//! Collection-management surface: list, create, get info, delete.
//!
//! These are the four endpoints that operate on collections as a
//! whole — vector-level CRUD lives in [`super::vectors`], search
//! over a collection lives in [`super::search`].

use crate::error::{Result, VectorizerError};
use crate::models::*;

use super::VectorizerClient;

impl VectorizerClient {
    /// List every collection visible to the authenticated principal.
    /// Accepts both the legacy bare-array response and the newer
    /// `{collections: [...]}` wrapper.
    pub async fn list_collections(&self) -> Result<Vec<Collection>> {
        let response = self.make_request("GET", "/collections", None).await?;
        let collections: Vec<Collection> = if let Ok(wrapper) =
            serde_json::from_str::<serde_json::Value>(&response)
        {
            if let Some(arr) = wrapper.get("collections").and_then(|v| v.as_array()) {
                serde_json::from_value(serde_json::Value::Array(arr.clone())).map_err(|e| {
                    VectorizerError::server(format!("Failed to parse collections array: {e}"))
                })?
            } else if wrapper.is_array() {
                serde_json::from_value(wrapper).map_err(|e| {
                    VectorizerError::server(format!("Failed to parse collections response: {e}"))
                })?
            } else {
                return Err(VectorizerError::server(
                    "Unexpected collections response format".to_string(),
                ));
            }
        } else {
            return Err(VectorizerError::server(
                "Failed to parse collections response".to_string(),
            ));
        };
        Ok(collections)
    }

    /// Create a new collection. The returned [`CollectionInfo`] is
    /// synthesised from the server's create-response plus the
    /// arguments — the server response only carries the collection
    /// name today.
    pub async fn create_collection(
        &self,
        name: &str,
        dimension: usize,
        metric: Option<SimilarityMetric>,
    ) -> Result<CollectionInfo> {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "name".to_string(),
            serde_json::Value::String(name.to_string()),
        );
        payload.insert(
            "dimension".to_string(),
            serde_json::Value::Number(dimension.into()),
        );
        payload.insert(
            "metric".to_string(),
            serde_json::Value::String(format!("{:?}", metric.unwrap_or_default()).to_lowercase()),
        );

        let response = self
            .make_request(
                "POST",
                "/collections",
                Some(serde_json::Value::Object(payload)),
            )
            .await?;
        let create_response: CreateCollectionResponse =
            serde_json::from_str(&response).map_err(|e| {
                VectorizerError::server(format!("Failed to parse create collection response: {e}"))
            })?;

        let info = CollectionInfo {
            name: create_response.collection,
            dimension,
            metric: format!("{:?}", metric.unwrap_or_default()).to_lowercase(),
            vector_count: 0,
            document_count: 0,
            created_at: String::new(),
            updated_at: String::new(),
            indexing_status: Some(crate::models::IndexingStatus {
                status: "created".to_string(),
                progress: 0.0,
                total_documents: 0,
                processed_documents: 0,
                vector_count: 0,
                estimated_time_remaining: None,
                last_updated: String::new(),
            }),
            size: None,
            quantization: None,
            normalization: None,
            status: Some("created".to_string()),
        };
        Ok(info)
    }

    /// Delete a collection by name.
    pub async fn delete_collection(&self, name: &str) -> Result<()> {
        self.make_request("DELETE", &format!("/collections/{name}"), None)
            .await?;
        Ok(())
    }

    /// Fetch metadata for a collection (vector count, dimension,
    /// metric, timestamps, indexing status).
    pub async fn get_collection_info(&self, collection: &str) -> Result<CollectionInfo> {
        let response = self
            .make_request("GET", &format!("/collections/{collection}"), None)
            .await?;
        let info: CollectionInfo = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse collection info: {e}"))
        })?;
        Ok(info)
    }
}
