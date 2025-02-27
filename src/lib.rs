pub mod operations_source;
use build_deps::fetch_operations;

pub fn list_operations() -> Vec<String> {
  fetch_operations()
}

pub fn run_operation(op: String, a: u32, b: u32) -> u32 {
  match op.as_str() {
    "add" => a + b,
    "mul" => a * b,
    _ => panic!("Unsupported operation"),
  }
}

include!("generated_operations.rs");
