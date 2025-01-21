mod math {
  tonic::include_proto!("math");
}

use crate::math::math_client::MathClient;
use crate::math::{compute_request::Operation, AddOperation, ComputeRequest};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let mut client = MathClient::connect("http://[::1]:50051").await?;
  let (tx, rx) = mpsc::channel(4);

  tokio::spawn(async move {
    for i in 0..10 {
      let operation = Operation::Add(AddOperation { a: i, b: i + 2 });
      let request = ComputeRequest { operation: Some(operation) };
      tx.send(request).await.unwrap();
    }
    drop(tx); // Close the sender to indicate the end of the stream
  });

  let mut response_stream = client.compute(ReceiverStream::new(rx)).await?.into_inner();

  while let Some(response) = response_stream.message().await? {
    println!("RESPONSE = {:?}", response);
  }

  Ok(())
}
