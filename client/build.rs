fn main() -> Result<(), Box<dyn std::error::Error>> {
  let _ = tonic_build::configure()
    .boxed("FibonacciResponse.data")
    .boxed("FibonacciRequest.data")
    .compile(&["../protos/fibonacci.proto"], &["../protos"]);

  Ok(())
}
