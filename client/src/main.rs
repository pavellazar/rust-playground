use tracing::info;
use runner_extended::{Runner, RunnerExtended};

fn main() {
    tracing_subscriber::fmt::init();
    
    let runner = Runner;
    let result = runner.run_add(10, 20);
    info!("add: 10 + 20 = {}", result);
    
    let result = runner.run_mul(15, 3);
    info!("mul: 15 * 3 = {}", result);
}