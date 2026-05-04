//! Search surface: text/vector search, intelligent search, semantic
//! search, contextual search, multi-collection search, hybrid
//! (dense + sparse) search.
//!
//! Six methods covering every search variant the v3 server exposes.
//! Discovery (multi-stage filter + score + expand) lives in
//! [`super::discovery`]; per-file search variants in [`super::files`].

use super::VectorizerClient;
use crate::error::{Result, VectorizerError};
use crate::models::hybrid_search::{
    HybridScoringAlgorithm, HybridSearchRequest, HybridSearchResponse,
};
use crate::models::{ExplainRequest, ExplainResponse, ExplainTrace, *};

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

    /// Search a collection for vectors associated with a given file path.
    ///
    /// Calls `POST /collections/{name}/search/file` with `{file_path, limit?}`.
    /// Returns a [`SearchResponse`] (may be empty if the file is not indexed).
    pub async fn search_by_file(
        &self,
        collection: &str,
        request: SearchByFileRequest,
    ) -> Result<SearchResponse> {
        let url = format!("/collections/{collection}/search/file");
        let payload = serde_json::json!({
            "file_path": request.file_path,
            "limit": request.limit.unwrap_or(10),
        });
        let response = self.make_request("POST", &url, Some(payload)).await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse search_by_file response: {e}"))
        })
    }

    // ── Phase-14: observability ────────────────────────────────────────────────

    /// Run a search and return the full HNSW execution trace (phase14).
    ///
    /// Calls `POST /collections/{name}/explain` with
    /// `{"vector": [f32…], "k": <u64>}`.
    ///
    /// The trace includes: `visited_nodes`, `ef_search`, `hnsw_search_ms`,
    /// `payload_filter_evals`, `quantization_score_ms`, and `total_ms`. The
    /// results are identical to a normal search — there is no separate explain
    /// engine; the real code path is instrumented.
    pub async fn explain_search(
        &self,
        collection: &str,
        request: crate::models::ExplainRequest,
    ) -> Result<crate::models::ExplainResponse> {
        let mut payload = serde_json::json!({ "vector": request.vector });
        if let Some(k) = request.k {
            payload
                .as_object_mut()
                .map(|o| o.insert("k".to_string(), serde_json::json!(k)));
        }
        let response = self
            .make_request(
                "POST",
                &format!("/collections/{collection}/explain"),
                Some(payload),
            )
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse explain_search response: {e}"))
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::models::{ExplainRequest, ExplainResponse, ExplainTrace};

    #[test]
    fn explain_request_serialize_with_k() {
        let req = ExplainRequest {
            vector: vec![0.1, 0.2, 0.3],
            k: Some(5),
        };
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["k"], 5);
        // f32 → f64 promotion introduces sub-microsecond error; use epsilon.
        let first = v["vector"][0].as_f64().unwrap();
        assert!((first - 0.1_f64).abs() < 1e-6, "unexpected value: {first}");
    }

    #[test]
    fn explain_request_serialize_without_k() {
        let req = ExplainRequest {
            vector: vec![0.1],
            k: None,
        };
        let v = serde_json::to_value(&req).unwrap();
        // k is skip_serializing_if = "Option::is_none"
        assert!(v.get("k").is_none());
    }

    #[test]
    fn explain_response_wire_shape() {
        // Mirror of `POST /collections/{name}/explain` response.
        let raw = json!({
            "collection": "docs",
            "k": 10,
            "results": [
                { "id": "vec-1", "score": 0.95, "payload": null }
            ],
            "trace": {
                "visited_nodes": 120,
                "ef_search": 100,
                "hnsw_search_ms": 1.23,
                "payload_filter_evals": 0,
                "quantization_score_ms": 0.45,
                "total_ms": 2.10,
            }
        });
        let resp: ExplainResponse = serde_json::from_value(raw).unwrap();
        assert_eq!(resp.collection, "docs");
        assert_eq!(resp.k, 10);
        assert_eq!(resp.results.len(), 1);
        assert_eq!(resp.trace.visited_nodes, 120);
        assert_eq!(resp.trace.ef_search, 100);
        assert!((resp.trace.hnsw_search_ms - 1.23).abs() < 1e-6);
    }

    #[test]
    fn explain_trace_deserializes_all_fields() {
        let raw = json!({
            "visited_nodes": 200,
            "ef_search": 64,
            "hnsw_search_ms": 3.5,
            "payload_filter_evals": 10,
            "quantization_score_ms": 0.8,
            "total_ms": 5.0,
        });
        let t: ExplainTrace = serde_json::from_value(raw).unwrap();
        assert_eq!(t.payload_filter_evals, 10);
        assert!((t.quantization_score_ms - 0.8).abs() < 1e-9);
    }
}
