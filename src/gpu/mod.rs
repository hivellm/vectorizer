//! Aceleração GPU usando Implementações Nativas (Metal, CUDA)
//!
//! Este módulo fornece aceleração GPU usando implementações nativas puras
//! sem dependências de wgpu para máxima performance e eficiência.
//!
//! ## Implementações Suportadas
//!
//! - **Metal Native**: macOS com Apple Silicon (M1/M2/M3/M4)
//! - **CUDA**: Linux/Windows com GPUs NVIDIA
//!
//! ## Operações Suportadas
//!
//! - Similaridade Coseno (Cosine Similarity)
//! - Distância Euclidiana (Euclidean Distance)
//! - Produto Escalar (Dot Product)
//! - Busca em Lote (Batch Search)
//! - Top-K Selection
//! - HNSW (Hierarchical Navigable Small World) para busca aproximada
//!
//! ## Uso
//!
//! ## Metal Native Example (macOS)
//!
//! ```rust
//! use vectorizer::gpu::MetalNativeCollection;
//! use vectorizer::models::DistanceMetric;
//!
//! // Criar coleção Metal Native
//! let mut collection = MetalNativeCollection::new(512, DistanceMetric::Cosine).unwrap();
//!
//! // Adicionar vetores
//! let vector1 = vectorizer::models::Vector::new("vec1".to_string(), vec![1.0; 512]);
//! let vector2 = vectorizer::models::Vector::new("vec2".to_string(), vec![0.0; 512]);
//! collection.add_vector(vector1).unwrap();
//! collection.add_vector(vector2).unwrap();
//!
//! // Buscar similaridades
//! let query = vec![1.0; 512];
//! let results = collection.search(&query, 10).unwrap();
//! ```
//!
//! ## CUDA Example (Linux/Windows)
//!
//! ```rust,no_run
//! # #[cfg(feature = "cuda")]
//! use vectorizer::gpu::cuda::{CudaContext, CudaVectorOps};
//!
//! # #[cfg(feature = "cuda")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Criar contexto CUDA
//! let context = CudaContext::new()?;
//!
//! // Realizar operações vetoriais
//! let query = vec![1.0, 2.0, 3.0];
//! let vectors = vec![
//!     vec![1.0, 0.0, 0.0],
//!     vec![0.0, 1.0, 0.0],
//!     vec![0.0, 0.0, 1.0],
//! ];
//!
//! let similarities = context.cosine_similarity(&query, &vectors)?;
//! # Ok(())
//! # }
//! ```

// Native GPU implementations only (no wgpu dependencies)

// Metal Native implementation (macOS only)
pub mod metal_native;
pub use metal_native::MetalNativeCollection;

// Metal Native HNSW implementation
pub mod metal_native_hnsw;
pub use metal_native_hnsw::{MetalNativeHnswGraph, MetalNativeContext};

// Metal buffer pool for optimized memory management
pub mod metal_buffer_pool;
pub use metal_buffer_pool::{MetalBufferPool, OptimizedMetalNativeCollection, BufferPoolStats, CollectionMemoryStats};

// VRAM monitoring and validation
pub mod vram_monitor;
pub use vram_monitor::{VramMonitor, VramValidator, VramStats, VramBufferInfo, VramBenchmarkResult};

// Native Metal helpers for synchronous buffer operations (macOS only)
#[cfg(target_os = "macos")]
pub mod metal_helpers;
#[cfg(target_os = "macos")]
pub use metal_helpers::{MetalBufferReader, create_system_metal_device};

// CUDA implementation (Linux/Windows)
#[cfg(feature = "cuda")]
pub mod cuda;

// Unified backend detection for native implementations
pub mod backends;
pub use backends::{GpuBackendType, detect_available_backends, select_best_backend};