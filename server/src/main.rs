use math::{AddOperationResult, MulOperationResult, PowOperationResult};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};

mod math {
  tonic::include_proto!("math");
}

use crate::math::math_server::{Math, MathServer};
use crate::math::{ComputeRequest, ComputeResponse};
struct MathService;

#[tonic::async_trait]
impl Math for MathService {
  type ComputeStream = ReceiverStream<Result<ComputeResponse, Status>>;

  async fn compute(
    &self,
    request: Request<Streaming<ComputeRequest>>,
  ) -> Result<Response<Self::ComputeStream>, Status> {
    let mut request_stream = request.into_inner();
    let (tx, rx) = mpsc::channel(4);

    tokio::spawn(async move {
      while let Some(req) = request_stream.message().await.transpose() {
        match req {
          Ok(req) => {
            let response = handle_compute_operation(req);
            if tx.send(Ok(response)).await.is_err() {
              break;
            }
          }
          Err(e) => {
            let _ = tx
              .send(Err(Status::invalid_argument(format!("error decoding message: {}", e))))
              .await;
            break;
          }
        }
      }
    });

    Ok(Response::new(ReceiverStream::new(rx)))
  }
}
fn handle_compute_operation(req: ComputeRequest) -> ComputeResponse {
  match req.operation {
    Some(math::compute_request::Operation::Add(op)) => ComputeResponse {
      result: Some(math::compute_response::Result::AddResult(AddOperationResult {
        result: op.a + op.b,
      })),
    },
    Some(math::compute_request::Operation::Mul(op)) => ComputeResponse {
      result: Some(math::compute_response::Result::MulResult(MulOperationResult {
        result: op.a * op.b,
      })),
    },
    Some(math::compute_request::Operation::Pow(op)) => ComputeResponse {
      result: Some(math::compute_response::Result::PowResult(PowOperationResult {
        result: op.base.pow(op.exponent),
      })),
    },
    None => ComputeResponse { result: None },
  }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let addr = "[::1]:50051".parse()?;
  let math_service = MathService;

  tonic::transport::Server::builder()
    .add_service(MathServer::new(math_service))
    .serve(addr)
    .await?;

  Ok(())
}
