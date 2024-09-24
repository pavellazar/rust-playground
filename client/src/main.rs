use fibonacci::fibonacci_client::FibonacciClient;
use fibonacci::{Data, FibonacciRequest};

pub mod fibonacci {
  tonic::include_proto!("fibonacci");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let mut client = FibonacciClient::connect("http://[::1]:50051").await?;

  let request = tonic::Request::new(FibonacciRequest {
    number: 10,
    data: Some(Box::new(Data { message: "Hello, World!".to_string() })),
  });
  let response = client.compute(request).await?;

  println!("RESPONSE={:?}", response);

  Ok(())
}
