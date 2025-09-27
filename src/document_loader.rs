//! Document loader and processor for automatic indexing

use crate::{
    VectorStore,
    api::handlers::IndexingProgressState,
    cache::{
        CacheConfig, CacheError, CacheManager, CacheResult, IncrementalConfig, IncrementalProcessor,
    },
    embedding::{
        BagOfWordsEmbedding, BertEmbedding, Bm25Embedding, CharNGramEmbedding, EmbeddingManager,
        MiniLmEmbedding, SvdEmbedding, TfIdfEmbedding,
    },
    models::{CollectionConfig, DistanceMetric, HnswConfig, Payload, Vector},
    models::collection_metadata::{CollectionMetadataFile, FileMetadata, CollectionIndexingConfig, EmbeddingModelInfo},
    utils::file_hash::{calculate_file_hash, get_file_modified_time},
};
use anyhow::{Context, Result};
use glob::Pattern;
use rayon::prelude::*;
use sha2::Digest;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};
use tracing::{debug, error, info, warn};

/// Document chunk with metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DocumentChunk {
    /// Unique identifier for the chunk
    pub id: String,
    /// Text content of the chunk
    pub content: String,
    /// Source file path
    pub file_path: String,
    /// Chunk index within the document
    pub chunk_index: usize,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Cache entry for processed documents
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CacheEntry {
    /// File modification time when cached
    modified_time: SystemTime,
    /// File size when cached
    file_size: u64,
    /// Processed chunks
    chunks: Vec<DocumentChunk>,
}

/// Project cache metadata
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ProjectCache {
    /// Cache entries by file path
    files: HashMap<String, CacheEntry>,
    /// Configuration used for processing
    config_hash: u64,
}

/// Document loader configuration
#[derive(Debug, Clone)]
pub struct LoaderConfig {
    /// Maximum chunk size in characters
    pub max_chunk_size: usize,
    /// Overlap between chunks in characters
    pub chunk_overlap: usize,
    /// File extensions to process (legacy - use include_patterns instead)
    pub allowed_extensions: Vec<String>,
    /// Glob patterns for files to include
    pub include_patterns: Vec<String>,
    /// Glob patterns for files/directories to exclude
    pub exclude_patterns: Vec<String>,
    /// Embedding dimension
    pub embedding_dimension: usize,
    /// Embedding type to use
    pub embedding_type: String,
    /// Collection name for documents
    pub collection_name: String,
    /// Maximum file size in bytes (default 1MB)
    pub max_file_size: usize,
}

impl Default for LoaderConfig {
    fn default() -> Self {
        Self {
            max_chunk_size: 2048, // Chunks maiores para melhor contexto
            chunk_overlap: 256,   // Overlap maior para melhor continuidade
            allowed_extensions: vec![
                // Documentos
                "md".to_string(),
                "txt".to_string(),
                "json".to_string(),
                "yaml".to_string(),
                "yml".to_string(),
                "toml".to_string(),
                "xml".to_string(),
                "csv".to_string(),
                // Código
                "rs".to_string(),
                "js".to_string(),
                "ts".to_string(),
                "tsx".to_string(),
                "jsx".to_string(),
                "py".to_string(),
                "java".to_string(),
                "c".to_string(),
                "cpp".to_string(),
                "cc".to_string(),
                "cxx".to_string(),
                "h".to_string(),
                "hpp".to_string(),
                "cs".to_string(),
                "php".to_string(),
                "rb".to_string(),
                "go".to_string(),
                "swift".to_string(),
                "kt".to_string(),
                "scala".to_string(),
                "sh".to_string(),
                "bash".to_string(),
                "zsh".to_string(),
                "fish".to_string(),
                "ps1".to_string(),
                "bat".to_string(),
                "cmd".to_string(),
                // Configuração
                "conf".to_string(),
                "config".to_string(),
                "ini".to_string(),
                "env".to_string(),
                "dockerfile".to_string(),
                "makefile".to_string(),
                "cmake".to_string(),
                // Web
                "html".to_string(),
                "htm".to_string(),
                "css".to_string(),
                "scss".to_string(),
                "sass".to_string(),
                "less".to_string(),
                "vue".to_string(),
                "svelte".to_string(),
                // Outros
                "sql".to_string(),
                "graphql".to_string(),
                "gql".to_string(),
                "proto".to_string(),
                "thrift".to_string(),
                "avro".to_string(),
            ],
            include_patterns: vec![
                "**/*.md".to_string(),
                "**/*.txt".to_string(),
                "**/*.json".to_string(),
            ],
            exclude_patterns: vec![
                "**/node_modules/**".to_string(),
                "**/target/**".to_string(),
                "**/dist/**".to_string(),
                "**/__pycache__/**".to_string(),
                "**/.git/**".to_string(),
                "**/.vectorizer/**".to_string(),
                "**/*.png".to_string(),
                "**/*.jpg".to_string(),
                "**/*.jpeg".to_string(),
                "**/*.gif".to_string(),
                "**/*.bmp".to_string(),
                "**/*.webp".to_string(),
                "**/*.svg".to_string(),
                "**/*.ico".to_string(),
                "**/*.mp4".to_string(),
                "**/*.avi".to_string(),
                "**/*.mov".to_string(),
                "**/*.wmv".to_string(),
                "**/*.flv".to_string(),
                "**/*.webm".to_string(),
                "**/*.mp3".to_string(),
                "**/*.wav".to_string(),
                "**/*.flac".to_string(),
                "**/*.aac".to_string(),
                "**/*.ogg".to_string(),
                "**/*.db".to_string(),
                "**/*.sqlite".to_string(),
                "**/*.sqlite3".to_string(),
                "**/*.bin".to_string(),
                "**/*.exe".to_string(),
                "**/*.dll".to_string(),
                "**/*.so".to_string(),
                "**/*.dylib".to_string(),
                "**/*.zip".to_string(),
                "**/*.tar".to_string(),
                "**/*.gz".to_string(),
                "**/*.rar".to_string(),
                "**/*.7z".to_string(),
                "**/*.pdf".to_string(),
                "**/*.doc".to_string(),
                "**/*.docx".to_string(),
                "**/*.xls".to_string(),
                "**/*.xlsx".to_string(),
                "**/*.ppt".to_string(),
                "**/*.pptx".to_string(),
            ],
            embedding_dimension: 512, // Valor fixo adequado para TF-IDF
            embedding_type: "bm25".to_string(), // BM25 como padrão
            collection_name: "documents".to_string(),
            max_file_size: 1024 * 1024, // 1MB por padrão
        }
    }
}

/// Document loader for processing project directories
pub struct DocumentLoader {
    /// Configuration
    config: LoaderConfig,
    /// Embedding manager
    embedding_manager: EmbeddingManager,
    /// Processed document chunks
    processed_chunks: Vec<String>,
    /// Cache manager
    pub cache_manager: Option<CacheManager>,
}

impl DocumentLoader {
    /// Check if a file path matches the include/exclude patterns
    fn matches_patterns(&self, file_path: &Path, project_root: &Path) -> bool {
        // Convert to relative path from project root for pattern matching
        let relative_path = match file_path.strip_prefix(project_root) {
            Ok(rel) => rel,
            Err(_) => return false,
        };

        let path_str = relative_path.to_string_lossy();

        // Debug logging for gov collections
        if self.config.collection_name.starts_with("gov-") {
            debug!("Checking file: {} against patterns for collection: {}", path_str, self.config.collection_name);
            debug!("Include patterns: {:?}", self.config.include_patterns);
            debug!("Exclude patterns: {:?}", self.config.exclude_patterns);
        }

        // Check exclude patterns first - if any match, exclude the file
        for exclude_pattern in &self.config.exclude_patterns {
            if let Ok(pattern) = Pattern::new(exclude_pattern) {
                if pattern.matches(&path_str) {
                    if self.config.collection_name.starts_with("gov-") {
                        debug!("File {} excluded by pattern: {}", path_str, exclude_pattern);
                    }
                    return false;
                }
            }
        }

        // Check include patterns - if any match, include the file
        for include_pattern in &self.config.include_patterns {
            if let Ok(pattern) = Pattern::new(include_pattern) {
                if pattern.matches(&path_str) {
                    if self.config.collection_name.starts_with("gov-") {
                        debug!("File {} included by pattern: {}", path_str, include_pattern);
                    }
                    return true;
                }
            }
        }

        // If include patterns are specified, don't fall back to extension-based matching
        // This ensures we only process files that match the specific patterns
        if !self.config.include_patterns.is_empty() {
            if self.config.collection_name.starts_with("gov-") {
                debug!("File {} not included (no pattern match, include patterns specified)", path_str);
            }
            return false;
        }

        // Only fall back to extension-based matching if no include patterns are specified (legacy mode)
        if let Some(extension) = file_path.extension().and_then(|e| e.to_str()) {
            let ext_lower = extension.to_lowercase();
            let result = self.config.allowed_extensions.contains(&ext_lower);
            if self.config.collection_name.starts_with("gov-") {
                debug!("File {} extension-based check: {} (extension: {})", path_str, result, ext_lower);
            }
            return result;
        }

        if self.config.collection_name.starts_with("gov-") {
            debug!("File {} rejected (no extension)", path_str);
        }
        false
    }

    /// Get mutable reference to embedding manager
    pub fn get_embedding_manager_mut(&mut self) -> &mut EmbeddingManager {
        &mut self.embedding_manager
    }

    /// Create a new document loader
    pub fn new(config: LoaderConfig) -> Self {
        let mut embedding_manager = EmbeddingManager::new();
        let processed_chunks = Vec::new();

        // Register all available embedding types
        let tfidf = TfIdfEmbedding::new(config.embedding_dimension);
        embedding_manager.register_provider("tfidf".to_string(), Box::new(tfidf));

        let bm25 = Bm25Embedding::new(config.embedding_dimension);
        embedding_manager.register_provider("bm25".to_string(), Box::new(bm25));

        let svd = SvdEmbedding::new(config.embedding_dimension, config.embedding_dimension);
        embedding_manager.register_provider("svd".to_string(), Box::new(svd));

        let bert = BertEmbedding::new(config.embedding_dimension);
        embedding_manager.register_provider("bert".to_string(), Box::new(bert));

        let minilm = MiniLmEmbedding::new(config.embedding_dimension);
        embedding_manager.register_provider("minilm".to_string(), Box::new(minilm));

        let bow = BagOfWordsEmbedding::new(config.embedding_dimension);
        embedding_manager.register_provider("bagofwords".to_string(), Box::new(bow));

        let char_ngram = CharNGramEmbedding::new(config.embedding_dimension, 3);
        embedding_manager.register_provider("charngram".to_string(), Box::new(char_ngram));

        // Set the configured embedding type as default
        embedding_manager
            .set_default_provider(&config.embedding_type)
            .unwrap();

        Self {
            config,
            embedding_manager,
            processed_chunks,
            cache_manager: None,
        }
    }

    /// Create a new document loader with cache management
    pub async fn new_with_cache(
        config: LoaderConfig,
        cache_config: CacheConfig,
    ) -> CacheResult<Self> {
        let mut loader = Self::new(config.clone());

        // Initialize cache manager
        let cache_manager = CacheManager::new(cache_config).await?;

        loader.cache_manager = Some(cache_manager);

        Ok(loader)
    }

    /// Load and index all documents from a project directory
    pub fn load_project(&mut self, project_path: &str, store: &VectorStore) -> Result<usize> {
        // This is now a simplified entry point that internally calls the async version.
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| anyhow::anyhow!("Failed to create Tokio runtime: {}", e))?;
        rt.block_on(self.full_project_indexing(project_path, store, None))
            .map(|(count, _)| count)
            .map_err(|e| anyhow::anyhow!("Failed to index project: {}", e))
    }

    /// Load and index all documents from a project directory with advanced cache management
    pub async fn load_project_with_cache(
        &mut self,
        project_path: &str,
        store: &VectorStore,
    ) -> CacheResult<(usize, bool)> {
        self.load_project_with_cache_and_progress(project_path, store, None).await
    }

    /// Load and index all documents from a project directory with progress callback
    pub async fn load_project_with_cache_and_progress(
        &mut self,
        project_path: &str,
        store: &VectorStore,
        progress_callback: Option<&IndexingProgressState>,
    ) -> CacheResult<(usize, bool)> {
        let collection_name = &self.config.collection_name;
        let vector_store_path = PathBuf::from(project_path).join(".vectorizer").join(format!("{}_vector_store.bin", collection_name));

        // 🚀 FAST PATH: If vector store already exists, load it IMMEDIATELY
        if vector_store_path.exists() {
            info!("🚀 Loading cached vector store for '{}'", collection_name);
            match self.load_persisted_store(&vector_store_path, store, collection_name) {
                Ok(count) => {
                    info!("✅ Loaded {} vectors from cache for '{}'", count, collection_name);
                    // Record cache hit if cache manager is available
                    if let Some(cache_manager) = &self.cache_manager {
                        let _ = cache_manager.record_hit().await;
                    }
                    return Ok((count, true));
                }
                Err(e) => {
                    warn!("Cache file exists but failed to load '{}': {}. Re-indexing...", vector_store_path.display(), e);
                }
            }
        }

        // SLOW PATH: Full indexing when no cache exists
        info!("📊 No cache found for '{}', performing full indexing", collection_name);

        if let Some(cache_manager) = &self.cache_manager {
            let _ = cache_manager.record_miss().await;
        }

        self.full_project_indexing(project_path, store, progress_callback).await
    }

    /// Performs a full indexing of the project.
    async fn full_project_indexing(&mut self, project_path: &str, store: &VectorStore, progress_callback: Option<&IndexingProgressState>) -> CacheResult<(usize, bool)> {
        // Update progress: Starting document collection (20%)
        if let Some(callback) = progress_callback {
            callback.update(&self.config.collection_name, "processing", 20.0, 0, 0);
        }

        let documents = self.collect_documents(project_path).map_err(|e| CacheError::Other(e.to_string()))?;
        if documents.is_empty() {
            warn!("No documents found in project directory for collection '{}'.", self.config.collection_name);
            return Ok((0, false));
        }

        // Update progress: Documents collected, chunking (40%)
        if let Some(callback) = progress_callback {
            callback.update(&self.config.collection_name, "processing", 40.0, documents.len() as usize, 0);
        }

        let all_chunks = self.chunk_documents(&documents).map_err(|e| CacheError::Other(e.to_string()))?;

        // Update progress: Chunks created, building vocabulary (60%)
        if let Some(callback) = progress_callback {
            callback.update(&self.config.collection_name, "processing", 60.0, documents.len() as usize, all_chunks.len() as usize);
        }

        self.build_vocabulary(&documents).map_err(|e| CacheError::Other(e.to_string()))?;

        // Save tokenizer after building vocabulary
        self.save_tokenizer(project_path).map_err(|e| CacheError::Other(e.to_string()))?;

        // Update progress: Vocabulary built, creating collection (70%)
        if let Some(callback) = progress_callback {
            callback.update(&self.config.collection_name, "processing", 70.0, documents.len() as usize, all_chunks.len() as usize);
        }

        self.create_collection(store).map_err(|e| CacheError::Other(e.to_string()))?;

        // Update progress: Collection created, storing vectors (80%)
        if let Some(callback) = progress_callback {
            callback.update(&self.config.collection_name, "processing", 80.0, documents.len() as usize, all_chunks.len() as usize);
        }

        let vector_count = self.store_chunks_parallel_with_progress(store, &all_chunks, progress_callback).map_err(|e| CacheError::Other(e.to_string()))?;

        info!(
            "Successfully processed {} documents into {} chunks for collection '{}'",
            documents.len(),
            all_chunks.len(),
            self.config.collection_name
        );

        // Save the newly indexed store.
        let vectorizer_dir = PathBuf::from(project_path).join(".vectorizer");
        if let Err(e) = fs::create_dir_all(&vectorizer_dir) {
            warn!(
                "Failed to create .vectorizer directory {}: {}",
                vectorizer_dir.display(),
                e
            );
        }
        let vector_store_path = vectorizer_dir.join(format!("{}_vector_store.bin", self.config.collection_name));
        
        if let Some(collection) = store.get_collection(&self.config.collection_name).ok() {
            // HNSW dump temporariamente desabilitado devido a problemas com a biblioteca hnsw_rs
            info!("⚠️ HNSW dump temporarily disabled for collection '{}' due to library issues", self.config.collection_name);
            
            // SEGUNDO: Criar sub_store e salvar vetores
            let sub_store = VectorStore::new();
            let meta = collection.metadata();
            sub_store.create_collection(&self.config.collection_name, meta.config.clone()).unwrap();
            sub_store.insert(&self.config.collection_name, collection.get_all_vectors()).unwrap();

            info!("🔄 Starting save of collection '{}' to '{}'", self.config.collection_name, vector_store_path.display());
            if let Err(e) = sub_store.save(&vector_store_path) {
                 error!("❌ Failed to save collection vector store to '{}': {}", vector_store_path.display(), e);
            } else {
                 info!("✅ Successfully saved collection vector store to '{}'", vector_store_path.display());
            }
        }

        if let Some(cache_manager) = &self.cache_manager {
            self.update_cache_metadata(cache_manager, project_path, store).await?;
        }

        // Save collection metadata
        let mut metadata = self.create_metadata_from_config(project_path);
        if let Err(e) = self.update_metadata_with_files(&mut metadata, &documents) {
            warn!("Failed to update metadata with files: {}", e);
        }
        if let Err(e) = self.save_metadata(project_path, &metadata) {
            warn!("Failed to save metadata: {}", e);
        }

        // Note: HNSW dump is now done BEFORE sub_store creation (above)
        
        Ok((vector_count, false))
    }

    /// Loads a persisted vector store into the main application store.
    fn load_persisted_store(&self, path: &Path, app_store: &VectorStore, collection_name: &str) -> Result<usize> {
        println!("🔄 Loading persisted store from: {}", path.display());
        println!("📖 Calling VectorStore::load...");
        let persisted_store = VectorStore::load(path)?;
        println!("📖 VectorStore loaded successfully");
        
        println!("🔍 Getting collection: {}", collection_name);
        let src_collection = persisted_store.get_collection(collection_name)?;
        println!("🔍 Collection retrieved successfully");
        
        let meta = src_collection.metadata();
        if app_store.get_collection(collection_name).is_err() {
            println!("🏗️ Creating collection in app store...");
            app_store.create_collection(collection_name, meta.config.clone())?;
            println!("🏗️ Collection created successfully");
        }

        println!("📊 Getting all vectors...");
        let vectors = src_collection.get_all_vectors();
        let vector_count = vectors.len();

        // Fast load: try to load HNSW dump first, fallback to rebuilding index
        let app_collection = app_store.get_collection(collection_name)?;

        // Try to load HNSW dump first
        let hnsw_loaded = if let Some(project_path) = path.parent().and_then(|p| p.parent()) {
            let cache_dir = project_path.join(".vectorizer");
            let basename = format!("{}_hnsw", collection_name);

            // Check if dump files exist
            let graph_file = cache_dir.join(format!("{}.hnsw.graph", basename));
            let data_file = cache_dir.join(format!("{}.hnsw.data", basename));

            if graph_file.exists() && data_file.exists() {
                println!("🎯 Found HNSW dump files, attempting to load...");
                match app_collection.load_hnsw_index_from_dump(&cache_dir, &basename) {
                    Ok(_) => {
                        println!("✅ Successfully loaded HNSW index from dump");
                        // Load vectors into memory without rebuilding index
                        app_collection.load_vectors_into_memory(vectors.clone())?;
                        println!("✅ Successfully loaded {} vectors into memory (dump mode)", vector_count);
                        true
                    }
                    Err(e) => {
                        println!("❌ Failed to load HNSW dump: {}, falling back to rebuild", e);
                        false
                    }
                }
            } else {
                println!("📝 No HNSW dump files found, rebuilding index...");
                false
            }
        } else {
            println!("📝 No project path available, rebuilding index...");
            false
        };

        // If HNSW dump loading failed, rebuild the index
        if !hnsw_loaded {
            println!("🔄 Rebuilding HNSW index from {} vectors...", vector_count);
            app_collection.fast_load_vectors(vectors)?;
            println!("✅ Successfully loaded {} vectors from cache (rebuild mode)", vector_count);

            // HNSW dump temporariamente desabilitado devido a problemas com a biblioteca hnsw_rs
            info!("⚠️ HNSW dump temporarily disabled for collection '{}' due to library issues", collection_name);
        }


        // Try to load metadata if it exists
        if let Some(project_path) = path.parent().and_then(|p| p.parent()) {
            if let Ok(Some(metadata)) = self.load_metadata(project_path.to_str().unwrap()) {
                debug!("Loaded metadata for collection '{}' with {} indexed files", collection_name, metadata.files.len());
            }
        }
        
        Ok(vector_count)
    }

    /// Save tokenizer to .vectorizer directory
    fn save_tokenizer(&self, project_path: &str) -> Result<()> {
        let vectorizer_dir = PathBuf::from(project_path).join(".vectorizer");
        fs::create_dir_all(&vectorizer_dir)?;
        
        // Try to save tokenizer for sparse embedding types
        let embedding_type = self.config.embedding_type.as_str();
        if matches!(embedding_type, "bm25" | "tfidf" | "char_ngram" | "cng" | "bag_of_words" | "bow") {
            let tokenizer_path = vectorizer_dir.join(format!("{}_tokenizer.json", self.config.collection_name));
            
            // Use the EmbeddingManager's save_vocabulary_json method
            match self.embedding_manager.save_vocabulary_json(embedding_type, &tokenizer_path) {
                Ok(_) => {
                    info!("✅ Saved tokenizer for '{}' to {}", self.config.collection_name, tokenizer_path.display());
            }
            Err(e) => {
                    warn!("Failed to save tokenizer for '{}': {}", embedding_type, e);
                }
            }
        } else {
            // For other embedding types (BERT, MiniLM, etc.), no tokenizer to save
            debug!("No tokenizer to save for embedding type: {}", embedding_type);
        }
        
        Ok(())
    }

    /// Update cache metadata after processing
    async fn update_cache_metadata(
        &self,
        cache_manager: &CacheManager,
        project_path: &str,
        store: &VectorStore,
    ) -> CacheResult<()> {
        let collection_name = &self.config.collection_name;

        // Collect documents to get file information
        let documents = self.collect_documents(project_path).map_err(|e| {
            CacheError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        // Create collection cache info
        let mut collection_info = crate::cache::CollectionCacheInfo::new(
            collection_name.clone(),
            self.config.embedding_type.clone(),
            "1.0.0".to_string(), // TODO: Get actual embedding version
        );

        // Update file information
        for (file_path, _content) in &documents {
            if let Ok(metadata) = std::fs::metadata(file_path) {
                let modified_time = chrono::DateTime::from_timestamp(
                    metadata
                        .modified()?
                        .duration_since(std::time::UNIX_EPOCH)?
                        .as_secs() as i64,
                    0,
                )
                .unwrap_or_else(chrono::Utc::now);

                // Calculate file hash
                let content_hash = self.calculate_file_hash(file_path).await?;

                let file_info = crate::cache::FileHashInfo::new(
                    content_hash,
                    metadata.len(),
                    modified_time,
                    1,      // TODO: Get actual chunk count
                    vec![], // Empty vector IDs - will be populated during actual indexing
                );

                collection_info.update_file_hash(file_path.clone(), file_info);
            }
        }

        collection_info.update_indexed();

        // Get actual vector count from the store
        if let Ok(collection) = store.get_collection(collection_name) {
            collection_info.vector_count = collection.vector_count();
        }

        // Update cache metadata
        cache_manager
            .update_collection_info(collection_info)
            .await?;

        Ok(())
    }

    /// Calculate file hash
    async fn calculate_file_hash(&self, file_path: &std::path::PathBuf) -> CacheResult<String> {
        let content = tokio::fs::read(file_path).await?;
        let mut hasher = sha2::Sha256::default();
        hasher.update(&content);
        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }

    /// Collect all documents from the project directory
    pub fn collect_documents(&self, project_path: &str) -> Result<Vec<(PathBuf, String)>> {
        let path = Path::new(project_path);
        let mut documents = Vec::new();
        self.collect_documents_recursive(path, path, &mut documents)?;
        info!(
            "📁 Found {} documents in '{}' for collection '{}'",
            documents.len(),
            project_path,
            self.config.collection_name
        );
        Ok(documents)
    }

    /// Recursively collect documents from directory
    #[allow(dead_code)]
    fn collect_documents_recursive(
        &self,
        dir: &Path,
        project_root: &Path,
        documents: &mut Vec<(PathBuf, String)>,
    ) -> Result<()> {
        debug!("Scanning directory: {}", dir.display());
        let entries = fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Skip hidden directories and common ignore patterns
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if dir_name.starts_with('.')
                        || dir_name == "node_modules"
                        || dir_name == "target"
                        || dir_name == "__pycache__"
                        || dir_name == "dist"
                        || dir_name == "build"
                        || dir_name == ".git"
                        || dir_name == ".vectorizer"
                        || dir_name == "Cargo.lock"
                        || dir_name == "package-lock.json"
                        || dir_name == "yarn.lock"
                        || dir_name == "pnpm-lock.yaml"
                        || dir_name == ".next"
                        || dir_name == ".nuxt"
                        || dir_name == ".vuepress"
                        || dir_name == "_site"
                        || dir_name == "public"
                        || dir_name == "static"
                        || dir_name == "assets"
                    {
                        continue;
                    }
                }
                self.collect_documents_recursive(&path, project_root, documents)?;
            } else if path.is_file() {
                // Skip specific file names that should never be indexed
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    let skip_file = file_name == "cache.bin"
                        || file_name.starts_with("tokenizer.")
                        || file_name.ends_with(".lock")
                        || file_name == "Cargo.lock"
                        || file_name == "package-lock.json"
                        || file_name == "yarn.lock"
                        || file_name == "pnpm-lock.yaml"
                        || file_name == ".gitignore"
                        || file_name == ".gitattributes"
                        || file_name.ends_with(".log")
                        || file_name.ends_with(".tmp")
                        || file_name.ends_with(".temp")
                        || file_name == ".DS_Store"
                        || file_name == "Thumbs.db";
                    if skip_file {
                        continue;
                    }
                }

                // Skip binary files by extension
                if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = extension.to_lowercase();
                    let binary_extensions = [
                        // Images
                        "png", "jpg", "jpeg", "gif", "bmp", "webp", "svg", "ico",
                        // Videos
                        "mp4", "avi", "mov", "wmv", "flv", "webm", "mkv",
                        // Audio
                        "mp3", "wav", "flac", "aac", "ogg", "m4a",
                        // Databases
                        "db", "sqlite", "sqlite3",
                        // Binaries
                        "exe", "dll", "so", "dylib", "bin",
                        // Archives
                        "zip", "tar", "gz", "rar", "7z", "bz2", "xz",
                        // Documents (binary formats)
                        "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx",
                    ];
                    
                    if binary_extensions.contains(&ext_lower.as_str()) {
                        continue;
                    }
                }

                if self.matches_patterns(&path, project_root) {
                    // Check file size
                    match fs::metadata(&path) {
                        Ok(metadata) => {
                            let file_size = metadata.len();
                            if file_size > self.config.max_file_size as u64 {
                                warn!(
                                    "Skipping file {} (size: {} bytes, max: {} bytes)",
                                    path.display(),
                                    file_size,
                                    self.config.max_file_size
                                );
                                continue;
                            }
                        }
                        Err(e) => {
                            warn!("Failed to read metadata for {}: {}", path.display(), e);
                            continue;
                        }
                    }

                    // Read file content
                    match fs::read_to_string(&path) {
                        Ok(content) => {
                            documents.push((path, content));
                        }
                        Err(e) => {
                            warn!("Failed to read file {}: {}", path.display(), e);
                            return Err(anyhow::anyhow!(
                                "Failed to read file {}: {}",
                                path.display(),
                                e
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Build vocabulary from all documents
    #[allow(dead_code)]
    fn build_vocabulary(&mut self, documents: &[(PathBuf, String)]) -> Result<()> {
        info!("Building vocabulary from {} documents", documents.len());

        // Build vocabulary for the configured embedding type
        let embedding_type = &self.config.embedding_type;
        info!("Building vocabulary for embedding type: {}", embedding_type);

        match embedding_type.as_str() {
            "tfidf" => {
                let texts: Vec<&str> = documents
                    .iter()
                    .map(|(_, content)| content.as_str())
                    .collect();

                if let Some(provider) = self.embedding_manager.get_provider_mut("tfidf") {
                    if let Some(tfidf) = provider.as_any_mut().downcast_mut::<TfIdfEmbedding>() {
                        tfidf.build_vocabulary(&texts);
                        info!("TF-IDF vocabulary built successfully");
                    } else {
                        return Err(anyhow::anyhow!("Failed to downcast to TfIdfEmbedding"));
                    }
                } else {
                    return Err(anyhow::anyhow!("TF-IDF provider not found"));
                }
            }
            "bm25" => {
                let texts: Vec<String> = documents
                    .iter()
                    .map(|(_, content)| content.clone())
                    .collect();

                if let Some(provider) = self.embedding_manager.get_provider_mut("bm25") {
                    if let Some(bm25) = provider.as_any_mut().downcast_mut::<Bm25Embedding>() {
                        bm25.build_vocabulary(&texts);
                        info!("BM25 vocabulary built successfully");
                    } else {
                        return Err(anyhow::anyhow!("Failed to downcast to Bm25Embedding"));
                    }
                } else {
                    return Err(anyhow::anyhow!("BM25 provider not found"));
                }
            }
            "bagofwords" => {
                let texts: Vec<&str> = documents
                    .iter()
                    .map(|(_, content)| content.as_str())
                    .collect();

                if let Some(provider) = self.embedding_manager.get_provider_mut("bagofwords") {
                    if let Some(bow) = provider.as_any_mut().downcast_mut::<BagOfWordsEmbedding>() {
                        bow.build_vocabulary(&texts);
                        info!("BagOfWords vocabulary built successfully");
                    } else {
                        return Err(anyhow::anyhow!("Failed to downcast to BagOfWordsEmbedding"));
                    }
                } else {
                    return Err(anyhow::anyhow!("BagOfWords provider not found"));
                }
            }
            "charngram" => {
                // CharNGramEmbedding does not require a pre-built vocabulary.
                // We log and proceed without additional preparation.
                info!("CharNGram embedding selected - no vocabulary build step required");
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported embedding type: {}",
                    embedding_type
                ));
            }
        }

        Ok(())
    }

    /// Store chunks in the vector store using parallel processing with batch control and progress updates
    fn store_chunks_parallel_with_progress(&self, store: &VectorStore, chunks: &[DocumentChunk], progress_callback: Option<&IndexingProgressState>) -> Result<usize> {
        info!(
            "📊 Processing {} chunks in batches for collection '{}'...",
            chunks.len(),
            self.config.collection_name
        );

        const PROCESSING_BATCH_SIZE: usize = 100; // Process 100 chunks at a time
        const INSERT_BATCH_SIZE: usize = 256;     // Insert 256 vectors at a time
        let mut total_vectors = 0;
        let mut all_vectors = Vec::new();
        let total_batches = (chunks.len() + PROCESSING_BATCH_SIZE - 1) / PROCESSING_BATCH_SIZE;

        // Process chunks in smaller batches to avoid memory issues
        for (batch_num, batch) in chunks.chunks(PROCESSING_BATCH_SIZE).enumerate() {
            info!(
                "🔄 Processing batch {}/{} ({} chunks) for collection '{}'...",
                batch_num + 1,
                total_batches,
                batch.len(),
                self.config.collection_name
            );

            // Update progress during batch processing
            if let Some(callback) = progress_callback {
                let progress = 80.0 + (batch_num as f32 / total_batches as f32) * 20.0; // 80-100%
                callback.update(&self.config.collection_name, "processing", progress, chunks.len(), batch_num * PROCESSING_BATCH_SIZE);
            }

            let batch_vectors: Vec<Vector> = batch
                .par_iter()
                .filter_map(|chunk| {
                    match self.embedding_manager.embed(&chunk.content) {
                        Ok(embedding) => {
                            if embedding.iter().all(|&x| x == 0.0) {
                                warn!(
                                    "Skipping chunk with zero embedding: '{}'",
                                    chunk.content.chars().take(100).collect::<String>()
                                );
                                return None;
                            }
                            
                            let vector = Vector {
                                id: uuid::Uuid::new_v4().to_string(),
                                data: embedding,
                                payload: Some(Payload {
                                    data: serde_json::json!({
                                        "content": chunk.content,
                                        "file_path": chunk.file_path,
                                        "chunk_index": chunk.chunk_index,
                                        "metadata": chunk.metadata
                                    }),
                                }),
                            };
                            Some(vector)
                        }
                        Err(e) => {
                            warn!("Failed to embed chunk '{}': {}", chunk.id, e);
                            None
                        }
                    }
                })
                .collect();

            all_vectors.extend(batch_vectors);

            // Insert vectors in batches to avoid memory issues
            if all_vectors.len() >= INSERT_BATCH_SIZE {
                let vectors_to_insert = all_vectors.drain(0..INSERT_BATCH_SIZE).collect::<Vec<_>>();
                if let Err(e) = store.insert(&self.config.collection_name, vectors_to_insert) {
                    error!("Failed to insert batch: {}", e);
                    return Err(e.into());
                }
                total_vectors += INSERT_BATCH_SIZE;
                info!("✅ Inserted {} vectors (total: {})", INSERT_BATCH_SIZE, total_vectors);
            }
        }

        // Insert remaining vectors
        if !all_vectors.is_empty() {
            let remaining_count = all_vectors.len();
            if let Err(e) = store.insert(&self.config.collection_name, all_vectors) {
                error!("Failed to insert final batch: {}", e);
                return Err(e.into());
            }
            total_vectors += remaining_count;
        }

        info!(
            "✅ Collection '{}' indexed successfully: {} vectors stored.",
            self.config.collection_name,
            total_vectors
        );
        Ok(total_vectors)
    }

    /// Store chunks in the vector store using parallel processing with batch control
    fn store_chunks_parallel(&self, store: &VectorStore, chunks: &[DocumentChunk]) -> Result<usize> {
        info!(
            "📊 Processing {} chunks in batches for collection '{}'...",
            chunks.len(),
            self.config.collection_name
        );

        const PROCESSING_BATCH_SIZE: usize = 100; // Process 100 chunks at a time
        const INSERT_BATCH_SIZE: usize = 256;     // Insert 256 vectors at a time
        let mut total_vectors = 0;
        let mut all_vectors = Vec::new();

        // Process chunks in smaller batches to avoid memory issues
        for (batch_num, batch) in chunks.chunks(PROCESSING_BATCH_SIZE).enumerate() {
            info!(
                "🔄 Processing batch {}/{} ({} chunks) for collection '{}'...",
                batch_num + 1,
                (chunks.len() + PROCESSING_BATCH_SIZE - 1) / PROCESSING_BATCH_SIZE,
                batch.len(),
                self.config.collection_name
            );

            let batch_vectors: Vec<Vector> = batch
                .par_iter()
                .filter_map(|chunk| {
                    match self.embedding_manager.embed(&chunk.content) {
                        Ok(embedding) => {
                            if embedding.iter().all(|&x| x == 0.0) {
                                warn!(
                                    "Skipping chunk with zero embedding: '{}'",
                                    chunk.content.chars().take(100).collect::<String>()
                                );
                                return None;
                            }
                            
                            let vector = Vector {
                                id: uuid::Uuid::new_v4().to_string(),
                                data: embedding,
                                payload: Some(Payload {
                                    data: serde_json::json!({
                                        "content": chunk.content,
                                        "file_path": chunk.file_path,
                                        "chunk_index": chunk.chunk_index,
                                        "metadata": chunk.metadata
                                    }),
                                }),
                            };
                            Some(vector)
                        }
                        Err(e) => {
                            warn!("Failed to embed chunk '{}': {}", chunk.id, e);
                            None
                        }
                    }
                })
                .collect();

            all_vectors.extend(batch_vectors);

            // Insert vectors in batches to avoid memory issues
            if all_vectors.len() >= INSERT_BATCH_SIZE {
                let vectors_to_insert = all_vectors.drain(0..INSERT_BATCH_SIZE).collect::<Vec<_>>();
                if let Err(e) = store.insert(&self.config.collection_name, vectors_to_insert) {
                    error!("Failed to insert batch: {}", e);
                    return Err(e.into());
                }
                total_vectors += INSERT_BATCH_SIZE;
                info!("✅ Inserted {} vectors (total: {})", INSERT_BATCH_SIZE, total_vectors);
            }
        }

        // Insert remaining vectors
        if !all_vectors.is_empty() {
            let remaining_count = all_vectors.len();
            if let Err(e) = store.insert(&self.config.collection_name, all_vectors) {
                error!("Failed to insert final batch: {}", e);
                return Err(e.into());
            }
            total_vectors += remaining_count;
        }

        info!(
            "✅ Collection '{}' indexed successfully: {} vectors stored.",
            self.config.collection_name,
            total_vectors
        );
        Ok(total_vectors)
    }

    /// Store chunks in the vector store
    #[allow(dead_code)]
    fn store_chunks(&self, store: &VectorStore, chunks: &[DocumentChunk]) -> Result<()> {
        let mut vectors = Vec::new();

        info!(
            "📊 Processing {} chunks for collection '{}' - this may take a while...",
            chunks.len(),
            self.config.collection_name
        );

        for (i, chunk) in chunks.iter().enumerate() {
            // Generate embedding for the chunk
            let embedding = match self.embedding_manager.embed(&chunk.content) {
                Ok(emb) => emb,
                Err(e) => {
                    warn!("Failed to embed chunk {}: {}", i, e);
                    continue; // Skip this chunk
                }
            };

            // Validate embedding - reject zero vectors
            let non_zero_count = embedding.iter().filter(|&&x| x != 0.0).count();
            if non_zero_count == 0 {
                warn!(
                    "Skipping chunk {} with zero embedding: '{}'",
                    i,
                    chunk.content.chars().take(100).collect::<String>()
                );
                continue; // Skip zero vectors
            }

            // Create vector data
            let vector = Vector {
                id: uuid::Uuid::new_v4().to_string(),
                data: embedding,
                payload: Some(Payload {
                    data: serde_json::json!({
                        "content": chunk.content,
                        "file_path": chunk.file_path,
                        "chunk_index": chunk.chunk_index,
                        "metadata": chunk.metadata
                    }),
                }),
            };

            vectors.push(vector);
        }

        // Insert vectors in larger batches for better performance
        const BATCH_SIZE: usize = 2000; // Lotes ainda maiores = ainda mais rápido
        for (batch_num, batch) in vectors.chunks(BATCH_SIZE).enumerate() {
            match store.insert(&self.config.collection_name, batch.to_vec()) {
                Ok(_) => {}
                Err(e) => {
                    error!("Failed to insert batch {}: {}", batch_num + 1, e);
                    return Err(e.into());
                }
            }
        }

        info!(
            "✅ Collection '{}' indexed successfully: {} chunks embedded and stored",
            self.config.collection_name,
            chunks.len()
        );
        Ok(())
    }

    /// Create the collection in the vector store
    #[allow(dead_code)]
    fn create_collection(&self, store: &VectorStore) -> Result<()> {
        // Check if collection already exists
        if store.get_collection(&self.config.collection_name).is_ok() {
            info!(
                "Collection '{}' already exists, skipping creation",
                self.config.collection_name
            );
            return Ok(());
        }

        let config = CollectionConfig {
            dimension: self.config.embedding_dimension,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig {
                m: 16,
                ef_construction: 200,
                ef_search: 64,
                seed: Some(42), // For reproducible results
            },
            quantization: None,
            compression: Default::default(),
        };

        store
            .create_collection(&self.config.collection_name, config)
            .with_context(|| {
                format!(
                    "Failed to create collection '{}'",
                    self.config.collection_name
                )
            })?;

        // Set the embedding type for this collection
        if let Ok(collection) = store.get_collection(&self.config.collection_name) {
            collection.set_embedding_type(self.config.embedding_type.clone());
            info!(
                "Set embedding type '{}' for collection '{}'",
                self.config.embedding_type, self.config.collection_name
            );
        }

        info!("Created collection: {}", self.config.collection_name);
        Ok(())
    }

    /// Split documents into chunks
    #[allow(dead_code)]
    fn chunk_documents(&mut self, documents: &[(PathBuf, String)]) -> Result<Vec<DocumentChunk>> {
        let mut chunks = Vec::new();

        for (i, (path, content)) in documents.iter().enumerate() {
            info!(
                "Processing document {}/{}: {}",
                i + 1,
                documents.len(),
                path.display()
            );
            let file_chunks = self.chunk_text(content, path)?;
            info!(
                "Created {} chunks from {}",
                file_chunks.len(),
                path.display()
            );
            chunks.extend(file_chunks);
        }

        Ok(chunks)
    }

    /// Split a single document into chunks
    #[allow(dead_code)]
    fn chunk_text(&mut self, text: &str, file_path: &Path) -> Result<Vec<DocumentChunk>> {
        let mut chunks = Vec::new();
        let mut start = 0;
        let mut chunk_index = 0;

        while start < text.len() {
            // Calculate the end position for this chunk
            let mut end = std::cmp::min(start + self.config.max_chunk_size, text.len());

            // If we're not at the end of the text, try to find a good break point
            if end < text.len() {
                // Ensure we're at a UTF-8 character boundary
                while end > start && !text.is_char_boundary(end) {
                    end -= 1;
                }

                // Try to break at a word boundary (whitespace, punctuation)
                if let Some(pos) = text[start..end].rfind(|c: char| {
                    c.is_whitespace() || c == '.' || c == '!' || c == '?' || c == '\n'
                }) {
                    end = start + pos + 1;
                }
            }

            // Extract the chunk text
            let chunk_text = text[start..end].trim();

            // Only create a chunk if it has content
            if !chunk_text.is_empty() {
                let chunk_id = format!("{}#{}", file_path.to_string_lossy(), chunk_index);

                let mut metadata = HashMap::new();
                metadata.insert(
                    "file_path".to_string(),
                    serde_json::Value::String(file_path.to_string_lossy().to_string()),
                );
                metadata.insert(
                    "chunk_index".to_string(),
                    serde_json::Value::Number(chunk_index.into()),
                );
                metadata.insert(
                    "file_extension".to_string(),
                    serde_json::Value::String(
                        file_path
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("unknown")
                            .to_string(),
                    ),
                );
                metadata.insert(
                    "chunk_size".to_string(),
                    serde_json::Value::Number(chunk_text.len().into()),
                );

                let chunk_content = chunk_text.to_string();
                chunks.push(DocumentChunk {
                    id: chunk_id,
                    content: chunk_content.clone(),
                    file_path: file_path.to_string_lossy().to_string(),
                    chunk_index,
                    metadata,
                });

                // Store processed chunk for benchmarking
                self.processed_chunks.push(chunk_content);

                chunk_index += 1;
            }

            // Calculate the next start position with overlap
            let next_start = if end >= self.config.chunk_overlap {
                end - self.config.chunk_overlap
            } else {
                end
            };

            // Ensure we make progress (prevent infinite loop)
            if next_start <= start {
                start = end;
            } else {
                start = next_start;
            }

            // Ensure start is at a UTF-8 character boundary
            while start < text.len() && !text.is_char_boundary(start) {
                start += 1;
            }
        }

        Ok(chunks)
    }

    /// Create vectors with embeddings from chunks
    #[allow(dead_code)]
    fn create_vectors(&mut self, chunks: &[DocumentChunk]) -> Result<Vec<Vector>> {
        let mut vectors = Vec::new();

        for chunk in chunks {
            // Generate embedding for the chunk content
            let embedding = self
                .embedding_manager
                .embed(&chunk.content)
                .with_context(|| format!("Failed to generate embedding for chunk: {}", chunk.id))?;

            // Create payload with metadata
            let mut payload_data = chunk.metadata.clone();
            payload_data.insert(
                "content".to_string(),
                serde_json::Value::String(chunk.content.clone()),
            );

            let payload = Payload::new(serde_json::Value::Object(
                payload_data.into_iter().collect(),
            ));

            vectors.push(Vector::with_payload(chunk.id.clone(), embedding, payload));
        }

        Ok(vectors)
    }

    /// Get processed document chunks
    pub fn get_processed_documents(&self) -> Vec<String> {
        self.processed_chunks.clone()
    }

    /// Get collection statistics
    pub fn get_stats(&self, store: &VectorStore) -> Result<serde_json::Value> {
        let metadata = store.get_collection_metadata(&self.config.collection_name)?;

        Ok(serde_json::json!({
            "collection_name": self.config.collection_name,
            "vector_count": metadata.vector_count,
            "dimension": metadata.config.dimension,
            "metric": format!("{:?}", metadata.config.metric),
            "created_at": metadata.created_at.to_rfc3339(),
            "updated_at": metadata.updated_at.to_rfc3339(),
        }))
    }

    /// Save collection metadata to disk
    fn save_metadata(&self, project_path: &str, metadata: &CollectionMetadataFile) -> Result<()> {
        let vectorizer_dir = PathBuf::from(project_path).join(".vectorizer");
        if let Err(e) = fs::create_dir_all(&vectorizer_dir) {
            warn!(
                "Failed to create .vectorizer directory {}: {}",
                vectorizer_dir.display(),
                e
            );
        }
        
        let metadata_path = vectorizer_dir.join(format!("{}_metadata.json", self.config.collection_name));
        let metadata_json = serde_json::to_string_pretty(metadata)?;
        fs::write(&metadata_path, metadata_json)?;
        
        debug!("Saved metadata for collection '{}' to {}", self.config.collection_name, metadata_path.display());
        Ok(())
    }

    /// Load collection metadata from disk
    fn load_metadata(&self, project_path: &str) -> Result<Option<CollectionMetadataFile>> {
        let vectorizer_dir = PathBuf::from(project_path).join(".vectorizer");
        let metadata_path = vectorizer_dir.join(format!("{}_metadata.json", self.config.collection_name));
        
        if !metadata_path.exists() {
            return Ok(None);
        }
        
        let metadata_json = fs::read_to_string(&metadata_path)?;
        let metadata: CollectionMetadataFile = serde_json::from_str(&metadata_json)?;
        
        debug!("Loaded metadata for collection '{}' from {}", self.config.collection_name, metadata_path.display());
        Ok(Some(metadata))
    }

    /// Create collection metadata from current configuration
    fn create_metadata_from_config(&self, project_path: &str) -> CollectionMetadataFile {
        let config = CollectionIndexingConfig {
            chunk_size: self.config.max_chunk_size,
            chunk_overlap: self.config.chunk_overlap,
            include_patterns: self.config.include_patterns.clone(),
            exclude_patterns: self.config.exclude_patterns.clone(),
            allowed_extensions: self.config.allowed_extensions.clone(),
            max_file_size: self.config.max_file_size,
        };

        let mut parameters = HashMap::new();
        // TODO: Add actual embedding parameters based on type
        match self.config.embedding_type.as_str() {
            "bm25" => {
                parameters.insert("k1".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(1.5).unwrap()));
                parameters.insert("b".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.75).unwrap()));
            }
            _ => {}
        }

        let embedding_model = EmbeddingModelInfo {
            model_type: format!("{:?}", self.config.embedding_type).to_lowercase(),
            dimension: self.config.embedding_dimension,
            parameters,
        };

        CollectionMetadataFile::new(
            self.config.collection_name.clone(),
            project_path.to_string(),
            config,
            embedding_model,
        )
    }

    /// Update metadata with file information
    fn update_metadata_with_files(&mut self, metadata: &mut CollectionMetadataFile, documents: &[(PathBuf, String)]) -> Result<()> {
        for (file_path, content) in documents {
            let relative_path = file_path.strip_prefix(&metadata.project_path)
                .unwrap_or(file_path)
                .to_string_lossy()
                .to_string();
            
            let file_modified = get_file_modified_time(file_path)?;
            let content_hash = calculate_file_hash(file_path)?;
            let file_size = fs::metadata(file_path)?.len();
            
            // Count chunks and vectors for this file
            let chunks = self.chunk_documents(&[(file_path.clone(), content.clone())])?;
            let vectors = self.create_vectors(&chunks)?;
            
            let file_metadata = FileMetadata {
                path: relative_path.clone(),
                size_bytes: file_size,
                chunk_count: chunks.len(),
                vector_count: vectors.len(),
                indexed_at: chrono::Utc::now(),
                file_modified_at: file_modified,
                content_hash,
            };
            
            metadata.add_file(file_metadata);
        }
        
        Ok(())
    }
}
