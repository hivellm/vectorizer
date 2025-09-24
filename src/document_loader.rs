//! Document loader and processor for automatic indexing

use crate::{
    VectorStore,
    embedding::{EmbeddingManager, TfIdfEmbedding, Bm25Embedding, SvdEmbedding, BertEmbedding, MiniLmEmbedding, BagOfWordsEmbedding, CharNGramEmbedding},
    models::{CollectionConfig, DistanceMetric, HnswConfig, Payload, Vector},
};
use anyhow::{Context, Result};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};
use tracing::{debug, info, warn, error};

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
    /// File extensions to process
    pub allowed_extensions: Vec<String>,
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
            max_chunk_size: 1000, // Valor fixo adequado
            chunk_overlap: 200,   // Valor fixo adequado
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
}

impl DocumentLoader {
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
        embedding_manager.set_default_provider(&config.embedding_type).unwrap();

        Self {
            config,
            embedding_manager,
            processed_chunks,
        }
    }

    /// Load and index all documents from a project directory with caching
    pub fn load_project(&mut self, project_path: &str, store: &VectorStore) -> Result<usize> {
        info!("Loading project from: {}", project_path);

        // Ensure .vectorizer directory exists
        let vectorizer_dir = PathBuf::from(project_path).join(".vectorizer");
        if let Err(e) = fs::create_dir_all(&vectorizer_dir) {
            warn!("Failed to create .vectorizer directory {}: {}", vectorizer_dir.display(), e);
        }

        // Use .vectorizer for cache file
        let cache_path = vectorizer_dir.join("cache.bin");
        
        // Try to load from cache first
        let mut project_cache = self.load_cache(&cache_path.to_string_lossy())?;
        let config_hash = self.calculate_config_hash();
        
        // Check if cache is valid for current config
        if project_cache.config_hash != config_hash {
            info!("Configuration changed, invalidating cache");
            project_cache = ProjectCache {
                files: HashMap::new(),
                config_hash,
            };
        }

        // Collect all documents
        let documents = self.collect_documents(project_path)?;
        info!("Found {} documents to process", documents.len());

        if documents.is_empty() {
            warn!("No documents found in project directory");
            return Ok(0);
        }

        // Process documents with cache
        let mut all_chunks = Vec::new();
        let mut cache_hits = 0;
        let mut cache_misses = 0;

        for (file_path, content) in &documents {
            let file_path_str = file_path.to_string_lossy().to_string();
            
            // Check if file is in cache and up to date
            if let Some(cache_entry) = project_cache.files.get(&file_path_str) {
                if let Ok(metadata) = fs::metadata(file_path) {
                    if let Ok(modified_time) = metadata.modified() {
                        if modified_time == cache_entry.modified_time && 
                           metadata.len() == cache_entry.file_size {
                            // Cache hit - use cached chunks
                            all_chunks.extend(cache_entry.chunks.clone());
                            cache_hits += 1;
                            debug!("Cache hit for: {}", file_path_str);
                            continue;
                        }
                    }
                }
            }
            
            // Cache miss - process file
            cache_misses += 1;
            debug!("Cache miss for: {}", file_path_str);
            
            let file_chunks = self.chunk_text(content, file_path)?;
            all_chunks.extend(file_chunks.clone());
            
            // Update cache
            if let Ok(metadata) = fs::metadata(file_path) {
                if let Ok(modified_time) = metadata.modified() {
                    project_cache.files.insert(file_path_str, CacheEntry {
                        modified_time,
                        file_size: metadata.len(),
                        chunks: file_chunks,
                    });
                }
            }
        }

        info!("Cache stats: {} hits, {} misses", cache_hits, cache_misses);

        // Save updated cache
        self.save_cache(&cache_path.to_string_lossy(), &project_cache)?;

        // Build vocabulary from all documents for configured embedding
        self.build_vocabulary(&documents)?;

        // After building vocabulary, persist tokenizer file for configured provider
        match self.config.embedding_type.as_str() {
            "bm25" => {
                if let Some(provider) = self.embedding_manager.get_provider_mut("bm25") {
                    if let Some(bm25) = provider.as_any_mut().downcast_mut::<Bm25Embedding>() {
                        let tokenizer_path = vectorizer_dir.join("tokenizer.bm25.json");
                        if let Err(e) = bm25.save_vocabulary_json(&tokenizer_path) {
                            warn!("Failed to save BM25 tokenizer to {}: {}", tokenizer_path.to_string_lossy(), e);
                        } else {
                            info!("Saved BM25 tokenizer to: {}", tokenizer_path.to_string_lossy());
                        }
                    }
                }
            }
            "tfidf" => {
                if let Some(provider) = self.embedding_manager.get_provider_mut("tfidf") {
                    if let Some(tfidf) = provider.as_any_mut().downcast_mut::<TfIdfEmbedding>() {
                        let tokenizer_path = vectorizer_dir.join("tokenizer.tfidf.json");
                        if let Err(e) = tfidf.save_vocabulary_json(&tokenizer_path) {
                            warn!("Failed to save TF-IDF tokenizer to {}: {}", tokenizer_path.to_string_lossy(), e);
                        } else {
                            info!("Saved TF-IDF tokenizer to: {}", tokenizer_path.to_string_lossy());
                        }
                    }
                }
            }
            "bagofwords" => {
                if let Some(provider) = self.embedding_manager.get_provider_mut("bagofwords") {
                    if let Some(bow) = provider.as_any_mut().downcast_mut::<BagOfWordsEmbedding>() {
                        let tokenizer_path = vectorizer_dir.join("tokenizer.bow.json");
                        if let Err(e) = bow.save_vocabulary_json(&tokenizer_path) {
                            warn!("Failed to save BagOfWords tokenizer to {}: {}", tokenizer_path.to_string_lossy(), e);
                        } else {
                            info!("Saved BagOfWords tokenizer to: {}", tokenizer_path.to_string_lossy());
                        }
                    }
                }
            }
            "charngram" => {
                if let Some(provider) = self.embedding_manager.get_provider_mut("charngram") {
                    if let Some(cng) = provider.as_any_mut().downcast_mut::<CharNGramEmbedding>() {
                        let tokenizer_path = vectorizer_dir.join("tokenizer.charngram.json");
                        if let Err(e) = cng.save_vocabulary_json(&tokenizer_path) {
                            warn!("Failed to save CharNGram tokenizer to {}: {}", tokenizer_path.to_string_lossy(), e);
                        } else {
                            info!("Saved CharNGram tokenizer to: {}", tokenizer_path.to_string_lossy());
                        }
                    }
                }
            }
            _ => {}
        }

        // Create collection in vector store
        self.create_collection(store)?;

        // Store chunks in vector store
        info!("Storing {} chunks in vector store", all_chunks.len());
        self.store_chunks(store, &all_chunks)?;

        info!(
            "Successfully processed {} documents into {} chunks",
            documents.len(),
            all_chunks.len()
        );
        Ok(all_chunks.len())
    }

    /// Collect all documents from the project directory
    pub fn collect_documents(&self, project_path: &str) -> Result<Vec<(PathBuf, String)>> {
        let path = Path::new(project_path);
        let mut documents = Vec::new();
        self.collect_documents_recursive(path, &mut documents)?;
        info!("collect_documents found {} documents", documents.len());
        Ok(documents)
    }

    /// Recursively collect documents from directory
    #[allow(dead_code)]
    fn collect_documents_recursive(
        &self,
        dir: &Path,
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
                self.collect_documents_recursive(&path, documents)?;
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

                if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = extension.to_lowercase();
                    if self.config.allowed_extensions.contains(&ext_lower) {
                        // Check file size
                        match fs::metadata(&path) {
                            Ok(metadata) => {
                                let file_size = metadata.len();
                                if file_size > self.config.max_file_size as u64 {
                                    warn!("Skipping file {} (size: {} bytes, max: {} bytes)", path.display(), file_size, self.config.max_file_size);
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
            },
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
            },
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
            },
            _ => {
                return Err(anyhow::anyhow!("Unsupported embedding type: {}", embedding_type));
            }
        }

        Ok(())
    }

    /// Store chunks in the vector store
    #[allow(dead_code)]
    fn store_chunks(&self, store: &VectorStore, chunks: &[DocumentChunk]) -> Result<()> {
        
        let mut vectors = Vec::new();
        
        info!("Starting to process {} chunks for embedding", chunks.len());
        
        for (i, chunk) in chunks.iter().enumerate() {
            if i % 100 == 0 {
                info!("Processing chunk {}/{}", i + 1, chunks.len());
            }
            
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
                warn!("Skipping chunk {} with zero embedding: '{}'", i, chunk.content.chars().take(100).collect::<String>());
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
        
        info!("Successfully created {} vectors, inserting in batches", vectors.len());
        
        // Insert vectors in larger batches for better performance
        const BATCH_SIZE: usize = 2000; // Lotes ainda maiores = ainda mais rápido
        for (batch_num, batch) in vectors.chunks(BATCH_SIZE).enumerate() {
            info!("Inserting batch {}/{} ({} vectors)", 
                  batch_num + 1, 
                  (vectors.len() + BATCH_SIZE - 1) / BATCH_SIZE,
                  batch.len());
            
            match store.insert(&self.config.collection_name, batch.to_vec()) {
                Ok(_) => {
                    info!("Successfully inserted batch {}", batch_num + 1);
                }
                Err(e) => {
                    error!("Failed to insert batch {}: {}", batch_num + 1, e);
                    return Err(e.into());
                }
            }
        }
        
        info!("Successfully stored {} chunks in collection '{}'", 
              chunks.len(), self.config.collection_name);
        Ok(())
    }

    /// Create the collection in the vector store
    #[allow(dead_code)]
    fn create_collection(&self, store: &VectorStore) -> Result<()> {
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

        // Delete existing collection if it exists
        let _ = store.delete_collection(&self.config.collection_name);

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
            info!("Set embedding type '{}' for collection '{}'", self.config.embedding_type, self.config.collection_name);
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

    /// Load cache from file
    fn load_cache(&self, cache_path: &str) -> Result<ProjectCache> {
        match fs::read_to_string(cache_path) {
            Ok(data) => {
                // Try to deserialize from JSON
                match serde_json::from_str(&data) {
                    Ok(cache) => {
                        info!("Loaded cache from: {}", cache_path);
                        Ok(cache)
                    }
                    Err(e) => {
                        // Try old bincode format for backward compatibility
                        if let Ok(binary_data) = fs::read(cache_path) {
                            if let Ok(cache) = bincode::deserialize::<ProjectCache>(&binary_data) {
                                info!("Loaded legacy bincode cache from: {}", cache_path);
                                return Ok(cache);
                            }
                        }
                        
                        warn!("Failed to deserialize cache: {}, creating new cache", e);
                        Ok(ProjectCache {
                            files: HashMap::new(),
                            config_hash: 0,
                        })
                    }
                }
            }
            Err(_) => {
                info!("No cache file found, creating new cache");
                Ok(ProjectCache {
                    files: HashMap::new(),
                    config_hash: 0,
                })
            }
        }
    }

    /// Save cache to file
    fn save_cache(&self, cache_path: &str, cache: &ProjectCache) -> Result<()> {
        // Use JSON instead of bincode to avoid deserialize_any issues
        let json_data = serde_json::to_string_pretty(cache)
            .with_context(|| "Failed to serialize cache to JSON")?;
        
        fs::write(cache_path, json_data)
            .with_context(|| format!("Failed to write cache to: {}", cache_path))?;
        
        info!("Saved cache to: {}", cache_path);
        Ok(())
    }

    /// Calculate hash of current configuration
    fn calculate_config_hash(&self) -> u64 {
        use std::hash::{Hash, Hasher, DefaultHasher};
        
        let mut hasher = DefaultHasher::new();
        self.config.max_chunk_size.hash(&mut hasher);
        self.config.chunk_overlap.hash(&mut hasher);
        self.config.embedding_dimension.hash(&mut hasher);
        self.config.embedding_type.hash(&mut hasher);
        self.config.allowed_extensions.hash(&mut hasher);
        self.config.max_file_size.hash(&mut hasher);
        
        hasher.finish()
    }

    /// Extract the embedding manager from the loader
    pub fn into_embedding_manager(mut self) -> EmbeddingManager {
        // CRITICAL FIX: Ensure vocabulary is preserved during transfer
        // Extract and save vocabulary data before the move

        let mut bm25_vocabulary_data = None;

        // Extract vocabulary data before move
        if let Some(provider) = self.embedding_manager.get_provider_mut("bm25") {
            if let Some(bm25) = provider.as_any_mut().downcast_ref::<crate::embedding::Bm25Embedding>() {
                if bm25.vocabulary_size() == 0 {
                    warn!("BM25 vocabulary is empty! This indicates a problem in build_vocabulary()");
                } else {
                    debug!("BM25 vocabulary has {} terms before transfer", bm25.vocabulary_size());
                    // Extract vocabulary data for restoration
                    bm25_vocabulary_data = Some(bm25.extract_vocabulary_data());
                }
            }
        }
        
        // Move the embedding manager
        let mut manager = self.embedding_manager;
        
        // Restore vocabulary data after move if needed
        if let Some(vocab_data) = bm25_vocabulary_data {
            if let Some(provider) = manager.get_provider_mut("bm25") {
                if let Some(bm25) = provider.as_any_mut().downcast_mut::<Bm25Embedding>() {
                    // Check if vocabulary was lost during move
                    if bm25.vocabulary_size() == 0 {
                        warn!("Vocabulary lost during move! Restoring...");
                        bm25.restore_vocabulary_data(
                            vocab_data.0,  // vocabulary
                            vocab_data.1,  // doc_freq
                            vocab_data.2,  // doc_lengths
                            vocab_data.3,  // avg_doc_length
                            vocab_data.4   // total_docs
                        );
                        debug!("Vocabulary restored with {} terms", bm25.vocabulary_size());
                    } else {
                        debug!("Vocabulary preserved during move: {} terms", bm25.vocabulary_size());
                    }
                }
            }
        }
        
        manager
    }
}
