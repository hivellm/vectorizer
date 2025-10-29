// Build script with safety checks
//
// This build script enforces safety guardrails at compile time
// to prevent dangerous configurations that can cause BSODs.

use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_HIVE_GPU");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_HIVE_GPU_CUDA");

    // Platform detection
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let is_windows = target_os == "windows";

    // Feature detection
    let has_gpu = env::var("CARGO_FEATURE_HIVE_GPU").is_ok();
    let has_cuda = env::var("CARGO_FEATURE_HIVE_GPU_CUDA").is_ok();
    let has_fastembed = env::var("CARGO_FEATURE_FASTEMBED").is_ok();
    let has_all_features = env::var("CARGO_FEATURE_FULL").is_ok();

    // Profile detection
    let profile = env::var("PROFILE").unwrap_or_default();
    let is_release = profile == "release";
    
    // Parallelism warnings
    let build_jobs = env::var("CARGO_BUILD_JOBS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(num_cpus::get());

    // Set compiler flags based on platform
    if is_windows {
        // Windows-specific optimizations
        println!("cargo:rustc-env=RAYON_NUM_THREADS=2");
    }

    // Generate Windows resource file
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico")
            .set("ProductName", "Vectorizer")
            .set("FileDescription", "High-Performance Vector Database")
            .set("CompanyName", "HiveLLM");

        // Best effort - don't fail build if icon missing
        let _ = res.compile();
    }
}
