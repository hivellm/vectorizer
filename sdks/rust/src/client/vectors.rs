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
    ///
    /// **Server caveat (observed on `hivehub/vectorizer:3.0.x`):** the
    /// `GET /collections/{c}/vectors/{id}` endpoint currently returns
    /// HTTP 200 with a synthetic uniform-vector payload
    /// (`[0.1, 0.1, …]`) even for ids that don't exist. Callers that
    /// need real miss detection should probe via
    /// [`VectorizerClient::list_vectors`] or search and not trust an
    /// `Ok(Vector)` as proof of existence until the server fix ships.
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

    /// Insert a batch of texts into a collection. The server embeds
    /// each entry with the collection's configured provider (BM25 by
    /// default; FastEmbed ONNX when selected in `config.yml`).
    ///
    /// Wire contract: the server's `POST /insert_texts` handler
    /// expects `{ "collection": "<name>", "texts": [...] }` — the
    /// collection is a top-level field in the JSON body, not a path
    /// segment. The earlier `POST /collections/{c}/documents` path
    /// this method used was never served (the 3.0.x server returns
    /// 404 for it) and has been removed.
    ///
    /// Per-entry `id` field: the server **reassigns** every inserted
    /// vector a server-generated UUID regardless of what the caller
    /// sent. The original client id is stashed as `client_id` on the
    /// response entry. Callers that need idempotency by client id
    /// should key off the `client_id` round-trip, not the
    /// server-assigned UUID.
    pub async fn insert_texts(
        &self,
        collection: &str,
        texts: Vec<BatchTextRequest>,
    ) -> Result<BatchResponse> {
        let payload = serde_json::json!({
            "collection": collection,
            "texts": texts,
        });
        let response = self
            .make_request("POST", "/insert_texts", Some(payload))
            .await?;
        let mut batch_response: BatchResponse = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse insert texts response: {e}"))
        })?;
        // v3 omits the pre-v3 `success` field and instead emits
        // `inserted` / `failed` counts (aliased onto
        // `successful_operations` / `failed_operations`). The struct
        // doc-comment tells callers to derive the flag themselves; do
        // that here once so existing consumers (and the SDK integration
        // suite) keep working across the shape change.
        if !batch_response.success
            && batch_response.failed_operations == 0
            && batch_response.successful_operations > 0
        {
            batch_response.success = true;
        }
        // v3 also drops the pre-v3 `operation` tag. The call site here
        // unambiguously *is* an insert, so fill it in if the server
        // didn't — callers that assert on the tag keep working.
        if batch_response.operation.is_empty() {
            batch_response.operation = "insert".to_string();
        }
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
