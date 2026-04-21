// Build scripts cannot use tracing — keep `println!` only.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use vendored protoc when PROTOC is not set (e.g. cross-compilation in CI
    // where the container has no protoc).
    if std::env::var("PROTOC").is_err()
        && let Ok(path) = protoc_bin_vendored::protoc_bin_path()
    {
        // SAFETY: build script runs in isolated process; no other threads read env
        unsafe { std::env::set_var("PROTOC", path) };
    }

    // Compile the first-party + cluster protos.
    //
    // Explicit rerun-if-changed directives: `compile_protos` does NOT
    // always emit these (observed during phase2_unify-search-result-type
    // when a `double score → float score` edit didn't trigger
    // regeneration on the next `cargo build`). Stating them here keeps
    // the build unambiguous.
    println!("cargo:rerun-if-changed=proto/vectorizer.proto");
    println!("cargo:rerun-if-changed=proto/cluster.proto");
    std::fs::create_dir_all("src/grpc_gen")?;
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true) // Enable client generation for tests + the Rust SDK
        .out_dir("src/grpc_gen")
        .compile_protos(
            &["proto/vectorizer.proto", "proto/cluster.proto"],
            &["proto"],
        )?;

    // Qdrant-compatible protos.
    println!("cargo:rerun-if-changed=proto/qdrant/");
    std::fs::create_dir_all("src/grpc_gen/qdrant")?;
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/grpc_gen/qdrant")
        .compile_protos(
            &[
                "proto/qdrant/collections_service.proto",
                "proto/qdrant/points_service.proto",
                "proto/qdrant/snapshots_service.proto",
            ],
            &["proto/qdrant"],
        )?;

    Ok(())
}
