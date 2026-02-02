// Build scripts cannot use tracing - reverting to println!
// use tracing::{info, error, warn, debug};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use vendored protoc when PROTOC is not set (e.g. cross-compilation in CI)
    if std::env::var("PROTOC").is_err() {
        if let Ok(path) = protoc_bin_vendored::protoc_bin_path() {
            // SAFETY: build script runs in isolated process; no other threads read env
            unsafe { std::env::set_var("PROTOC", path) };
        }
    }

    // Compile protobuf definitions with tonic-build
    // Using tonic-build 0.12 for stable API compatibility
    // Note: This will recompile if proto files change (expected behavior)
    // To avoid unnecessary rebuilds, proto files should be committed and only changed when needed
    tonic_build::configure()
        .build_server(true)
        .build_client(true) // Enable client generation for tests
        .out_dir("src/grpc")
        .compile_protos(
            &["proto/vectorizer.proto", "proto/cluster.proto"],
            &["proto"],
        )?;

    // Compile Qdrant-compatible gRPC proto definitions
    println!("cargo:rerun-if-changed=proto/qdrant/");
    std::fs::create_dir_all("src/grpc/qdrant")?;
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/grpc/qdrant")
        .compile_protos(
            &[
                "proto/qdrant/collections_service.proto",
                "proto/qdrant/points_service.proto",
                "proto/qdrant/snapshots_service.proto",
            ],
            &["proto/qdrant"],
        )?;

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

    Ok(())
}
