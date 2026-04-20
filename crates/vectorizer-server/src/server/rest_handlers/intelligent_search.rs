//! Intelligent search REST handlers.
//!
//! High-level orchestrator endpoints that compose multiple underlying
//! searches (domain expansion, reranking, cross-collection fan-out) into
//! a single API call. Each handler delegates to an
//! [`intelligent_search::rest_api::RESTAPIHandler`] built from the
//! server's shared `VectorStore`.
//!
//! - `intelligent_search`       — POST /search/intelligent
//! - `multi_collection_search`  — POST /search/multi-collection
//! - `semantic_search`          — POST /search/semantic
//! - `contextual_search`        — POST /search/contextual

use axum::extract::State;
use axum::response::Json;
use serde_json::{Value, json};
use tracing::{debug, error};

use crate::server::VectorizerServer;
use crate::server::error_middleware::{
    ErrorResponse, create_bad_request_error, create_validation_error,
};

pub async fn intelligent_search(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    use vectorizer::cache::query_cache::QueryKey;
    use vectorizer::intelligent_search::rest_api::{IntelligentSearchRequest, RESTAPIHandler};
    use vectorizer::monitoring::metrics::METRICS;

    // Start latency timer
    let label_wildcard = "*".to_string();
    let label_intelligent = "intelligent".to_string();
    let timer = METRICS
        .search_latency_seconds
        .with_label_values(&[&label_wildcard, &label_intelligent])
        .start_timer();

    // Extract parameters from JSON payload
    let query = payload
        .get("query")
        .and_then(|q| q.as_str())
        .ok_or_else(|| create_validation_error("query", "missing or invalid query parameter"))?;

    let collections = payload
        .get("collections")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        });

    let max_results = payload
        .get("max_results")
        .and_then(|m| m.as_u64())
        .map(|m| m as usize)
        .unwrap_or(10);

    let domain_expansion = payload.get("domain_expansion").and_then(|d| d.as_bool());
    let technical_focus = payload.get("technical_focus").and_then(|t| t.as_bool());
    let mmr_enabled = payload.get("mmr_enabled").and_then(|m| m.as_bool());
    let mmr_lambda = payload
        .get("mmr_lambda")
        .and_then(|l| l.as_f64())
        .map(|l| l as f32);

    // Create cache key (use "*" as collection name for multi-collection searches)
    let collection_key = collections
        .as_ref()
        .map(|c| c.join(","))
        .unwrap_or_else(|| "*".to_string());
    let cache_key = QueryKey::new(
        collection_key,
        format!(
            "intelligent:{}:{}:{}:{}:{}",
            query,
            max_results,
            domain_expansion.unwrap_or(true),
            technical_focus.unwrap_or(true),
            mmr_enabled.unwrap_or(false)
        ),
        max_results,
        None,
    );

    // Check cache first
    if let Some(cached_result) = state.query_cache.get(&cache_key) {
        debug!("💾 Cache hit for intelligent search query '{}'", query);
        drop(timer);
        return Ok(Json(cached_result));
    }

    // Create handler with the actual server instances
    let handler = RESTAPIHandler::new_with_store(state.store.clone());

    // Create intelligent search request
    let request = IntelligentSearchRequest {
        query: query.to_string(),
        collections,
        max_results: Some(max_results),
        domain_expansion,
        technical_focus,
        mmr_enabled,
        mmr_lambda,
    };

    match handler.handle_intelligent_search(request).await {
        Ok(response) => {
            // Convert response to JSON
            let response_json = serde_json::to_value(&response).unwrap_or(json!({}));

            // Cache the result
            state.query_cache.insert(cache_key, response_json.clone());

            // Record success metrics
            let result_count = response.results.len();
            let label_wildcard = "*";
            let label_intelligent = "intelligent";
            let label_success = "success";
            METRICS
                .search_requests_total
                .with_label_values(&[label_wildcard, label_intelligent, label_success])
                .inc();
            let label_wildcard_str = "*".to_string();
            let label_intelligent_str = "intelligent".to_string();
            METRICS
                .search_results_count
                .with_label_values(&[&label_wildcard_str, &label_intelligent_str])
                .observe(result_count as f64);
            drop(timer);

            Ok(Json(response_json))
        }
        Err(e) => {
            // Record error metrics
            let label_wildcard = "*";
            let label_intelligent = "intelligent";
            let label_error = "error";
            METRICS
                .search_requests_total
                .with_label_values(&[label_wildcard, label_intelligent, label_error])
                .inc();
            drop(timer);

            error!("Intelligent search error: {:?}", e);
            Err(create_bad_request_error(&format!(
                "Intelligent search failed: {:?}",
                e
            )))
        }
    }
}

pub async fn multi_collection_search(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    use vectorizer::intelligent_search::rest_api::{MultiCollectionSearchRequest, RESTAPIHandler};

    // Create handler with the actual server instances
    let handler = RESTAPIHandler::new_with_store(state.store.clone());

    let query = payload
        .get("query")
        .and_then(|q| q.as_str())
        .ok_or_else(|| create_validation_error("query", "missing or invalid query parameter"))?;

    let collections = payload
        .get("collections")
        .and_then(|c| c.as_array())
        .ok_or_else(|| {
            create_validation_error("collections", "missing or invalid collections parameter")
        })?
        .iter()
        .filter_map(|v| v.as_str())
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    let max_per_collection = payload
        .get("max_per_collection")
        .and_then(|m| m.as_u64())
        .map(|m| m as usize);

    let max_total_results = payload
        .get("max_total_results")
        .and_then(|m| m.as_u64())
        .map(|m| m as usize);

    let cross_collection_reranking = payload
        .get("cross_collection_reranking")
        .and_then(|c| c.as_bool());

    let request = MultiCollectionSearchRequest {
        query: query.to_string(),
        collections,
        max_per_collection,
        max_total_results,
        cross_collection_reranking,
    };

    match handler.handle_multi_collection_search(request).await {
        Ok(response) => Ok(Json(serde_json::to_value(response).unwrap_or(json!({})))),
        Err(e) => {
            error!("Multi collection search error: {:?}", e);
            Err(create_bad_request_error(&format!(
                "Multi collection search failed: {:?}",
                e
            )))
        }
    }
}

pub async fn semantic_search(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    use vectorizer::intelligent_search::rest_api::{RESTAPIHandler, SemanticSearchRequest};

    // Create handler with the actual server instances
    let handler = RESTAPIHandler::new_with_store(state.store.clone());

    let query = payload
        .get("query")
        .and_then(|q| q.as_str())
        .ok_or_else(|| create_validation_error("query", "missing or invalid query parameter"))?;

    let collection = payload
        .get("collection")
        .and_then(|c| c.as_str())
        .ok_or_else(|| {
            create_validation_error("collection", "missing or invalid collection parameter")
        })?;

    let max_results = payload
        .get("max_results")
        .and_then(|m| m.as_u64())
        .map(|m| m as usize);

    let semantic_reranking = payload.get("semantic_reranking").and_then(|s| s.as_bool());

    let cross_encoder_reranking = payload
        .get("cross_encoder_reranking")
        .and_then(|c| c.as_bool());

    let similarity_threshold = payload
        .get("similarity_threshold")
        .and_then(|s| s.as_f64())
        .map(|s| s as f32);

    let request = SemanticSearchRequest {
        query: query.to_string(),
        collection: collection.to_string(),
        max_results,
        semantic_reranking,
        cross_encoder_reranking,
        similarity_threshold,
    };

    match handler.handle_semantic_search(request).await {
        Ok(response) => Ok(Json(serde_json::to_value(response).unwrap_or(json!({})))),
        Err(e) => {
            error!("Semantic search error: {:?}", e);
            Err(create_bad_request_error(&format!(
                "Semantic search failed: {:?}",
                e
            )))
        }
    }
}

pub async fn contextual_search(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    use vectorizer::intelligent_search::rest_api::{ContextualSearchRequest, RESTAPIHandler};

    // Create handler with the actual server instances
    let handler = RESTAPIHandler::new_with_store(state.store.clone());

    let query = payload
        .get("query")
        .and_then(|q| q.as_str())
        .ok_or_else(|| create_validation_error("query", "missing or invalid query parameter"))?;

    let collection = payload
        .get("collection")
        .and_then(|c| c.as_str())
        .ok_or_else(|| {
            create_validation_error("collection", "missing or invalid collection parameter")
        })?;

    let context_filters = payload
        .get("context_filters")
        .and_then(|f| f.as_object())
        .map(|obj| {
            let mut map = std::collections::HashMap::new();
            for (k, v) in obj {
                map.insert(k.clone(), v.clone());
            }
            map
        });

    let context_weight = payload
        .get("context_weight")
        .and_then(|w| w.as_f64())
        .map(|w| w as f32);

    let context_reranking = payload.get("context_reranking").and_then(|r| r.as_bool());

    let max_results = payload
        .get("max_results")
        .and_then(|m| m.as_u64())
        .map(|m| m as usize);

    let request = ContextualSearchRequest {
        query: query.to_string(),
        collection: collection.to_string(),
        context_filters,
        context_weight,
        context_reranking,
        max_results,
    };

    match handler.handle_contextual_search(request).await {
        Ok(response) => Ok(Json(serde_json::to_value(response).unwrap_or(json!({})))),
        Err(e) => {
            error!("Contextual search error: {:?}", e);
            Err(create_bad_request_error(&format!(
                "Contextual search failed: {:?}",
                e
            )))
        }
    }
}
