//! Main VectorStore implementation

use crate::{
    error::{Result, VectorizerError},
    models::{CollectionConfig, CollectionMetadata, SearchResult, Vector},
    cuda::CudaConfig,
};
use dashmap::DashMap;
use std::sync::Arc;
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
        }
    }
    
    /// Create a new vector store with automatic GPU detection
    /// Priority: Metal (Mac Silicon) > CUDA > CPU
    pub fn new_auto() -> Self {
        // Always print for debug
        eprintln!("🔍 VectorStore::new_auto() called - starting GPU detection...");
        
        // 1. Try Metal first (Mac Silicon with wgpu-gpu feature)
        #[cfg(all(target_os = "macos", target_arch = "aarch64", feature = "wgpu-gpu"))]
        {
            eprintln!("🍎 Detecting Metal GPU on Mac Silicon...");
            info!("🍎 Detecting Metal GPU on Mac Silicon...");
            let metal_config = crate::gpu::GpuConfig::for_metal_silicon();
            if let Ok(_) = pollster::block_on(crate::gpu::GpuContext::new(metal_config.clone())) {
                eprintln!("✅ Metal GPU detected and enabled!");
                info!("✅ Metal GPU detected and enabled!");
                return Self::new_with_metal_config(metal_config);
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
        Self::new_with_cuda_config(CudaConfig { enabled: false, ..Default::default() })
    }
    
    /// Universal GPU detection across all backends (Vulkan, DirectX, CUDA, Metal)
    /// Priority: Metal (macOS) > Vulkan (AMD/Universal) > DirectX12 (Windows) > CUDA (NVIDIA) > CPU
    #[cfg(feature = "wgpu-gpu")]
    pub fn new_auto_universal() -> Self {
        use crate::gpu::{detect_available_backends, select_best_backend, GpuBackendType};
        
        eprintln!("\n🌍 VectorStore::new_auto_universal() - Universal Multi-GPU Detection");
        info!("🔍 Starting universal GPU backend detection...");
        
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
                        return Self::new_with_metal_config(metal_config);
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
                    return Self::new_with_vulkan_config(vulkan_config);
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
                    return Self::new_with_cuda_config(cuda_config);
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
        Self::new()
    }

    /// Create a new collection
    pub fn create_collection(&self, name: &str, config: CollectionConfig) -> Result<()> {
        debug!("Creating collection '{}' with config: {:?}", name, config);

        if self.collections.contains_key(name) {
            return Err(VectorizerError::CollectionAlreadyExists(name.to_string()));
        }

        // Prioridade: Metal > CUDA > CPU
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
            "Inserting {} vectors into collection '{}' (parallel)",
            vectors.len(),
            collection_name
        );

        let collection_ref = self.get_collection(collection_name)?;

        // Use parallel iteration for better performance
        use rayon::prelude::*;
        vectors.into_par_iter().try_for_each(|vector| {
            collection_ref.add_vector(vector.id.clone(), vector)
        })?;

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
