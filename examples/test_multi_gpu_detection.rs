//! Test Multi-GPU Backend Detection
//!
//! This example demonstrates the automatic detection of available GPU backends
//! including Metal (macOS), Vulkan (AMD/NVIDIA/Intel), DirectX 12 (Windows), and CUDA.

use vectorizer::gpu::{detect_available_backends, select_best_backend, GpuBackendType};

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    println!("\n🚀 ==========================================");
    println!("   Multi-GPU Backend Detection Test");
    println!("==========================================\n");

    // Detect all available backends
    println!("🔍 Scanning for available GPU backends...\n");
    let available = detect_available_backends();

    if available.is_empty() {
        println!("❌ No GPU backends detected!");
        return;
    }

    println!("📊 Detection Results:\n");
    println!("┌───────────────────────────────────────┐");
    println!("│  Backend          │ Status  │ Priority │");
    println!("├───────────────────────────────────────┤");

    for backend in &available {
        let status = "✅ Available";
        let priority = backend.priority();
        println!("│  {:16} │ {:8} │    {:3}    │", backend.name(), status, priority);
    }

    println!("└───────────────────────────────────────┘\n");

    // Select best backend
    let best = select_best_backend(&available);

    println!("🎯 Best Backend Selected:");
    println!("   {} {}\n", best.icon(), best.name());

    // Show platform info
    println!("📍 Platform Information:");
    println!("   OS: {}", std::env::consts::OS);
    println!("   Arch: {}", std::env::consts::ARCH);

    #[cfg(feature = "cuda")]
    println!("   CUDA: Enabled");

    #[cfg(not(feature = "cuda"))]
    println!("   CUDA: Disabled");

    #[cfg(feature = "wgpu-gpu")]
    println!("   wgpu-gpu: Enabled");

    #[cfg(not(feature = "wgpu-gpu"))]
    println!("   wgpu-gpu: Disabled");

    println!("\n🎉 Detection complete!\n");

    // Show recommendations
    println!("💡 Recommendations:");
    match best {
        GpuBackendType::Metal => {
            println!("   • Metal is the best option for Apple Silicon");
            println!("   • Optimized for M1/M2/M3/M4 chips");
            println!("   • No additional drivers required");
        }
        GpuBackendType::Vulkan => {
            println!("   • Vulkan is available and universal");
            println!("   • Works with AMD, NVIDIA, and Intel GPUs");
            println!("   • Ensure Vulkan drivers are up to date");
        }
        GpuBackendType::DirectX12 => {
            println!("   • DirectX 12 is the best option for Windows");
            println!("   • Native Windows GPU acceleration");
            println!("   • Supports all modern GPUs");
        }
        GpuBackendType::CudaNative => {
            println!("   • CUDA is available for NVIDIA GPUs");
            println!("   • Specialized for deep learning workloads");
            println!("   • Ensure CUDA toolkit is installed");
        }
        GpuBackendType::Cpu => {
            println!("   ⚠️  No GPU detected, using CPU fallback");
            println!("   • Consider installing GPU drivers");
            println!("   • Performance will be limited");
        }
    }

    println!("\n==========================================\n");
}

