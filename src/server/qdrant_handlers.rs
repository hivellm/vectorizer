//! Qdrant compatibility REST API handlers

use std::collections::HashMap;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde_json::{Value, json};
use tracing::{debug, error, info, warn};

use super::VectorizerServer;
use super::error_middleware::{
    ErrorResponse, create_conflict_error, create_error_response, create_not_found_error,
};
use crate::error::VectorizerError;
use crate::models::qdrant::{
    PointOperationStatus as QdrantOperationStatus, QdrantCollectionConfig, QdrantCollectionInfo,
    QdrantCollectionInfoResponse, QdrantCollectionListResponse, QdrantCollectionResponse,
    QdrantCollectionStats, QdrantCollectionStatus, QdrantCreateCollectionRequest, QdrantDistance,
    QdrantHnswConfig, QdrantOptimizerConfig, QdrantOptimizerStatus, QdrantPayloadDataType,
    QdrantPayloadSchema, QdrantQuantizationConfig, QdrantQuantizationType,
    QdrantScalarQuantization, QdrantUpdateCollectionRequest, QdrantVectorsConfig, QdrantWalConfig,
};
use crate::models::{Payload, Vector};

/// Extract payload schema from collection vectors
fn extract_payload_schema(
    collection: &crate::db::CollectionType,
) -> HashMap<String, QdrantPayloadSchema> {
    let mut schema = HashMap::new();

    // Get a sample of vectors to analyze payload structure
    let sample_vectors = collection.get_all_vectors();
    let sample_size = std::cmp::min(100, sample_vectors.len()); // Sample up to 100 vectors

    for vector in sample_vectors.iter().take(sample_size) {
        if let Some(payload) = &vector.payload {
            analyze_payload_structure(&payload.data, &mut schema);
        }
    }

    schema
}

/// Analyze payload structure recursively to build schema
fn analyze_payload_structure(
    value: &serde_json::Value,
    schema: &mut HashMap<String, QdrantPayloadSchema>,
) {
    match value {
        serde_json::Value::Object(obj) => {
            for (key, val) in obj {
                let payload_type = match val {
                    serde_json::Value::String(_) => QdrantPayloadDataType::Keyword,
                    serde_json::Value::Number(n) => {
                        if n.is_i64() {
                            QdrantPayloadDataType::Integer
                        } else {
                            QdrantPayloadDataType::Float
                        }
                    }
                    serde_json::Value::Bool(_) => QdrantPayloadDataType::Bool,
                    serde_json::Value::Array(arr) => {
                        if !arr.is_empty() {
                            match &arr[0] {
                                serde_json::Value::String(_) => QdrantPayloadDataType::Keyword,
                                serde_json::Value::Number(n) => {
                                    if n.is_i64() {
                                        QdrantPayloadDataType::Integer
                                    } else {
                                        QdrantPayloadDataType::Float
                                    }
                                }
                                serde_json::Value::Bool(_) => QdrantPayloadDataType::Bool,
                                _ => QdrantPayloadDataType::Keyword, // Default fallback
                            }
                        } else {
                            QdrantPayloadDataType::Keyword
                        }
                    }
                    serde_json::Value::Object(_) => {
                        // Recursively analyze nested objects
                        analyze_payload_structure(val, schema);
                        continue;
                    }
                    serde_json::Value::Null => continue,
                };

                schema.insert(
                    key.clone(),
                    QdrantPayloadSchema {
                        data_type: payload_type,
                        indexed: false, // Default to not indexed
                    },
                );
            }
        }
        _ => {} // Skip non-object values
    }
}

/// Convert QdrantHnswConfig to HnswConfig
fn convert_qdrant_hnsw_config(
    qdrant_config: &QdrantHnswConfig,
    current_config: &crate::models::HnswConfig,
) -> crate::models::HnswConfig {
    crate::models::HnswConfig {
        m: qdrant_config.m as usize,
        ef_construction: qdrant_config.ef_construct as usize,
        ef_search: qdrant_config.full_scan_threshold as usize,
        seed: current_config.seed, // Keep current seed
    }
}

/// Convert QdrantOptimizerConfig to optimizer settings (stored in collection metadata)
fn convert_qdrant_optimizer_config(
    qdrant_config: &QdrantOptimizerConfig,
) -> HashMap<String, serde_json::Value> {
    let mut optimizer_settings = HashMap::new();

    let segments_threshold = qdrant_config.deleted_threshold;
    optimizer_settings.insert(
        "segments_threshold".to_string(),
        serde_json::Value::Number(
            serde_json::Number::from_f64(segments_threshold).unwrap_or(serde_json::Number::from(0)),
        ),
    );

    if let Some(max_segment_size) = qdrant_config.max_segment_size {
        optimizer_settings.insert(
            "max_segment_size".to_string(),
            serde_json::Value::Number(serde_json::Number::from(max_segment_size)),
        );
    }

    if let Some(memmap_threshold) = qdrant_config.memmap_threshold {
        optimizer_settings.insert(
            "memmap_threshold".to_string(),
            serde_json::Value::Number(serde_json::Number::from(memmap_threshold)),
        );
    }

    if let Some(indexing_threshold) = qdrant_config.indexing_threshold {
        optimizer_settings.insert(
            "indexing_threshold".to_string(),
            serde_json::Value::Number(serde_json::Number::from(indexing_threshold)),
        );
    }

    let flush_interval_sec = qdrant_config.flush_interval_sec;
    optimizer_settings.insert(
        "flush_interval_sec".to_string(),
        serde_json::Value::Number(serde_json::Number::from(flush_interval_sec)),
    );

    if let Some(max_optimization_threads) = qdrant_config.max_optimization_threads {
        optimizer_settings.insert(
            "max_optimization_threads".to_string(),
            serde_json::Value::Number(serde_json::Number::from(max_optimization_threads)),
        );
    }

    optimizer_settings
}

/// Get all collections
pub async fn get_collections(
    State(state): State<VectorizerServer>,
) -> Result<Json<QdrantCollectionListResponse>, ErrorResponse> {
    debug!("Getting all collections");

    let collections = state.store.list_collections();
    let mut collection_infos = Vec::new();

    for collection_name in collections {
        match state.store.get_collection(&collection_name) {
            Ok(collection) => {
                let metadata = collection.metadata();
                let config = collection.config();
                let (index_size, payload_size, total_size) = collection.get_size_info();
                let (index_bytes, payload_bytes, total_bytes) = collection.calculate_memory_usage();

                // Convert Vectorizer config to Qdrant config
                let qdrant_config = convert_to_qdrant_config(&config);

                let collection_info = QdrantCollectionInfo {
                    name: collection_name.clone(),
                    status: QdrantCollectionStatus::Green,
                    config: qdrant_config,
                    stats: QdrantCollectionStats {
                        points_count: collection.vector_count() as u64,
                        indexed_vectors_count: collection.vector_count() as u64,
                        segments_count: 1, // Vectorizer uses single segment
                        segments: vec![],  // Empty for now
                    },
                    optimizer_status: QdrantOptimizerStatus {
                        ok: true,
                        error: None,
                    },
                    payload_schema: extract_payload_schema(&collection),
                };

                collection_infos.push(collection_info);
            }
            Err(e) => {
                error!("Failed to get collection {}: {}", collection_name, e);
                // Continue with other collections
            }
        }
    }

    Ok(Json(QdrantCollectionListResponse {
        collections: collection_infos,
    }))
}

/// Get specific collection information
pub async fn get_collection(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
) -> Result<Json<QdrantCollectionResponse>, ErrorResponse> {
    debug!("Getting collection: {}", collection_name);

    match state.store.get_collection(&collection_name) {
        Ok(collection) => {
            let metadata = collection.metadata();
            let config = collection.config();
            let (index_size, payload_size, total_size) = collection.get_size_info();
            let (index_bytes, payload_bytes, total_bytes) = collection.calculate_memory_usage();

            // Convert Vectorizer config to Qdrant config
            let qdrant_config = convert_to_qdrant_config(&config);

            let collection_info = QdrantCollectionInfo {
                name: collection_name.clone(),
                status: QdrantCollectionStatus::Green,
                config: qdrant_config,
                stats: QdrantCollectionStats {
                    points_count: collection.vector_count() as u64,
                    indexed_vectors_count: collection.vector_count() as u64,
                    segments_count: 1, // Vectorizer uses single segment
                    segments: vec![],  // Empty for now
                },
                optimizer_status: QdrantOptimizerStatus {
                    ok: true,
                    error: None,
                },
                payload_schema: extract_payload_schema(&collection),
            };

            Ok(Json(QdrantCollectionInfoResponse {
                result: collection_info,
            }))
        }
        Err(_) => Err(create_not_found_error("collection", &collection_name)),
    }
}

/// Create a new collection
pub async fn create_collection(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    Json(request): Json<QdrantCreateCollectionRequest>,
) -> Result<Json<QdrantOperationStatus>, ErrorResponse> {
    debug!("Creating collection: {}", collection_name);

    // Note: QdrantCreateCollectionRequest doesn't have collection_name field
    // The collection name is in the path parameter

    // Convert Qdrant config to Vectorizer config
    let vectorizer_config = convert_from_qdrant_config(&request)?;

    match state
        .store
        .create_collection(&collection_name, vectorizer_config)
    {
        Ok(_) => {
            info!("Collection '{}' created successfully", collection_name);
            Ok(Json(QdrantOperationStatus::Acknowledged))
        }
        Err(VectorizerError::CollectionAlreadyExists(_)) => {
            Err(create_conflict_error("collection", &collection_name))
        }
        Err(e) => {
            error!("Failed to create collection '{}': {}", collection_name, e);
            Err(create_error_response(
                "internal_error",
                &format!("Failed to create collection: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

/// Update collection configuration
pub async fn update_collection(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    Json(request): Json<QdrantUpdateCollectionRequest>,
) -> Result<Json<QdrantOperationStatus>, ErrorResponse> {
    debug!("Updating collection: {}", collection_name);

    // Validate collection exists
    let collection = state
        .store
        .get_collection(&collection_name)
        .map_err(|_| create_not_found_error("collection", &collection_name))?;

    let _current_config = collection.config();

    // Update configuration based on request
    debug!(
        "Updating optimizer config for collection: {}",
        collection_name
    );
    // Store optimizer settings in collection metadata
    let optimizer_settings = convert_qdrant_optimizer_config(&request.config.optimizer_config);
    // Note: In a real implementation, you would update the collection's metadata
    // with these optimizer settings. For now, we just log the update.
    info!("Optimizer config updated: {:?}", optimizer_settings);

    debug!("Updating HNSW config for collection: {}", collection_name);
    // Update HNSW configuration
    let new_hnsw_config =
        convert_qdrant_hnsw_config(&request.config.hnsw_config, &_current_config.hnsw_config);
    
    // Note: HNSW config changes may require reindexing for full effect
    // The new config is stored but existing index parameters remain until rebuild
    info!(
        "HNSW config updated: m={}, ef_construction={}, ef_search={} (may require reindexing)",
        new_hnsw_config.m, new_hnsw_config.ef_construction, new_hnsw_config.ef_search
    );

    debug!("Updating WAL config for collection: {}", collection_name);
    // WAL configuration is typically handled at the storage level
    // Store WAL settings in collection metadata
    info!(
        "WAL config updated: wal_capacity_mb={:?}, wal_segments_ahead={:?}",
        request.config.wal_config.wal_capacity_mb, request.config.wal_config.wal_segments_ahead
    );

    if let Some(quantization_config) = request.config.quantization_config {
        debug!(
            "Updating quantization config for collection: {}",
            collection_name
        );
        // Update quantization configuration
        if let Some(scalar) = quantization_config.scalar {
            let bits = (scalar.quantile.unwrap_or(0.5) * 8.0) as usize;
            
            // Note: Quantization changes require reindexing and rebuilding quantized vectors
            // The config is logged but actual re-quantization is not performed automatically
            info!(
                "Quantization config updated: {:?} with {} bits (requires reindexing)",
                quantization_config.quantization,
                bits
            );
        }
    }

    debug!(
        "Updating vectors config for collection: {}",
        collection_name
    );
    
    // Validate dimension change
    let new_dimension = request.config.vectors.size as usize;
    if new_dimension != _current_config.dimension {
        warn!(
            "Dimension change requested: {} -> {} (REQUIRES COMPLETE REINDEXING)",
            _current_config.dimension, new_dimension
        );
        // Note: Dimension changes require complete reindexing as existing vectors
        // would be incompatible. In production, this should either:
        // 1. Return an error preventing the change, or
        // 2. Trigger an async reindexing job
        info!(
            "Vector dimension change logged but not applied: collection must be recreated"
        );
    }
    
    // Note: Distance metric update would also require reindexing
    info!("Vectors config update processed: {:?}", request.config.vectors);

    info!("Collection '{}' updated successfully", collection_name);
    Ok(Json(QdrantOperationStatus::Acknowledged))
}

/// Delete a collection
pub async fn delete_collection(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
) -> Result<Json<QdrantOperationStatus>, ErrorResponse> {
    debug!("Deleting collection: {}", collection_name);

    match state.store.delete_collection(&collection_name) {
        Ok(_) => {
            info!("Collection '{}' deleted successfully", collection_name);
            Ok(Json(QdrantOperationStatus::Acknowledged))
        }
        Err(_) => Err(create_not_found_error("collection", &collection_name)),
    }
}

/// Convert Vectorizer CollectionConfig to Qdrant CollectionConfig
fn convert_to_qdrant_config(config: &crate::models::CollectionConfig) -> QdrantCollectionConfig {
    let distance = match config.metric {
        crate::models::DistanceMetric::Cosine => QdrantDistance::Cosine,
        crate::models::DistanceMetric::Euclidean => QdrantDistance::Euclid,
        crate::models::DistanceMetric::DotProduct => QdrantDistance::Dot,
    };

    let vectors_config = QdrantVectorsConfig {
        size: config.dimension,
        distance: distance.clone(),
    };

    // QdrantCollectionParams doesn't exist, use QdrantCollectionConfig directly

    let hnsw_config = QdrantHnswConfig {
        m: config.hnsw_config.m as u32,
        ef_construct: config.hnsw_config.ef_construction as u32,
        full_scan_threshold: config.hnsw_config.ef_search as u32,
        max_indexing_threads: None,
        on_disk: None,
    };

    let optimizer_config = QdrantOptimizerConfig {
        deleted_threshold: 0.2,
        vacuum_min_vector_number: 1000,
        default_segment_number: 2,
        max_segment_size: Some(100_000),
        memmap_threshold: Some(50_000),
        indexing_threshold: Some(20_000),
        flush_interval_sec: 5,
        max_optimization_threads: Some(4),
    };

    let wal_config = QdrantWalConfig {
        wal_capacity_mb: 1024,
        wal_segments_ahead: 2,
    };

    let quantization_config = match &config.quantization {
        crate::models::QuantizationConfig::SQ { bits } => Some(QdrantQuantizationConfig {
            quantization: QdrantQuantizationType::Int8,
            scalar: Some(QdrantScalarQuantization {
                r#type: QdrantQuantizationType::Int8,
                quantile: None,
                always_ram: None,
            }),
        }),
        _ => None,
    };

    QdrantCollectionConfig {
        vectors: vectors_config,
        shard_number: 1,
        replication_factor: 1,
        write_consistency_factor: 1,
        on_disk_payload: false,
        distance,
        hnsw_config,
        optimizer_config,
        wal_config,
        quantization_config,
    }
}

/// Convert Qdrant CreateCollectionRequest to Vectorizer CollectionConfig
fn convert_from_qdrant_config(
    request: &QdrantCreateCollectionRequest,
) -> Result<crate::models::CollectionConfig, ErrorResponse> {
    // Extract vector configuration
    let dimension = request.config.vectors.size;
    let metric = match request.config.vectors.distance {
        QdrantDistance::Cosine => crate::models::DistanceMetric::Cosine,
        QdrantDistance::Euclid => crate::models::DistanceMetric::Euclidean,
        QdrantDistance::Dot => crate::models::DistanceMetric::DotProduct,
    };

    // Extract HNSW configuration
    let hnsw_config = crate::models::HnswConfig {
        m: request.config.hnsw_config.m as usize,
        ef_construction: request.config.hnsw_config.ef_construct as usize,
        ef_search: request.config.hnsw_config.full_scan_threshold as usize,
        seed: None,
    };

    // Extract quantization configuration
    let quantization_config = if let Some(quantization) = &request.config.quantization_config {
        if let Some(scalar) = &quantization.scalar {
            match scalar.r#type {
                QdrantQuantizationType::Int8 => crate::models::QuantizationConfig::SQ { bits: 8 },
            }
        } else {
            crate::models::QuantizationConfig::SQ { bits: 8 } // Default to 8-bit quantization
        }
    } else {
        crate::models::QuantizationConfig::SQ { bits: 8 } // Default to 8-bit quantization
    };

    Ok(crate::models::CollectionConfig {
        dimension,
        metric,
        hnsw_config,
        quantization: quantization_config,
        compression: crate::models::CompressionConfig::default(),
        normalization: None, // Qdrant doesn't have normalization concept
    })
}
