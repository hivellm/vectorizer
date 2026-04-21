//! Vector-level surface: get, batch-insert texts, embed.
//!
//! Single-vector retrieval, batch text insertion, and on-server
//! embedding generation. Search lives in [`super::search`];
//! collection-level CRUD in [`super::collections`].

use super::VectorizerClient;
use crate::error::{Result, VectorizerError};
use crate::models::*;

impl VectorizerClient {
    /// Fetch one vector by id.
    pub async fn get_vector(&self, collection: &str, vector_id: &str) -> Result<Vector> {
        let response = self
            .make_request(
                "GET",
                &format!("/collections/{collection}/vectors/{vector_id}"),
                None,
            )
            .await?;
        let vector: Vector = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse get vector response: {e}"))
        })?;
        Ok(vector)
    }

    /// Insert a batch of texts into a collection. The server
    /// embeds them with the collection's configured provider.
    pub async fn insert_texts(
        &self,
        collection: &str,
        texts: Vec<BatchTextRequest>,
    ) -> Result<BatchResponse> {
        let payload = serde_json::json!({ "texts": texts });
        let response = self
            .make_request(
                "POST",
                &format!("/collections/{collection}/documents"),
                Some(serde_json::to_value(payload)?),
            )
            .await?;
        let batch_response: BatchResponse = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse insert texts response: {e}"))
        })?;
        Ok(batch_response)
    }

    /// Generate an embedding for `text` using either the supplied
    /// `model` name or the server default.
    pub async fn embed_text(&self, text: &str, model: Option<&str>) -> Result<EmbeddingResponse> {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "text".to_string(),
            serde_json::Value::String(text.to_string()),
        );
        if let Some(model) = model {
            payload.insert(
                "model".to_string(),
                serde_json::Value::String(model.to_string()),
            );
        }
        let response = self
            .make_request("POST", "/embed", Some(serde_json::Value::Object(payload)))
            .await?;
        let embedding_response: EmbeddingResponse =
            serde_json::from_str(&response).map_err(|e| {
                VectorizerError::server(format!("Failed to parse embedding response: {e}"))
            })?;
        Ok(embedding_response)
    }
}
