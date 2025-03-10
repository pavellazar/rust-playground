use crate::paas::proof_service_client::ProofServiceClient;
use crate::paas::proof_service_server::{ProofService, ProofServiceServer};
use crate::paas::{ProofRequest, ProofResponse};
use backoff::future::retry;
use backoff::ExponentialBackoff;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tonic::{
  transport::{Channel, Server},
  Request, Response, Status, Streaming,
};

pub mod paas {
  tonic::include_proto!("paas");
}

#[derive(Debug, Clone)]
struct ProxyService {
  backend_tx: mpsc::Sender<ProofRequest>, // Send requests to the backend stream
  workers:
    Arc<RwLock<std::collections::HashMap<usize, mpsc::Sender<Result<ProofResponse, Status>>>>>, // Worker response channels
  next_worker_id: Arc<RwLock<usize>>, // Assign unique worker IDs
}

impl ProxyService {
  async fn new(service_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
    let client = connect_to_service(service_url).await?;
    let (tx, rx) = mpsc::channel::<ProofRequest>(1024); // Buffer for backend requests
    let proxy = ProxyService {
      backend_tx: tx,
      workers: Arc::new(RwLock::new(std::collections::HashMap::new())),
      next_worker_id: Arc::new(RwLock::new(0)),
    };

    let proxy_clone = proxy.clone();
    tokio::spawn(async move {
      if let Err(e) = proxy_clone.run_backend_stream(client, rx).await {
        eprintln!("Backend stream failed: {}", e);
      }
    });

    Ok(proxy)
  }

  async fn run_backend_stream(
    &self,
    mut client: ProofServiceClient<Channel>,
    rx: mpsc::Receiver<ProofRequest>,
  ) -> Result<(), Status> {
    let inbound_stream = ReceiverStream::new(rx);
    let mut outbound_stream = client
      .run(Request::new(inbound_stream))
      .await
      .map_err(|e| Status::internal(format!("Failed to start backend stream: {}", e)))?
      .into_inner();

    while let Some(response) = outbound_stream.next().await {
      let response = response; // Already Result<ProofResponse, Status>
      let workers = self.workers.read().await;
      for sender in workers.values() {
        if let Err(e) = sender.send(response.clone()).await {
          eprintln!("Failed to send to worker: {:?}", e);
        }
      }
    }
    Err(Status::internal("Backend stream ended unexpectedly"))
  }

  async fn register_worker(&self) -> (usize, mpsc::Receiver<Result<ProofResponse, Status>>) {
    let (tx, rx) = mpsc::channel::<Result<ProofResponse, Status>>(1024);
    let mut next_id = self.next_worker_id.write().await;
    let worker_id = *next_id;
    *next_id += 1;

    let mut workers = self.workers.write().await;
    workers.insert(worker_id, tx);
    (worker_id, rx)
  }

  async fn unregister_worker(&self, worker_id: usize) {
    let mut workers = self.workers.write().await;
    workers.remove(&worker_id);
  }
}

#[tonic::async_trait]
impl ProofService for ProxyService {
  type RunStream = ReceiverStream<Result<ProofResponse, Status>>;

  async fn run(
    &self,
    request: Request<Streaming<ProofRequest>>,
  ) -> Result<Response<Self::RunStream>, Status> {
    let mut stream = request.into_inner();
    let (worker_id, worker_rx) = self.register_worker().await; // Use the helper function
    let proxy = self.clone();
    let backend_tx = self.backend_tx.clone();

    tokio::spawn(async move {
      while let Some(request) = stream.next().await {
        match request {
          Ok(req) => {
            if let Err(e) = backend_tx.send(req).await {
              eprintln!("Failed to forward request to backend: {:?}", e);
              break;
            }
          }
          Err(e) => {
            eprintln!("Worker stream error: {}", e);
            break;
          }
        }
      }
      proxy.unregister_worker(worker_id).await;
    });

    Ok(Response::new(ReceiverStream::new(worker_rx)))
  }
}

async fn connect_to_service(
  url: &str,
) -> Result<ProofServiceClient<Channel>, tonic::transport::Error> {
  let backoff = ExponentialBackoff {
    initial_interval: std::time::Duration::from_secs(1),
    max_interval: std::time::Duration::from_secs(60),
    multiplier: 2.0,
    max_elapsed_time: Some(std::time::Duration::from_secs(300)),
    ..Default::default()
  };

  retry(backoff, || {
    let url = url.to_string();
    async move {
      println!("Attempting to connect to {}", url);
      ProofServiceClient::connect(url)
        .await
        .map_err(backoff::Error::transient)
    }
  })
  .await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let service_url = "http://[::1]:50051";
  let proxy_addr = "[::1]:50052".parse()?;

  let proxy_service = ProxyService::new(service_url).await?;

  println!("Proxy running on {}", proxy_addr);

  Server::builder()
    .add_service(ProofServiceServer::new(proxy_service))
    .serve(proxy_addr)
    .await?;

  Ok(())
}
