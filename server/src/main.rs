use fibonacci::fibonacci_server::{Fibonacci, FibonacciServer};
use fibonacci::{FibonacciRequest, FibonacciResponse};
use tonic::{transport::Server, Request, Response, Status};

pub mod fibonacci {
    tonic::include_proto!("fibonacci");
}

#[derive(Default)]
pub struct MyFibonacci {}

#[tonic::async_trait]
impl Fibonacci for MyFibonacci {
    async fn compute(
        &self,
        request: Request<FibonacciRequest>,
    ) -> Result<Response<FibonacciResponse>, Status> {
        let request = request.into_inner();
        let (number, data) = (request.number, request.data);
        let result = fibonacci(number);
        Ok(Response::new(FibonacciResponse { result, data }))
    }
}

fn fibonacci(n: i32) -> i32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let fibonacci = MyFibonacci::default();

    Server::builder()
        .add_service(FibonacciServer::new(fibonacci))
        .serve(addr)
        .await?;

    Ok(())
}