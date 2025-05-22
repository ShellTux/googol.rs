fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("protos/googol.proto")?;
    tonic_build::compile_protos("protos/helloworld.proto")?;
    Ok(())
}
