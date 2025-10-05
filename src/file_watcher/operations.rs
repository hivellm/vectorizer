//! Vector operations for file watcher

use std::sync::Arc;
use tokio::sync::RwLock;
use crate::{VectorStore, embedding::EmbeddingManager};
use crate::error::{Result, VectorizerError};

/// Vector operations for file watcher
pub struct VectorOperations {
    vector_store: Arc<VectorStore>,
    embedding_manager: Arc<RwLock<EmbeddingManager>>,
}

impl VectorOperations {
    pub fn new(
        vector_store: Arc<VectorStore>,
        embedding_manager: Arc<RwLock<EmbeddingManager>>,
    ) -> Self {
        Self {
            vector_store,
            embedding_manager,
        }
    }

    /// Process file change event
    pub async fn process_file_change(&self, event: &crate::file_watcher::FileChangeEvent) -> Result<()> {
        match event {
            crate::file_watcher::FileChangeEvent::Created(_) | crate::file_watcher::FileChangeEvent::Modified(_) => {
                // TODO: Implement file indexing
                tracing::info!("File changed: {:?}", event);
            }
            crate::file_watcher::FileChangeEvent::Deleted(_) => {
                // TODO: Implement file removal from index
                tracing::info!("File deleted: {:?}", event);
            }
            crate::file_watcher::FileChangeEvent::Renamed(_, _) => {
                // TODO: Implement file rename handling
                tracing::info!("File renamed: {:?}", event);
            }
        }
        Ok(())
    }

    /// Index file content
    pub async fn index_file(&self, file_path: &str, content: &str, collection_name: &str) -> Result<()> {
        let embedding_manager = self.embedding_manager.read().await;
        
        // Generate embedding for the content
        let embedding = embedding_manager.embed(content)?;
        
        // Create vector with file path as ID
        let vector = crate::models::Vector::with_payload(
            file_path.to_string(),
            embedding,
            crate::models::Payload::new(serde_json::json!({
                "file_path": file_path,
                "content": content,
                "collection": collection_name
            }))
        );
        
        // Insert into collection
        self.vector_store.insert(collection_name, vec![vector])?;
        
        tracing::info!("Indexed file: {} in collection: {}", file_path, collection_name);
        Ok(())
    }

    /// Remove file from index
    pub async fn remove_file(&self, file_path: &str, collection_name: &str) -> Result<()> {
        // Remove vector by ID (file path)
        self.vector_store.delete(collection_name, file_path)?;
        
        tracing::info!("Removed file: {} from collection: {}", file_path, collection_name);
        Ok(())
    }

    /// Update file in index
    pub async fn update_file(&self, file_path: &str, content: &str, collection_name: &str) -> Result<()> {
        // For now, just re-index the file (remove and add again)
        self.remove_file(file_path, collection_name).await?;
        self.index_file(file_path, content, collection_name).await?;
        
        tracing::info!("Updated file: {} in collection: {}", file_path, collection_name);
        Ok(())
    }
}
