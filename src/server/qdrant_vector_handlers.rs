//! Qdrant vector operations REST API handlers

use std::collections::HashMap;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde_json::{Value, json};
use tracing::{debug, error, info};

use super::VectorizerServer;
use super::error_middleware::{ErrorResponse, create_error_response, create_not_found_error};
use crate::error::VectorizerError;
use crate::models::qdrant::{
    PointCountResult as QdrantCountResult, PointOperationStatus as QdrantOperationStatus,
    PointScrollResult as QdrantScrollResult, QdrantCountPointsRequest, QdrantCountPointsResponse,
    QdrantDeletePointsRequest, QdrantPointCountRequest, QdrantPointCountResponse,
    QdrantPointDeleteRequest, QdrantPointId, QdrantPointOperationResult,
    QdrantPointRetrieveRequest, QdrantPointRetrieveResponse, QdrantPointScrollRequest,
    QdrantPointScrollResponse, QdrantPointStruct, QdrantPointUpsertRequest,
    QdrantRetrievePointsRequest, QdrantRetrievePointsResponse, QdrantScrollPointsRequest,
    QdrantScrollPointsResponse, QdrantUpsertPointsRequest, QdrantValue, QdrantVector,
};
use crate::models::{Payload, Vector};

/// Convert QdrantValue to serde_json::Value
fn qdrant_value_to_json_value(value: QdrantValue) -> serde_json::Value {
    match value {
        QdrantValue::String(s) => serde_json::Value::String(s),
        QdrantValue::Integer(i) => serde_json::Value::Number(serde_json::Number::from(i)),
        QdrantValue::Float(f) => serde_json::Value::Number(
            serde_json::Number::from_f64(f).unwrap_or(serde_json::Number::from(0)),
        ),
        QdrantValue::Boolean(b) => serde_json::Value::Bool(b),
        QdrantValue::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(qdrant_value_to_json_value).collect())
        }
        QdrantValue::Object(obj) => serde_json::Value::Object(
            obj.into_iter()
                .map(|(k, v)| (k, qdrant_value_to_json_value(v)))
                .collect(),
        ),
        QdrantValue::Null => serde_json::Value::Null,
    }
}

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

/// Upsert points to a collection
pub async fn upsert_points(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    req: axum::extract::Request,
) -> Result<Json<QdrantPointOperationResult>, ErrorResponse> {
    // Read body with configurable limit
    let max_size_bytes = state.max_request_size_mb * 1024 * 1024;
    let body_bytes = axum::body::to_bytes(req.into_body(), max_size_bytes)
        .await
        .map_err(|e| {
            error!("Failed to read request body: {}", e);
            if e.to_string().contains("too large") || e.to_string().contains("limit") {
                create_error_response(
                    "payload_too_large",
                    &format!("Request body exceeds {}MB limit", state.max_request_size_mb),
                    StatusCode::PAYLOAD_TOO_LARGE,
                )
            } else {
                create_error_response(
                    "bad_request",
                    &format!("Failed to read request body: {}", e),
                    StatusCode::BAD_REQUEST,
                )
            }
        })?;

    // Validate body size
    if body_bytes.len() > max_size_bytes {
        return Err(create_error_response(
            "payload_too_large",
            &format!(
                "Request body exceeds {}MB limit (got {}MB)",
                state.max_request_size_mb,
                body_bytes.len() / 1024 / 1024
            ),
            StatusCode::PAYLOAD_TOO_LARGE,
        ));
    }

    let request: QdrantPointUpsertRequest = serde_json::from_slice(&body_bytes).map_err(|e| {
        error!("Failed to parse request JSON: {}", e);
        create_error_response(
            "bad_request",
            &format!("Invalid JSON: {}", e),
            StatusCode::BAD_REQUEST,
        )
    })?;
    info!(
        "🔵 [UPSERT] Received request to upsert {} points to collection: {}",
        request.points.len(),
        collection_name
    );
    debug!(
        "Upserting {} points to collection: {}",
        request.points.len(),
        collection_name
    );

    // Validate collection exists
    let collection = state
        .store
        .get_collection(&collection_name)
        .map_err(|_| create_not_found_error("collection", &collection_name))?;

    let config = collection.config();
    info!(
        "Collection '{}' expects dimension: {}",
        collection_name, config.dimension
    );

    let mut vectors = Vec::new();

    let points_count = request.points.len();
    for (idx, point) in request.points.into_iter().enumerate() {
        // Log vector dimension before conversion
        let vector_dim = match &point.vector {
            QdrantVector::Dense(data) => data.len(),
            QdrantVector::Named(_) => 0,
        };
        debug!(
            "Point {}: vector dimension = {}, expected = {}",
            idx, vector_dim, config.dimension
        );

        // Convert Qdrant point to Vectorizer vector
        let vector = convert_qdrant_point_to_vector(point, &config).map_err(|e| {
            error!(
                "Failed to convert point {}: dimension mismatch or invalid format",
                idx
            );
            e
        })?;
        vectors.push(vector);
    }

    info!(
        "Successfully converted {} points, ready to insert",
        vectors.len()
    );

    // Fire-and-forget: Return response immediately and process in background
    // This improves response time for large batches
    let store_clone = state.store.clone();
    let collection_name_for_bg = collection_name.clone();
    let points_count_for_bg = points_count;

    // Spawn background task for insertion (fire-and-forget)
    tokio::spawn(async move {
        let start_time = std::time::Instant::now();
        let collection_name_bg = collection_name_for_bg.clone();

        // Run the synchronous insert in a blocking task
        let insert_result = tokio::task::spawn_blocking(move || {
            store_clone.insert(&collection_name_for_bg, vectors)
        })
        .await;

        match insert_result {
            Ok(Ok(_)) => {
                let duration = start_time.elapsed();
                info!(
                    "Successfully upserted {} points to collection: {} in {:.3}s (background)",
                    points_count_for_bg,
                    collection_name_bg,
                    duration.as_secs_f64()
                );
            }
            Ok(Err(e)) => {
                error!(
                    "Failed to upsert points to collection '{}' (background): {}",
                    collection_name_bg, e
                );
            }
            Err(e) => {
                error!(
                    "Insert task join error for collection '{}' (background): {}",
                    collection_name_bg, e
                );
            }
        }
    });

    // Return immediately - insertion happens in background
    info!(
        "Accepted {} points for insertion to collection: {} (processing in background)",
        points_count, collection_name
    );

    Ok(Json(QdrantPointOperationResult {
        status: QdrantOperationStatus::Acknowledged,
        operation_id: None,
    }))
}

/// Retrieve points from a collection
pub async fn retrieve_points(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    Json(request): Json<QdrantPointRetrieveRequest>,
) -> Result<Json<QdrantPointRetrieveResponse>, ErrorResponse> {
    debug!(
        "Retrieving {} points from collection: {}",
        request.ids.len(),
        collection_name
    );

    // Validate collection exists
    let collection = state
        .store
        .get_collection(&collection_name)
        .map_err(|_| create_not_found_error("collection", &collection_name))?;

    let mut points = Vec::new();

    let with_payload = request.with_payload.unwrap_or(true);
    let with_vector = request.with_vector.unwrap_or(false);

    let start_time = std::time::Instant::now();
    for point_id in request.ids {
        let id_str = match point_id {
            QdrantPointId::Numeric(n) => n.to_string(),
            QdrantPointId::Uuid(s) => s,
        };

        match collection.get_vector(&id_str) {
            Ok(vector) => {
                let qdrant_point =
                    convert_vector_to_qdrant_point(vector, with_payload, with_vector);
                points.push(qdrant_point);
            }
            Err(_) => {
                // Point not found, skip it
                debug!(
                    "Point '{}' not found in collection '{}'",
                    id_str, collection_name
                );
            }
        }
    }

    let duration = start_time.elapsed();
    Ok(Json(QdrantPointRetrieveResponse { result: points }))
}

/// Delete points from a collection
pub async fn delete_points(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    Json(request): Json<QdrantPointDeleteRequest>,
) -> Result<Json<QdrantPointOperationResult>, ErrorResponse> {
    debug!(
        "Deleting {} points from collection: {}",
        request.points.len(),
        collection_name
    );

    // Validate collection exists
    let collection = state
        .store
        .get_collection(&collection_name)
        .map_err(|_| create_not_found_error("collection", &collection_name))?;

    let mut deleted_count = 0;

    let start_time = std::time::Instant::now();
    for point_id in request.points {
        let id_str = match point_id {
            QdrantPointId::Numeric(n) => n.to_string(),
            QdrantPointId::Uuid(s) => s,
        };

        match state.store.delete(&collection_name, &id_str) {
            Ok(_) => {
                deleted_count += 1;
            }
            Err(_) => {
                // Point not found, skip it
                debug!(
                    "Point '{}' not found in collection '{}'",
                    id_str, collection_name
                );
            }
        }
    }

    let duration = start_time.elapsed();
    info!(
        "Successfully deleted {} points from collection: {} in {:.3}s",
        deleted_count,
        collection_name,
        duration.as_secs_f64()
    );
    Ok(Json(QdrantPointOperationResult {
        status: QdrantOperationStatus::Acknowledged,
        operation_id: None,
    }))
}

/// Scroll points from a collection
pub async fn scroll_points(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    Json(request): Json<QdrantPointScrollRequest>,
) -> Result<Json<QdrantPointScrollResponse>, ErrorResponse> {
    debug!("Scrolling points from collection: {}", collection_name);

    // Validate collection exists
    let collection = state
        .store
        .get_collection(&collection_name)
        .map_err(|_| create_not_found_error("collection", &collection_name))?;

    // Get all vectors from collection
    let start_time = std::time::Instant::now();
    let all_vectors = collection.get_all_vectors();
    let limit = request.limit.unwrap_or(10) as usize;
    let offset = request.offset.map(|id| match id {
        QdrantPointId::Numeric(n) => n.to_string(),
        QdrantPointId::Uuid(s) => s,
    });

    // Apply offset if provided
    let start_index = if let Some(offset_id) = offset {
        all_vectors
            .iter()
            .position(|v| v.id == offset_id)
            .unwrap_or(0)
    } else {
        0
    };

    // Get limited results
    let vectors = all_vectors
        .clone()
        .into_iter()
        .skip(start_index)
        .take(limit)
        .collect::<Vec<_>>();

    // Convert to Qdrant points
    let points: Vec<_> = vectors
        .into_iter()
        .map(|vector| QdrantPointStruct {
            id: QdrantPointId::Uuid(vector.id),
            vector: QdrantVector::Dense(vector.data),
            payload: vector.payload.map(|p| {
                p.data
                    .as_object()
                    .unwrap()
                    .iter()
                    .map(|(k, v)| (k.clone(), json_value_to_qdrant_value(v.clone())))
                    .collect()
            }),
        })
        .collect();

    // Calculate next page offset
    let next_page_offset = if start_index + limit < all_vectors.len() {
        // There are more results, use the last point's ID as offset
        points.last().map(|point| point.id.clone())
    } else {
        // No more results
        None
    };

    let duration = start_time.elapsed();
    Ok(Json(QdrantPointScrollResponse {
        result: QdrantScrollResult {
            points,
            next_page_offset,
        },
    }))
}

/// Count points in a collection
pub async fn count_points(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    Json(request): Json<QdrantPointCountRequest>,
) -> Result<Json<QdrantPointCountResponse>, ErrorResponse> {
    debug!("Counting points in collection: {}", collection_name);

    // Validate collection exists
    let collection = state
        .store
        .get_collection(&collection_name)
        .map_err(|_| create_not_found_error("collection", &collection_name))?;

    let start_time = std::time::Instant::now();
    let count = collection.vector_count() as u64;
    let duration = start_time.elapsed();

    Ok(Json(QdrantPointCountResponse {
        result: QdrantCountResult { count },
    }))
}

/// Convert Qdrant point to Vectorizer vector
fn convert_qdrant_point_to_vector(
    point: QdrantPointStruct,
    config: &crate::models::CollectionConfig,
) -> Result<Vector, ErrorResponse> {
    // Extract vector data
    let vector_data = match point.vector {
        QdrantVector::Dense(data) => data,
        QdrantVector::Named(_) => {
            return Err(create_error_response(
                "bad_request",
                "Named vectors not yet supported",
                StatusCode::BAD_REQUEST,
            ));
        }
    };

    // Validate dimension
    if vector_data.len() != config.dimension {
        return Err(create_error_response(
            "vector_dimension_mismatch",
            &format!(
                "Vector dimension mismatch: expected {}, got {}",
                config.dimension,
                vector_data.len()
            ),
            StatusCode::BAD_REQUEST,
        ));
    }

    // Convert payload
    let payload = if let Some(payload_data) = point.payload {
        Some(Payload::new(serde_json::Value::Object(
            payload_data
                .into_iter()
                .map(|(k, v)| (k, qdrant_value_to_json_value(v)))
                .collect(),
        )))
    } else {
        None
    };

    let id = match point.id {
        QdrantPointId::Numeric(n) => n.to_string(),
        QdrantPointId::Uuid(s) => s,
    };

    Ok(Vector {
        id,
        data: vector_data,
        sparse: None,
        payload,
    })
}

/// Convert Vectorizer vector to Qdrant point
fn convert_vector_to_qdrant_point(
    vector: Vector,
    with_payload: bool,
    with_vector: bool,
) -> QdrantPointStruct {
    let id = QdrantPointId::Uuid(vector.id.clone());

    let vector_data = if with_vector {
        Some(QdrantVector::Dense(vector.data))
    } else {
        None
    };

    let payload = if with_payload {
        vector.payload.map(|p| {
            p.data
                .as_object()
                .unwrap()
                .iter()
                .map(|(k, v)| (k.clone(), json_value_to_qdrant_value(v.clone())))
                .collect()
        })
    } else {
        None
    };

    QdrantPointStruct {
        id,
        vector: vector_data.unwrap_or(QdrantVector::Dense(vec![])),
        payload,
    }
}
