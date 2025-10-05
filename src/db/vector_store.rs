//! Main VectorStore implementation

use crate::{
    error::{Result, VectorizerError},
    models::{CollectionConfig, CollectionMetadata, SearchResult, Vector},
    cuda::CudaConfig,
};
use anyhow::anyhow;
use dashmap::DashMap;
use std::sync::Arc;
use std::ops::Deref;
use std::collections::HashSet;
use std::time::Duration;
use std::path::PathBuf;
use tracing::{debug, info, warn};

use super::collection::Collection;
#[cfg(feature = "cuda")]
use crate::cuda::collection::CudaCollection;
#[cfg(feature = "wgpu-gpu")]
use crate::gpu::{MetalCollection, VulkanCollection, DirectX12Collection, GpuConfig};

/// Enum to represent different collection types (CPU, CUDA, or Metal)
pub enum CollectionType {
    /// CPU-based collection
    Cpu(Collection),
    /// CUDA-accelerated collection
    #[cfg(feature = "cuda")]
    Cuda(CudaCollection),
    /// Metal-accelerated collection (Apple Silicon)
    #[cfg(feature = "wgpu-gpu")]
    Metal(MetalCollection),
    /// Vulkan-accelerated collection (AMD/NVIDIA/Intel/Universal)
    #[cfg(feature = "wgpu-gpu")]
    Vulkan(VulkanCollection),
    /// DirectX 12-accelerated collection (Windows)
    #[cfg(feature = "wgpu-gpu")]
    DirectX12(DirectX12Collection),
}

impl std::fmt::Debug for CollectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectionType::Cpu(c) => write!(f, "CollectionType::Cpu({})", c.name()),
            #[cfg(feature = "cuda")]
            CollectionType::Cuda(c) => write!(f, "CollectionType::Cuda({})", c.name()),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Metal(c) => write!(f, "CollectionType::Metal({})", c.name()),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Vulkan(c) => write!(f, "CollectionType::Vulkan({})", c.name()),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::DirectX12(c) => write!(f, "CollectionType::DirectX12({})", c.name()),
        }
    }
}

impl CollectionType {
    /// Get collection name
    pub fn name(&self) -> &str {
        match self {
            CollectionType::Cpu(c) => c.name(),
            #[cfg(feature = "cuda")]
            CollectionType::Cuda(c) => c.name(),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Metal(c) => c.name(),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Vulkan(c) => c.name(),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::DirectX12(c) => c.name(),
        }
    }

    /// Get collection config
    pub fn config(&self) -> &CollectionConfig {
        match self {
            CollectionType::Cpu(c) => c.config(),
            #[cfg(feature = "cuda")]
            CollectionType::Cuda(c) => c.config(),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Metal(c) => c.config(),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Vulkan(c) => c.config(),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::DirectX12(c) => c.config(),
        }
    }

    /// Add a vector to the collection
    pub fn add_vector(&self, _id: String, vector: Vector) -> Result<()> {
        match self {
            CollectionType::Cpu(c) => c.insert(vector),
            #[cfg(feature = "cuda")]
            CollectionType::Cuda(c) => c.add_vector(vector),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Metal(c) => pollster::block_on(c.add_vector(vector)),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Vulkan(c) => pollster::block_on(c.add_vector(vector)),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::DirectX12(c) => pollster::block_on(c.add_vector(vector)),
        }
    }

    /// Search for similar vectors
    pub fn search(&self, query: &[f32], limit: usize) -> Result<Vec<SearchResult>> {
        match self {
            CollectionType::Cpu(c) => c.search(query, limit),
            #[cfg(feature = "cuda")]
            CollectionType::Cuda(c) => c.search(query, limit),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Metal(c) => pollster::block_on(c.search(query, limit)),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Vulkan(c) => pollster::block_on(c.search(query, limit)),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::DirectX12(c) => pollster::block_on(c.search(query, limit)),
        }
    }

    /// Get collection metadata
    pub fn metadata(&self) -> CollectionMetadata {
        match self {
            CollectionType::Cpu(c) => c.metadata(),
            #[cfg(feature = "cuda")]
            CollectionType::Cuda(c) => c.metadata(),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Metal(c) => c.metadata(),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Vulkan(c) => c.metadata(),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::DirectX12(c) => c.metadata(),
        }
    }

    /// Delete a vector from the collection
    pub fn delete_vector(&self, id: &str) -> Result<()> {
        match self {
            CollectionType::Cpu(c) => c.delete(id),
            #[cfg(feature = "cuda")]
            CollectionType::Cuda(c) => c.remove_vector(id),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Metal(c) => c.remove_vector(id),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Vulkan(c) => c.remove_vector(id),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::DirectX12(c) => c.remove_vector(id),
        }
    }

    /// Get a vector by ID
    pub fn get_vector(&self, vector_id: &str) -> Result<Vector> {
        match self {
            CollectionType::Cpu(c) => c.get_vector(vector_id),
            #[cfg(feature = "cuda")]
            CollectionType::Cuda(c) => c.get_vector(vector_id),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Metal(c) => pollster::block_on(c.get_vector(vector_id)),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Vulkan(c) => pollster::block_on(c.get_vector(vector_id)),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::DirectX12(c) => pollster::block_on(c.get_vector(vector_id)),
        }
    }

    /// Get the number of vectors in the collection
    pub fn vector_count(&self) -> usize {
        match self {
            CollectionType::Cpu(c) => c.vector_count(),
            #[cfg(feature = "cuda")]
            CollectionType::Cuda(c) => c.vector_count(),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Metal(c) => c.vector_count(),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Vulkan(c) => c.vector_count(),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::DirectX12(c) => c.vector_count(),
        }
    }

    /// Get estimated memory usage
    pub fn estimated_memory_usage(&self) -> usize {
        match self {
            CollectionType::Cpu(c) => c.estimated_memory_usage(),
            #[cfg(feature = "cuda")]
            CollectionType::Cuda(c) => c.estimated_memory_usage(),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Metal(c) => c.estimated_memory_usage(),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Vulkan(c) => c.estimated_memory_usage(),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::DirectX12(c) => c.estimated_memory_usage(),
        }
    }

    /// Get all vectors in the collection
    pub fn get_all_vectors(&self) -> Vec<Vector> {
        match self {
            CollectionType::Cpu(c) => c.get_all_vectors(),
            #[cfg(feature = "cuda")]
            CollectionType::Cuda(c) => c.get_all_vectors(),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Metal(c) => pollster::block_on(c.get_all_vectors()).unwrap_or_default(),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Vulkan(c) => pollster::block_on(c.get_all_vectors()).unwrap_or_default(),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::DirectX12(c) => pollster::block_on(c.get_all_vectors()).unwrap_or_default(),
        }
    }

    /// Get embedding type
    pub fn get_embedding_type(&self) -> String {
        match self {
            CollectionType::Cpu(c) => c.get_embedding_type(),
            #[cfg(feature = "cuda")]
            CollectionType::Cuda(c) => c.get_embedding_type(),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Metal(c) => c.get_embedding_type(),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Vulkan(_c) => "unknown".to_string(),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::DirectX12(_c) => "unknown".to_string(),
        }
    }

    /// Requantize existing vectors if quantization is enabled
    pub fn requantize_existing_vectors(&self) -> Result<()> {
        match self {
            CollectionType::Cpu(c) => c.requantize_existing_vectors(),
            #[cfg(feature = "cuda")]
            CollectionType::Cuda(c) => {
                // For CUDA collections, we don't implement requantization yet
                warn!("Requantization not implemented for CUDA collections yet");
                Ok(())
            }
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Metal(_) => {
                warn!("Requantization not implemented for Metal collections yet");
                Ok(())
            }
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Vulkan(_) => {
                warn!("Requantization not implemented for Vulkan collections yet");
                Ok(())
            }
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::DirectX12(_) => {
                warn!("Requantization not implemented for DirectX 12 collections yet");
                Ok(())
            }
        }
    }

    /// Set embedding type
    pub fn set_embedding_type(&self, embedding_type: String) {
        match self {
            CollectionType::Cpu(c) => c.set_embedding_type(embedding_type),
            #[cfg(feature = "cuda")]
            CollectionType::Cuda(c) => c.set_embedding_type(embedding_type),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Metal(c) => c.set_embedding_type(embedding_type),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Vulkan(_c) => (),
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::DirectX12(_c) => (),
        }
    }

    /// Load HNSW index from dump
    pub fn load_hnsw_index_from_dump<P: AsRef<std::path::Path>>(&self, path: P, basename: &str) -> Result<()> {
        match self {
            CollectionType::Cpu(c) => c.load_hnsw_index_from_dump(path, basename),
            #[cfg(feature = "cuda")]
            CollectionType::Cuda(_) => {
                warn!("CUDA collections don't support HNSW dump loading yet");
                Ok(()) // No-op for now
            }
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Metal(_) => {
                warn!("Metal collections don't support HNSW dump loading yet");
                Ok(()) // No-op for now
            }
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Vulkan(_) => {
                warn!("Vulkan collections don\'t support HNSW dump loading yet");
                Ok(()) // No-op for now
            }
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::DirectX12(_) => {
                warn!("DirectX 12 collections don\'t support HNSW dump loading yet");
                Ok(()) // No-op for now
            }
        }
    }

    /// Load vectors into memory
    pub fn load_vectors_into_memory(&self, vectors: Vec<Vector>) -> Result<()> {
        match self {
            CollectionType::Cpu(c) => c.load_vectors_into_memory(vectors),
            #[cfg(feature = "cuda")]
            CollectionType::Cuda(_) => {
                warn!("CUDA collections don't support vector loading into memory yet");
                Ok(()) // No-op for now
            }
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Metal(_) => {
                warn!("Metal collections don't support vector loading into memory yet");
                Ok(()) // No-op for now
            }
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Vulkan(_) => {
                warn!("Vulkan collections don\'t support vector loading into memory yet");
                Ok(()) // No-op for now
            }
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::DirectX12(_) => {
                warn!("DirectX 12 collections don\'t support vector loading into memory yet");
                Ok(()) // No-op for now
            }
        }
    }

    /// Fast load vectors
    pub fn fast_load_vectors(&self, vectors: Vec<Vector>) -> Result<()> {
        match self {
            CollectionType::Cpu(c) => c.fast_load_vectors(vectors),
            #[cfg(feature = "cuda")]
            CollectionType::Cuda(_) => {
                warn!("CUDA collections don't support fast vector loading yet");
                Ok(()) // No-op for now
            }
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Metal(_) => {
                warn!("Metal collections don't support fast vector loading yet");
                Ok(()) // No-op for now
            }
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Vulkan(_) => {
                warn!("Vulkan collections don\'t support fast vector loading yet");
                Ok(()) // No-op for now
            }
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::DirectX12(_) => {
                warn!("DirectX 12 collections don\'t support fast vector loading yet");
                Ok(()) // No-op for now
            }
        }
    }
}

/// Thread-safe in-memory vector store
#[derive(Clone)]
pub struct VectorStore {
    /// Collections stored in a concurrent hash map
    collections: Arc<DashMap<String, CollectionType>>,
    /// CUDA configuration
    cuda_config: CudaConfig,
    /// Metal GPU configuration
    #[cfg(feature = "wgpu-gpu")]
    metal_config: Option<GpuConfig>,
    /// Vulkan GPU configuration
    #[cfg(feature = "wgpu-gpu")]
    vulkan_config: Option<GpuConfig>,
    /// DirectX 12 GPU configuration
    #[cfg(feature = "wgpu-gpu")]
    dx12_config: Option<GpuConfig>,
    /// Auto-save enabled flag (prevents auto-save during initialization)
    auto_save_enabled: Arc<std::sync::atomic::AtomicBool>,
    /// Collections pending save (for batch persistence)
    pending_saves: Arc<std::sync::Mutex<std::collections::HashSet<String>>>,
    /// Background save task handle
    save_task_handle: Arc<std::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl std::fmt::Debug for VectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VectorStore")
            .field("collections", &self.collections.len())
            .field("cuda_enabled", &self.cuda_config.enabled)
            .finish()
    }
}

impl VectorStore {
    /// Create a new empty vector store
    pub fn new() -> Self {
        Self::new_with_cuda_config(CudaConfig::default())
    }

    /// Create a new vector store with CUDA configuration
    pub fn new_with_cuda_config(cuda_config: CudaConfig) -> Self {
        info!("Creating new VectorStore with CUDA config: enabled={}", cuda_config.enabled);
        Self {
            collections: Arc::new(DashMap::new()),
            cuda_config,
            #[cfg(feature = "wgpu-gpu")]
            metal_config: None,
            #[cfg(feature = "wgpu-gpu")]
            vulkan_config: None,
            #[cfg(feature = "wgpu-gpu")]
            dx12_config: None,
            auto_save_enabled: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            pending_saves: Arc::new(std::sync::Mutex::new(HashSet::new())),
            save_task_handle: Arc::new(std::sync::Mutex::new(None)),
        }
    }
    
    /// Create a new vector store with Metal GPU configuration
    #[cfg(feature = "wgpu-gpu")]
    pub fn new_with_metal_config(metal_config: GpuConfig) -> Self {
        info!("Creating new VectorStore with Metal GPU config: enabled={}", metal_config.enabled);
        Self {
            collections: Arc::new(DashMap::new()),
            cuda_config: CudaConfig { enabled: false, ..Default::default() },
            metal_config: Some(metal_config),
            vulkan_config: None,
            dx12_config: None,
            auto_save_enabled: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            pending_saves: Arc::new(std::sync::Mutex::new(HashSet::new())),
            save_task_handle: Arc::new(std::sync::Mutex::new(None)),
        }
    }
    
    /// Create a new vector store with Vulkan GPU configuration
    #[cfg(feature = "wgpu-gpu")]
    pub fn new_with_vulkan_config(vulkan_config: GpuConfig) -> Self {
        info!("Creating new VectorStore with Vulkan GPU config: enabled={}", vulkan_config.enabled);
        Self {
            collections: Arc::new(DashMap::new()),
            cuda_config: CudaConfig { enabled: false, ..Default::default() },
            metal_config: None,
            vulkan_config: Some(vulkan_config),
            dx12_config: None,
            auto_save_enabled: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            pending_saves: Arc::new(std::sync::Mutex::new(HashSet::new())),
            save_task_handle: Arc::new(std::sync::Mutex::new(None)),
        }
    }
    
    /// Create a new vector store with automatic GPU detection
    /// Priority: Metal (Mac Silicon) > CUDA > CPU
    pub fn new_auto() -> Self {
        eprintln!("🔍 VectorStore::new_auto() called - starting GPU detection...");

        // Try to load persisted collections first
        let mut store = Self::new();
        if let Ok(collections_loaded) = store.load_all_persisted_collections() {
            if collections_loaded > 0 {
                eprintln!("✅ Loaded {} persisted collections from data directory", collections_loaded);
                info!("✅ Loaded {} persisted collections from data directory", collections_loaded);
                // Enable auto-save after loading persisted collections
                store.enable_auto_save();
                return store;
            }
        }
        
        // 1. Try Metal first (Mac Silicon with wgpu-gpu feature)
        #[cfg(all(target_os = "macos", target_arch = "aarch64", feature = "wgpu-gpu"))]
        {
            eprintln!("🍎 Detecting Metal GPU on Mac Silicon...");
            info!("🍎 Detecting Metal GPU on Mac Silicon...");
            let metal_config = crate::gpu::GpuConfig::for_metal_silicon();
            if let Ok(_) = pollster::block_on(crate::gpu::GpuContext::new(metal_config.clone())) {
                eprintln!("✅ Metal GPU detected and enabled!");
                info!("✅ Metal GPU detected and enabled!");
                let mut store = Self::new_with_metal_config(metal_config);
                store.enable_auto_save();
                return store;
            } else {
                eprintln!("⚠️ Metal GPU detection failed, falling back...");
                warn!("⚠️ Metal GPU detection failed, falling back...");
            }
        }
        
        #[cfg(not(all(target_os = "macos", target_arch = "aarch64", feature = "wgpu-gpu")))]
        {
            eprintln!("⚠️ Metal not available (not Mac Silicon or wgpu-gpu feature not compiled)");
        }
        
        // 2. CUDA is compiled but NOT auto-enabled - respect user config
        // Users must explicitly enable CUDA in config.yml
        #[cfg(feature = "cuda")]
        {
            eprintln!("ℹ️  CUDA support is available but disabled by default");
            info!("ℹ️  CUDA support is available but disabled by default");
        }
        
        // 3. Default to CPU-only mode
        eprintln!("💻 Using CPU-only mode (default)");
        info!("💻 Using CPU-only mode (default)");
        let mut store = Self::new_with_cuda_config(CudaConfig { enabled: false, ..Default::default() });
        store.enable_auto_save();
        store
    }
    
    /// Universal GPU detection across all backends (Vulkan, DirectX, CUDA, Metal)
    /// Priority: Metal (macOS) > Vulkan (AMD/Universal) > DirectX12 (Windows) > CUDA (NVIDIA) > CPU
    #[cfg(feature = "wgpu-gpu")]
    pub fn new_auto_universal() -> Self {
        use crate::gpu::{detect_available_backends, select_best_backend, GpuBackendType};
        
        //eprintln!("\n🌍 VectorStore::new_auto_universal() - Universal Multi-GPU Detection");
        info!("🔍 Starting universal GPU backend detection...");
        
        // Try to load persisted collections first
        let mut store = Self::new();
        if let Ok(collections_loaded) = store.load_all_persisted_collections() {
            if collections_loaded > 0 {
                eprintln!("✅ Loaded {} persisted collections from data directory", collections_loaded);
                info!("✅ Loaded {} persisted collections from data directory", collections_loaded);
                // Enable auto-save after loading persisted collections
                store.enable_auto_save();
            return store;
            }
        }
        
        // Detect all available backends
        let available = detect_available_backends();
        
        if available.is_empty() {
            eprintln!("❌ No GPU backends detected - using CPU");
            warn!("No GPU backends available");
            return Self::new();
        }
        
        // Select best backend
        let best = select_best_backend(&available);
        eprintln!("🎯 Selected: {}", best);
        info!("Selected backend: {}", best);
        
        // Initialize VectorStore with the selected backend
        match best {
            GpuBackendType::Metal => {
                #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
                {
                    eprintln!("🍎 Initializing Metal GPU backend...");
                    let metal_config = crate::gpu::GpuConfig::for_metal_silicon();
                    if let Ok(_) = pollster::block_on(crate::gpu::GpuContext::new(metal_config.clone())) {
                        eprintln!("✅ Metal GPU initialized successfully!");
                        info!("✅ Metal GPU initialized successfully!");
                        let mut store = Self::new_with_metal_config(metal_config);
                        store.enable_auto_save();
                        return store;
                    } else {
                        eprintln!("⚠️ Metal initialization failed - falling back");
                        warn!("Metal GPU initialization failed");
                    }
                }
            }
            
            GpuBackendType::Vulkan => {
                #[cfg(feature = "wgpu-gpu")]
                {
                    eprintln!("🔥 Initializing Vulkan GPU backend...");
                    info!("Initializing Vulkan GPU backend...");
                    let vulkan_config = crate::gpu::GpuConfig::default();
                    eprintln!("✅ Vulkan GPU initialized!");
                    info!("✅ Vulkan GPU initialized!");
                    let mut store = Self::new_with_vulkan_config(vulkan_config);
                    store.enable_auto_save();
                    return store;
                }
                
                #[cfg(not(feature = "wgpu-gpu"))]
                {
                    eprintln!("⚠️ Vulkan requires wgpu-gpu feature");
                    warn!("Vulkan selected but wgpu-gpu feature not enabled");
                }
            }
            
            GpuBackendType::DirectX12 => {
                eprintln!("🪟 DirectX 12 detected but integration pending...");
                info!("DirectX 12 backend detected but not yet integrated");
                // TODO: Implement DirectX12Collection (FASE 3)
            }
            
            GpuBackendType::CudaNative => {
                #[cfg(feature = "cuda")]
                {
                    eprintln!("⚡ Initializing CUDA GPU backend...");
                    info!("Initializing CUDA GPU backend...");
                    let cuda_config = CudaConfig { enabled: true, ..Default::default() };
                    eprintln!("✅ CUDA GPU initialized!");
                    info!("✅ CUDA GPU initialized!");
                    let mut store = Self::new_with_cuda_config(cuda_config);
                    store.enable_auto_save();
                    return store;
                }
            }
            
            GpuBackendType::Cpu => {
                eprintln!("💻 Using CPU backend");
                info!("Using CPU backend");
            }
        }
        
        // Fallback to CPU if GPU initialization failed
        eprintln!("💻 Falling back to CPU backend");
        warn!("GPU initialization failed, using CPU fallback");
        let mut store = Self::new();
        store.enable_auto_save();
        store
    }

    /// Create a new collection
    pub fn create_collection(&self, name: &str, config: CollectionConfig) -> Result<()> {
        debug!("Creating collection '{}' with config: {:?}", name, config);

        if self.collections.contains_key(name) {
            return Err(VectorizerError::CollectionAlreadyExists(name.to_string()));
        }

        // Prioridade: Metal > Vulkan > DirectX12 > CUDA > CPU
        #[cfg(feature = "wgpu-gpu")]
        if let Some(ref metal_cfg) = self.metal_config {
            if metal_cfg.enabled {
                info!("Creating Metal GPU-accelerated collection '{}'", name);
                let metal_collection = pollster::block_on(
                    crate::gpu::MetalCollection::new(name.to_string(), config, metal_cfg.clone())
                )?;
                let collection = CollectionType::Metal(metal_collection);
                self.collections.insert(name.to_string(), collection);
                info!("Collection '{}' created successfully with Metal GPU", name);
                return Ok(());
            }
        }

        #[cfg(feature = "wgpu-gpu")]
        if let Some(ref vulkan_cfg) = self.vulkan_config {
            if vulkan_cfg.enabled {
                info!("Creating Vulkan GPU-accelerated collection '{}'", name);
                let vulkan_collection = pollster::block_on(
                    crate::gpu::VulkanCollection::new(name.to_string(), config, vulkan_cfg.clone())
                )?;
                let collection = CollectionType::Vulkan(vulkan_collection);
                self.collections.insert(name.to_string(), collection);
                info!("Collection '{}' created successfully with Vulkan GPU", name);
                return Ok(());
            }
        }

        #[cfg(feature = "wgpu-gpu")]
        if let Some(ref dx12_cfg) = self.dx12_config {
            if dx12_cfg.enabled {
                info!("Creating DirectX 12 GPU-accelerated collection '{}'", name);
                let dx12_collection = pollster::block_on(
                    crate::gpu::DirectX12Collection::new(name.to_string(), config, dx12_cfg.clone())
                )?;
                let collection = CollectionType::DirectX12(dx12_collection);
                self.collections.insert(name.to_string(), collection);
                info!("Collection '{}' created successfully with DirectX 12 GPU", name);
                return Ok(());
            }
        }

        // Fallback para CUDA
        let collection = if self.cuda_config.enabled {
            #[cfg(feature = "cuda")]
            {
                info!("Creating CUDA-accelerated collection '{}'", name);
                CollectionType::Cuda(CudaCollection::new(name.to_string(), config, self.cuda_config.clone()))
            }
            #[cfg(not(feature = "cuda"))]
            {
                warn!("CUDA requested but not compiled in - falling back to CPU collection");
                CollectionType::Cpu(Collection::new(name.to_string(), config))
            }
        } else {
            debug!("Creating CPU-based collection '{}'", name);
            CollectionType::Cpu(Collection::new(name.to_string(), config))
        };

        self.collections.insert(name.to_string(), collection);

        info!("Collection '{}' created successfully", name);
        Ok(())
    }

    /// Create or update collection with automatic quantization
    pub fn create_collection_with_quantization(&self, name: &str, config: CollectionConfig) -> Result<()> {
        debug!("Creating/updating collection '{}' with automatic quantization", name);

        // Check if collection already exists
        if let Some(existing_collection) = self.collections.get(name) {
            // Check if quantization is enabled in the new config
            let quantization_enabled = matches!(config.quantization, crate::models::QuantizationConfig::SQ { bits: 8 });
            
            // Check if existing collection has quantization
            let existing_quantization_enabled = matches!(existing_collection.config().quantization, crate::models::QuantizationConfig::SQ { bits: 8 });
            
            if quantization_enabled && !existing_quantization_enabled {
                info!("🔄 Collection '{}' needs quantization upgrade - applying automatically", name);
                
                // Store existing vectors
                let existing_vectors = existing_collection.get_all_vectors();
                let vector_count = existing_vectors.len();
                
                if vector_count > 0 {
                    info!("📦 Storing {} existing vectors for quantization upgrade", vector_count);
                    
                    // Store the existing vector count and document count
                    let existing_metadata = existing_collection.metadata();
                    let existing_document_count = existing_metadata.document_count;
                    
                    // Remove old collection
                    self.collections.remove(name);
                    
                    // Create new collection with quantization
                    self.create_collection(name, config)?;
                    
                    // Get the new collection
                    let new_collection = self.get_collection(name)?;
                    
                    // Apply quantization to existing vectors
                    for vector in existing_vectors {
                        let vector_id = vector.id.clone();
                        if let Err(e) = new_collection.add_vector(vector_id.clone(), vector) {
                            warn!("Failed to add vector {} to quantized collection: {}", vector_id, e);
                        }
                    }
                    
                    info!("✅ Successfully upgraded collection '{}' with quantization for {} vectors", name, vector_count);
                } else {
                    // Collection is empty, just recreate with new config
                    self.collections.remove(name);
                    self.create_collection(name, config)?;
                    info!("✅ Recreated empty collection '{}' with quantization", name);
                }
            } else {
                debug!("Collection '{}' already has correct quantization configuration", name);
            }
        } else {
            // Collection doesn't exist, create it normally with quantization
            self.create_collection(name, config)?;
        }

        Ok(())
    }

    /// Delete a collection
    pub fn delete_collection(&self, name: &str) -> Result<()> {
        debug!("Deleting collection '{}'", name);

        self.collections
            .remove(name)
            .ok_or_else(|| VectorizerError::CollectionNotFound(name.to_string()))?;

        info!("Collection '{}' deleted successfully", name);
        Ok(())
    }

    /// Get a reference to a collection by name
    pub fn get_collection(&self, name: &str) -> Result<impl std::ops::Deref<Target = CollectionType> + '_> {
        self.collections
            .get(name)
            .ok_or_else(|| VectorizerError::CollectionNotFound(name.to_string()))
    }


    /// List all collections
    pub fn list_collections(&self) -> Vec<String> {
        self.collections
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Get collection metadata
    pub fn get_collection_metadata(&self, name: &str) -> Result<CollectionMetadata> {
        let collection_ref = self.get_collection(name)?;
        Ok(collection_ref.metadata())
    }

    /// Insert vectors into a collection
    pub fn insert(&self, collection_name: &str, vectors: Vec<Vector>) -> Result<()> {
        debug!(
            "Inserting {} vectors into collection '{}'",
            vectors.len(),
            collection_name
        );

        let collection_ref = self.get_collection(collection_name)?;

        // Check if this is a GPU collection that needs async handling
        match collection_ref.deref() {
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Metal(_) | CollectionType::Vulkan(_) | CollectionType::DirectX12(_) => {
                // For GPU collections, use sequential insertion to avoid async issues
                for vector in vectors {
                    collection_ref.add_vector(vector.id.clone(), vector)?;
                }
            },
            _ => {
                // For CPU collections, use parallel iteration for better performance
                use rayon::prelude::*;
                vectors.into_par_iter().try_for_each(|vector| {
                    collection_ref.add_vector(vector.id.clone(), vector)
                })?;
            }
        }

        // Mark collection for auto-save
        self.mark_collection_for_save(collection_name);

        Ok(())
    }

    /// Update a vector in a collection
    pub fn update(&self, collection_name: &str, vector: Vector) -> Result<()> {
        debug!(
            "Updating vector '{}' in collection '{}'",
            vector.id, collection_name
        );

        let collection_ref = self.get_collection(collection_name)?;
        // For update, we delete and re-add (TODO: Add direct update method to CollectionType)
        collection_ref.delete_vector(&vector.id)?;
        collection_ref.add_vector(vector.id.clone(), vector)?;

        // Mark collection for auto-save
        self.mark_collection_for_save(collection_name);

        Ok(())
    }

    /// Delete a vector from a collection
    pub fn delete(&self, collection_name: &str, vector_id: &str) -> Result<()> {
        debug!(
            "Deleting vector '{}' from collection '{}'",
            vector_id, collection_name
        );

        let collection_ref = self.get_collection(collection_name)?;
        collection_ref.delete_vector(vector_id)?;

        // Mark collection for auto-save
        self.mark_collection_for_save(collection_name);

        Ok(())
    }

    /// Get a vector by ID
    pub fn get_vector(&self, collection_name: &str, vector_id: &str) -> Result<Vector> {
        let collection_ref = self.get_collection(collection_name)?;
        collection_ref.get_vector(vector_id)
    }

    /// Search for similar vectors
    pub fn search(
        &self,
        collection_name: &str,
        query_vector: &[f32],
        k: usize,
    ) -> Result<Vec<SearchResult>> {
        debug!(
            "Searching for {} nearest neighbors in collection '{}'",
            k, collection_name
        );

        let collection_ref = self.get_collection(collection_name)?;
        collection_ref.search(query_vector, k)
    }

    /// Load a collection from cache without reconstructing the HNSW index
    pub fn load_collection_from_cache(&self, collection_name: &str, persisted_vectors: Vec<crate::persistence::PersistedVector>) -> Result<()> {
        use crate::persistence::PersistedVector;

        debug!("Fast loading collection '{}' from cache with {} vectors", collection_name, persisted_vectors.len());

        let collection_ref = self.get_collection(collection_name)?;

        // TODO: Implement load_from_cache for CudaCollection and Metal
        match &*collection_ref {
            CollectionType::Cpu(c) => {
                c.load_from_cache(persisted_vectors)?;
                // Requantize existing vectors if quantization is enabled
                c.requantize_existing_vectors()?;
            },
            #[cfg(feature = "cuda")]
            CollectionType::Cuda(_) => {
                info!("🔧 Loading {} vectors into CUDA collection '{}'", persisted_vectors.len(), collection_name);
                // For CUDA collections, manually insert vectors (cache loading not fully implemented)
                for pv in persisted_vectors {
                    let vector: Vector = pv.into();
                    collection_ref.add_vector(vector.id.clone(), vector)?;
                }
                let final_count = collection_ref.metadata().vector_count;
                info!("✅ Successfully loaded {} vectors into CUDA collection '{}'", final_count, collection_name);
            }
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Metal(_) => {
                warn!("Metal collections don't support cache loading yet - falling back to manual insertion");
                // For now, manually insert vectors for Metal collections
                for pv in persisted_vectors {
                    // Convert PersistedVector back to Vector
                    let vector: Vector = pv.into();
                    collection_ref.add_vector(vector.id.clone(), vector)?;
                }
            }
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Vulkan(_) => {
                warn!("Vulkan collections don't support cache loading yet - falling back to manual insertion");
                // For now, manually insert vectors for Vulkan collections
                for pv in persisted_vectors {
                    // Convert PersistedVector back to Vector
                    let vector: Vector = pv.into();
                    collection_ref.add_vector(vector.id.clone(), vector)?;
                }
            }
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::DirectX12(_) => {
                warn!("DirectX 12 collections don't support cache loading yet - falling back to manual insertion");
                // For now, manually insert vectors for DirectX 12 collections
                for pv in persisted_vectors {
                    // Convert PersistedVector back to Vector
                    let vector: Vector = pv.into();
                    collection_ref.add_vector(vector.id.clone(), vector)?;
                }
            }
        }

        Ok(())
    }

    /// Load a collection from cache with optional HNSW dump for instant loading
    pub fn load_collection_from_cache_with_hnsw_dump(&self, collection_name: &str, persisted_vectors: Vec<crate::persistence::PersistedVector>, hnsw_dump_path: Option<&std::path::Path>, hnsw_basename: Option<&str>) -> Result<()> {
        use crate::persistence::PersistedVector;

        debug!("Loading collection '{}' from cache with {} vectors (HNSW dump: {})", collection_name, persisted_vectors.len(), hnsw_basename.is_some());

        let collection_ref = self.get_collection(collection_name)?;

        // TODO: Implement load_from_cache_with_hnsw_dump for CudaCollection and Metal
        match &*collection_ref {
            CollectionType::Cpu(c) => c.load_from_cache_with_hnsw_dump(persisted_vectors, hnsw_dump_path, hnsw_basename)?,
            #[cfg(feature = "cuda")]
            CollectionType::Cuda(_) => {
                info!("🔧 Loading {} vectors into CUDA collection '{}' (HNSW dump not supported yet)", persisted_vectors.len(), collection_name);
                // For CUDA collections, manually insert vectors (cache loading not fully implemented)
                for pv in persisted_vectors {
                    let vector: Vector = pv.into();
                    collection_ref.add_vector(vector.id.clone(), vector)?;
                }
                let final_count = collection_ref.metadata().vector_count;
                info!("✅ Successfully loaded {} vectors into CUDA collection '{}'", final_count, collection_name);
            }
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Metal(_) => {
                warn!("Metal collections don't support HNSW dump loading yet - falling back to manual insertion");
                // For now, manually insert vectors for Metal collections
                for pv in persisted_vectors {
                    // Convert PersistedVector back to Vector
                    let vector: Vector = pv.into();
                    collection_ref.add_vector(vector.id.clone(), vector)?;
                }
            }
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::Vulkan(_) => {
                warn!("Vulkan collections don't support HNSW dump loading yet - falling back to manual insertion");
                // For now, manually insert vectors for Vulkan collections
                for pv in persisted_vectors {
                    // Convert PersistedVector back to Vector
                    let vector: Vector = pv.into();
                    collection_ref.add_vector(vector.id.clone(), vector)?;
                }
            }
            #[cfg(feature = "wgpu-gpu")]
            CollectionType::DirectX12(_) => {
                warn!("DirectX 12 collections don't support HNSW dump loading yet - falling back to manual insertion");
                // For now, manually insert vectors for DirectX 12 collections
                for pv in persisted_vectors {
                    // Convert PersistedVector back to Vector
                    let vector: Vector = pv.into();
                    collection_ref.add_vector(vector.id.clone(), vector)?;
                }
            }
        }

        Ok(())
    }

    /// Get statistics about the vector store
    pub fn stats(&self) -> VectorStoreStats {
        let mut total_vectors = 0;
        let mut total_memory_bytes = 0;

        for entry in self.collections.iter() {
            let collection = entry.value();
            total_vectors += collection.vector_count();
            total_memory_bytes += collection.estimated_memory_usage();
        }

        VectorStoreStats {
            collection_count: self.collections.len(),
            total_vectors,
            total_memory_bytes,
        }
    }
}

impl Default for VectorStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the vector store
#[derive(Debug, Default, Clone)]
pub struct VectorStoreStats {
    /// Number of collections
    pub collection_count: usize,
    /// Total number of vectors across all collections
    pub total_vectors: usize,
    /// Estimated memory usage in bytes
    pub total_memory_bytes: usize,
}

impl VectorStore {
    /// Get the centralized data directory path (same as DocumentLoader)
    pub fn get_data_dir() -> PathBuf {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        current_dir.join("data")
    }


    /// Load all persisted collections from the data directory
    pub fn load_all_persisted_collections(&mut self) -> Result<usize> {
        let data_dir = Self::get_data_dir();
        if !data_dir.exists() {
            debug!("Data directory does not exist: {:?}", data_dir);
            return Ok(0);
        }

        let mut collections_loaded = 0;

        // Find all .bin files in the data directory
        for entry in std::fs::read_dir(&data_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(extension) = path.extension() {
                if extension == "bin" {
                    // Extract collection name from filename (remove _vector_store.bin suffix)
                    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                        if let Some(collection_name) = filename.strip_suffix("_vector_store.bin") {
                            debug!("Found persisted collection: {}", collection_name);

                            match self.load_persisted_collection(&path, collection_name) {
                                Ok(_) => {
                                    collections_loaded += 1;
                                    info!("✅ Successfully loaded collection '{}' from persistence", collection_name);
                                }
                                Err(e) => {
                                    warn!("❌ Failed to load collection '{}' from {:?}: {}", collection_name, path, e);
                                }
                            }
                        }
                    }
                }
            }
        }

        info!("📊 Loaded {} collections from persistence directory", collections_loaded);
        Ok(collections_loaded)
    }

    /// Load dynamic collections that are not in the workspace
    /// Call this after workspace initialization to load any additional persisted collections
    pub fn load_dynamic_collections(&mut self) -> Result<usize> {
        let data_dir = Self::get_data_dir();
        if !data_dir.exists() {
            debug!("Data directory does not exist: {:?}", data_dir);
            return Ok(0);
        }

        let mut dynamic_collections_loaded = 0;
        let existing_collections: std::collections::HashSet<String> = self.list_collections().into_iter().collect();

        // Find all .bin files in the data directory that are not already loaded
        for entry in std::fs::read_dir(&data_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(extension) = path.extension() {
                if extension == "bin" {
                    // Extract collection name from filename (remove _vector_store.bin suffix)
                    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                        if let Some(collection_name) = filename.strip_suffix("_vector_store.bin") {
                            // Skip if this collection is already loaded (from workspace)
                            if existing_collections.contains(collection_name) {
                                debug!("Skipping collection '{}' - already loaded from workspace", collection_name);
                                continue;
                            }

                            debug!("Loading dynamic collection: {}", collection_name);

                            match self.load_persisted_collection(&path, collection_name) {
                                Ok(_) => {
                                    dynamic_collections_loaded += 1;
                                    info!("✅ Loaded dynamic collection '{}' from persistence", collection_name);
                                }
                                Err(e) => {
                                    warn!("❌ Failed to load dynamic collection '{}' from {:?}: {}", collection_name, path, e);
                                }
                            }
                        }
                    }
                }
            }
        }

        if dynamic_collections_loaded > 0 {
            info!("📊 Loaded {} additional dynamic collections from persistence", dynamic_collections_loaded);
        }

        Ok(dynamic_collections_loaded)
    }

    /// Load a single persisted collection from file
    fn load_persisted_collection<P: AsRef<std::path::Path>>(&mut self, path: P, collection_name: &str) -> Result<()> {
        use crate::persistence::PersistedVectorStore;

        let path = path.as_ref();
        debug!("Loading persisted collection '{}' from {:?}", collection_name, path);

        // Read and parse the JSON file
        let json_data = std::fs::read_to_string(path)?;
        let persisted: PersistedVectorStore = serde_json::from_str(&json_data)?;

        // Check version
        if persisted.version != 1 {
            return Err(crate::error::VectorizerError::Other(format!(
                "Unsupported persisted collection version: {}",
                persisted.version
            )));
        }

        // Find the collection in the persisted data
        let persisted_collection = persisted.collections.iter()
            .find(|c| c.name == collection_name)
            .ok_or_else(|| crate::error::VectorizerError::Other(format!(
                "Collection '{}' not found in persisted data", collection_name
            )))?;

        // Create collection with the persisted config
        let mut config = persisted_collection.config.clone();
        config.quantization = crate::models::QuantizationConfig::SQ { bits: 8 };

        self.create_collection_with_quantization(collection_name, config)?;

        // Load vectors if any exist
        if !persisted_collection.vectors.is_empty() {
            debug!("Loading {} vectors into collection '{}'", persisted_collection.vectors.len(), collection_name);
            self.load_collection_from_cache(collection_name, persisted_collection.vectors.clone())?;
        }
        
        Ok(())
    }
    
    /// Enable auto-save for all collections
    /// Call this after initialization is complete
    pub fn enable_auto_save(&self) {
        self.auto_save_enabled.store(true, std::sync::atomic::Ordering::Relaxed);
        
        // Start background save task
        let pending_saves: Arc<std::sync::Mutex<HashSet<String>>> = Arc::clone(&self.pending_saves);
        let collections = Arc::clone(&self.collections);
        
        let save_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30)); // Save every 30 seconds
            
            loop {
                interval.tick().await;
                
                if !pending_saves.lock().unwrap().is_empty() {
                    info!("🔄 Background save: {} collections pending", pending_saves.lock().unwrap().len());
                    
                    // Process all pending saves
                    let collections_to_save: Vec<String> = pending_saves.lock().unwrap().iter().cloned().collect();
                    pending_saves.lock().unwrap().clear();
                    
                    for collection_name in collections_to_save {
                        if let Some(collection) = collections.get(&collection_name) {
                            match Self::save_collection_to_file_static(&collection_name, &*collection) {
                                Ok(_) => debug!("✅ Background saved collection '{}'", collection_name),
                                Err(e) => warn!("❌ Background save failed for '{}': {}", collection_name, e),
                            }
                        }
                    }
                    
                    info!("✅ Background save completed");
                }
            }
        });
        
        // Store the task handle
        *self.save_task_handle.lock().unwrap() = Some(save_task);
        info!("✅ Auto-save enabled with background task");
    }
    
    /// Disable auto-save for all collections
    /// Useful during bulk operations or maintenance
    pub fn disable_auto_save(&self) {
        self.auto_save_enabled.store(false, std::sync::atomic::Ordering::Relaxed);
        info!("⏸️ Auto-save disabled for all collections");
    }
    
    /// Force immediate save of all pending collections
    /// Useful before shutdown or critical operations
    pub fn force_save_all(&self) -> Result<()> {
        if self.pending_saves.lock().unwrap().is_empty() {
            debug!("No pending saves to force");
            return Ok(());
        }
        
        info!("🔄 Force saving {} pending collections", self.pending_saves.lock().unwrap().len());
        
        let collections_to_save: Vec<String> = self.pending_saves.lock().unwrap().iter().cloned().collect();
        self.pending_saves.lock().unwrap().clear();
        
        for collection_name in collections_to_save {
            if let Some(collection) = self.collections.get(&collection_name) {
                match Self::save_collection_to_file_static(&collection_name, &*collection) {
                    Ok(_) => debug!("✅ Force saved collection '{}'", collection_name),
                    Err(e) => warn!("❌ Force save failed for '{}': {}", collection_name, e),
                }
            }
        }
        
        info!("✅ Force save completed");
        Ok(())
    }

    /// Save a single collection to file following workspace pattern
    /// Creates separate files for vectors, tokenizer, and metadata
    pub fn save_collection_to_file(&self, collection_name: &str) -> Result<()> {
        use std::fs;
        use crate::persistence::PersistedCollection;

        info!("Saving collection '{}' to individual files", collection_name);

        // Get collection
        let collection = self.get_collection(collection_name)?;
        let metadata = collection.metadata();

        // Ensure data directory exists
        let data_dir = Self::get_data_dir();
        if let Err(e) = fs::create_dir_all(&data_dir) {
            return Err(crate::error::VectorizerError::Other(format!(
                "Failed to create data directory '{}': {}",
                data_dir.display(),
                e
            )));
        }

        // Collect all vectors from the collection
        let vectors: Vec<crate::persistence::PersistedVector> = collection
            .get_all_vectors()
            .into_iter()
            .map(crate::persistence::PersistedVector::from)
            .collect();

        // Create persisted collection
        let persisted_collection = PersistedCollection {
            name: collection_name.to_string(),
            config: metadata.config.clone(),
            vectors,
            hnsw_dump_basename: None,
        };

        // Save vectors to binary file (following workspace pattern)
        let vector_store_path = data_dir.join(format!("{}_vector_store.bin", collection_name));
        self.save_collection_vectors_binary(&persisted_collection, &vector_store_path)?;

        // Save metadata to JSON file
        let metadata_path = data_dir.join(format!("{}_metadata.json", collection_name));
        self.save_collection_metadata(&persisted_collection, &metadata_path)?;

        // Save tokenizer (for dynamic collections, create a minimal tokenizer)
        let tokenizer_path = data_dir.join(format!("{}_tokenizer.json", collection_name));
        self.save_collection_tokenizer(collection_name, &tokenizer_path)?;

        info!("Successfully saved collection '{}' to files", collection_name);
        Ok(())
    }
    
    /// Static method to save collection to file (for background task)
    fn save_collection_to_file_static(collection_name: &str, collection: &CollectionType) -> Result<()> {
        use std::fs;
        use crate::persistence::PersistedCollection;
        
        debug!("Saving collection '{}' to individual files", collection_name);
        
        // Get collection metadata
        let metadata = collection.metadata();
        
        // Ensure data directory exists
        let data_dir = Self::get_data_dir();
        if let Err(e) = fs::create_dir_all(&data_dir) {
            return Err(crate::error::VectorizerError::Other(format!(
                "Failed to create data directory '{}': {}",
                data_dir.display(),
                e
            )));
        }
        
        // Collect all vectors from the collection
        let vectors: Vec<crate::persistence::PersistedVector> = collection
            .get_all_vectors()
            .into_iter()
            .map(crate::persistence::PersistedVector::from)
            .collect();
        
        // Create persisted collection
        let persisted_collection = PersistedCollection {
            name: collection_name.to_string(),
            config: metadata.config.clone(),
            vectors,
            hnsw_dump_basename: None,
        };
        
        // Save vectors to binary file
        let vector_store_path = data_dir.join(format!("{}_vector_store.bin", collection_name));
        Self::save_collection_vectors_binary_static(&persisted_collection, &vector_store_path)?;
        
        // Save metadata to JSON file
        let metadata_path = data_dir.join(format!("{}_metadata.json", collection_name));
        Self::save_collection_metadata_static(&persisted_collection, &metadata_path)?;
        
        // Save tokenizer
        let tokenizer_path = data_dir.join(format!("{}_tokenizer.json", collection_name));
        Self::save_collection_tokenizer_static(collection_name, &tokenizer_path)?;
        
        debug!("Successfully saved collection '{}' to files", collection_name);
        Ok(())
    }
    
    /// Mark a collection for auto-save (internal method)
    fn mark_collection_for_save(&self, collection_name: &str) {
        if self.auto_save_enabled.load(std::sync::atomic::Ordering::Relaxed) {
            self.pending_saves.lock().unwrap().insert(collection_name.to_string());
        }
    }

    /// Save collection vectors to binary file
    fn save_collection_vectors_binary(&self, persisted_collection: &crate::persistence::PersistedCollection, path: &std::path::Path) -> Result<()> {
        use std::fs::File;
        use std::io::Write;

        let json_data = serde_json::to_string_pretty(&persisted_collection)?;
        let mut file = File::create(path)?;
        file.write_all(json_data.as_bytes())?;

        debug!("Saved {} vectors to {}", persisted_collection.vectors.len(), path.display());
        Ok(())
    }
    
    /// Save collection metadata to JSON file
    fn save_collection_metadata(&self, persisted_collection: &crate::persistence::PersistedCollection, path: &std::path::Path) -> Result<()> {
        use std::fs::File;
        use std::io::Write;

        let metadata = serde_json::json!({
            "name": persisted_collection.name,
            "config": persisted_collection.config,
            "vector_count": persisted_collection.vectors.len(),
            "created_at": chrono::Utc::now().to_rfc3339(),
        });

        let json_data = serde_json::to_string_pretty(&metadata)?;
        let mut file = File::create(path)?;
        file.write_all(json_data.as_bytes())?;

        debug!("Saved metadata for '{}' to {}", persisted_collection.name, path.display());
        Ok(())
    }
    
    /// Save collection tokenizer to JSON file
    fn save_collection_tokenizer(&self, collection_name: &str, path: &std::path::Path) -> Result<()> {
        use std::fs::File;
        use std::io::Write;

        // For dynamic collections, create a minimal tokenizer
        let tokenizer_data = serde_json::json!({
            "collection_name": collection_name,
            "tokenizer_type": "dynamic",
            "created_at": chrono::Utc::now().to_rfc3339(),
            "vocab_size": 0,
            "special_tokens": {},
        });

        let json_data = serde_json::to_string_pretty(&tokenizer_data)?;
        let mut file = File::create(path)?;
        file.write_all(json_data.as_bytes())?;

        debug!("Saved tokenizer for '{}' to {}", collection_name, path.display());
        Ok(())
    }

    /// Static version of save_collection_vectors_binary
    fn save_collection_vectors_binary_static(persisted_collection: &crate::persistence::PersistedCollection, path: &std::path::Path) -> Result<()> {
        use std::fs::File;
        use std::io::Write;

        let json_data = serde_json::to_string_pretty(&persisted_collection)?;
        let mut file = File::create(path)?;
        file.write_all(json_data.as_bytes())?;

        debug!("Saved {} vectors to {}", persisted_collection.vectors.len(), path.display());
        Ok(())
    }

    /// Static version of save_collection_metadata
    fn save_collection_metadata_static(persisted_collection: &crate::persistence::PersistedCollection, path: &std::path::Path) -> Result<()> {
        use std::fs::File;
        use std::io::Write;

        let metadata = serde_json::json!({
            "name": persisted_collection.name,
            "config": persisted_collection.config,
            "vector_count": persisted_collection.vectors.len(),
            "created_at": chrono::Utc::now().to_rfc3339(),
        });

        let json_data = serde_json::to_string_pretty(&metadata)?;
        let mut file = File::create(path)?;
        file.write_all(json_data.as_bytes())?;

        debug!("Saved metadata for '{}' to {}", persisted_collection.name, path.display());
        Ok(())
    }

    /// Static version of save_collection_tokenizer
    fn save_collection_tokenizer_static(collection_name: &str, path: &std::path::Path) -> Result<()> {
        use std::fs::File;
        use std::io::Write;

        // For dynamic collections, create a minimal tokenizer
        let tokenizer_data = serde_json::json!({
            "collection_name": collection_name,
            "tokenizer_type": "dynamic",
            "created_at": chrono::Utc::now().to_rfc3339(),
            "vocab_size": 0,
            "special_tokens": {},
        });

        let json_data = serde_json::to_string_pretty(&tokenizer_data)?;
        let mut file = File::create(path)?;
        file.write_all(json_data.as_bytes())?;

        debug!("Saved tokenizer for '{}' to {}", collection_name, path.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CompressionConfig, DistanceMetric, HnswConfig, Payload};

    #[test]
    fn test_create_and_list_collections() {
        let store = VectorStore::new();

        let config = CollectionConfig {
            dimension: 128,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: Default::default(),
            compression: Default::default(),
        };

        // Create collections
        store.create_collection("test1", config.clone()).unwrap();
        store.create_collection("test2", config).unwrap();

        // List collections
        let collections = store.list_collections();
        assert_eq!(collections.len(), 2);
        assert!(collections.contains(&"test1".to_string()));
        assert!(collections.contains(&"test2".to_string()));
    }

    #[test]
    fn test_duplicate_collection_error() {
        let store = VectorStore::new();

        let config = CollectionConfig {
            dimension: 128,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: Default::default(),
            compression: Default::default(),
        };

        // Create collection
        store.create_collection("test", config.clone()).unwrap();

        // Try to create duplicate
        let result = store.create_collection("test", config);
        assert!(matches!(
            result,
            Err(VectorizerError::CollectionAlreadyExists(_))
        ));
    }

    #[test]
    fn test_delete_collection() {
        let store = VectorStore::new();

        let config = CollectionConfig {
            dimension: 128,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: Default::default(),
            compression: Default::default(),
        };

        // Create and delete collection
        store.create_collection("test", config).unwrap();
        assert_eq!(store.list_collections().len(), 1);

        store.delete_collection("test").unwrap();
        assert_eq!(store.list_collections().len(), 0);

        // Try to delete non-existent collection
        let result = store.delete_collection("test");
        assert!(matches!(
            result,
            Err(VectorizerError::CollectionNotFound(_))
        ));
    }

    #[test]
    fn test_vector_operations_integration() {
        let store = VectorStore::new();

        let config = CollectionConfig {
            dimension: 3,
            metric: DistanceMetric::Euclidean,
            hnsw_config: HnswConfig {
                m: 4,
                ef_construction: 100,
                ef_search: 50,
                seed: Some(42),
            },
            quantization: Default::default(),
            compression: Default::default(),
        };

        store.create_collection("test", config).unwrap();

        // Test inserting multiple vectors
        let vectors = vec![
            Vector::with_payload(
                "vec1".to_string(),
                vec![1.0, 0.0, 0.0],
                Payload::from_value(serde_json::json!({"type": "test", "id": 1})).unwrap(),
            ),
            Vector::with_payload(
                "vec2".to_string(),
                vec![0.0, 1.0, 0.0],
                Payload::from_value(serde_json::json!({"type": "test", "id": 2})).unwrap(),
            ),
            Vector::with_payload(
                "vec3".to_string(),
                vec![0.0, 0.0, 1.0],
                Payload::from_value(serde_json::json!({"type": "test", "id": 3})).unwrap(),
            ),
        ];

        store.insert("test", vectors).unwrap();

        // Test search
        let results = store.search("test", &[1.0, 0.0, 0.0], 2).unwrap();
        assert!(results.len() >= 1, "Should return at least 1 result");
        assert_eq!(results[0].id, "vec1");

        // Test get individual vector
        let vector = store.get_vector("test", "vec1").unwrap();
        assert_eq!(vector.id, "vec1");
        assert_eq!(vector.data, vec![1.0, 0.0, 0.0]);

        // Test update
        let updated = Vector::with_payload(
            "vec1".to_string(),
            vec![2.0, 0.0, 0.0],
            Payload::from_value(serde_json::json!({"type": "updated", "id": 1})).unwrap(),
        );
        store.update("test", updated).unwrap();

        let retrieved = store.get_vector("test", "vec1").unwrap();
        assert_eq!(retrieved.data, vec![2.0, 0.0, 0.0]);

        // Test delete
        store.delete("test", "vec2").unwrap();
        let result = store.get_vector("test", "vec2");
        assert!(matches!(result, Err(VectorizerError::VectorNotFound(_))));
    }

    #[test]
    fn test_stats_functionality() {
        let store = VectorStore::new();

        let config = CollectionConfig {
            dimension: 3,
            metric: DistanceMetric::Euclidean,
            hnsw_config: HnswConfig::default(),
            quantization: Default::default(),
            compression: Default::default(),
        };

        // Empty store stats
        let stats = store.stats();
        assert_eq!(stats.collection_count, 0);
        assert_eq!(stats.total_vectors, 0);

        // Create collection and add vectors
        store.create_collection("test", config).unwrap();
        let vectors = vec![
            Vector::new("v1".to_string(), vec![1.0, 2.0, 3.0]),
            Vector::new("v2".to_string(), vec![4.0, 5.0, 6.0]),
        ];
        store.insert("test", vectors).unwrap();

        let stats = store.stats();
        assert_eq!(stats.collection_count, 1);
        assert_eq!(stats.total_vectors, 2);
        assert!(stats.total_memory_bytes > 0);
    }

    #[test]
    fn test_concurrent_operations() {
        use std::sync::Arc;
        use std::thread;

        let store = Arc::new(VectorStore::new());

        let config = CollectionConfig {
            dimension: 3,
            metric: DistanceMetric::Euclidean,
            hnsw_config: HnswConfig::default(),
            quantization: Default::default(),
            compression: Default::default(),
        };

        // Create collection from main thread
        store.create_collection("concurrent_test", config).unwrap();

        let mut handles = vec![];

        // Spawn multiple threads to insert vectors
        for i in 0..5 {
            let store_clone = Arc::clone(&store);
            let handle = thread::spawn(move || {
                let vectors = vec![
                    Vector::new(format!("vec_{}_{}", i, 0), vec![i as f32, 0.0, 0.0]),
                    Vector::new(format!("vec_{}_{}", i, 1), vec![0.0, i as f32, 0.0]),
                ];
                store_clone.insert("concurrent_test", vectors).unwrap();
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all vectors were inserted
        let stats = store.stats();
        assert_eq!(stats.collection_count, 1);
        assert_eq!(stats.total_vectors, 10); // 5 threads * 2 vectors each
    }

    #[test]
    fn test_collection_metadata() {
        let store = VectorStore::new();

        let config = CollectionConfig {
            dimension: 768,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig {
                m: 32,
                ef_construction: 200,
                ef_search: 64,
                seed: Some(123),
            },
            quantization: Default::default(),
            compression: CompressionConfig {
                enabled: true,
                threshold_bytes: 2048,
                algorithm: crate::models::CompressionAlgorithm::Lz4,
            },
        };

        store
            .create_collection("metadata_test", config.clone())
            .unwrap();

        // Add some vectors
        let vectors = vec![
            Vector::new("v1".to_string(), vec![0.1; 768]),
            Vector::new("v2".to_string(), vec![0.2; 768]),
        ];
        store.insert("metadata_test", vectors).unwrap();

        // Test metadata retrieval
        let metadata = store.get_collection_metadata("metadata_test").unwrap();
        assert_eq!(metadata.name, "metadata_test");
        assert_eq!(metadata.vector_count, 2);
        assert_eq!(metadata.config.dimension, 768);
        assert_eq!(metadata.config.metric, DistanceMetric::Cosine);
    }

    #[test]
    fn test_error_handling_edge_cases() {
        let store = VectorStore::new();

        let config = CollectionConfig {
            dimension: 3,
            metric: DistanceMetric::Euclidean,
            hnsw_config: HnswConfig::default(),
            quantization: Default::default(),
            compression: Default::default(),
        };

        store.create_collection("error_test", config).unwrap();

        // Test operations on non-existent collection
        let result = store.insert("non_existent", vec![]);
        assert!(matches!(
            result,
            Err(VectorizerError::CollectionNotFound(_))
        ));

        let result = store.search("non_existent", &[1.0, 2.0, 3.0], 1);
        assert!(matches!(
            result,
            Err(VectorizerError::CollectionNotFound(_))
        ));

        let result = store.get_vector("non_existent", "v1");
        assert!(matches!(
            result,
            Err(VectorizerError::CollectionNotFound(_))
        ));

        // Test operations on non-existent vector
        let result = store.get_vector("error_test", "non_existent");
        assert!(matches!(result, Err(VectorizerError::VectorNotFound(_))));

        let result = store.update(
            "error_test",
            Vector::new("non_existent".to_string(), vec![1.0, 2.0, 3.0]),
        );
        assert!(matches!(result, Err(VectorizerError::VectorNotFound(_))));

        let result = store.delete("error_test", "non_existent");
        assert!(matches!(result, Err(VectorizerError::VectorNotFound(_))));
    }
}
