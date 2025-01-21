fn main() -> Result<(), Box<dyn std::error::Error>> {
  tonic_build::configure().compile_protos(&["../protos/math.proto"], &["../protos"])?;
  Ok(())
}
