//! Search surface: text/vector search, intelligent search, semantic
//! search, contextual search, multi-collection search, hybrid
//! (dense + sparse) search.
//!
//! Six methods covering every search variant the v3 server exposes.
//! Discovery (multi-stage filter + score + expand) lives in
//! [`super::discovery`]; per-file search variants in [`super::files`].

use crate::error::{Result, VectorizerError};
use crate::models::hybrid_search::{
    HybridScoringAlgorithm, HybridSearchRequest, HybridSearchResponse,
};
use crate::models::*;

use super::VectorizerClient;

impl VectorizerClient {
    /// Text search against one collection. The server embeds the
    /// query with the collection's provider, runs ANN search, and
    /// returns up to `limit` (default 10) hits scored above the
    /// optional `score_threshold`.
    pub async fn search_vectors(
        &self,
        collection: &str,
        query: &str,
        limit: Option<usize>,
        score_threshold: Option<f32>,
    ) -> Result<SearchResponse> {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "query".to_string(),
            serde_json::Value::String(query.to_string()),
        );
        payload.insert(
            "limit".to_string(),
            serde_json::Value::Number(limit.unwrap_or(10).into()),
        );
        if let Some(threshold) = score_threshold {
            payload.insert(
                "score_threshold".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(threshold as f64).unwrap()),
            );
        }
        let response = self
            .make_request(
                "POST",
                &format!("/collections/{collection}/search/text"),
                Some(serde_json::Value::Object(payload)),
            )
            .await?;
        let search_response: SearchResponse = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse search response: {e}"))
        })?;
        Ok(search_response)
    }

    /// Intelligent search — multi-query expansion + MMR
    /// diversification + domain term boosting.
    pub async fn intelligent_search(
        &self,
        request: IntelligentSearchRequest,
    ) -> Result<IntelligentSearchResponse> {
        let response = self
            .make_request(
                "POST",
                "/intelligent_search",
                Some(serde_json::to_value(request).unwrap()),
            )
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse intelligent search response: {e}"))
        })
    }

    /// Semantic search — advanced reranking + similarity-threshold
    /// filtering on top of the base text search.
    pub async fn semantic_search(
        &self,
        request: SemanticSearchRequest,
    ) -> Result<SemanticSearchResponse> {
        let response = self
            .make_request(
                "POST",
                "/semantic_search",
                Some(serde_json::to_value(request).unwrap()),
            )
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse semantic search response: {e}"))
        })
    }

    /// Context-aware search with metadata filtering and contextual
    /// reranking.
    pub async fn contextual_search(
        &self,
        request: ContextualSearchRequest,
    ) -> Result<ContextualSearchResponse> {
        let response = self
            .make_request(
                "POST",
                "/contextual_search",
                Some(serde_json::to_value(request).unwrap()),
            )
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse contextual search response: {e}"))
        })
    }

    /// Multi-collection search with cross-collection reranking and
    /// aggregation.
    pub async fn multi_collection_search(
        &self,
        request: MultiCollectionSearchRequest,
    ) -> Result<MultiCollectionSearchResponse> {
        let response = self
            .make_request(
                "POST",
                "/multi_collection_search",
                Some(serde_json::to_value(request).unwrap()),
            )
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse multi-collection search response: {e}"
            ))
        })
    }

    /// Hybrid search combining dense and sparse vectors with one of
    /// three scoring algorithms (RRF, weighted, alpha-blending).
    pub async fn hybrid_search(
        &self,
        request: HybridSearchRequest,
    ) -> Result<HybridSearchResponse> {
        let url = format!("/collections/{}/hybrid_search", request.collection);
        let payload = serde_json::json!({
            "query": request.query,
            "alpha": request.alpha,
            "algorithm": match request.algorithm {
                HybridScoringAlgorithm::ReciprocalRankFusion => "rrf",
                HybridScoringAlgorithm::WeightedCombination => "weighted",
                HybridScoringAlgorithm::AlphaBlending => "alpha",
            },
            "dense_k": request.dense_k,
            "sparse_k": request.sparse_k,
            "final_k": request.final_k,
            "query_sparse": request.query_sparse.as_ref().map(|sv| serde_json::json!({
                "indices": sv.indices,
                "values": sv.values,
            })),
        });
        let response = self.make_request("POST", &url, Some(payload)).await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse hybrid search response: {e}"))
        })
    }
}
