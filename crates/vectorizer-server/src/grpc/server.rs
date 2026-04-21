//! gRPC server implementation for Vectorizer
//!
//! This module implements the VectorizerService trait generated from the protobuf definitions.

use std::sync::Arc;
use std::time::Instant;

use ::vectorizer::db::hybrid_search::HybridScoringAlgorithm;
use ::vectorizer::db::{HybridSearchConfig, VectorStore};
use ::vectorizer::grpc_conversions::*;
use ::vectorizer::models::{CollectionConfig, Payload, QuantizationConfig, SparseVector, Vector};
use once_cell::sync::Lazy;
use tonic::{Request, Response, Status};
use tracing::{debug, error, info};
use vectorizer_core::error::VectorizerError;

use super::vectorizer as proto;
use super::vectorizer::vectorizer_service_server::VectorizerService;

/// Server start time for uptime tracking
static SERVER_START_TIME: Lazy<Instant> = Lazy::new(Instant::now);

/// Convert internal QuantizationConfig to proto QuantizationConfig
fn quantization_config_to_proto(config: &QuantizationConfig) -> Option<proto::QuantizationConfig> {
    match config {
        QuantizationConfig::None => None,
        QuantizationConfig::SQ { bits } => Some(proto::QuantizationConfig {
            config: Some(proto::quantization_config::Config::Scalar(
                proto::ScalarQuantization { bits: *bits as u32 },
            )),
        }),
        QuantizationConfig::PQ {
            n_centroids,
            n_subquantizers,
        } => Some(proto::QuantizationConfig {
            config: Some(proto::quantization_config::Config::Product(
                proto::ProductQuantization {
                    subvectors: *n_subquantizers as u32,
                    centroids: *n_centroids as u32,
                },
            )),
        }),
        QuantizationConfig::Binary => Some(proto::QuantizationConfig {
            config: Some(proto::quantization_config::Config::Binary(
                proto::BinaryQuantization {},
            )),
        }),
    }
}

/// Vectorizer gRPC service implementation
#[derive(Clone)]
pub struct VectorizerGrpcService {
    store: Arc<VectorStore>,
}

impl VectorizerGrpcService {
    /// Create a new gRPC service instance
    pub fn new(store: Arc<VectorStore>) -> Self {
        // Initialize the start time on first service creation
        let _ = *SERVER_START_TIME;
        Self { store }
    }

    /// Get server uptime in seconds
    pub fn uptime_seconds() -> u64 {
        SERVER_START_TIME.elapsed().as_secs()
    }
}

#[tonic::async_trait]
impl VectorizerService for VectorizerGrpcService {
    async fn list_collections(
        &self,
        _request: Request<proto::ListCollectionsRequest>,
    ) -> Result<Response<proto::ListCollectionsResponse>, Status> {
        debug!("gRPC: ListCollections request");

        let collections = self.store.list_collections();

        Ok(Response::new(proto::ListCollectionsResponse {
            collection_names: collections,
        }))
    }

    async fn create_collection(
        &self,
        request: Request<proto::CreateCollectionRequest>,
    ) -> Result<Response<proto::CreateCollectionResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC: CreateCollection request for '{}'", req.name);

        let config: vectorizer::models::CollectionConfig = req
            .config
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("Collection config is required"))?
            .try_into()
            .map_err(|e: VectorizerError| Status::invalid_argument(e.to_string()))?;

        match self.store.create_collection(&req.name, config) {
            Ok(_) => Ok(Response::new(proto::CreateCollectionResponse {
                success: true,
                message: format!("Collection '{}' created successfully", req.name),
            })),
            Err(e) => {
                error!("Failed to create collection: {}", e);
                Ok(Response::new(proto::CreateCollectionResponse {
                    success: false,
                    message: e.to_string(),
                }))
            }
        }
    }

    async fn get_collection_info(
        &self,
        request: Request<proto::GetCollectionInfoRequest>,
    ) -> Result<Response<proto::GetCollectionInfoResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC: GetCollectionInfo request for '{}'",
            req.collection_name
        );

        let collection = self
            .store
            .get_collection(&req.collection_name)
            .map_err(|e| Status::not_found(e.to_string()))?;

        let metadata = collection.metadata();
        let config = collection.config();

        let proto_config = proto::CollectionConfig {
            dimension: config.dimension as u32,
            metric: match config.metric {
                vectorizer::models::DistanceMetric::Cosine => proto::DistanceMetric::Cosine as i32,
                vectorizer::models::DistanceMetric::Euclidean => {
                    proto::DistanceMetric::Euclidean as i32
                }
                vectorizer::models::DistanceMetric::DotProduct => {
                    proto::DistanceMetric::DotProduct as i32
                }
            },
            hnsw_config: Some(proto::HnswConfig {
                m: config.hnsw_config.m as u32,
                ef_construction: config.hnsw_config.ef_construction as u32,
                ef: config.hnsw_config.ef_search as u32, // model uses 'ef_search', proto uses 'ef'
                seed: config.hnsw_config.seed.unwrap_or(0),
            }),
            quantization: quantization_config_to_proto(&config.quantization),
            storage_type: match config.storage_type {
                Some(vectorizer::models::StorageType::Memory) => proto::StorageType::Memory as i32,
                Some(vectorizer::models::StorageType::Mmap) => proto::StorageType::Mmap as i32,
                None => proto::StorageType::Memory as i32,
            },
        };

        let info = proto::CollectionInfo {
            name: req.collection_name.clone(),
            config: Some(proto_config),
            vector_count: collection.vector_count() as u64,
            created_at: metadata.created_at.timestamp(),
            updated_at: metadata.updated_at.timestamp(),
        };

        Ok(Response::new(proto::GetCollectionInfoResponse {
            info: Some(info),
        }))
    }

    async fn delete_collection(
        &self,
        request: Request<proto::DeleteCollectionRequest>,
    ) -> Result<Response<proto::DeleteCollectionResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC: DeleteCollection request for '{}'",
            req.collection_name
        );

        match self.store.delete_collection(&req.collection_name) {
            Ok(_) => Ok(Response::new(proto::DeleteCollectionResponse {
                success: true,
                message: format!("Collection '{}' deleted successfully", req.collection_name),
            })),
            Err(e) => {
                error!("Failed to delete collection: {}", e);
                Ok(Response::new(proto::DeleteCollectionResponse {
                    success: false,
                    message: e.to_string(),
                }))
            }
        }
    }

    async fn insert_vector(
        &self,
        request: Request<proto::InsertVectorRequest>,
    ) -> Result<Response<proto::InsertVectorResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC: InsertVector request for collection '{}', vector '{}'",
            req.collection_name, req.vector_id
        );

        let vector: Vector = (&req)
            .try_into()
            .map_err(|e: VectorizerError| Status::invalid_argument(e.to_string()))?;

        match self.store.insert(&req.collection_name, vec![vector]) {
            Ok(_) => Ok(Response::new(proto::InsertVectorResponse {
                success: true,
                message: "Vector inserted successfully".to_string(),
            })),
            Err(e) => {
                error!("Failed to insert vector: {}", e);
                Ok(Response::new(proto::InsertVectorResponse {
                    success: false,
                    message: e.to_string(),
                }))
            }
        }
    }

    async fn insert_vectors(
        &self,
        request: Request<tonic::Streaming<proto::InsertVectorRequest>>,
    ) -> Result<Response<proto::InsertVectorsResponse>, Status> {
        debug!("gRPC: InsertVectors streaming request");

        let mut stream = request.into_inner();
        let mut inserted_count = 0u32;
        let mut failed_count = 0u32;
        let mut errors = Vec::new();
        let mut current_collection: Option<String> = None;
        let mut batch = Vec::new();

        while let Some(req) = stream.message().await? {
            if current_collection.is_none() {
                current_collection = Some(req.collection_name.clone());
            }

            if current_collection.as_ref() != Some(&req.collection_name) {
                return Err(Status::invalid_argument(
                    "All vectors in stream must belong to the same collection",
                ));
            }

            match Vector::try_from(&req) {
                Ok(vector) => batch.push(vector),
                Err(e) => {
                    failed_count += 1;
                    errors.push(e.to_string());
                }
            }
        }

        if let Some(collection_name) = current_collection {
            let batch_len = batch.len() as u32;
            match self.store.insert(&collection_name, batch) {
                Ok(_) => {
                    inserted_count = batch_len;
                }
                Err(e) => {
                    failed_count = batch_len;
                    errors.push(e.to_string());
                }
            }
        }

        Ok(Response::new(proto::InsertVectorsResponse {
            inserted_count,
            failed_count,
            errors,
        }))
    }

    async fn get_vector(
        &self,
        request: Request<proto::GetVectorRequest>,
    ) -> Result<Response<proto::GetVectorResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC: GetVector request for collection '{}', vector '{}'",
            req.collection_name, req.vector_id
        );

        let vector = self
            .store
            .get_vector(&req.collection_name, &req.vector_id)
            .map_err(|e| Status::not_found(e.to_string()))?;

        use std::collections::HashMap;
        let payload: HashMap<String, String> = vector
            .payload
            .as_ref()
            .and_then(|p| {
                // Payload is a wrapper around serde_json::Value
                if let serde_json::Value::Object(map) = &p.data {
                    Some(
                        map.iter()
                            .map(|(k, v)| (k.clone(), v.to_string()))
                            .collect::<HashMap<String, String>>(),
                    )
                } else {
                    None
                }
            })
            .unwrap_or_default();

        Ok(Response::new(proto::GetVectorResponse {
            vector_id: vector.id,
            data: vector.data,
            payload,
        }))
    }

    async fn update_vector(
        &self,
        request: Request<proto::UpdateVectorRequest>,
    ) -> Result<Response<proto::UpdateVectorResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC: UpdateVector request for collection '{}', vector '{}'",
            req.collection_name, req.vector_id
        );

        use std::collections::HashMap;
        let vector = Vector {
            id: req.vector_id,
            data: req.data,
            sparse: None, // gRPC doesn't support sparse vectors directly yet
            payload: if req.payload.is_empty() {
                None
            } else {
                // Convert HashMap<String, String> to Payload (which wraps serde_json::Value)
                let json_map: serde_json::Map<String, serde_json::Value> = req
                    .payload
                    .into_iter()
                    .map(|(k, v)| (k, serde_json::Value::String(v)))
                    .collect();
                Some(Payload::new(serde_json::Value::Object(json_map)))
            },
            document_id: None,
        };

        match self.store.update(&req.collection_name, vector) {
            Ok(_) => Ok(Response::new(proto::UpdateVectorResponse {
                success: true,
                message: "Vector updated successfully".to_string(),
            })),
            Err(e) => {
                error!("Failed to update vector: {}", e);
                Ok(Response::new(proto::UpdateVectorResponse {
                    success: false,
                    message: e.to_string(),
                }))
            }
        }
    }

    async fn delete_vector(
        &self,
        request: Request<proto::DeleteVectorRequest>,
    ) -> Result<Response<proto::DeleteVectorResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC: DeleteVector request for collection '{}', vector '{}'",
            req.collection_name, req.vector_id
        );

        match self.store.delete(&req.collection_name, &req.vector_id) {
            Ok(_) => Ok(Response::new(proto::DeleteVectorResponse {
                success: true,
                message: "Vector deleted successfully".to_string(),
            })),
            Err(e) => {
                error!("Failed to delete vector: {}", e);
                Ok(Response::new(proto::DeleteVectorResponse {
                    success: false,
                    message: e.to_string(),
                }))
            }
        }
    }

    async fn search(
        &self,
        request: Request<proto::SearchRequest>,
    ) -> Result<Response<proto::SearchResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC: Search request for collection '{}', limit={}",
            req.collection_name, req.limit
        );

        let results = self
            .store
            .search(&req.collection_name, &req.query_vector, req.limit as usize)
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_results: Vec<proto::SearchResult> = results.iter().map(|r| r.into()).collect();

        Ok(Response::new(proto::SearchResponse {
            results: proto_results,
        }))
    }

    async fn batch_search(
        &self,
        request: Request<proto::BatchSearchRequest>,
    ) -> Result<Response<proto::BatchSearchResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC: BatchSearch request for collection '{}', {} queries",
            req.collection_name,
            req.queries.len()
        );

        let mut batch_results = Vec::new();

        for query in req.queries {
            match self.store.search(
                &req.collection_name,
                &query.query_vector,
                query.limit as usize,
            ) {
                Ok(results) => {
                    let proto_results: Vec<proto::SearchResult> =
                        results.iter().map(|r| r.into()).collect();
                    batch_results.push(proto::SearchResponse {
                        results: proto_results,
                    });
                }
                Err(e) => {
                    error!("Batch search failed for query: {}", e);
                    batch_results.push(proto::SearchResponse { results: vec![] });
                }
            }
        }

        Ok(Response::new(proto::BatchSearchResponse {
            results: batch_results,
        }))
    }

    async fn hybrid_search(
        &self,
        request: Request<proto::HybridSearchRequest>,
    ) -> Result<Response<proto::HybridSearchResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC: HybridSearch request for collection '{}'",
            req.collection_name
        );

        let sparse_query = req.sparse_query.as_ref().map(|sv| SparseVector {
            indices: sv.indices.iter().map(|&i| i as usize).collect(),
            values: sv.values.clone(),
        });

        let config = req.config.as_ref().map(|c| HybridSearchConfig {
            dense_k: c.dense_k as usize,
            sparse_k: c.sparse_k as usize,
            final_k: c.final_k as usize,
            alpha: c.alpha as f32,
            algorithm: match c.algorithm() {
                proto::HybridScoringAlgorithm::Rrf => HybridScoringAlgorithm::ReciprocalRankFusion,
                proto::HybridScoringAlgorithm::Weighted => {
                    HybridScoringAlgorithm::WeightedCombination
                }
                proto::HybridScoringAlgorithm::AlphaBlend => HybridScoringAlgorithm::AlphaBlending,
            },
        });

        let config = config.unwrap_or_else(|| HybridSearchConfig {
            dense_k: 10,
            sparse_k: 10,
            final_k: 10,
            alpha: 0.5,
            algorithm: HybridScoringAlgorithm::ReciprocalRankFusion,
        });

        let results = self
            .store
            .hybrid_search(
                &req.collection_name,
                &req.dense_query,
                sparse_query.as_ref(),
                config,
            )
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_results: Vec<proto::HybridSearchResult> = results
            .iter()
            .map(|r| proto::HybridSearchResult {
                id: r.id.clone(),
                // proto fields are now `float` (f32). See phase2_unify-search-result-type.
                hybrid_score: r.score,
                dense_score: r.dense_score.unwrap_or(0.0),
                sparse_score: r.sparse_score.unwrap_or(0.0),
                vector: r.vector.as_ref().map(|v| v.clone()).unwrap_or_default(),
                payload: r
                    .payload
                    .as_ref()
                    .and_then(|p| {
                        use std::collections::HashMap;
                        if let serde_json::Value::Object(map) = &p.data {
                            Some(
                                map.iter()
                                    .map(|(k, v)| (k.clone(), v.to_string()))
                                    .collect::<HashMap<String, String>>(),
                            )
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default(),
            })
            .collect();

        Ok(Response::new(proto::HybridSearchResponse {
            results: proto_results,
        }))
    }

    async fn health_check(
        &self,
        _request: Request<proto::HealthCheckRequest>,
    ) -> Result<Response<proto::HealthCheckResponse>, Status> {
        Ok(Response::new(proto::HealthCheckResponse {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        }))
    }

    async fn get_stats(
        &self,
        _request: Request<proto::GetStatsRequest>,
    ) -> Result<Response<proto::GetStatsResponse>, Status> {
        let collections = self.store.list_collections();
        let total_vectors: usize = collections
            .iter()
            .map(|name| {
                self.store
                    .get_collection(name)
                    .map(|c| c.vector_count())
                    .unwrap_or(0)
            })
            .sum();

        Ok(Response::new(proto::GetStatsResponse {
            collections_count: collections.len() as u32,
            total_vectors: total_vectors as u64,
            uptime_seconds: Self::uptime_seconds() as i64,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }))
    }
}
