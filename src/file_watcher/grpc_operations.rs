//! GRPC operations for vector database updates

use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::RwLock;
use crate::VectorStore;
use crate::models::QuantizationConfig;
use crate::embedding::EmbeddingManager;
use crate::grpc::vectorizer::vectorizer_service_client::VectorizerServiceClient;
use tonic::{Request, Response};
use crate::file_watcher::{FileChangeEvent, FileChangeEventWithMetadata, Result, FileWatcherError};

/// GRPC operations for vector database
pub struct GrpcVectorOperations {
    vector_store: Arc<VectorStore>,
    embedding_manager: Arc<RwLock<EmbeddingManager>>,
    grpc_client: Option<Arc<VectorizerServiceClient<tonic::transport::Channel>>>,
}

impl GrpcVectorOperations {
    /// Create a new GRPC vector operations instance
    pub fn new(
        vector_store: Arc<VectorStore>,
        embedding_manager: Arc<RwLock<EmbeddingManager>>,
        grpc_client: Option<Arc<VectorizerServiceClient<tonic::transport::Channel>>>,
    ) -> Self {
        Self {
            vector_store,
            embedding_manager,
            grpc_client,
        }
    }

    /// Process a file change event
    pub async fn process_file_change(
        &self,
        event: FileChangeEventWithMetadata,
        collection_name: &str,
    ) -> Result<()> {
        match event.event {
            FileChangeEvent::Created(path) | FileChangeEvent::Modified(path) => {
                self.index_file(&path, collection_name).await
            }
            FileChangeEvent::Deleted(path) => {
                self.remove_file(&path, collection_name).await
            }
            FileChangeEvent::Renamed(old_path, new_path) => {
                // Remove old file and index new file
                self.remove_file(&old_path, collection_name).await?;
                self.index_file(&new_path, collection_name).await
            }
        }
    }

    /// Index a file into the vector database
    pub async fn index_file(&self, file_path: &std::path::Path, collection_name: &str) -> Result<()> {
        // Check if file exists and is readable
        if !file_path.exists() {
            return Err(FileWatcherError::FileSystem(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {:?}", file_path),
            )));
        }

        // Skip directories
        if file_path.is_dir() {
            tracing::debug!("Skipping directory: {:?}", file_path);
            return Ok(());
        }

        // Skip hidden files and temporary files
        if let Some(file_name) = file_path.file_name() {
            let name = file_name.to_string_lossy();
            if name.starts_with('.') || name.starts_with('~') || name.ends_with(".tmp") || 
               name.contains(".tmp") || name.ends_with(".part") || name.ends_with(".lock") {
                tracing::debug!("Skipping temporary/hidden file: {:?}", file_path);
                return Ok(());
            }
        }

        // Skip files in target directory (Rust build artifacts)
        if file_path.to_string_lossy().contains("/target/") || 
           file_path.to_string_lossy().contains("\\target\\") {
            tracing::debug!("Skipping build artifact: {:?}", file_path);
            return Ok(());
        }

        // Read file content
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| FileWatcherError::FileSystem(e))?;

        // Generate embedding
        let embedding = self.generate_embedding(&content).await?;

        // Create vector ID from file path
        let vector_id = self.create_vector_id(file_path);

        // Create vector with metadata
        let vector = crate::models::Vector::with_payload(
            vector_id,
            embedding,
            crate::models::Payload::from_value(serde_json::json!({
                "file_path": file_path.to_string_lossy(),
                "file_size": file_path.metadata().map(|m| m.len()).unwrap_or(0),
                "last_modified": chrono::Utc::now().to_rfc3339(),
                "content_preview": content.chars().take(200).collect::<String>(),
            }))
            .map_err(|e| FileWatcherError::Embedding(e.to_string()))?,
        );

        // Insert vector into collection
        if let Some(grpc_client) = &self.grpc_client {
            self.insert_vector_grpc(grpc_client, collection_name, vector).await?;
        } else {
            self.insert_vector_local(collection_name, vector).await?;
        }

        tracing::info!("Indexed file: {:?}", file_path);
        Ok(())
    }

    /// Remove a file from the vector database
    pub async fn remove_file(&self, file_path: &std::path::Path, collection_name: &str) -> Result<()> {
        let vector_id = self.create_vector_id(file_path);

        if let Some(grpc_client) = &self.grpc_client {
            self.remove_vector_grpc(grpc_client, collection_name, &vector_id).await?;
        } else {
            self.remove_vector_local(collection_name, &vector_id).await?;
        }

        tracing::info!("Removed file: {:?}", file_path);
        Ok(())
    }

    /// Remove a vector by ID from the collection
    pub async fn remove_vector(&self, vector_id: &str, collection_name: &str) -> Result<()> {
        if let Some(grpc_client) = &self.grpc_client {
            self.remove_vector_grpc(grpc_client, collection_name, vector_id).await?;
        } else {
            self.remove_vector_local(collection_name, vector_id).await?;
        }

        tracing::info!("Removed vector: {} from collection: {}", vector_id, collection_name);
        Ok(())
    }

    /// Generate embedding for text content
    async fn generate_embedding(&self, content: &str) -> Result<Vec<f32>> {
        let manager = self.embedding_manager.read().await;
        manager.embed(content)
            .map_err(|e| FileWatcherError::Embedding(e.to_string()))
    }

    /// Create vector ID from file path
    pub fn create_vector_id(&self, file_path: &std::path::Path) -> String {
        // Use file path as ID, but make it safe for vector database
        file_path.to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "_")
            .replace(":", "_")
            .replace(" ", "_")
    }

    /// Insert vector using GRPC
    async fn insert_vector_grpc(
        &self,
        grpc_client: &VectorizerServiceClient<tonic::transport::Channel>,
        collection_name: &str,
        vector: crate::models::Vector,
    ) -> Result<()> {
        use crate::grpc::vectorizer::{
            InsertTextsRequest, TextData, InsertTextsResponse
        };

        // Extrair texto do payload para embedding
        let text = {
            let mut text_content = String::new();
            if let Ok(value) = serde_json::to_value(vector.payload.clone()) {
                if let Some(obj) = value.as_object() {
                    if let Some(content) = obj.get("content").and_then(|v| v.as_str()) {
                        text_content = content.to_string();
                    }
                }
            }
            text_content
        };

        let text_data = TextData {
            id: vector.id,
            text,
            metadata: {
                let mut map = std::collections::HashMap::new();
                if let Ok(value) = serde_json::to_value(vector.payload.clone()) {
                    if let Some(obj) = value.as_object() {
                        for (k, v) in obj {
                            if k != "content" { // Não incluir content no metadata
                                map.insert(k.clone(), v.to_string());
                            }
                        }
                    }
                }
                map
            },
        };

        let request = InsertTextsRequest {
            collection: collection_name.to_string(),
            texts: vec![text_data],
            provider: "bm25".to_string(), // Provider padrão
        };

        let mut grpc_client = grpc_client.clone();
        let response: Response<InsertTextsResponse> = grpc_client
            .insert_texts(Request::new(request))
            .await
            .map_err(|e| FileWatcherError::Grpc(e))?;

        if response.get_ref().status == "success" {
            Ok(())
        } else {
            Err(FileWatcherError::Embedding(
                response.get_ref().message.clone()
            ))
        }
    }

    /// Insert vector locally
    async fn insert_vector_local(
        &self,
        collection_name: &str,
        vector: crate::models::Vector,
    ) -> Result<()> {
        self.vector_store
            .insert(collection_name, vec![vector])
            .map_err(|e| FileWatcherError::Embedding(e.to_string()))?;
        Ok(())
    }

    /// Remove vector using GRPC
    async fn remove_vector_grpc(
        &self,
        client: &VectorizerServiceClient<tonic::transport::Channel>,
        collection_name: &str,
        vector_id: &str,
    ) -> Result<()> {
        use crate::grpc::vectorizer::{
            DeleteVectorsRequest, DeleteVectorsResponse
        };

        let request = DeleteVectorsRequest {
            collection: collection_name.to_string(),
            vector_ids: vec![vector_id.to_string()],
        };

        let mut grpc_client = client.clone();
        let response: Response<DeleteVectorsResponse> = grpc_client
            .delete_vectors(Request::new(request))
            .await
            .map_err(|e| FileWatcherError::Grpc(e))?;

        if response.get_ref().status == "success" {
            Ok(())
        } else {
            Err(FileWatcherError::Embedding(
                response.get_ref().message.clone()
            ))
        }
    }

    /// Remove vector locally
    async fn remove_vector_local(
        &self,
        collection_name: &str,
        vector_id: &str,
    ) -> Result<()> {
        self.vector_store
            .delete(collection_name, vector_id)
            .map_err(|e| FileWatcherError::Embedding(e.to_string()))?;
        Ok(())
    }

    /// Batch process multiple file changes
    pub async fn batch_process_file_changes(
        &self,
        events: Vec<FileChangeEventWithMetadata>,
        collection_name: &str,
    ) -> Result<()> {
        for event in events {
            if let Err(e) = self.process_file_change(event, collection_name).await {
                tracing::error!("Failed to process file change: {}", e);
                // Continue processing other events
            }
        }
        Ok(())
    }

    /// Update vector in the database
    pub async fn update_vector(
        &self,
        file_path: &std::path::Path,
        collection_name: &str,
    ) -> Result<()> {
        // For update, we remove the old vector and insert the new one
        self.remove_file(file_path, collection_name).await?;
        self.index_file(file_path, collection_name).await?;
        Ok(())
    }

    /// Check if collection exists
    pub async fn collection_exists(&self, collection_name: &str) -> bool {
        self.vector_store.list_collections().contains(&collection_name.to_string())
    }

    /// Create collection if it doesn't exist
    pub async fn ensure_collection_exists(
        &self,
        collection_name: &str,
        dimension: usize,
    ) -> Result<()> {
        if !self.collection_exists(collection_name).await {
            let config = crate::models::CollectionConfig {
                dimension,
                metric: crate::models::DistanceMetric::Cosine,
                hnsw_config: crate::models::HnswConfig::default(),
                quantization: QuantizationConfig::SQ { bits: 8 },
                compression: Default::default(),
            };

            self.vector_store
                .create_collection(collection_name, config)
                .map_err(|e| FileWatcherError::Embedding(e.to_string()))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[tokio::test]
    async fn test_vector_id_creation() {
        let operations = GrpcVectorOperations::new(
            Arc::new(VectorStore::new()),
            Arc::new(RwLock::new(EmbeddingManager::new())),
            None,
        );

        let path = PathBuf::from("test/file.txt");
        let id = operations.create_vector_id(&path);
        assert_eq!(id, "test_file.txt");

        let path_with_spaces = PathBuf::from("test file with spaces.txt");
        let id_with_spaces = operations.create_vector_id(&path_with_spaces);
        assert_eq!(id_with_spaces, "test_file_with_spaces.txt");
    }

    #[tokio::test]
    async fn test_collection_operations() {
        let vector_store = Arc::new(VectorStore::new());
        let operations = GrpcVectorOperations::new(
            vector_store.clone(),
            Arc::new(RwLock::new(EmbeddingManager::new())),
            None,
        );

        let collection_name = "test_collection";
        
        // Check if collection exists (should be false initially)
        assert!(!operations.collection_exists(collection_name).await);

        // Create collection
        operations.ensure_collection_exists(collection_name, 128).await.unwrap();
        
        // Check if collection exists (should be true now)
        assert!(operations.collection_exists(collection_name).await);
    }

    #[tokio::test]
    async fn test_file_indexing() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "This is a test file content").unwrap();

        let vector_store = Arc::new(VectorStore::new());
        let mut embedding_manager = EmbeddingManager::new();
        
        // Register a simple embedding provider for testing
        let tfidf = crate::embedding::TfIdfEmbedding::new(64);
        embedding_manager.register_provider("tfidf".to_string(), Box::new(tfidf));
        embedding_manager.set_default_provider("tfidf").unwrap();

        let operations = GrpcVectorOperations::new(
            vector_store.clone(),
            Arc::new(RwLock::new(embedding_manager)),
            None,
        );

        let collection_name = "test_collection";
        operations.ensure_collection_exists(collection_name, 64).await.unwrap();

        // Index the file
        operations.index_file(&file_path, collection_name).await.unwrap();

        // Check if vector was inserted
        let metadata = vector_store.get_collection_metadata(collection_name).unwrap();
        assert_eq!(metadata.vector_count, 1);
    }

    #[tokio::test]
    async fn test_file_removal() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "This is a test file content").unwrap();

        let vector_store = Arc::new(VectorStore::new());
        let mut embedding_manager = EmbeddingManager::new();
        
        let tfidf = crate::embedding::TfIdfEmbedding::new(64);
        embedding_manager.register_provider("tfidf".to_string(), Box::new(tfidf));
        embedding_manager.set_default_provider("tfidf").unwrap();

        let operations = GrpcVectorOperations::new(
            vector_store.clone(),
            Arc::new(RwLock::new(embedding_manager)),
            None,
        );

        let collection_name = "test_collection";
        operations.ensure_collection_exists(collection_name, 64).await.unwrap();

        // Index the file
        operations.index_file(&file_path, collection_name).await.unwrap();

        // Remove the file
        operations.remove_file(&file_path, collection_name).await.unwrap();

        // Check if vector was removed
        let metadata = vector_store.get_collection_metadata(collection_name).unwrap();
        assert_eq!(metadata.vector_count, 0);
    }
}
