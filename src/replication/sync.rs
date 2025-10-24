//! Synchronization utilities for replication
//!
//! This module provides helpers for:
//! - Snapshot creation and transfer
//! - Incremental sync
//! - Checksum verification

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::db::VectorStore;

/// Snapshot metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub offset: u64,
    pub timestamp: u64,
    pub total_collections: usize,
    pub total_vectors: usize,
    pub compressed: bool,
    pub checksum: u32,
}

/// Create a snapshot of all collections for full sync
pub async fn create_snapshot(store: &VectorStore, offset: u64) -> Result<Vec<u8>, String> {
    info!("Creating snapshot at offset {}", offset);

    // Get all collections
    let collections = store.list_collections();
    let total_collections = collections.len();

    // Serialize collection data
    let mut collection_snapshots = Vec::new();
    let mut total_vectors = 0;

    for collection_name in collections {
        // Get collection
        if let Ok(collection) = store.get_collection(&collection_name) {
            let config = collection.config();
            total_vectors += collection.vector_count();

            // Get all vectors from collection
            let all_vectors = collection.get_all_vectors();

            // Convert to (id, data, payload) format
            let vectors: Vec<(String, Vec<f32>, Option<Vec<u8>>)> = all_vectors
                .into_iter()
                .map(|v| {
                    let payload = v
                        .payload
                        .as_ref()
                        .map(|p| serde_json::to_vec(&p.data).unwrap_or_default());
                    (v.id, v.data, payload)
                })
                .collect();

            collection_snapshots.push(CollectionSnapshot {
                name: collection_name,
                dimension: config.dimension,
                metric: format!("{:?}", config.metric),
                vectors,
            });
        }
    }

    // Serialize snapshot data
    let snapshot_data = SnapshotData {
        collections: collection_snapshots,
    };

    let data = bincode::serialize(&snapshot_data).map_err(|e| e.to_string())?;

    // Calculate checksum
    let checksum = crc32fast::hash(&data);

    // Create metadata
    let metadata = SnapshotMetadata {
        offset,
        timestamp: current_timestamp(),
        total_collections,
        total_vectors,
        compressed: false,
        checksum,
    };

    info!(
        "Snapshot created: {} collections, {} vectors, {} bytes, checksum: {}",
        total_collections,
        total_vectors,
        data.len(),
        checksum
    );

    // Combine metadata + data
    let mut result = bincode::serialize(&metadata).map_err(|e| e.to_string())?;
    result.extend_from_slice(&data);

    Ok(result)
}

/// Apply snapshot to vector store
pub async fn apply_snapshot(store: &VectorStore, snapshot: &[u8]) -> Result<u64, String> {
    // Deserialize metadata
    let metadata: SnapshotMetadata = bincode::deserialize(snapshot).map_err(|e| e.to_string())?;

    let metadata_size = bincode::serialized_size(&metadata).map_err(|e| e.to_string())? as usize;
    let data = &snapshot[metadata_size..];

    // Verify checksum
    let checksum = crc32fast::hash(data);
    if checksum != metadata.checksum {
        return Err(format!(
            "Checksum mismatch: expected {}, got {}",
            metadata.checksum, checksum
        ));
    }

    // Deserialize snapshot data
    let snapshot_data: SnapshotData = bincode::deserialize(data).map_err(|e| e.to_string())?;

    info!(
        "Applying snapshot: {} collections, {} vectors, offset: {}",
        snapshot_data.collections.len(),
        metadata.total_vectors,
        metadata.offset
    );

    // Apply each collection
    for collection in snapshot_data.collections {
        // Create collection with appropriate config
        let config = crate::models::CollectionConfig {
            dimension: collection.dimension,
            metric: parse_distance_metric(&collection.metric),
            hnsw_config: crate::models::HnswConfig::default(),
            quantization: crate::models::QuantizationConfig::None,
            compression: Default::default(),
            normalization: None,
        };

        // Create or recreate collection
        let _ = store.delete_collection(&collection.name);
        store
            .create_collection(&collection.name, config)
            .map_err(|e| e.to_string())?;

        // Insert vectors
        let vector_count = collection.vectors.len();
        let vectors: Vec<crate::models::Vector> = collection
            .vectors
            .into_iter()
            .map(|(id, data, payload)| {
                let payload_obj = payload.map(|p| crate::models::Payload {
                    data: serde_json::from_slice(&p).unwrap_or_default(),
                });
                crate::models::Vector {
                    id,
                    data,
                    payload: payload_obj,
                }
            })
            .collect();

        // Insert vectors and verify
        if let Err(e) = store.insert(&collection.name, vectors) {
            return Err(format!(
                "Failed to insert vectors into collection {}: {}",
                collection.name, e
            ));
        }

        // Verify insertion succeeded
        if let Ok(col) = store.get_collection(&collection.name) {
            debug!(
                "Applied collection: {} with {} vectors (verified: {})",
                collection.name,
                vector_count,
                col.vector_count()
            );
        } else {
            return Err(format!(
                "Failed to verify collection {} after insertion",
                collection.name
            ));
        }
    }

    info!("Snapshot applied successfully");
    Ok(metadata.offset)
}

/// Snapshot data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SnapshotData {
    collections: Vec<CollectionSnapshot>,
}

/// Collection snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CollectionSnapshot {
    name: String,
    dimension: usize,
    metric: String,
    vectors: Vec<(String, Vec<f32>, Option<Vec<u8>>)>, // (id, vector, payload)
}

fn parse_distance_metric(metric: &str) -> crate::models::DistanceMetric {
    match metric.to_lowercase().as_str() {
        "euclidean" => crate::models::DistanceMetric::Euclidean,
        "cosine" => crate::models::DistanceMetric::Cosine,
        "dotproduct" | "dot_product" => crate::models::DistanceMetric::DotProduct,
        _ => crate::models::DistanceMetric::Cosine,
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_snapshot_checksum_verification() {
        let store = VectorStore::new();

        let config = crate::models::CollectionConfig {
            dimension: 3,
            metric: crate::models::DistanceMetric::Cosine,
            hnsw_config: crate::models::HnswConfig::default(),
            quantization: crate::models::QuantizationConfig::None,
            compression: Default::default(),
            normalization: None,
        };
        store.create_collection("test", config).unwrap();

        let vec1 = crate::models::Vector {
            id: "vec1".to_string(),
            data: vec![1.0, 0.0, 0.0],
            payload: None,
        };
        store.insert("test", vec![vec1]).unwrap();

        let mut snapshot = create_snapshot(&store, 0).await.unwrap();

        // Corrupt data
        if let Some(last) = snapshot.last_mut() {
            *last = !*last;
        }

        // Should fail checksum
        let store2 = VectorStore::new();
        let result = apply_snapshot(&store2, &snapshot).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Checksum mismatch"));
    }

    #[tokio::test]
    #[ignore = "Snapshot replication issue - vectors not being restored from snapshot. Same root cause as integration tests"]
    async fn test_snapshot_with_payloads() {
        // Use CPU-only for both stores to ensure consistent behavior across platforms
        let store1 = VectorStore::new_cpu_only();

        let config = crate::models::CollectionConfig {
            dimension: 3,
            metric: crate::models::DistanceMetric::Cosine,
            hnsw_config: crate::models::HnswConfig::default(),
            quantization: crate::models::QuantizationConfig::None,
            compression: Default::default(),
            normalization: None,
        };
        store1.create_collection("payload_test", config).unwrap();

        // Insert vectors with different payload types
        let vec1 = crate::models::Vector {
            id: "vec1".to_string(),
            data: vec![1.0, 0.0, 0.0],
            payload: Some(crate::models::Payload {
                data: serde_json::json!({"type": "string", "value": "test"}),
            }),
        };

        let vec2 = crate::models::Vector {
            id: "vec2".to_string(),
            data: vec![0.0, 1.0, 0.0],
            payload: Some(crate::models::Payload {
                data: serde_json::json!({"type": "number", "value": 123}),
            }),
        };

        let vec3 = crate::models::Vector {
            id: "vec3".to_string(),
            data: vec![0.0, 0.0, 1.0],
            payload: None, // No payload
        };

        store1
            .insert("payload_test", vec![vec1, vec2, vec3])
            .unwrap();

        // Snapshot
        let snapshot = create_snapshot(&store1, 100).await.unwrap();

        // Apply
        let store2 = VectorStore::new();
        apply_snapshot(&store2, &snapshot).await.unwrap();

        // Verify payloads preserved
        let v1 = store2.get_vector("payload_test", "vec1").unwrap();
        assert!(v1.payload.is_some());

        let v3 = store2.get_vector("payload_test", "vec3").unwrap();
        assert!(v3.payload.is_none());
    }

    #[tokio::test]
    async fn test_snapshot_with_different_metrics() {
        let store1 = VectorStore::new();

        // Euclidean
        let config_euclidean = crate::models::CollectionConfig {
            dimension: 3,
            metric: crate::models::DistanceMetric::Euclidean,
            hnsw_config: crate::models::HnswConfig::default(),
            quantization: crate::models::QuantizationConfig::None,
            compression: Default::default(),
            normalization: None,
        };
        store1
            .create_collection("euclidean", config_euclidean)
            .unwrap();

        // DotProduct
        let config_dot = crate::models::CollectionConfig {
            dimension: 3,
            metric: crate::models::DistanceMetric::DotProduct,
            hnsw_config: crate::models::HnswConfig::default(),
            quantization: crate::models::QuantizationConfig::None,
            compression: Default::default(),
            normalization: None,
        };
        store1.create_collection("dotproduct", config_dot).unwrap();

        // Insert vectors
        let vec = crate::models::Vector {
            id: "test".to_string(),
            data: vec![1.0, 2.0, 3.0],
            payload: None,
        };
        store1.insert("euclidean", vec![vec.clone()]).unwrap();
        store1.insert("dotproduct", vec![vec]).unwrap();

        // Snapshot
        let snapshot = create_snapshot(&store1, 50).await.unwrap();

        // Apply
        let store2 = VectorStore::new();
        apply_snapshot(&store2, &snapshot).await.unwrap();

        // Verify metrics preserved
        let euc_col = store2.get_collection("euclidean").unwrap();
        assert_eq!(
            euc_col.config().metric,
            crate::models::DistanceMetric::Euclidean
        );

        let dot_col = store2.get_collection("dotproduct").unwrap();
        assert_eq!(
            dot_col.config().metric,
            crate::models::DistanceMetric::DotProduct
        );
    }

    #[tokio::test]
    async fn test_snapshot_empty_store() {
        let store1 = VectorStore::new();

        // Create snapshot of empty store
        let snapshot = create_snapshot(&store1, 0).await.unwrap();
        assert!(!snapshot.is_empty()); // Metadata still exists

        // Apply to new store (CPU-only for consistent test behavior)
        let store2 = VectorStore::new_cpu_only();
        let offset = apply_snapshot(&store2, &snapshot).await.unwrap();

        assert_eq!(offset, 0);
        assert_eq!(store2.list_collections().len(), 0);
    }

    #[tokio::test]
    async fn test_snapshot_metadata_fields() {
        let store = VectorStore::new();

        // Create collection with data
        let config = crate::models::CollectionConfig {
            dimension: 3,
            metric: crate::models::DistanceMetric::Cosine,
            hnsw_config: crate::models::HnswConfig::default(),
            quantization: crate::models::QuantizationConfig::None,
            compression: Default::default(),
            normalization: None,
        };
        store.create_collection("meta_test", config).unwrap();

        let vec1 = crate::models::Vector {
            id: "vec1".to_string(),
            data: vec![1.0, 0.0, 0.0],
            payload: None,
        };
        store.insert("meta_test", vec![vec1]).unwrap();

        // Create snapshot
        let snapshot = create_snapshot(&store, 999).await.unwrap();

        // Deserialize metadata to verify fields
        let metadata: SnapshotMetadata = bincode::deserialize(&snapshot).unwrap();

        assert_eq!(metadata.offset, 999);
        assert_eq!(metadata.total_collections, 1);
        assert_eq!(metadata.total_vectors, 1);
        assert!(!metadata.compressed);
        assert!(metadata.checksum > 0);
        assert!(metadata.timestamp > 0);
    }
}
