pub struct Runner;

impl Runner {
  pub fn list_operations(&self) -> Vec<String> {
    vec!["add".to_string(), "mul".to_string()]
  }

  pub fn run_operation(&self, op: String, a: u32, b: u32) -> u32 {
    match op.as_str() {
      "add" => a + b,
      "mul" => a * b,
      _ => panic!("Unsupported operation"),
    }
  }
}