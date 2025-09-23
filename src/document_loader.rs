//! Document loader and processor for automatic indexing

use crate::{
    embedding::{EmbeddingManager, TfIdfEmbedding},
    models::{CollectionConfig, DistanceMetric, HnswConfig, Payload, Vector},
    VectorStore,
};
use anyhow::{Context, Result};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use tracing::{debug, error, info, warn};

/// Document chunk with metadata
#[derive(Debug, Clone)]
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
    /// Collection name for documents
    pub collection_name: String,
    /// Maximum file size in bytes (default 1MB)
    pub max_file_size: usize,
}

impl Default for LoaderConfig {
    fn default() -> Self {
        Self {
            max_chunk_size: 1000,  // Valor fixo adequado
            chunk_overlap: 200,    // Valor fixo adequado
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
            embedding_dimension: 384,  // Valor fixo adequado para TF-IDF
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
        
        // Initialize TF-IDF embedding
        let tfidf = TfIdfEmbedding::new(config.embedding_dimension);
        embedding_manager.register_provider("tfidf".to_string(), Box::new(tfidf));
        embedding_manager.set_default_provider("tfidf").unwrap();

        Self {
            config,
            embedding_manager,
            processed_chunks,
        }
    }

    /// Load and index all documents from a project directory
    pub fn load_project(&mut self, project_path: &str) -> Result<usize> {
        info!("Loading project from: {}", project_path);

        // Collect all documents
        let documents = self.collect_documents(project_path)?;
        info!("Found {} documents to process", documents.len());

        if documents.is_empty() {
            warn!("No documents found in project directory");
            return Ok(0);
        }

        // Build vocabulary from all documents for TF-IDF
        self.build_vocabulary(&documents)?;

        // Process documents in chunks and store content
        info!("Starting to chunk {} documents", documents.len());
        let chunks = self.chunk_documents(&documents)?;
        info!("Created {} chunks from documents", chunks.len());

        info!("Successfully processed {} documents into {} chunks", documents.len(), chunks.len());
        Ok(chunks.len())
    }

    /// Collect all documents from the project directory
    fn collect_documents(&self, project_path: &str) -> Result<Vec<(PathBuf, String)>> {
        let mut documents = Vec::new();
        self.collect_documents_recursive(Path::new(project_path), &mut documents)?;
        Ok(documents)
    }

    /// Recursively collect documents from directory
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
                        || dir_name == "__pycache__" {
                        debug!("Skipping directory: {}", dir_name);
                        continue;
                    }
                }
                self.collect_documents_recursive(&path, documents)?;
            } else if path.is_file() {
                if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = format!(".{}", extension.to_lowercase());
                    debug!("File {} has extension: {}", path.display(), ext_lower);
                    if self.config.allowed_extensions.contains(&ext_lower) {
                        debug!("Extension {} is allowed, checking file size", ext_lower);
                        // Check file size
                        match fs::metadata(&path) {
                            Ok(metadata) => {
                                let file_size = metadata.len();
                                debug!("File size: {} bytes, max allowed: {} bytes", file_size, self.config.max_file_size);
                                if file_size > self.config.max_file_size as u64 {
                                    debug!("Skipping file {} (size: {} bytes, max: {} bytes)",
                                           path.display(), file_size, self.config.max_file_size);
                                    continue;
                                }
                            }
                            Err(e) => {
                                warn!("Failed to read metadata for {}: {}", path.display(), e);
                                continue;
                            }
                        }

                        // Read file content
                        debug!("Processing file: {}", path.display());
                        match fs::read_to_string(&path) {
                            Ok(content) => {
                                debug!("Loaded document: {} ({} bytes)", path.display(), content.len());
                                documents.push((path, content));
                            }
                            Err(e) => {
                                warn!("Failed to read file {}: {}", path.display(), e);
                                return Err(anyhow::anyhow!("Failed to read file {}: {}", path.display(), e));
                            }
                        }
                    } else {
                        debug!("Extension {} is not allowed. Allowed: {:?}", ext_lower, self.config.allowed_extensions);
                    }
                }
            }
        }

        Ok(())
    }

    /// Build vocabulary from all documents
    fn build_vocabulary(&mut self, documents: &[(PathBuf, String)]) -> Result<()> {
        info!("Building vocabulary from {} documents", documents.len());

        // Extract text content for vocabulary building
        let texts: Vec<&str> = documents.iter().map(|(_, content)| content.as_str()).collect();

        // Get the TF-IDF provider and build vocabulary
        if let Some(provider) = self.embedding_manager.get_provider_mut("tfidf") {
            if let Some(tfidf) = provider.as_any_mut().downcast_mut::<TfIdfEmbedding>() {
                tfidf.build_vocabulary(&texts);
                info!("Vocabulary built successfully");
            } else {
                return Err(anyhow::anyhow!("Failed to downcast to TfIdfEmbedding"));
            }
        } else {
            return Err(anyhow::anyhow!("TF-IDF provider not found"));
        }

        Ok(())
    }

    /// Create the collection in the vector store
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

        store.create_collection(&self.config.collection_name, config)
            .with_context(|| format!("Failed to create collection '{}'", self.config.collection_name))?;

        info!("Created collection: {}", self.config.collection_name);
        Ok(())
    }

    /// Split documents into chunks
    fn chunk_documents(&mut self, documents: &[(PathBuf, String)]) -> Result<Vec<DocumentChunk>> {
        let mut chunks = Vec::new();

        for (i, (path, content)) in documents.iter().enumerate() {
            info!("Processing document {}/{}: {}", i + 1, documents.len(), path.display());
            let file_chunks = self.chunk_text(content, path)?;
            info!("Created {} chunks from {}", file_chunks.len(), path.display());
            chunks.extend(file_chunks);
        }

        Ok(chunks)
    }

    /// Split a single document into chunks
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
                if let Some(pos) = text[start..end].rfind(|c: char| c.is_whitespace() || c == '.' || c == '!' || c == '?' || c == '\n') {
                    end = start + pos + 1;
                }
            }

            // Extract the chunk text
            let chunk_text = text[start..end].trim();
            
            // Only create a chunk if it has content
            if !chunk_text.is_empty() {
                let chunk_id = format!("{}#{}", file_path.to_string_lossy(), chunk_index);

                let mut metadata = HashMap::new();
                metadata.insert("file_path".to_string(), 
                    serde_json::Value::String(file_path.to_string_lossy().to_string()));
                metadata.insert("chunk_index".to_string(), 
                    serde_json::Value::Number(chunk_index.into()));
                metadata.insert("file_extension".to_string(), 
                    serde_json::Value::String(
                        file_path.extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("unknown")
                            .to_string()
                    ));
                metadata.insert("chunk_size".to_string(), 
                    serde_json::Value::Number(chunk_text.len().into()));

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
    fn create_vectors(&mut self, chunks: &[DocumentChunk]) -> Result<Vec<Vector>> {
        let mut vectors = Vec::new();

        for chunk in chunks {
            // Generate embedding for the chunk content
            let embedding = self.embedding_manager.embed(&chunk.content)
                .with_context(|| format!("Failed to generate embedding for chunk: {}", chunk.id))?;

            // Create payload with metadata
            let mut payload_data = chunk.metadata.clone();
            payload_data.insert("content".to_string(), 
                serde_json::Value::String(chunk.content.clone()));

            let payload = Payload::new(serde_json::Value::Object(
                payload_data.into_iter().collect()
            ));

            vectors.push(Vector::with_payload(
                chunk.id.clone(),
                embedding,
                payload,
            ));
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
}
