fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile protobuf definitions
    // Note: tonic-build 0.14 uses a different API
    // For now, we'll use prost-build directly until we verify the exact API
    let mut config = prost_build::Config::new();
    config.out_dir("src/grpc");
    config.compile_protos(&["proto/vectorizer.proto"], &["proto"])?;

    // Embed Windows icon resource
    #[cfg(all(target_os = "windows", not(target_env = "msvc")))]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "Vectorizer");
        res.set("FileDescription", "High-performance vector database");
        res.set("CompanyName", "HiveLLM Contributors");
        res.set("LegalCopyright", "Copyright © 2025 HiveLLM Contributors");
        res.set("ProductVersion", env!("CARGO_PKG_VERSION"));
        res.set("FileVersion", env!("CARGO_PKG_VERSION"));

        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile Windows resource: {}", e);
        }
    }

    // Skip resource generation on MSVC to avoid CVT1100 duplicate resource error
    // The winres crate can conflict with MSVC toolchain's default resource handling
    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    {
        println!(
            "cargo:warning=Skipping winres resource generation on MSVC to avoid duplicate resource errors"
        );
    }

    Ok(())
}
