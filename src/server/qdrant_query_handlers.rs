//! Qdrant Query API handlers (Qdrant 1.7+)
//!
//! This module provides handlers for the unified Query API introduced in Qdrant 1.7.
//! Includes support for prefetch operations for multi-stage retrieval.

use std::collections::{HashMap, HashSet};

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use tracing::{debug, error, info, warn};

use super::VectorizerServer;
use super::error_middleware::{ErrorResponse, create_error_response, create_not_found_error};
use crate::models::qdrant::point::{QdrantPointId, QdrantValue, QdrantVector};
use crate::models::qdrant::{
    FilterProcessor, QdrantBatchQueryRequest, QdrantBatchQueryResponse, QdrantComplexQuery,
    QdrantFilter, QdrantGroupsResult, QdrantPointGroup, QdrantPrefetch, QdrantQuery,
    QdrantQueryGroupsRequest, QdrantQueryGroupsResponse, QdrantQueryRequest, QdrantQueryResponse,
    QdrantScoredPoint, QdrantVectorInput, QdrantWithPayload, QdrantWithVector,
};

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

/// Check if we should include payload based on QdrantWithPayload
fn should_include_payload(with_payload: &Option<QdrantWithPayload>) -> bool {
    match with_payload {
        None => true, // Default to including payload
        Some(QdrantWithPayload::Bool(b)) => *b,
        Some(QdrantWithPayload::Include(_)) => true,
        Some(QdrantWithPayload::Selector(_)) => true,
    }
}

/// Check if we should include vector based on QdrantWithVector
fn should_include_vector(with_vector: &Option<QdrantWithVector>) -> bool {
    match with_vector {
        None => false, // Default to not including vector
        Some(QdrantWithVector::Bool(b)) => *b,
        Some(QdrantWithVector::Include(_)) => true,
    }
}

/// Extract query vector from QdrantQuery
fn extract_query_vector(
    query: &QdrantQuery,
    collection: &crate::db::CollectionType,
) -> Result<Vec<f32>, String> {
    match query {
        QdrantQuery::Vector(v) => Ok(v.clone()),
        QdrantQuery::PointId(id) => {
            // Lookup the point and get its vector
            let id_str = match id {
                QdrantPointId::Numeric(n) => n.to_string(),
                QdrantPointId::Uuid(s) => s.clone(),
            };
            collection
                .get_vector(&id_str)
                .map(|v| v.data)
                .map_err(|e| format!("Failed to lookup point {}: {}", id_str, e))
        }
        QdrantQuery::Complex(complex) => match complex.as_ref() {
            QdrantComplexQuery::Nearest(nearest) => {
                extract_vector_input(&nearest.nearest, collection)
            }
            QdrantComplexQuery::Recommend(recommend) => {
                // For recommend, average the positive vectors
                let positive = recommend
                    .positive
                    .as_ref()
                    .ok_or("Recommend query requires positive examples")?;
                if positive.is_empty() {
                    return Err(
                        "Recommend query requires at least one positive example".to_string()
                    );
                }

                let mut sum_vector: Option<Vec<f32>> = None;
                let mut count = 0;

                for input in positive {
                    let vec = extract_vector_input(input, collection)?;
                    if let Some(ref mut sum) = sum_vector {
                        for (i, val) in vec.iter().enumerate() {
                            if i < sum.len() {
                                sum[i] += val;
                            }
                        }
                    } else {
                        sum_vector = Some(vec);
                    }
                    count += 1;
                }

                let mut avg = sum_vector.unwrap();
                for val in &mut avg {
                    *val /= count as f32;
                }

                // Subtract negative vectors if present
                if let Some(negative) = &recommend.negative {
                    for input in negative {
                        if let Ok(neg_vec) = extract_vector_input(input, collection) {
                            for (i, val) in neg_vec.iter().enumerate() {
                                if i < avg.len() {
                                    avg[i] -= val / count as f32;
                                }
                            }
                        }
                    }
                }

                Ok(avg)
            }
            QdrantComplexQuery::Discover(discover) => {
                // For discover, use the target vector
                extract_vector_input(&discover.target, collection)
            }
            QdrantComplexQuery::Context(context) => {
                // For context, average all positive vectors from context pairs
                if context.context.is_empty() {
                    return Err("Context query requires at least one context pair".to_string());
                }

                let mut sum_vector: Option<Vec<f32>> = None;
                let mut count = 0;

                for pair in &context.context {
                    let vec = extract_vector_input(&pair.positive, collection)?;
                    if let Some(ref mut sum) = sum_vector {
                        for (i, val) in vec.iter().enumerate() {
                            if i < sum.len() {
                                sum[i] += val;
                            }
                        }
                    } else {
                        sum_vector = Some(vec);
                    }
                    count += 1;
                }

                let mut avg = sum_vector.unwrap();
                for val in &mut avg {
                    *val /= count as f32;
                }
                Ok(avg)
            }
            QdrantComplexQuery::Fusion(_) => {
                Err("Fusion queries are not yet supported".to_string())
            }
            QdrantComplexQuery::OrderBy(_) => {
                Err("OrderBy queries require payload ordering, not vector search".to_string())
            }
        },
    }
}

/// Extract vector from QdrantVectorInput
fn extract_vector_input(
    input: &QdrantVectorInput,
    collection: &crate::db::CollectionType,
) -> Result<Vec<f32>, String> {
    match input {
        QdrantVectorInput::Vector(v) => Ok(v.clone()),
        QdrantVectorInput::PointId(id) => {
            let id_str = match id {
                QdrantPointId::Numeric(n) => n.to_string(),
                QdrantPointId::Uuid(s) => s.clone(),
            };
            collection
                .get_vector(&id_str)
                .map(|v| v.data)
                .map_err(|e| format!("Failed to lookup point {}: {}", id_str, e))
        }
    }
}

/// Process prefetch operations recursively and return candidate point IDs
///
/// Prefetch is a multi-stage retrieval feature that:
/// 1. Executes sub-queries first to get candidate points
/// 2. Uses those candidates as the search space for subsequent queries
/// 3. Supports nested prefetch for multi-stage pipelines
fn process_prefetch(
    prefetch_list: &[QdrantPrefetch],
    collection: &crate::db::CollectionType,
    config: &crate::models::CollectionConfig,
) -> Result<HashSet<String>, String> {
    let mut candidate_ids: HashSet<String> = HashSet::new();

    for prefetch in prefetch_list {
        // First, process any nested prefetch (recursive)
        let nested_candidates = if let Some(nested_prefetch) = &prefetch.prefetch {
            process_prefetch(nested_prefetch, collection, config)?
        } else {
            HashSet::new()
        };

        // Get the limit for this prefetch stage
        let prefetch_limit = prefetch.limit.unwrap_or(100) as usize;

        // Execute the prefetch query if provided
        if let Some(query) = &prefetch.query {
            let query_vector = extract_query_vector(query, collection)?;

            // Validate vector dimension
            if query_vector.len() != config.dimension {
                return Err(format!(
                    "Prefetch: Expected dimension {}, got {}",
                    config.dimension,
                    query_vector.len()
                ));
            }

            // Perform the prefetch search
            let search_results = collection
                .search(&query_vector, prefetch_limit * 2) // Over-fetch to account for filtering
                .map_err(|e| format!("Prefetch search failed: {}", e))?;

            // Filter results based on prefetch configuration
            for result in search_results.into_iter().take(prefetch_limit) {
                // Apply score threshold if provided
                if let Some(threshold) = prefetch.score_threshold {
                    if result.score < threshold {
                        continue;
                    }
                }

                // Apply filter if provided
                if let Some(filter) = &prefetch.filter {
                    if let Some(payload) = &result.payload {
                        if !FilterProcessor::apply_filter(filter, payload) {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }

                // If we have nested candidates, only include results that match
                if !nested_candidates.is_empty() && !nested_candidates.contains(&result.id) {
                    continue;
                }

                candidate_ids.insert(result.id);
            }
        } else if !nested_candidates.is_empty() {
            // No query in this prefetch, just pass through nested candidates
            candidate_ids.extend(nested_candidates);
        }
    }

    debug!(
        "Prefetch completed: {} candidate IDs collected",
        candidate_ids.len()
    );
    Ok(candidate_ids)
}

/// Query points in a collection (Qdrant 1.7+ unified Query API)
/// POST /qdrant/collections/{name}/points/query
///
/// Supports prefetch for multi-stage retrieval:
/// - Prefetch queries are executed first to collect candidate point IDs
/// - The main query then searches within those candidates
pub async fn query_points(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    Json(request): Json<QdrantQueryRequest>,
) -> Result<Json<QdrantQueryResponse>, ErrorResponse> {
    let has_prefetch = request.prefetch.is_some();
    info!(
        collection = %collection_name,
        has_query = request.query.is_some(),
        has_prefetch = has_prefetch,
        limit = ?request.limit,
        "Query API: Querying points in collection"
    );

    // Get collection from store
    let collection = state
        .store
        .get_collection(&collection_name)
        .map_err(|_| create_not_found_error("collection", &collection_name))?;

    let config = collection.config();
    let limit = request.limit.unwrap_or(10) as usize;
    let offset = request.offset.unwrap_or(0) as usize;

    // Process prefetch if provided - this gives us a set of candidate IDs
    let prefetch_candidates: Option<HashSet<String>> =
        if let Some(prefetch_list) = &request.prefetch {
            if !prefetch_list.is_empty() {
                let candidates =
                    process_prefetch(prefetch_list, &collection, &config).map_err(|e| {
                        create_error_response(&e, "Prefetch failed", StatusCode::BAD_REQUEST)
                    })?;

                if candidates.is_empty() {
                    warn!(
                        collection = %collection_name,
                        "Prefetch returned no candidates, returning empty results"
                    );
                    return Ok(Json(QdrantQueryResponse { result: vec![] }));
                }

                debug!(
                    collection = %collection_name,
                    candidate_count = candidates.len(),
                    "Prefetch collected candidates for re-ranking"
                );
                Some(candidates)
            } else {
                None
            }
        } else {
            None
        };

    // If no query provided, return empty results (Qdrant behavior)
    let query = match &request.query {
        Some(q) => q,
        None => {
            return Ok(Json(QdrantQueryResponse { result: vec![] }));
        }
    };

    // Extract query vector from the query type
    let query_vector = extract_query_vector(query, &collection)
        .map_err(|e| create_error_response(&e, "Invalid query", StatusCode::BAD_REQUEST))?;

    // Validate vector dimension
    if query_vector.len() != config.dimension {
        return Err(create_error_response(
            &format!(
                "Expected dimension {}, got {}",
                config.dimension,
                query_vector.len()
            ),
            "Vector dimension mismatch",
            StatusCode::BAD_REQUEST,
        ));
    }

    // Perform search - if we have prefetch candidates, we need more results to filter
    let search_limit = if prefetch_candidates.is_some() {
        // When using prefetch, we're filtering, so fetch more to account for filtering loss
        (limit + offset) * 3
    } else {
        limit + offset
    };

    let search_results = collection
        .search(&query_vector, search_limit)
        .map_err(|e| {
            create_error_response(
                &format!("{}", e),
                "Query failed",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

    // Apply offset, filters, prefetch candidates, and convert to response format
    let include_payload = should_include_payload(&request.with_payload);
    let include_vector = should_include_vector(&request.with_vector);

    let results: Vec<QdrantScoredPoint> = search_results
        .into_iter()
        .filter(|result| {
            // Apply prefetch candidate filter first (most selective)
            if let Some(ref candidates) = prefetch_candidates {
                if !candidates.contains(&result.id) {
                    return false;
                }
            }

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
                    return false;
                }
            }

            true
        })
        .skip(offset)
        .take(limit)
        .map(|result| {
            let id = match result.id.parse::<u64>() {
                Ok(numeric_id) => QdrantPointId::Numeric(numeric_id),
                Err(_) => QdrantPointId::Uuid(result.id),
            };

            let vector = if include_vector {
                Some(QdrantVector::Dense(result.vector.unwrap_or_default()))
            } else {
                None
            };

            let payload = if include_payload {
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
        used_prefetch = prefetch_candidates.is_some(),
        "Query API: Query completed successfully"
    );

    Ok(Json(QdrantQueryResponse { result: results }))
}

/// Batch query points in a collection
/// POST /qdrant/collections/{name}/points/query/batch
///
/// Supports prefetch for multi-stage retrieval per batch item
pub async fn batch_query_points(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    Json(request): Json<QdrantBatchQueryRequest>,
) -> Result<Json<QdrantBatchQueryResponse>, ErrorResponse> {
    info!(
        collection = %collection_name,
        batch_size = request.searches.len(),
        "Query API: Batch querying points in collection"
    );

    // Get collection from store
    let collection = state
        .store
        .get_collection(&collection_name)
        .map_err(|_| create_not_found_error("collection", &collection_name))?;

    let config = collection.config();

    let mut results = Vec::new();
    for (i, query_request) in request.searches.into_iter().enumerate() {
        let limit = query_request.limit.unwrap_or(10) as usize;
        let offset = query_request.offset.unwrap_or(0) as usize;

        // Process prefetch if provided for this batch item
        let prefetch_candidates: Option<HashSet<String>> =
            if let Some(prefetch_list) = &query_request.prefetch {
                if !prefetch_list.is_empty() {
                    match process_prefetch(prefetch_list, &collection, &config) {
                        Ok(candidates) => {
                            if candidates.is_empty() {
                                // Prefetch returned nothing, skip to empty result
                                results.push(QdrantQueryResponse { result: vec![] });
                                continue;
                            }
                            Some(candidates)
                        }
                        Err(e) => {
                            return Err(create_error_response(
                                &format!("Query {}: Prefetch failed: {}", i, e),
                                "Prefetch failed",
                                StatusCode::BAD_REQUEST,
                            ));
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            };

        // If no query provided, return empty results
        let query = match &query_request.query {
            Some(q) => q,
            None => {
                results.push(QdrantQueryResponse { result: vec![] });
                continue;
            }
        };

        // Extract query vector
        let query_vector = match extract_query_vector(query, &collection) {
            Ok(v) => v,
            Err(e) => {
                return Err(create_error_response(
                    &format!("Query {}: {}", i, e),
                    "Invalid query",
                    StatusCode::BAD_REQUEST,
                ));
            }
        };

        // Validate vector dimension
        if query_vector.len() != config.dimension {
            return Err(create_error_response(
                &format!(
                    "Query {}: Expected dimension {}, got {}",
                    i,
                    config.dimension,
                    query_vector.len()
                ),
                "Vector dimension mismatch",
                StatusCode::BAD_REQUEST,
            ));
        }

        // Perform search - increase limit if using prefetch
        let search_limit = if prefetch_candidates.is_some() {
            (limit + offset) * 3
        } else {
            limit + offset
        };

        let search_results = collection
            .search(&query_vector, search_limit)
            .map_err(|e| {
                create_error_response(
                    &format!("Query {}: {}", i, e),
                    "Query failed",
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
            })?;

        let include_payload = should_include_payload(&query_request.with_payload);
        let include_vector = should_include_vector(&query_request.with_vector);

        let batch_results: Vec<QdrantScoredPoint> = search_results
            .into_iter()
            .filter(|result| {
                // Apply prefetch candidate filter first
                if let Some(ref candidates) = prefetch_candidates {
                    if !candidates.contains(&result.id) {
                        return false;
                    }
                }

                if let Some(threshold) = query_request.score_threshold {
                    if result.score < threshold {
                        return false;
                    }
                }

                if let Some(filter) = &query_request.filter {
                    if let Some(payload) = &result.payload {
                        return FilterProcessor::apply_filter(filter, payload);
                    } else {
                        return false;
                    }
                }

                true
            })
            .skip(offset)
            .take(limit)
            .map(|result| {
                let id = match result.id.parse::<u64>() {
                    Ok(numeric_id) => QdrantPointId::Numeric(numeric_id),
                    Err(_) => QdrantPointId::Uuid(result.id),
                };

                let vector = if include_vector {
                    Some(QdrantVector::Dense(result.vector.unwrap_or_default()))
                } else {
                    None
                };

                let payload = if include_payload {
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

        results.push(QdrantQueryResponse {
            result: batch_results,
        });
    }

    info!(
        collection = %collection_name,
        batch_results = results.len(),
        "Query API: Batch query completed successfully"
    );

    Ok(Json(QdrantBatchQueryResponse { result: results }))
}

/// Query points with grouping
/// POST /qdrant/collections/{name}/points/query/groups
///
/// Supports prefetch for multi-stage retrieval with grouped results
pub async fn query_points_groups(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    Json(request): Json<QdrantQueryGroupsRequest>,
) -> Result<Json<QdrantQueryGroupsResponse>, ErrorResponse> {
    let has_prefetch = request.prefetch.is_some();
    info!(
        collection = %collection_name,
        group_by = %request.group_by,
        has_query = request.query.is_some(),
        has_prefetch = has_prefetch,
        limit = ?request.limit,
        "Query API: Querying points with groups"
    );

    // Get collection from store
    let collection = state
        .store
        .get_collection(&collection_name)
        .map_err(|_| create_not_found_error("collection", &collection_name))?;

    let config = collection.config();
    let group_limit = request.limit.unwrap_or(10) as usize;
    let group_size = request.group_size.unwrap_or(3) as usize;

    // Process prefetch if provided
    let prefetch_candidates: Option<HashSet<String>> =
        if let Some(prefetch_list) = &request.prefetch {
            if !prefetch_list.is_empty() {
                let candidates =
                    process_prefetch(prefetch_list, &collection, &config).map_err(|e| {
                        create_error_response(&e, "Prefetch failed", StatusCode::BAD_REQUEST)
                    })?;

                if candidates.is_empty() {
                    warn!(
                        collection = %collection_name,
                        "Prefetch returned no candidates for groups query"
                    );
                    return Ok(Json(QdrantQueryGroupsResponse {
                        result: QdrantGroupsResult { groups: vec![] },
                    }));
                }
                Some(candidates)
            } else {
                None
            }
        } else {
            None
        };

    // If no query provided, return empty groups
    let query = match &request.query {
        Some(q) => q,
        None => {
            return Ok(Json(QdrantQueryGroupsResponse {
                result: QdrantGroupsResult { groups: vec![] },
            }));
        }
    };

    // Extract query vector
    let query_vector = extract_query_vector(query, &collection)
        .map_err(|e| create_error_response(&e, "Invalid query", StatusCode::BAD_REQUEST))?;

    // Validate vector dimension
    if query_vector.len() != config.dimension {
        return Err(create_error_response(
            &format!(
                "Expected dimension {}, got {}",
                config.dimension,
                query_vector.len()
            ),
            "Vector dimension mismatch",
            StatusCode::BAD_REQUEST,
        ));
    }

    // Perform search with a larger limit to account for grouping
    // Increase further if using prefetch to account for filtering
    let search_limit = if prefetch_candidates.is_some() {
        group_limit * group_size * 5
    } else {
        group_limit * group_size * 3
    };
    let search_results = collection
        .search(&query_vector, search_limit)
        .map_err(|e| {
            create_error_response(
                &format!("{}", e),
                "Query failed",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

    let include_payload = should_include_payload(&request.with_payload);
    let include_vector = should_include_vector(&request.with_vector);

    // Group results by the specified field
    let mut groups: HashMap<String, Vec<QdrantScoredPoint>> = HashMap::new();

    for result in search_results {
        // Apply prefetch candidate filter first
        if let Some(ref candidates) = prefetch_candidates {
            if !candidates.contains(&result.id) {
                continue;
            }
        }

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

        // Extract group key from payload
        let group_key = if let Some(payload) = &result.payload {
            payload
                .data
                .get(&request.group_by)
                .map(|v| v.to_string())
                .unwrap_or_else(|| "null".to_string())
        } else {
            "null".to_string()
        };

        // Check if this group already has enough results
        let group = groups.entry(group_key).or_insert_with(Vec::new);
        if group.len() >= group_size {
            continue;
        }

        // Convert to QdrantScoredPoint
        let id = match result.id.parse::<u64>() {
            Ok(numeric_id) => QdrantPointId::Numeric(numeric_id),
            Err(_) => QdrantPointId::Uuid(result.id),
        };

        let vector = if include_vector {
            Some(QdrantVector::Dense(result.vector.unwrap_or_default()))
        } else {
            None
        };

        let payload = if include_payload {
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

        group.push(QdrantScoredPoint {
            id,
            vector,
            payload,
            score: result.score,
        });

        // Check if we have enough groups
        if groups.len() >= group_limit && groups.values().all(|g| g.len() >= group_size) {
            break;
        }
    }

    // Convert HashMap to Vec<QdrantPointGroup>
    let mut point_groups: Vec<QdrantPointGroup> = groups
        .into_iter()
        .map(|(id, hits)| QdrantPointGroup {
            id: serde_json::Value::String(id),
            hits,
            lookup: None,
        })
        .collect();

    // Sort groups by the best score in each group
    point_groups.sort_by(|a, b| {
        let a_best = a.hits.first().map(|h| h.score).unwrap_or(0.0);
        let b_best = b.hits.first().map(|h| h.score).unwrap_or(0.0);
        b_best
            .partial_cmp(&a_best)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Limit to requested number of groups
    point_groups.truncate(group_limit);

    info!(
        collection = %collection_name,
        groups_count = point_groups.len(),
        "Query API: Query groups completed successfully"
    );

    Ok(Json(QdrantQueryGroupsResponse {
        result: QdrantGroupsResult {
            groups: point_groups,
        },
    }))
}
