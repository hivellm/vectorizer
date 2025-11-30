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
    QdrantBatchSearchRequest, QdrantBatchSearchResponse, QdrantDistancePair, QdrantGroupsResult,
    QdrantMatrixOffsetsResult, QdrantMatrixPairsResult, QdrantPointGroup, QdrantRecommendRequest,
    QdrantRecommendResponse, QdrantRecommendStrategy, QdrantScoredPoint, QdrantSearchGroupsRequest,
    QdrantSearchGroupsResponse, QdrantSearchMatrixOffsetsRequest,
    QdrantSearchMatrixOffsetsResponse, QdrantSearchMatrixPairsRequest,
    QdrantSearchMatrixPairsResponse, QdrantSearchRequest, QdrantSearchResponse, QdrantWithPayload,
    QdrantWithVector,
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

// =============================================================================
// Search Groups Handler
// =============================================================================

/// Search points and group results by a payload field
pub async fn search_points_groups(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    Json(request): Json<QdrantSearchGroupsRequest>,
) -> Result<Json<QdrantSearchGroupsResponse>, ErrorResponse> {
    info!(
        collection = %collection_name,
        group_by = %request.group_by,
        vector_dim = request.vector.len(),
        limit = ?request.limit,
        group_size = ?request.group_size,
        "Searching points with grouping"
    );

    // Get collection from store
    let collection = state
        .store
        .get_collection(&collection_name)
        .map_err(|_e| create_not_found_error("collection", &collection_name))?;

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

    // Defaults
    let group_limit = request.limit.unwrap_or(10) as usize;
    let group_size = request.group_size.unwrap_or(3) as usize;

    // Perform search with enough results to fill groups
    // We need at least group_limit * group_size results, but more to account for filtering
    let search_limit = group_limit * group_size * 3;
    let search_results = collection
        .search(&request.vector, search_limit)
        .map_err(|e| {
            create_error_response(
                &format!("{}", e),
                "Search failed",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

    // Group results by the group_by field
    let mut groups_map: HashMap<String, Vec<QdrantScoredPoint>> = HashMap::new();

    for result in search_results {
        // Apply score threshold if provided
        if let Some(threshold) = request.score_threshold {
            if result.score < threshold {
                continue;
            }
        }

        // Apply filters if provided
        if let Some(filter) = &request.filter {
            if let Some(payload) = &result.payload {
                if !FilterProcessor::apply_filter(filter, payload) {
                    continue;
                }
            } else {
                continue;
            }
        }

        // Extract group value from payload
        let group_value = if let Some(payload) = &result.payload {
            payload
                .data
                .get(&request.group_by)
                .map(|v| match v {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => v.to_string(),
                })
                .unwrap_or_else(|| "__null__".to_string())
        } else {
            "__null__".to_string()
        };

        // Check if we have enough groups and this would create a new one
        let group_exists = groups_map.contains_key(&group_value);
        if groups_map.len() >= group_limit && !group_exists {
            continue;
        }

        // Check if we still need more groups or items in existing groups
        let group_entries = groups_map.entry(group_value).or_insert_with(Vec::new);
        if group_entries.len() >= group_size {
            continue;
        }

        // Convert to QdrantScoredPoint
        let id = match result.id.parse::<u64>() {
            Ok(numeric_id) => QdrantPointId::Numeric(numeric_id),
            Err(_) => QdrantPointId::Uuid(result.id),
        };

        let with_vector = match &request.with_vector {
            Some(QdrantWithVector::Bool(b)) => *b,
            Some(QdrantWithVector::Include(_)) => true,
            None => false,
        };

        let vector = if with_vector {
            Some(QdrantVector::Dense(result.vector.unwrap_or_default()))
        } else {
            None
        };

        let with_payload = match &request.with_payload {
            Some(QdrantWithPayload::Bool(b)) => *b,
            Some(_) => true,
            None => true,
        };

        let payload = if with_payload {
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

        group_entries.push(QdrantScoredPoint {
            id,
            vector,
            payload,
            score: result.score,
        });
    }

    // Convert groups_map to response format
    let groups: Vec<QdrantPointGroup> = groups_map
        .into_iter()
        .take(group_limit)
        .map(|(group_id, hits)| QdrantPointGroup {
            id: serde_json::Value::String(group_id),
            hits,
            lookup: None, // with_lookup not implemented yet
        })
        .collect();

    info!(
        collection = %collection_name,
        groups_count = groups.len(),
        "Search groups completed successfully"
    );

    Ok(Json(QdrantSearchGroupsResponse {
        result: QdrantGroupsResult { groups },
    }))
}

// =============================================================================
// Search Matrix Handlers
// =============================================================================

/// Compute pairwise distances between sampled points
pub async fn search_matrix_pairs(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    Json(request): Json<QdrantSearchMatrixPairsRequest>,
) -> Result<Json<QdrantSearchMatrixPairsResponse>, ErrorResponse> {
    let sample_size = request.sample.unwrap_or(10) as usize;
    let limit = request.limit.unwrap_or(100) as usize;

    info!(
        collection = %collection_name,
        sample_size = sample_size,
        limit = limit,
        "Computing search matrix pairs"
    );

    // Get collection from store
    let collection = state
        .store
        .get_collection(&collection_name)
        .map_err(|_e| create_not_found_error("collection", &collection_name))?;

    // Get all vectors from collection
    let all_vectors = collection.get_all_vectors();

    // Apply filter if provided and sample points
    let filtered_vectors: Vec<_> = all_vectors
        .into_iter()
        .filter(|v| {
            if let Some(filter) = &request.filter {
                if let Some(payload) = &v.payload {
                    FilterProcessor::apply_filter(filter, payload)
                } else {
                    false
                }
            } else {
                true
            }
        })
        .take(sample_size)
        .collect();

    if filtered_vectors.len() < 2 {
        return Ok(Json(QdrantSearchMatrixPairsResponse {
            result: QdrantMatrixPairsResult { pairs: vec![] },
        }));
    }

    // Compute pairwise distances
    let metric = collection.config().metric;
    let mut pairs: Vec<QdrantDistancePair> = Vec::new();

    for i in 0..filtered_vectors.len() {
        for j in (i + 1)..filtered_vectors.len() {
            if pairs.len() >= limit {
                break;
            }

            let vec_a = &filtered_vectors[i];
            let vec_b = &filtered_vectors[j];

            // Compute distance/similarity
            let score = compute_similarity(&vec_a.data, &vec_b.data, &metric);

            let id_a = match vec_a.id.parse::<u64>() {
                Ok(n) => QdrantPointId::Numeric(n),
                Err(_) => QdrantPointId::Uuid(vec_a.id.clone()),
            };

            let id_b = match vec_b.id.parse::<u64>() {
                Ok(n) => QdrantPointId::Numeric(n),
                Err(_) => QdrantPointId::Uuid(vec_b.id.clone()),
            };

            pairs.push(QdrantDistancePair {
                a: id_a,
                b: id_b,
                score,
            });
        }

        if pairs.len() >= limit {
            break;
        }
    }

    info!(
        collection = %collection_name,
        pairs_count = pairs.len(),
        "Search matrix pairs completed"
    );

    Ok(Json(QdrantSearchMatrixPairsResponse {
        result: QdrantMatrixPairsResult { pairs },
    }))
}

/// Compute distances as sparse offset matrix
pub async fn search_matrix_offsets(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    Json(request): Json<QdrantSearchMatrixOffsetsRequest>,
) -> Result<Json<QdrantSearchMatrixOffsetsResponse>, ErrorResponse> {
    let sample_size = request.sample.unwrap_or(10) as usize;
    let limit = request.limit.unwrap_or(100) as usize;

    info!(
        collection = %collection_name,
        sample_size = sample_size,
        limit = limit,
        "Computing search matrix offsets"
    );

    // Get collection from store
    let collection = state
        .store
        .get_collection(&collection_name)
        .map_err(|_e| create_not_found_error("collection", &collection_name))?;

    // Get all vectors from collection
    let all_vectors = collection.get_all_vectors();

    // Apply filter if provided and sample points
    let filtered_vectors: Vec<_> = all_vectors
        .into_iter()
        .filter(|v| {
            if let Some(filter) = &request.filter {
                if let Some(payload) = &v.payload {
                    FilterProcessor::apply_filter(filter, payload)
                } else {
                    false
                }
            } else {
                true
            }
        })
        .take(sample_size)
        .collect();

    if filtered_vectors.is_empty() {
        return Ok(Json(QdrantSearchMatrixOffsetsResponse {
            result: QdrantMatrixOffsetsResult {
                ids: vec![],
                offsets: vec![],
                scores: vec![],
            },
        }));
    }

    let n = filtered_vectors.len();
    let metric = collection.config().metric;

    // Build IDs list
    let ids: Vec<QdrantPointId> = filtered_vectors
        .iter()
        .map(|v| match v.id.parse::<u64>() {
            Ok(n) => QdrantPointId::Numeric(n),
            Err(_) => QdrantPointId::Uuid(v.id.clone()),
        })
        .collect();

    // Build sparse matrix representation
    // For each row i, we store scores for columns j > i (upper triangular)
    let mut offsets: Vec<u64> = Vec::with_capacity(n + 1);
    let mut scores: Vec<f32> = Vec::new();
    let mut total_entries = 0u64;

    for i in 0..n {
        offsets.push(total_entries);

        for j in (i + 1)..n {
            if scores.len() >= limit {
                break;
            }

            let score = compute_similarity(
                &filtered_vectors[i].data,
                &filtered_vectors[j].data,
                &metric,
            );
            scores.push(score);
            total_entries += 1;
        }

        if scores.len() >= limit {
            break;
        }
    }

    // Add final offset
    offsets.push(total_entries);

    info!(
        collection = %collection_name,
        ids_count = ids.len(),
        scores_count = scores.len(),
        "Search matrix offsets completed"
    );

    Ok(Json(QdrantSearchMatrixOffsetsResponse {
        result: QdrantMatrixOffsetsResult {
            ids,
            offsets,
            scores,
        },
    }))
}

/// Compute similarity between two vectors based on metric
fn compute_similarity(a: &[f32], b: &[f32], metric: &crate::models::DistanceMetric) -> f32 {
    use crate::models::DistanceMetric;

    match metric {
        DistanceMetric::Cosine => {
            let mut dot = 0.0f32;
            let mut norm_a = 0.0f32;
            let mut norm_b = 0.0f32;
            for (x, y) in a.iter().zip(b.iter()) {
                dot += x * y;
                norm_a += x * x;
                norm_b += y * y;
            }
            let denom = (norm_a.sqrt() * norm_b.sqrt()).max(1e-10);
            dot / denom
        }
        DistanceMetric::Euclidean => {
            let mut sum = 0.0f32;
            for (x, y) in a.iter().zip(b.iter()) {
                let diff = x - y;
                sum += diff * diff;
            }
            // Return negative distance so higher is better
            -sum.sqrt()
        }
        DistanceMetric::DotProduct => {
            let mut dot = 0.0f32;
            for (x, y) in a.iter().zip(b.iter()) {
                dot += x * y;
            }
            dot
        }
    }
}
