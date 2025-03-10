use paas::proof_service_server::{ProofService, ProofServiceServer};
use paas::{ProofRequest, ProofResponse};
use tokio_stream::StreamExt;
use tonic::{transport::Server, Request, Response, Status, Streaming};

pub mod paas {
  tonic::include_proto!("paas");
}

#[derive(Debug, Default)]
struct PaaSService;

#[tonic::async_trait]
impl ProofService for PaaSService {
  type RunStream = tokio_stream::wrappers::ReceiverStream<Result<ProofResponse, Status>>;

  async fn run(
    &self,
    request: Request<Streaming<ProofRequest>>,
  ) -> Result<Response<Self::RunStream>, Status> {
    let mut stream = request.into_inner();
    let (tx, rx) = tokio::sync::mpsc::channel(1024);

    tokio::spawn(async move {
      while let Some(request) = stream.next().await {
        match request {
          Ok(req) => {
            let n = req.n;
            if n > 93 {
              let _ = tx
                .send(Err(Status::invalid_argument("Input too large, max is 93")))
                .await;
              continue;
            }
            let result = fibonacci(n);
            let response = ProofResponse { result };
            if tx.send(Ok(response)).await.is_err() {
              break; // Receiver dropped
            }
          }
          Err(e) => {
            let _ = tx.send(Err(e)).await;
            break;
          }
        }
      }
    });

    Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(
      rx,
    )))
  }
}

fn fibonacci(n: u32) -> u64 {
  if n <= 1 {
    return n as u64;
  }
  let mut a = 0u64;
  let mut b = 1u64;
  for _ in 2..=n {
    let temp = a + b;
    a = b;
    b = temp;
  }
  b
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let addr = "[::1]:50051".parse()?;
  let service = PaaSService::default();

  println!("Service running on {}", addr);

  Server::builder()
    .add_service(ProofServiceServer::new(service))
    .serve(addr)
    .await?;

  Ok(())
}
