use backoff::future::retry;
use backoff::ExponentialBackoff;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tonic::Request;

pub mod paas {
  tonic::include_proto!("paas");
}

use paas::proof_service_client::ProofServiceClient;
use paas::ProofRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let url = "http://[::1]:50052"; // Connect to proxy, not server

  let backoff = ExponentialBackoff {
    initial_interval: std::time::Duration::from_secs(1),
    max_interval: std::time::Duration::from_secs(60),
    multiplier: 2.0,
    max_elapsed_time: Some(std::time::Duration::from_secs(300)),
    ..Default::default()
  };

  let mut client = retry(backoff, || async {
    println!("Attempting to connect to {}", url);
    ProofServiceClient::connect(url)
      .await
      .map_err(backoff::Error::transient)
  })
  .await?;

  // Create a stream of requests
  let (tx, rx) = tokio::sync::mpsc::channel(1024);
  let request_stream = ReceiverStream::new(rx);
  let mut response_stream = client.run(Request::new(request_stream)).await?.into_inner();

  // Spawn a task to send requests
  tokio::spawn(async move {
    for n in [1, 2, 3, 42, 93, 94].iter() {
      // Example sequence, including an invalid one
      let request = ProofRequest { n: *n };
      if tx.send(request).await.is_err() {
        break; // Stream closed
      }
      tokio::time::sleep(std::time::Duration::from_millis(500)).await; // Simulate work
    }
  });

  // Receive and print responses
  while let Some(response) = response_stream.next().await {
    match response {
      Ok(resp) => println!("Response: {}", resp.result),
      Err(e) => println!("Error: {}", e),
    }
  }

  Ok(())
}
