use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=proto/");
    println!("cargo:rerun-if-changed=config.yml");

    // Build protobuf files
    tonic_build::compile_protos("proto/vectorizer.proto").expect("Failed to compile proto files");

    // Check for CUDA libraries if cuda_real feature is enabled AND cuda is enabled in config
    if env::var("CARGO_FEATURE_CUDA_REAL").is_ok() && cuda_enabled_in_config() {
        setup_cuda_libraries();
    }
}

fn setup_cuda_libraries() {
    println!("cargo:rerun-if-changed=lib/");

    // Check for pre-built CUDA libraries in lib/ directory
    let lib_dir = PathBuf::from("lib");

    if !lib_dir.exists() {
        println!("cargo:warning=CUDA lib directory not found - CUDA libraries not available");
        return;
    }

    // Check if CUDA libraries exist and are valid
    let lib_name = if cfg!(target_os = "windows") {
        "cuhnsw.lib"
    } else {
        "libcuhnsw.a"
    };

    let lib_path = lib_dir.join(lib_name);

    let has_valid_library = lib_path.exists() &&
        std::fs::metadata(&lib_path).map(|m| m.len() > 1_000).unwrap_or(false); // Require at least 1KB for a valid library (real or stub)

    if has_valid_library {
        println!("cargo:warning=Found complete CUDA library: {} ({} bytes)", lib_path.display(),
                std::fs::metadata(&lib_path).unwrap().len());

        // Enable CUDA library feature
        println!("cargo:rustc-cfg=feature=\"cuda_library_available\"");

        // Link to the lib directory
        println!("cargo:rustc-link-search=native=lib");

        // Link the cuhnsw library
        println!("cargo:rustc-link-lib=static=cuhnsw");

        // Try to find CUDA installation for linking CUDA runtime libraries
        let cuda_path = find_cuda_path();
        let cuda_lib = if cfg!(target_os = "windows") {
            format!("{}/lib/x64", cuda_path)
        } else {
            format!("{}/lib64", cuda_path)
        };

        // Try to link CUDA runtime libraries if CUDA is available
        if Path::new(&cuda_lib).exists() {
            println!("cargo:rustc-link-search=native={}", cuda_lib);
            println!("cargo:rustc-link-lib=cudart");
            println!("cargo:rustc-link-lib=curand");
            println!("cargo:rustc-link-lib=cublas");
            println!("cargo:warning=CUDA runtime libraries linked successfully");
        } else {
            println!("cargo:warning=CUDA runtime libraries not found, CUDA may not work properly");
        }

        // Link standard libraries
        if cfg!(target_os = "windows") {
            println!("cargo:rustc-link-lib=msvcrt");
        } else {
            println!("cargo:rustc-link-lib=stdc++");
        }

        println!("cargo:warning=CUDA acceleration enabled with real GPU support");

    } else if lib_path.exists() {
        println!("cargo:warning=CUDA library found but too small ({} bytes) - using CPU fallback",
                std::fs::metadata(&lib_path).unwrap().len());
    } else {
        println!("cargo:warning=No CUDA library found in lib/ - using CPU fallback");
    }
}

fn cuda_enabled_in_config() -> bool {
    // Try to read config.yml
    match std::fs::read_to_string("config.yml") {
        Ok(content) => {
            // Simple check for cuda.enabled: true
            if content.contains("cuda:") {
                // Look for enabled: true in the cuda section
                let lines: Vec<&str> = content.lines().collect();
                let mut in_cuda_section = false;

                for line in lines {
                    let trimmed = line.trim();

                    // Check if we're entering the cuda section
                    if trimmed.starts_with("cuda:") {
                        in_cuda_section = true;
                        continue;
                    }

                    // If we're in cuda section and find enabled: true, return true
                    if in_cuda_section {
                        if trimmed.starts_with("enabled:") {
                            return trimmed.contains("true");
                        }
                        // Exit cuda section if we hit another top-level key
                        if !trimmed.starts_with(" ") && !trimmed.starts_with("#") && trimmed.contains(":") && !trimmed.starts_with("enabled:") {
                            break;
                        }
                    }
                }

                // If we found cuda section but no enabled setting, default to false
                false
            } else {
                // No cuda section found, disable CUDA
                false
            }
        }
        Err(_) => {
            // Can't read config.yml, disable CUDA to be safe
            println!("cargo:warning=config.yml not found or unreadable, CUDA disabled by default");
            false
        }
    }
}

fn find_cuda_path() -> String {
    // Try environment variables first
    if let Ok(path) = env::var("CUDA_PATH") {
        return path;
    }
    if let Ok(path) = env::var("CUDA_HOME") {
        return path;
    }

    // Default paths
    if cfg!(target_os = "windows") {
        // Try common Windows CUDA installation paths
        let paths = vec![
            "C:/Program Files/NVIDIA GPU Computing Toolkit/CUDA/v13.0",
            "C:/Program Files/NVIDIA GPU Computing Toolkit/CUDA/v12.0",
            "C:/Program Files/NVIDIA GPU Computing Toolkit/CUDA/v11.8",
            "C:/Program Files/NVIDIA GPU Computing Toolkit/CUDA/v11.7",
        ];

        for path in paths {
            if Path::new(path).exists() {
                return path.to_string();
            }
        }

        panic!("CUDA not found. Please set CUDA_PATH environment variable.");
    } else {
        "/usr/local/cuda".to_string()
    }
}