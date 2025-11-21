//! gRPC server implementation for Vectorizer
//!
//! This module implements the VectorizerService trait generated from the protobuf definitions.

use std::sync::Arc;

use tonic::{Request, Response, Status};
use tracing::{debug, error, info};

use crate::db::{hybrid_search::HybridScoringAlgorithm, HybridSearchConfig, VectorStore};
use crate::error::VectorizerError;
use crate::models::{CollectionConfig, Payload, SparseVector, Vector};

use super::conversions::*;
use super::vectorizer::vectorizer_service_server::VectorizerService;
use super::vectorizer;

/// Vectorizer gRPC service implementation
#[derive(Clone)]
pub struct VectorizerGrpcService {
    store: Arc<VectorStore>,
}

impl VectorizerGrpcService {
    /// Create a new gRPC service instance
    pub fn new(store: Arc<VectorStore>) -> Self {
        Self { store }
    }
}

#[tonic::async_trait]
impl VectorizerService for VectorizerGrpcService {
    async fn list_collections(
        &self,
        _request: Request<vectorizer::ListCollectionsRequest>,
    ) -> Result<Response<vectorizer::ListCollectionsResponse>, Status> {
        debug!("gRPC: ListCollections request");

        let collections = self.store.list_collections();

        Ok(Response::new(vectorizer::ListCollectionsResponse {
            collection_names: collections,
        }))
    }

    async fn create_collection(
        &self,
        request: Request<vectorizer::CreateCollectionRequest>,
    ) -> Result<Response<vectorizer::CreateCollectionResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC: CreateCollection request for '{}'", req.name);

        let config: crate::models::CollectionConfig = req
            .config
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("Collection config is required"))?
            .try_into()
            .map_err(|e: VectorizerError| Status::invalid_argument(e.to_string()))?;

        match self.store.create_collection(&req.name, config) {
            Ok(_) => Ok(Response::new(vectorizer::CreateCollectionResponse {
                success: true,
                message: format!("Collection '{}' created successfully", req.name),
            })),
            Err(e) => {
                error!("Failed to create collection: {}", e);
                Ok(Response::new(vectorizer::CreateCollectionResponse {
                    success: false,
                    message: e.to_string(),
                }))
            }
        }
    }

    async fn get_collection_info(
        &self,
        request: Request<vectorizer::GetCollectionInfoRequest>,
    ) -> Result<Response<vectorizer::GetCollectionInfoResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC: GetCollectionInfo request for '{}'", req.collection_name);

        let collection = self
            .store
            .get_collection(&req.collection_name)
            .map_err(|e| Status::not_found(e.to_string()))?;

        let metadata = collection.metadata();
        let config = collection.config();

        let proto_config = vectorizer::CollectionConfig {
            dimension: config.dimension as u32,
            metric: match config.metric {
                crate::models::DistanceMetric::Cosine => vectorizer::DistanceMetric::Cosine as i32,
                crate::models::DistanceMetric::Euclidean => vectorizer::DistanceMetric::Euclidean as i32,
                crate::models::DistanceMetric::DotProduct => vectorizer::DistanceMetric::DotProduct as i32,
            },
            hnsw_config: Some(vectorizer::HnswConfig {
                m: config.hnsw_config.m as u32,
                ef_construction: config.hnsw_config.ef_construction as u32,
                ef: config.hnsw_config.ef_search as u32, // model uses 'ef_search', proto uses 'ef'
                seed: config.hnsw_config.seed.unwrap_or(0),
            }),
            quantization: None, // TODO: Convert quantization config
            storage_type: match config.storage_type {
                Some(crate::models::StorageType::Memory) => vectorizer::StorageType::Memory as i32,
                Some(crate::models::StorageType::Mmap) => vectorizer::StorageType::Mmap as i32,
                None => vectorizer::StorageType::Memory as i32,
            },
        };

        let info = vectorizer::CollectionInfo {
            name: req.collection_name.clone(),
            config: Some(proto_config),
            vector_count: collection.vector_count() as u64,
            created_at: metadata.created_at.timestamp(),
            updated_at: metadata.updated_at.timestamp(),
        };

        Ok(Response::new(vectorizer::GetCollectionInfoResponse {
            info: Some(info),
        }))
    }

    async fn delete_collection(
        &self,
        request: Request<vectorizer::DeleteCollectionRequest>,
    ) -> Result<Response<vectorizer::DeleteCollectionResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC: DeleteCollection request for '{}'", req.collection_name);

        match self.store.delete_collection(&req.collection_name) {
            Ok(_) => Ok(Response::new(vectorizer::DeleteCollectionResponse {
                success: true,
                message: format!("Collection '{}' deleted successfully", req.collection_name),
            })),
            Err(e) => {
                error!("Failed to delete collection: {}", e);
                Ok(Response::new(vectorizer::DeleteCollectionResponse {
                    success: false,
                    message: e.to_string(),
                }))
            }
        }
    }

    async fn insert_vector(
        &self,
        request: Request<vectorizer::InsertVectorRequest>,
    ) -> Result<Response<vectorizer::InsertVectorResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC: InsertVector request for collection '{}', vector '{}'",
            req.collection_name, req.vector_id
        );

        let vector: Vector = (&req)
            .try_into()
            .map_err(|e: VectorizerError| Status::invalid_argument(e.to_string()))?;

        match self.store.insert(&req.collection_name, vec![vector]) {
            Ok(_) => Ok(Response::new(vectorizer::InsertVectorResponse {
                success: true,
                message: "Vector inserted successfully".to_string(),
            })),
            Err(e) => {
                error!("Failed to insert vector: {}", e);
                Ok(Response::new(vectorizer::InsertVectorResponse {
                    success: false,
                    message: e.to_string(),
                }))
            }
        }
    }

    async fn insert_vectors(
        &self,
        request: Request<tonic::Streaming<vectorizer::InsertVectorRequest>>,
    ) -> Result<Response<vectorizer::InsertVectorsResponse>, Status> {
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

        Ok(Response::new(vectorizer::InsertVectorsResponse {
            inserted_count,
            failed_count,
            errors,
        }))
    }

    async fn get_vector(
        &self,
        request: Request<vectorizer::GetVectorRequest>,
    ) -> Result<Response<vectorizer::GetVectorResponse>, Status> {
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

        Ok(Response::new(vectorizer::GetVectorResponse {
            vector_id: vector.id,
            data: vector.data,
            payload,
        }))
    }

    async fn update_vector(
        &self,
        request: Request<vectorizer::UpdateVectorRequest>,
    ) -> Result<Response<vectorizer::UpdateVectorResponse>, Status> {
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
        };

        match self.store.update(&req.collection_name, vector) {
            Ok(_) => Ok(Response::new(vectorizer::UpdateVectorResponse {
                success: true,
                message: "Vector updated successfully".to_string(),
            })),
            Err(e) => {
                error!("Failed to update vector: {}", e);
                Ok(Response::new(vectorizer::UpdateVectorResponse {
                    success: false,
                    message: e.to_string(),
                }))
            }
        }
    }

    async fn delete_vector(
        &self,
        request: Request<vectorizer::DeleteVectorRequest>,
    ) -> Result<Response<vectorizer::DeleteVectorResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC: DeleteVector request for collection '{}', vector '{}'",
            req.collection_name, req.vector_id
        );

        match self.store.delete(&req.collection_name, &req.vector_id) {
            Ok(_) => Ok(Response::new(vectorizer::DeleteVectorResponse {
                success: true,
                message: "Vector deleted successfully".to_string(),
            })),
            Err(e) => {
                error!("Failed to delete vector: {}", e);
                Ok(Response::new(vectorizer::DeleteVectorResponse {
                    success: false,
                    message: e.to_string(),
                }))
            }
        }
    }

    async fn search(
        &self,
        request: Request<vectorizer::SearchRequest>,
    ) -> Result<Response<vectorizer::SearchResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC: Search request for collection '{}', limit={}",
            req.collection_name, req.limit
        );

        let results = self
            .store
            .search(&req.collection_name, &req.query_vector, req.limit as usize)
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_results: Vec<vectorizer::SearchResult> = results.iter().map(|r| r.into()).collect();

        Ok(Response::new(vectorizer::SearchResponse {
            results: proto_results,
        }))
    }

    async fn batch_search(
        &self,
        request: Request<vectorizer::BatchSearchRequest>,
    ) -> Result<Response<vectorizer::BatchSearchResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC: BatchSearch request for collection '{}', {} queries",
            req.collection_name,
            req.queries.len()
        );

        let mut batch_results = Vec::new();

        for query in req.queries {
            match self
                .store
                .search(&req.collection_name, &query.query_vector, query.limit as usize)
            {
                Ok(results) => {
                    let proto_results: Vec<vectorizer::SearchResult> = results.iter().map(|r| r.into()).collect();
                    batch_results.push(vectorizer::SearchResponse {
                        results: proto_results,
                    });
                }
                Err(e) => {
                    error!("Batch search failed for query: {}", e);
                    batch_results.push(vectorizer::SearchResponse { results: vec![] });
                }
            }
        }

        Ok(Response::new(vectorizer::BatchSearchResponse {
            results: batch_results,
        }))
    }

    async fn hybrid_search(
        &self,
        request: Request<vectorizer::HybridSearchRequest>,
    ) -> Result<Response<vectorizer::HybridSearchResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC: HybridSearch request for collection '{}'", req.collection_name);

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
                vectorizer::HybridScoringAlgorithm::Rrf => HybridScoringAlgorithm::ReciprocalRankFusion,
                vectorizer::HybridScoringAlgorithm::Weighted => HybridScoringAlgorithm::WeightedCombination,
                vectorizer::HybridScoringAlgorithm::AlphaBlend => HybridScoringAlgorithm::AlphaBlending,
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
            .hybrid_search(&req.collection_name, &req.dense_query, sparse_query.as_ref(), config)
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_results: Vec<vectorizer::HybridSearchResult> = results
            .iter()
            .map(|r| vectorizer::HybridSearchResult {
                id: r.id.clone(),
                hybrid_score: r.score as f64,
                dense_score: r.score as f64, // TODO: Extract actual dense/sparse scores
                sparse_score: 0.0,
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

        Ok(Response::new(vectorizer::HybridSearchResponse {
            results: proto_results,
        }))
    }

    async fn health_check(
        &self,
        _request: Request<vectorizer::HealthCheckRequest>,
    ) -> Result<Response<vectorizer::HealthCheckResponse>, Status> {
        Ok(Response::new(vectorizer::HealthCheckResponse {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        }))
    }

    async fn get_stats(
        &self,
        _request: Request<vectorizer::GetStatsRequest>,
    ) -> Result<Response<vectorizer::GetStatsResponse>, Status> {
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

        Ok(Response::new(vectorizer::GetStatsResponse {
            collections_count: collections.len() as u32,
            total_vectors: total_vectors as u64,
            uptime_seconds: 0, // TODO: Track uptime
            version: env!("CARGO_PKG_VERSION").to_string(),
        }))
    }
}

