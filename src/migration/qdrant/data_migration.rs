//! Qdrant data migration tools
//!
//! Tools for exporting data from Qdrant and importing into Vectorizer.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::db::VectorStore;
use crate::error::{Result, VectorizerError};
use crate::models::Vector;

/// Qdrant data exporter
pub struct QdrantDataExporter;

impl QdrantDataExporter {
    /// Export collection data from Qdrant API
    ///
    /// Fetches all points from a Qdrant collection via REST API
    pub async fn export_collection(
        qdrant_url: &str,
        collection_name: &str,
    ) -> Result<ExportedCollection> {
        info!(
            "📤 Exporting collection '{}' from Qdrant at {}",
            collection_name, qdrant_url
        );

        // Get collection info
        let collection_info = Self::fetch_collection_info(qdrant_url, collection_name).await?;

        // Scroll through all points
        let mut all_points = Vec::new();
        let mut offset: Option<String> = None;
        let batch_size = 1000;

        loop {
            let batch =
                Self::scroll_points(qdrant_url, collection_name, offset.clone(), batch_size)
                    .await?;

            if batch.result.points.is_empty() {
                break;
            }

            all_points.extend(batch.result.points);

            if batch.result.next_page_offset.is_none() {
                break;
            }

            offset = batch.result.next_page_offset;
        }

        info!(
            "✅ Exported {} points from collection '{}'",
            all_points.len(),
            collection_name
        );

        Ok(ExportedCollection {
            name: collection_name.to_string(),
            config: collection_info.config,
            points: all_points,
        })
    }

    /// Fetch collection info from Qdrant
    async fn fetch_collection_info(
        qdrant_url: &str,
        collection_name: &str,
    ) -> Result<QdrantCollectionInfo> {
        let url = format!("{}/collections/{}", qdrant_url, collection_name);

        let response = reqwest::Client::new().get(&url).send().await.map_err(|e| {
            VectorizerError::Other(format!("Failed to fetch collection info: {}", e))
        })?;

        if !response.status().is_success() {
            return Err(VectorizerError::NotFound(format!(
                "Collection '{}' not found in Qdrant",
                collection_name
            )));
        }

        let info: QdrantCollectionInfoResponse = response.json().await.map_err(|e| {
            VectorizerError::Deserialization(format!("Failed to parse response: {}", e))
        })?;

        Ok(info.result)
    }

    /// Scroll points from Qdrant
    async fn scroll_points(
        qdrant_url: &str,
        collection_name: &str,
        offset: Option<String>,
        limit: usize,
    ) -> Result<QdrantScrollResponse> {
        let url = format!(
            "{}/collections/{}/points/scroll",
            qdrant_url, collection_name
        );

        let mut request_body = serde_json::json!({
            "limit": limit,
            "with_payload": true,
            "with_vector": true
        });

        if let Some(offset_id) = offset {
            request_body["offset"] = serde_json::Value::String(offset_id);
        }

        let response = reqwest::Client::new()
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| VectorizerError::Other(format!("Failed to scroll points: {}", e)))?;

        if !response.status().is_success() {
            return Err(VectorizerError::Other(format!(
                "Failed to scroll points: HTTP {}",
                response.status()
            )));
        }

        let scroll_response: QdrantScrollResponseWrapper = response.json().await.map_err(|e| {
            VectorizerError::Deserialization(format!("Failed to parse response: {}", e))
        })?;

        Ok(QdrantScrollResponse {
            result: scroll_response.result,
        })
    }

    /// Export to JSON file
    pub fn export_to_file<P: AsRef<std::path::Path>>(
        exported: &ExportedCollection,
        path: P,
    ) -> Result<()> {
        let json = serde_json::to_string_pretty(exported)
            .map_err(|e| VectorizerError::Serialization(format!("Failed to serialize: {}", e)))?;

        std::fs::write(path.as_ref(), json).map_err(|e| VectorizerError::Io(e))?;

        info!("💾 Exported collection to {}", path.as_ref().display());
        Ok(())
    }
}

/// Qdrant data importer
pub struct QdrantDataImporter;

impl QdrantDataImporter {
    /// Import collection data into Vectorizer
    pub async fn import_collection(
        store: &VectorStore,
        exported: &ExportedCollection,
    ) -> Result<ImportResult> {
        info!(
            "📥 Importing collection '{}' into Vectorizer",
            exported.name
        );

        // Convert Qdrant config to Vectorizer config
        let vectorizer_config = Self::convert_config(&exported.config)?;

        // Create collection
        if let Err(e) = store.create_collection(&exported.name, vectorizer_config.clone()) {
            if e.to_string().contains("already exists") {
                warn!(
                    "Collection '{}' already exists, skipping creation",
                    exported.name
                );
            } else {
                return Err(e);
            }
        }

        // Convert and insert points
        let mut imported_count = 0;
        let mut errors = Vec::new();

        for point in &exported.points {
            match Self::convert_point(point) {
                Ok(vector) => {
                    if let Err(e) = store.insert(&exported.name, vec![vector]) {
                        errors.push(format!("Failed to insert point {}: {}", point.id, e));
                    } else {
                        imported_count += 1;
                    }
                }
                Err(e) => {
                    errors.push(format!("Failed to convert point {}: {}", point.id, e));
                }
            }
        }

        info!(
            "✅ Imported {} points ({} errors)",
            imported_count,
            errors.len()
        );

        Ok(ImportResult {
            collection_name: exported.name.clone(),
            imported_count,
            error_count: errors.len(),
            errors,
        })
    }

    /// Import from JSON file
    pub fn import_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<ExportedCollection> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| VectorizerError::Io(e))?;

        let exported: ExportedCollection = serde_json::from_str(&content).map_err(|e| {
            VectorizerError::Deserialization(format!("Failed to parse export file: {}", e))
        })?;

        info!("📖 Loaded export file: {} points", exported.points.len());
        Ok(exported)
    }

    /// Convert Qdrant collection config to Vectorizer config
    fn convert_config(
        qdrant_config: &QdrantCollectionConfig,
    ) -> Result<crate::models::CollectionConfig> {
        use crate::models::{
            CollectionConfig, CompressionConfig, DistanceMetric, HnswConfig, QuantizationConfig,
        };

        let dimension = match &qdrant_config.params.vectors {
            QdrantVectorsConfigResponse::Vector { size, distance: _ } => *size as usize,
            QdrantVectorsConfigResponse::NamedVectors { .. } => {
                return Err(VectorizerError::Other(
                    "Named vectors not supported".to_string(),
                ));
            }
        };

        let metric = match &qdrant_config.params.vectors {
            QdrantVectorsConfigResponse::Vector { size: _, distance } => match distance.as_str() {
                "Cosine" => DistanceMetric::Cosine,
                "Euclidean" => DistanceMetric::Euclidean,
                "Dot" => DistanceMetric::DotProduct,
                _ => DistanceMetric::Cosine,
            },
            _ => DistanceMetric::Cosine,
        };

        let hnsw_config = if let Some(hnsw) = &qdrant_config.params.hnsw_config {
            HnswConfig {
                m: hnsw.m as usize,
                ef_construction: hnsw.ef_construct as usize,
                ef_search: hnsw.ef.unwrap_or(hnsw.ef_construct) as usize,
                seed: None,
            }
        } else {
            HnswConfig::default()
        };

        let quantization = if let Some(quant) = &qdrant_config.params.quantization_config {
            match quant.quantization {
                QdrantQuantizationTypeResponse::Int8 => QuantizationConfig::SQ { bits: 8 },
                _ => QuantizationConfig::SQ { bits: 8 },
            }
        } else {
            QuantizationConfig::SQ { bits: 8 }
        };

        Ok(CollectionConfig {
            dimension,
            metric,
            hnsw_config,
            quantization,
            compression: CompressionConfig::default(),
            normalization: None,
            storage_type: Some(crate::models::StorageType::Memory),
            sharding: None,
            graph: None,
            encryption: None,
        })
    }

    /// Convert Qdrant point to Vectorizer vector
    fn convert_point(qdrant_point: &QdrantPoint) -> Result<Vector> {
        let vector_data: Vec<f32> = match &qdrant_point.vector {
            QdrantVector::Dense(data) => data.clone(),
            QdrantVector::Sparse(_) => {
                return Err(VectorizerError::Other(
                    "Sparse vectors not supported in migration".to_string(),
                ));
            }
        };

        use crate::models::Payload;

        let payload: Option<Payload> = qdrant_point.payload.clone().map(|p| Payload { data: p });

        Ok(Vector {
            id: qdrant_point.id.clone(),
            data: vector_data,
            payload,
            sparse: None,
        })
    }
}

/// Exported collection data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedCollection {
    /// Collection name
    pub name: String,
    /// Collection configuration
    pub config: QdrantCollectionConfig,
    /// All points in the collection
    pub points: Vec<QdrantPoint>,
}

/// Qdrant collection info response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct QdrantCollectionInfoResponse {
    result: QdrantCollectionInfo,
}

/// Qdrant collection info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCollectionInfo {
    config: QdrantCollectionConfig,
}

/// Qdrant collection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCollectionConfig {
    pub params: QdrantCollectionParams,
}

/// Qdrant collection parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCollectionParams {
    pub vectors: QdrantVectorsConfigResponse,
    #[serde(rename = "hnsw_config")]
    pub hnsw_config: Option<QdrantHnswConfigResponse>,
    #[serde(rename = "quantization_config")]
    pub quantization_config: Option<QdrantQuantizationConfigResponse>,
}

/// Qdrant vectors configuration response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QdrantVectorsConfigResponse {
    Vector { size: u32, distance: String },
    NamedVectors(HashMap<String, QdrantNamedVectorConfigResponse>),
}

/// Qdrant named vector config response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantNamedVectorConfigResponse {
    size: u32,
    distance: String,
}

/// Qdrant HNSW config response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantHnswConfigResponse {
    m: u32,
    #[serde(rename = "ef_construct")]
    ef_construct: u32,
    ef: Option<u32>,
}

/// Qdrant quantization config response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantQuantizationConfigResponse {
    pub quantization: QdrantQuantizationTypeResponse,
}

/// Qdrant quantization type response
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QdrantQuantizationTypeResponse {
    Int8,
    Product,
    Binary,
}

/// Qdrant scroll response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
struct QdrantScrollResponseWrapper {
    result: QdrantScrollResult,
}

/// Qdrant scroll response
#[derive(Debug, Clone)]
pub struct QdrantScrollResponse {
    pub result: QdrantScrollResult,
}

/// Qdrant scroll result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantScrollResult {
    pub points: Vec<QdrantPoint>,
    #[serde(rename = "next_page_offset")]
    pub next_page_offset: Option<String>,
}

/// Qdrant point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantPoint {
    /// Point ID
    pub id: String,
    /// Vector data
    pub vector: QdrantVector,
    /// Payload data
    pub payload: Option<serde_json::Value>,
}

/// Qdrant vector (dense or sparse)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QdrantVector {
    /// Dense vector
    Dense(Vec<f32>),
    /// Sparse vector (not supported in migration)
    Sparse(QdrantSparseVector),
}

/// Qdrant sparse vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantSparseVector {
    pub indices: Vec<u32>,
    pub values: Vec<f32>,
}

/// Import result
#[derive(Debug, Clone)]
pub struct ImportResult {
    /// Collection name
    pub collection_name: String,
    /// Number of points imported
    pub imported_count: usize,
    /// Number of errors
    pub error_count: usize,
    /// Error messages
    pub errors: Vec<String>,
}
