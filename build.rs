fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .type_attribute(
            "TopSearch",
            "#[derive(serde::Deserialize, serde::Serialize)]",
        )
        .compile_protos(&["protos/googol.proto", "protos/helloworld.proto"], &["."])
        .unwrap_or_else(|e| panic!("protobuf compile error: {}", e));
    Ok(())
}
