//! `impl Collections for QdrantGrpcService` — extracted from the prior
//! monolithic `qdrant_grpc.rs` (phase3_split-qdrant-grpc). The impl block
//! itself is unchanged; only the file it lives in is new.

use std::time::Instant;

use tonic::{Request, Response, Status};
use tracing::{error, info};

use super::QdrantGrpcService;
use crate::grpc::qdrant_proto::collections_server::Collections;
use crate::grpc::qdrant_proto::*;

// ============================================================================
// Collections Service Implementation
// ============================================================================

#[tonic::async_trait]
impl Collections for QdrantGrpcService {
    async fn get(
        &self,
        request: Request<GetCollectionInfoRequest>,
    ) -> Result<Response<GetCollectionInfoResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Get collection info");

        let collection = self
            .store
            .get_collection(&req.collection_name)
            .map_err(|e| Status::not_found(format!("Collection not found: {}", e)))?;

        let config = collection.config();
        let vector_count = collection.vector_count() as u64;

        // Convert metric to distance
        let distance = match config.metric {
            crate::models::DistanceMetric::Cosine => Distance::Cosine as i32,
            crate::models::DistanceMetric::Euclidean => Distance::Euclid as i32,
            crate::models::DistanceMetric::DotProduct => Distance::Dot as i32,
        };

        let result = GetCollectionInfoResponse {
            result: Some(CollectionInfo {
                status: CollectionStatus::Green as i32,
                optimizer_status: Some(OptimizerStatus {
                    ok: true,
                    error: String::new(),
                }),
                points_count: Some(vector_count),
                indexed_vectors_count: Some(vector_count),
                segments_count: 1,
                config: Some(CollectionConfig {
                    params: Some(CollectionParams {
                        vectors_config: Some(VectorsConfig {
                            config: Some(vectors_config::Config::Params(VectorParams {
                                size: config.dimension as u64,
                                distance,
                                hnsw_config: None,
                                quantization_config: None,
                                on_disk: Some(false),
                                datatype: None,
                                multivector_config: None,
                            })),
                        }),
                        shard_number: 1,
                        sharding_method: None,
                        replication_factor: Some(1),
                        write_consistency_factor: Some(1),
                        read_fan_out_factor: None,
                        on_disk_payload: false,
                        sparse_vectors_config: None,
                    }),
                    hnsw_config: Some(HnswConfigDiff {
                        m: Some(config.hnsw_config.m as u64),
                        ef_construct: Some(config.hnsw_config.ef_construction as u64),
                        full_scan_threshold: Some(10000),
                        max_indexing_threads: Some(0),
                        on_disk: Some(false),
                        payload_m: None,
                        inline_storage: None,
                    }),
                    optimizer_config: None,
                    wal_config: None,
                    quantization_config: None,
                    strict_mode_config: None,
                    metadata: std::collections::HashMap::new(),
                }),
                payload_schema: std::collections::HashMap::new(),
                warnings: vec![],
            }),
            time: start.elapsed().as_secs_f64(),
        };

        Ok(Response::new(result))
    }

    async fn list(
        &self,
        _request: Request<ListCollectionsRequest>,
    ) -> Result<Response<ListCollectionsResponse>, Status> {
        let start = Instant::now();
        info!("Qdrant gRPC: List collections");

        let collections: Vec<CollectionDescription> = self
            .store
            .list_collections()
            .into_iter()
            .map(|name| CollectionDescription { name })
            .collect();

        let result = ListCollectionsResponse {
            collections,
            time: start.elapsed().as_secs_f64(),
        };

        Ok(Response::new(result))
    }

    async fn create(
        &self,
        request: Request<CreateCollection>,
    ) -> Result<Response<CollectionOperationResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Create collection");

        // Extract dimension from vectors config
        let dimension = req
            .vectors_config
            .as_ref()
            .and_then(|vc| match &vc.config {
                Some(vectors_config::Config::Params(p)) => Some(p.size as usize),
                Some(vectors_config::Config::ParamsMap(m)) => {
                    m.map.values().next().map(|p| p.size as usize)
                }
                None => None,
            })
            .unwrap_or(128);

        // Extract distance metric
        let distance = req
            .vectors_config
            .as_ref()
            .and_then(|vc| match &vc.config {
                Some(vectors_config::Config::Params(p)) => Some(p.distance),
                Some(vectors_config::Config::ParamsMap(m)) => {
                    m.map.values().next().map(|p| p.distance)
                }
                None => None,
            })
            .unwrap_or(Distance::Cosine as i32);

        let metric = match distance {
            d if d == Distance::Cosine as i32 => crate::models::DistanceMetric::Cosine,
            d if d == Distance::Euclid as i32 => crate::models::DistanceMetric::Euclidean,
            d if d == Distance::Dot as i32 => crate::models::DistanceMetric::DotProduct,
            _ => crate::models::DistanceMetric::Cosine,
        };

        // Build config
        let mut config = crate::models::CollectionConfig::default();
        config.dimension = dimension;
        config.metric = metric;

        // Extract HNSW config if provided
        if let Some(hnsw) = req.hnsw_config {
            if let Some(m) = hnsw.m {
                config.hnsw_config.m = m as usize;
            }
            if let Some(ef) = hnsw.ef_construct {
                config.hnsw_config.ef_construction = ef as usize;
            }
        }

        self.store
            .create_collection(&req.collection_name, config)
            .map_err(|e| Status::internal(format!("Failed to create collection: {}", e)))?;

        Ok(Response::new(CollectionOperationResponse {
            result: true,
            time: start.elapsed().as_secs_f64(),
        }))
    }

    async fn update(
        &self,
        request: Request<UpdateCollection>,
    ) -> Result<Response<CollectionOperationResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Update collection");

        // Verify collection exists
        let _collection = self
            .store
            .get_collection(&req.collection_name)
            .map_err(|e| Status::not_found(format!("Collection not found: {}", e)))?;

        Ok(Response::new(CollectionOperationResponse {
            result: true,
            time: start.elapsed().as_secs_f64(),
        }))
    }

    async fn delete(
        &self,
        request: Request<DeleteCollection>,
    ) -> Result<Response<CollectionOperationResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Delete collection");

        self.store
            .delete_collection(&req.collection_name)
            .map_err(|e| Status::internal(format!("Failed to delete collection: {}", e)))?;

        Ok(Response::new(CollectionOperationResponse {
            result: true,
            time: start.elapsed().as_secs_f64(),
        }))
    }

    async fn update_aliases(
        &self,
        request: Request<ChangeAliases>,
    ) -> Result<Response<CollectionOperationResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!("Qdrant gRPC: Update aliases");

        for action in req.actions {
            if let Some(action_type) = action.action {
                match action_type {
                    alias_operations::Action::CreateAlias(create) => {
                        self.store
                            .create_alias(&create.alias_name, &create.collection_name)
                            .map_err(|e| {
                                Status::internal(format!("Failed to create alias: {}", e))
                            })?;
                    }
                    alias_operations::Action::DeleteAlias(delete) => {
                        self.store.delete_alias(&delete.alias_name).map_err(|e| {
                            Status::internal(format!("Failed to delete alias: {}", e))
                        })?;
                    }
                    alias_operations::Action::RenameAlias(rename) => {
                        let _ = self.store.delete_alias(&rename.old_alias_name);
                        self.store
                            .create_alias(&rename.new_alias_name, &rename.old_alias_name)
                            .map_err(|e| {
                                Status::internal(format!("Failed to rename alias: {}", e))
                            })?;
                    }
                }
            }
        }

        Ok(Response::new(CollectionOperationResponse {
            result: true,
            time: start.elapsed().as_secs_f64(),
        }))
    }

    async fn list_collection_aliases(
        &self,
        request: Request<ListCollectionAliasesRequest>,
    ) -> Result<Response<ListAliasesResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: List collection aliases");

        let aliases: Vec<AliasDescription> = self
            .store
            .list_aliases()
            .into_iter()
            .filter(|(_, collection)| collection == &req.collection_name)
            .map(|(alias, collection)| AliasDescription {
                alias_name: alias,
                collection_name: collection,
            })
            .collect();

        Ok(Response::new(ListAliasesResponse {
            aliases,
            time: start.elapsed().as_secs_f64(),
        }))
    }

    async fn list_aliases(
        &self,
        _request: Request<ListAliasesRequest>,
    ) -> Result<Response<ListAliasesResponse>, Status> {
        let start = Instant::now();
        info!("Qdrant gRPC: List all aliases");

        let aliases: Vec<AliasDescription> = self
            .store
            .list_aliases()
            .into_iter()
            .map(|(alias, collection)| AliasDescription {
                alias_name: alias,
                collection_name: collection,
            })
            .collect();

        Ok(Response::new(ListAliasesResponse {
            aliases,
            time: start.elapsed().as_secs_f64(),
        }))
    }

    async fn collection_cluster_info(
        &self,
        request: Request<CollectionClusterInfoRequest>,
    ) -> Result<Response<CollectionClusterInfoResponse>, Status> {
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Get collection cluster info");

        let _collection = self
            .store
            .get_collection(&req.collection_name)
            .map_err(|e| Status::not_found(format!("Collection not found: {}", e)))?;

        Ok(Response::new(CollectionClusterInfoResponse {
            peer_id: 0,
            shard_count: 1,
            local_shards: vec![LocalShardInfo {
                shard_id: 0,
                shard_key: None,
                points_count: 0,
                state: ReplicaState::Active as i32,
            }],
            remote_shards: vec![],
            shard_transfers: vec![],
            resharding_operations: vec![],
        }))
    }

    async fn collection_exists(
        &self,
        request: Request<CollectionExistsRequest>,
    ) -> Result<Response<CollectionExistsResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();

        let exists = self.store.get_collection(&req.collection_name).is_ok();

        Ok(Response::new(CollectionExistsResponse {
            result: Some(CollectionExists { exists }),
            time: start.elapsed().as_secs_f64(),
        }))
    }

    async fn update_collection_cluster_setup(
        &self,
        request: Request<UpdateCollectionClusterSetupRequest>,
    ) -> Result<Response<UpdateCollectionClusterSetupResponse>, Status> {
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Update collection cluster setup");

        let _collection = self
            .store
            .get_collection(&req.collection_name)
            .map_err(|e| Status::not_found(format!("Collection not found: {}", e)))?;

        Ok(Response::new(UpdateCollectionClusterSetupResponse {
            result: true,
        }))
    }

    async fn create_shard_key(
        &self,
        request: Request<CreateShardKeyRequest>,
    ) -> Result<Response<CreateShardKeyResponse>, Status> {
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Create shard key");

        let _collection = self
            .store
            .get_collection(&req.collection_name)
            .map_err(|e| Status::not_found(format!("Collection not found: {}", e)))?;

        Ok(Response::new(CreateShardKeyResponse { result: true }))
    }

    async fn delete_shard_key(
        &self,
        request: Request<DeleteShardKeyRequest>,
    ) -> Result<Response<DeleteShardKeyResponse>, Status> {
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Delete shard key");

        let _collection = self
            .store
            .get_collection(&req.collection_name)
            .map_err(|e| Status::not_found(format!("Collection not found: {}", e)))?;

        Ok(Response::new(DeleteShardKeyResponse { result: true }))
    }
}
