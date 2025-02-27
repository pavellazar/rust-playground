use rust_playground::{run_add, run_mul};
use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  tracing_subscriber::fmt::init();

  // Test the generated functions
  info!("run_add(5, 10) = {}", run_add(5, 10));
  info!("run_mul(5, 10) = {}", run_mul(5, 10));

  Ok(())
}
