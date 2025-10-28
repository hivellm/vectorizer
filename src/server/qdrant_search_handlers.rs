//! Qdrant search operations REST API handlers

use std::collections::HashMap;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde_json::{Value, json};
use tracing::{debug, error, info};

use super::VectorizerServer;
use super::error_middleware::{ErrorResponse, create_error_response, create_not_found_error};
use crate::error::VectorizerError;
use crate::models::qdrant::point::{QdrantPointId, QdrantValue, QdrantVector};
use crate::models::qdrant::{
    FilterProcessor, QdrantBatchRecommendRequest, QdrantBatchRecommendResponse,
    QdrantBatchSearchRequest, QdrantBatchSearchResponse, QdrantRecommendRequest,
    QdrantRecommendResponse, QdrantRecommendStrategy, QdrantScoredPoint, QdrantSearchRequest,
    QdrantSearchResponse,
};
use crate::models::{Payload, SearchResult, Vector};

/// Convert serde_json::Value to QdrantValue
fn json_value_to_qdrant_value(value: serde_json::Value) -> QdrantValue {
    match value {
        serde_json::Value::String(s) => QdrantValue::String(s),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                QdrantValue::Integer(i)
            } else if let Some(f) = n.as_f64() {
                QdrantValue::Float(f)
            } else {
                QdrantValue::String(n.to_string())
            }
        }
        serde_json::Value::Bool(b) => QdrantValue::Boolean(b),
        serde_json::Value::Array(arr) => {
            QdrantValue::Array(arr.into_iter().map(json_value_to_qdrant_value).collect())
        }
        serde_json::Value::Object(obj) => QdrantValue::Object(
            obj.into_iter()
                .map(|(k, v)| (k, json_value_to_qdrant_value(v)))
                .collect(),
        ),
        serde_json::Value::Null => QdrantValue::Null,
    }
}

/// Search points in a collection
pub async fn search_points(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    Json(request): Json<QdrantSearchRequest>,
) -> Result<Json<QdrantSearchResponse>, ErrorResponse> {
    info!(
        collection = %collection_name,
        vector_dim = request.vector.len(),
        limit = ?request.limit,
        "Searching points in collection"
    );

    // Get collection from store
    let collection = state
        .store
        .get_collection(&collection_name)
        .map_err(|e| create_not_found_error("collection", &collection_name))?;

    // Validate vector dimension
    let expected_dim = collection.config().dimension;
    if request.vector.len() != expected_dim {
        return Err(create_error_response(
            &format!(
                "Expected dimension {}, got {}",
                expected_dim,
                request.vector.len()
            ),
            "Vector dimension mismatch",
            StatusCode::BAD_REQUEST,
        ));
    }

    // Set default limit if not provided
    let limit = request.limit.unwrap_or(10) as usize;
    let offset = request.offset.unwrap_or(0) as usize;

    // Perform search
    let search_results = collection
        .search(&request.vector, limit + offset)
        .map_err(|e| {
            create_error_response(
                &format!("{}", e),
                "Search failed",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

    // Apply offset, filters, and limit
    let results: Vec<QdrantScoredPoint> = search_results
        .into_iter()
        .skip(offset)
        .take(limit)
        .filter(|result| {
            // Apply score threshold if provided
            if let Some(threshold) = request.score_threshold {
                if result.score < threshold {
                    return false;
                }
            }

            // Apply filters if provided
            if let Some(filter) = &request.filter {
                if let Some(payload) = &result.payload {
                    return FilterProcessor::apply_filter(filter, payload);
                } else {
                    // No payload, filter cannot match
                    return false;
                }
            }

            true
        })
        .map(|result| {
            // Convert to QdrantScoredPoint
            let id = match result.id.parse::<u64>() {
                Ok(numeric_id) => QdrantPointId::Numeric(numeric_id),
                Err(_) => QdrantPointId::Uuid(result.id),
            };

            let vector = if request.with_vector.unwrap_or(false) {
                Some(QdrantVector::Dense(result.vector.unwrap_or_default()))
            } else {
                None
            };

            let payload = if request.with_payload.unwrap_or(true) {
                result.payload.map(|p| {
                    p.data
                        .as_object()
                        .unwrap_or(&serde_json::Map::new())
                        .iter()
                        .map(|(k, v)| (k.clone(), json_value_to_qdrant_value(v.clone())))
                        .collect()
                })
            } else {
                None
            };

            QdrantScoredPoint {
                id,
                vector,
                payload,
                score: result.score,
            }
        })
        .collect();

    info!(
        collection = %collection_name,
        results_count = results.len(),
        "Search completed successfully"
    );

    Ok(Json(QdrantSearchResponse { result: results }))
}

/// Recommend points in a collection
pub async fn recommend_points(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    Json(request): Json<QdrantRecommendRequest>,
) -> Result<Json<QdrantRecommendResponse>, ErrorResponse> {
    info!(
        collection = %collection_name,
        positive_count = request.positive.len(),
        negative_count = request.negative.as_ref().map_or(0, |v| v.len()),
        "Recommending points in collection"
    );

    // Get collection from store
    let collection = state
        .store
        .get_collection(&collection_name)
        .map_err(|e| create_not_found_error("collection", &collection_name))?;

    // Convert point IDs to strings for retrieval
    let positive_ids: Vec<String> = request
        .positive
        .into_iter()
        .map(|id| match id {
            QdrantPointId::Numeric(n) => n.to_string(),
            QdrantPointId::Uuid(s) => s,
        })
        .collect();

    let negative_ids: Vec<String> = request
        .negative
        .unwrap_or_default()
        .into_iter()
        .map(|id| match id {
            QdrantPointId::Numeric(n) => n.to_string(),
            QdrantPointId::Uuid(s) => s,
        })
        .collect();

    // Retrieve positive examples
    let positive_vectors: Vec<Vec<f32>> = positive_ids
        .iter()
        .filter_map(|id| collection.get_vector(id).ok().map(|v| v.data))
        .collect();

    if positive_vectors.is_empty() {
        return Err(create_error_response(
            "At least one positive example must exist in the collection",
            "No valid positive examples found",
            StatusCode::BAD_REQUEST,
        ));
    }

    // Retrieve negative examples
    let _negative_vectors: Vec<Vec<f32>> = negative_ids
        .iter()
        .filter_map(|id| collection.get_vector(id).ok().map(|v| v.data))
        .collect();

    // Calculate recommendation vector based on strategy
    let recommendation_vector = match request
        .strategy
        .unwrap_or(QdrantRecommendStrategy::AverageVector)
    {
        QdrantRecommendStrategy::AverageVector => {
            // Average all positive vectors
            let mut avg_vector = vec![0.0; positive_vectors[0].len()];
            for vector in &positive_vectors {
                for (i, &val) in vector.iter().enumerate() {
                    avg_vector[i] += val;
                }
            }
            let count = positive_vectors.len() as f32;
            for val in &mut avg_vector {
                *val /= count;
            }
            avg_vector
        }
        QdrantRecommendStrategy::BestScore => {
            // Use the first positive vector as the query
            positive_vectors[0].clone()
        }
    };

    // Set default limit if not provided
    let limit = request.limit.unwrap_or(10) as usize;
    let offset = request.offset.unwrap_or(0) as usize;

    // Perform search with the recommendation vector
    let search_results = collection
        .search(&recommendation_vector, limit + offset)
        .map_err(|e| {
            create_error_response(
                &format!("{}", e),
                "Recommendation search failed",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

    // Apply offset, filters, and limit
    let results: Vec<QdrantScoredPoint> = search_results
        .into_iter()
        .skip(offset)
        .take(limit)
        .filter(|result| {
            // Apply score threshold if provided
            if let Some(threshold) = request.score_threshold {
                if result.score < threshold {
                    return false;
                }
            }

            // Apply filters if provided
            if let Some(filter) = &request.filter {
                if let Some(payload) = &result.payload {
                    return FilterProcessor::apply_filter(filter, payload);
                } else {
                    // No payload, filter cannot match
                    return false;
                }
            }

            true
        })
        .map(|result| {
            // Convert to QdrantScoredPoint
            let id = match result.id.parse::<u64>() {
                Ok(numeric_id) => QdrantPointId::Numeric(numeric_id),
                Err(_) => QdrantPointId::Uuid(result.id),
            };

            let vector = if request.with_vector.unwrap_or(false) {
                Some(QdrantVector::Dense(result.vector.unwrap_or_default()))
            } else {
                None
            };

            let payload = if request.with_payload.unwrap_or(true) {
                result.payload.map(|p| {
                    p.data
                        .as_object()
                        .unwrap_or(&serde_json::Map::new())
                        .iter()
                        .map(|(k, v)| (k.clone(), json_value_to_qdrant_value(v.clone())))
                        .collect()
                })
            } else {
                None
            };

            QdrantScoredPoint {
                id,
                vector,
                payload,
                score: result.score,
            }
        })
        .collect();

    info!(
        collection = %collection_name,
        results_count = results.len(),
        "Recommendation completed successfully"
    );

    Ok(Json(QdrantRecommendResponse { result: results }))
}

/// Batch search points in a collection
pub async fn batch_search_points(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    Json(request): Json<QdrantBatchSearchRequest>,
) -> Result<Json<QdrantBatchSearchResponse>, ErrorResponse> {
    info!(
        collection = %collection_name,
        batch_size = request.searches.len(),
        "Batch searching points in collection"
    );

    // Get collection from store
    let collection = state
        .store
        .get_collection(&collection_name)
        .map_err(|e| create_not_found_error("collection", &collection_name))?;

    let expected_dim = collection.config().dimension;

    // Process each search request in the batch
    let mut results = Vec::new();
    for (i, search_request) in request.searches.into_iter().enumerate() {
        // Validate vector dimension
        if search_request.vector.len() != expected_dim {
            return Err(create_error_response(
                &format!(
                    "Search {}: Expected dimension {}, got {}",
                    i,
                    expected_dim,
                    search_request.vector.len()
                ),
                "Vector dimension mismatch",
                StatusCode::BAD_REQUEST,
            ));
        }

        // Set default limit if not provided
        let limit = search_request.limit.unwrap_or(10) as usize;
        let offset = search_request.offset.unwrap_or(0) as usize;

        // Perform search
        let search_results = collection
            .search(&search_request.vector, limit + offset)
            .map_err(|e| {
                create_error_response(
                    &format!("Search {}: {}", i, e),
                    "Batch search failed",
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
            })?;

        // Apply offset, filters, and limit
        let batch_results: Vec<QdrantScoredPoint> = search_results
            .into_iter()
            .skip(offset)
            .take(limit)
            .filter(|result| {
                // Apply score threshold if provided
                if let Some(threshold) = search_request.score_threshold {
                    if result.score < threshold {
                        return false;
                    }
                }

                // Apply filters if provided
                if let Some(filter) = &search_request.filter {
                    if let Some(payload) = &result.payload {
                        return FilterProcessor::apply_filter(filter, payload);
                    } else {
                        // No payload, filter cannot match
                        return false;
                    }
                }

                true
            })
            .map(|result| {
                // Convert to QdrantScoredPoint
                let id = match result.id.parse::<u64>() {
                    Ok(numeric_id) => QdrantPointId::Numeric(numeric_id),
                    Err(_) => QdrantPointId::Uuid(result.id),
                };

                let vector = if search_request.with_vector.unwrap_or(false) {
                    Some(QdrantVector::Dense(result.vector.unwrap_or_default()))
                } else {
                    None
                };

                let payload = if search_request.with_payload.unwrap_or(true) {
                    result.payload.map(|p| {
                        p.data
                            .as_object()
                            .unwrap_or(&serde_json::Map::new())
                            .iter()
                            .map(|(k, v)| (k.clone(), json_value_to_qdrant_value(v.clone())))
                            .collect()
                    })
                } else {
                    None
                };

                QdrantScoredPoint {
                    id,
                    vector,
                    payload,
                    score: result.score,
                }
            })
            .collect();

        results.push(QdrantSearchResponse {
            result: batch_results,
        });
    }

    info!(
        collection = %collection_name,
        batch_results = results.len(),
        "Batch search completed successfully"
    );

    Ok(Json(QdrantBatchSearchResponse { result: results }))
}

/// Batch recommend points in a collection
pub async fn batch_recommend_points(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    Json(request): Json<QdrantBatchRecommendRequest>,
) -> Result<Json<QdrantBatchRecommendResponse>, ErrorResponse> {
    info!(
        collection = %collection_name,
        batch_size = request.searches.len(),
        "Batch recommending points in collection"
    );

    // Get collection from store
    let collection = state
        .store
        .get_collection(&collection_name)
        .map_err(|e| create_not_found_error("collection", &collection_name))?;

    // Process each recommend request in the batch
    let mut results = Vec::new();
    for (i, recommend_request) in request.searches.into_iter().enumerate() {
        // Convert point IDs to strings for retrieval
        let positive_ids: Vec<String> = recommend_request
            .positive
            .into_iter()
            .map(|id| match id {
                QdrantPointId::Numeric(n) => n.to_string(),
                QdrantPointId::Uuid(s) => s,
            })
            .collect();

        let negative_ids: Vec<String> = recommend_request
            .negative
            .unwrap_or_default()
            .into_iter()
            .map(|id| match id {
                QdrantPointId::Numeric(n) => n.to_string(),
                QdrantPointId::Uuid(s) => s,
            })
            .collect();

        // Retrieve positive examples
        let positive_vectors: Vec<Vec<f32>> = positive_ids
            .iter()
            .filter_map(|id| collection.get_vector(id).ok().map(|v| v.data))
            .collect();

        if positive_vectors.is_empty() {
            return Err(create_error_response(
                &format!(
                    "Recommend {}: At least one positive example must exist in the collection",
                    i
                ),
                "No valid positive examples found",
                StatusCode::BAD_REQUEST,
            ));
        }

        // Retrieve negative examples
        let _negative_vectors: Vec<Vec<f32>> = negative_ids
            .iter()
            .filter_map(|id| collection.get_vector(id).ok().map(|v| v.data))
            .collect();

        // Calculate recommendation vector based on strategy
        let recommendation_vector = match recommend_request
            .strategy
            .unwrap_or(QdrantRecommendStrategy::AverageVector)
        {
            QdrantRecommendStrategy::AverageVector => {
                // Average all positive vectors
                let mut avg_vector = vec![0.0; positive_vectors[0].len()];
                for vector in &positive_vectors {
                    for (i, &val) in vector.iter().enumerate() {
                        avg_vector[i] += val;
                    }
                }
                let count = positive_vectors.len() as f32;
                for val in &mut avg_vector {
                    *val /= count;
                }
                avg_vector
            }
            QdrantRecommendStrategy::BestScore => {
                // Use the first positive vector as the query
                positive_vectors[0].clone()
            }
        };

        // Set default limit if not provided
        let limit = recommend_request.limit.unwrap_or(10) as usize;
        let offset = recommend_request.offset.unwrap_or(0) as usize;

        // Perform search with the recommendation vector
        let search_results = collection
            .search(&recommendation_vector, limit + offset)
            .map_err(|e| {
                create_error_response(
                    &format!("Recommend {}: {}", i, e),
                    "Batch recommendation search failed",
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
            })?;

        // Apply offset, filters, and limit
        let batch_results: Vec<QdrantScoredPoint> = search_results
            .into_iter()
            .skip(offset)
            .take(limit)
            .filter(|result| {
                // Apply score threshold if provided
                if let Some(threshold) = recommend_request.score_threshold {
                    if result.score < threshold {
                        return false;
                    }
                }

                // Apply filters if provided
                if let Some(filter) = &recommend_request.filter {
                    if let Some(payload) = &result.payload {
                        return FilterProcessor::apply_filter(filter, payload);
                    } else {
                        // No payload, filter cannot match
                        return false;
                    }
                }

                true
            })
            .map(|result| {
                // Convert to QdrantScoredPoint
                let id = match result.id.parse::<u64>() {
                    Ok(numeric_id) => QdrantPointId::Numeric(numeric_id),
                    Err(_) => QdrantPointId::Uuid(result.id),
                };

                let vector = if recommend_request.with_vector.unwrap_or(false) {
                    Some(QdrantVector::Dense(result.vector.unwrap_or_default()))
                } else {
                    None
                };

                let payload = if recommend_request.with_payload.unwrap_or(true) {
                    result.payload.map(|p| {
                        p.data
                            .as_object()
                            .unwrap_or(&serde_json::Map::new())
                            .iter()
                            .map(|(k, v)| (k.clone(), json_value_to_qdrant_value(v.clone())))
                            .collect()
                    })
                } else {
                    None
                };

                QdrantScoredPoint {
                    id,
                    vector,
                    payload,
                    score: result.score,
                }
            })
            .collect();

        results.push(QdrantRecommendResponse {
            result: batch_results,
        });
    }

    info!(
        collection = %collection_name,
        batch_results = results.len(),
        "Batch recommendation completed successfully"
    );

    Ok(Json(QdrantBatchRecommendResponse { result: results }))
}
