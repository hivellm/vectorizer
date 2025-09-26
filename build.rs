fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path("target/vectorizer_descriptor.bin")
        .compile(&["proto/vectorizer.proto"], &["proto"])?;
    Ok(())
}
