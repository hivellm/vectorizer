//! Qdrant-compatible gRPC service implementations
//!
//! This module implements the Qdrant gRPC API on top of Vectorizer,
//! enabling drop-in replacement for Qdrant clients using gRPC.

use std::sync::Arc;
use std::time::Instant;

use tonic::{Request, Response, Status};
use tracing::{error, info};

use crate::VectorStore;
use crate::grpc::qdrant_proto::collections_server::Collections;
use crate::grpc::qdrant_proto::points_server::Points;
use crate::grpc::qdrant_proto::snapshots_server::Snapshots;
use crate::grpc::qdrant_proto::*;
use crate::models::{Payload, Vector};

/// Qdrant-compatible gRPC service
#[derive(Clone)]
pub struct QdrantGrpcService {
    store: Arc<VectorStore>,
    snapshot_manager: Option<Arc<crate::storage::SnapshotManager>>,
}

impl QdrantGrpcService {
    pub fn new(store: Arc<VectorStore>) -> Self {
        Self {
            store,
            snapshot_manager: None,
        }
    }

    pub fn with_snapshot_manager(
        store: Arc<VectorStore>,
        snapshot_manager: Arc<crate::storage::SnapshotManager>,
    ) -> Self {
        Self {
            store,
            snapshot_manager: Some(snapshot_manager),
        }
    }
}

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

// ============================================================================
// Points Service Implementation
// ============================================================================

#[tonic::async_trait]
impl Points for QdrantGrpcService {
    async fn upsert(
        &self,
        request: Request<UpsertPoints>,
    ) -> Result<Response<PointsOperationResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Upsert points");

        let mut collection = self
            .store
            .get_collection_mut(&req.collection_name)
            .map_err(|e| Status::not_found(format!("Collection not found: {}", e)))?;

        for point in req.points {
            let id = match point.id.and_then(|p| p.point_id_options) {
                Some(point_id::PointIdOptions::Num(n)) => n.to_string(),
                Some(point_id::PointIdOptions::Uuid(u)) => u,
                None => continue,
            };

            let vector_data = match point.vectors.and_then(|v| v.vectors_options) {
                Some(vectors::VectorsOptions::Vector(v)) => v.data,
                Some(vectors::VectorsOptions::Vectors(map)) => map
                    .vectors
                    .values()
                    .next()
                    .map(|v| v.data.clone())
                    .unwrap_or_default(),
                None => continue,
            };

            let payload: Option<Payload> = if point.payload.is_empty() {
                None
            } else {
                Some(Payload {
                    data: convert_payload_to_json(&point.payload),
                })
            };

            let vec = if let Some(p) = payload {
                Vector::with_payload(id.clone(), vector_data, p)
            } else {
                Vector::new(id.clone(), vector_data)
            };

            if let Err(e) = collection.add_vector(id.clone(), vec) {
                error!("Failed to upsert point {}: {}", id, e);
            }
        }

        Ok(Response::new(PointsOperationResponse {
            result: Some(UpdateResult {
                operation_id: Some(0),
                status: UpdateStatus::Completed as i32,
            }),
            time: start.elapsed().as_secs_f64(),
            usage: None,
        }))
    }

    async fn delete(
        &self,
        request: Request<DeletePoints>,
    ) -> Result<Response<PointsOperationResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Delete points");

        let mut collection = self
            .store
            .get_collection_mut(&req.collection_name)
            .map_err(|e| Status::not_found(format!("Collection not found: {}", e)))?;

        if let Some(selector) = req.points {
            match selector.points_selector_one_of {
                Some(points_selector::PointsSelectorOneOf::Points(ids)) => {
                    for point_id in ids.ids {
                        let id = match point_id.point_id_options {
                            Some(point_id::PointIdOptions::Num(n)) => n.to_string(),
                            Some(point_id::PointIdOptions::Uuid(u)) => u,
                            None => continue,
                        };
                        let _ = collection.delete_vector(&id);
                    }
                }
                Some(points_selector::PointsSelectorOneOf::Filter(_filter)) => {
                    info!("Filter-based deletion not fully implemented in gRPC");
                }
                None => {}
            }
        }

        Ok(Response::new(PointsOperationResponse {
            result: Some(UpdateResult {
                operation_id: Some(0),
                status: UpdateStatus::Completed as i32,
            }),
            time: start.elapsed().as_secs_f64(),
            usage: None,
        }))
    }

    async fn get(&self, request: Request<GetPoints>) -> Result<Response<GetResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Get points");

        let collection = self
            .store
            .get_collection(&req.collection_name)
            .map_err(|e| Status::not_found(format!("Collection not found: {}", e)))?;

        let with_payload = req
            .with_payload
            .map(|w| w.selector_options.is_some())
            .unwrap_or(true);
        let with_vectors = req
            .with_vectors
            .map(|w| w.selector_options.is_some())
            .unwrap_or(true);

        let mut result_points = vec![];

        for point_id in req.ids {
            let id = match point_id.point_id_options {
                Some(point_id::PointIdOptions::Num(n)) => n.to_string(),
                Some(point_id::PointIdOptions::Uuid(u)) => u,
                None => continue,
            };

            if let Ok(vec) = collection.get_vector(&id) {
                let empty_json = serde_json::Value::Object(serde_json::Map::new());
                let payload_json = vec.payload.as_ref().map(|p| &p.data).unwrap_or(&empty_json);

                result_points.push(RetrievedPoint {
                    id: Some(PointId {
                        point_id_options: Some(point_id::PointIdOptions::Uuid(id)),
                    }),
                    payload: if with_payload {
                        convert_json_to_payload(payload_json)
                    } else {
                        std::collections::HashMap::new()
                    },
                    vectors: if with_vectors {
                        Some(VectorsOutput {
                            vectors_options: Some(vectors_output::VectorsOptions::Vector(
                                VectorOutput {
                                    data: vec.data.clone(),
                                    indices: None,
                                    vectors_count: None,
                                    vector: Some(vector_output::Vector::Dense(DenseVector {
                                        data: vec.data.clone(),
                                    })),
                                },
                            )),
                        })
                    } else {
                        None
                    },
                    shard_key: None,
                    order_value: None,
                });
            }
        }

        Ok(Response::new(GetResponse {
            result: result_points,
            time: start.elapsed().as_secs_f64(),
            usage: None,
        }))
    }

    async fn update_vectors(
        &self,
        request: Request<UpdatePointVectors>,
    ) -> Result<Response<PointsOperationResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Update vectors");

        let mut collection = self
            .store
            .get_collection_mut(&req.collection_name)
            .map_err(|e| Status::not_found(format!("Collection not found: {}", e)))?;

        for point in req.points {
            let id = match point.id.and_then(|p| p.point_id_options) {
                Some(point_id::PointIdOptions::Num(n)) => n.to_string(),
                Some(point_id::PointIdOptions::Uuid(u)) => u,
                None => continue,
            };

            let vector_data = match point.vectors.and_then(|v| v.vectors_options) {
                Some(vectors::VectorsOptions::Vector(v)) => v.data,
                Some(vectors::VectorsOptions::Vectors(map)) => map
                    .vectors
                    .values()
                    .next()
                    .map(|v| v.data.clone())
                    .unwrap_or_default(),
                None => continue,
            };

            let payload = collection
                .get_vector(&id)
                .ok()
                .and_then(|v| v.payload.clone());

            let vec = if let Some(p) = payload {
                Vector::with_payload(id, vector_data, p)
            } else {
                Vector::new(id, vector_data)
            };
            let _ = collection.update_vector(vec);
        }

        Ok(Response::new(PointsOperationResponse {
            result: Some(UpdateResult {
                operation_id: Some(0),
                status: UpdateStatus::Completed as i32,
            }),
            time: start.elapsed().as_secs_f64(),
            usage: None,
        }))
    }

    async fn delete_vectors(
        &self,
        request: Request<DeletePointVectors>,
    ) -> Result<Response<PointsOperationResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Delete vectors");

        Ok(Response::new(PointsOperationResponse {
            result: Some(UpdateResult {
                operation_id: Some(0),
                status: UpdateStatus::Completed as i32,
            }),
            time: start.elapsed().as_secs_f64(),
            usage: None,
        }))
    }

    async fn set_payload(
        &self,
        request: Request<SetPayloadPoints>,
    ) -> Result<Response<PointsOperationResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Set payload");

        let mut collection = self
            .store
            .get_collection_mut(&req.collection_name)
            .map_err(|e| Status::not_found(format!("Collection not found: {}", e)))?;

        let new_payload_json = convert_payload_to_json(&req.payload);

        if let Some(selector) = req.points_selector {
            match selector.points_selector_one_of {
                Some(points_selector::PointsSelectorOneOf::Points(ids)) => {
                    for point_id in ids.ids {
                        let id = match point_id.point_id_options {
                            Some(point_id::PointIdOptions::Num(n)) => n.to_string(),
                            Some(point_id::PointIdOptions::Uuid(u)) => u,
                            None => continue,
                        };

                        if let Ok(vec) = collection.get_vector(&id) {
                            let mut merged = vec
                                .payload
                                .as_ref()
                                .map(|p| p.data.clone())
                                .unwrap_or(serde_json::json!({}));

                            if let serde_json::Value::Object(m1) = &mut merged {
                                if let serde_json::Value::Object(m2) = &new_payload_json {
                                    for (k, v) in m2 {
                                        m1.insert(k.clone(), v.clone());
                                    }
                                }
                            }

                            let updated = Vector::with_payload(
                                id,
                                vec.data.clone(),
                                Payload { data: merged },
                            );
                            let _ = collection.update_vector(updated);
                        }
                    }
                }
                Some(points_selector::PointsSelectorOneOf::Filter(_)) => {
                    info!("Filter-based payload update not fully implemented in gRPC");
                }
                None => {}
            }
        }

        Ok(Response::new(PointsOperationResponse {
            result: Some(UpdateResult {
                operation_id: Some(0),
                status: UpdateStatus::Completed as i32,
            }),
            time: start.elapsed().as_secs_f64(),
            usage: None,
        }))
    }

    async fn overwrite_payload(
        &self,
        request: Request<SetPayloadPoints>,
    ) -> Result<Response<PointsOperationResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Overwrite payload");

        let mut collection = self
            .store
            .get_collection_mut(&req.collection_name)
            .map_err(|e| Status::not_found(format!("Collection not found: {}", e)))?;

        let new_payload = convert_payload_to_json(&req.payload);

        if let Some(selector) = req.points_selector {
            match selector.points_selector_one_of {
                Some(points_selector::PointsSelectorOneOf::Points(ids)) => {
                    for point_id in ids.ids {
                        let id = match point_id.point_id_options {
                            Some(point_id::PointIdOptions::Num(n)) => n.to_string(),
                            Some(point_id::PointIdOptions::Uuid(u)) => u,
                            None => continue,
                        };

                        if let Ok(vec) = collection.get_vector(&id) {
                            let updated = Vector::with_payload(
                                id,
                                vec.data.clone(),
                                Payload {
                                    data: new_payload.clone(),
                                },
                            );
                            let _ = collection.update_vector(updated);
                        }
                    }
                }
                Some(points_selector::PointsSelectorOneOf::Filter(_)) => {
                    info!("Filter-based payload overwrite not fully implemented in gRPC");
                }
                None => {}
            }
        }

        Ok(Response::new(PointsOperationResponse {
            result: Some(UpdateResult {
                operation_id: Some(0),
                status: UpdateStatus::Completed as i32,
            }),
            time: start.elapsed().as_secs_f64(),
            usage: None,
        }))
    }

    async fn delete_payload(
        &self,
        request: Request<DeletePayloadPoints>,
    ) -> Result<Response<PointsOperationResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Delete payload keys");

        let mut collection = self
            .store
            .get_collection_mut(&req.collection_name)
            .map_err(|e| Status::not_found(format!("Collection not found: {}", e)))?;

        if let Some(selector) = req.points_selector {
            match selector.points_selector_one_of {
                Some(points_selector::PointsSelectorOneOf::Points(ids)) => {
                    for point_id in ids.ids {
                        let id = match point_id.point_id_options {
                            Some(point_id::PointIdOptions::Num(n)) => n.to_string(),
                            Some(point_id::PointIdOptions::Uuid(u)) => u,
                            None => continue,
                        };

                        if let Ok(vec) = collection.get_vector(&id) {
                            let mut payload = vec
                                .payload
                                .as_ref()
                                .map(|p| p.data.clone())
                                .unwrap_or(serde_json::json!({}));

                            if let serde_json::Value::Object(ref mut map) = payload {
                                for key in &req.keys {
                                    map.remove(key);
                                }
                            }
                            let updated = Vector::with_payload(
                                id,
                                vec.data.clone(),
                                Payload { data: payload },
                            );
                            let _ = collection.update_vector(updated);
                        }
                    }
                }
                Some(points_selector::PointsSelectorOneOf::Filter(_)) => {
                    info!("Filter-based payload deletion not fully implemented in gRPC");
                }
                None => {}
            }
        }

        Ok(Response::new(PointsOperationResponse {
            result: Some(UpdateResult {
                operation_id: Some(0),
                status: UpdateStatus::Completed as i32,
            }),
            time: start.elapsed().as_secs_f64(),
            usage: None,
        }))
    }

    async fn clear_payload(
        &self,
        request: Request<ClearPayloadPoints>,
    ) -> Result<Response<PointsOperationResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Clear payload");

        let mut collection = self
            .store
            .get_collection_mut(&req.collection_name)
            .map_err(|e| Status::not_found(format!("Collection not found: {}", e)))?;

        if let Some(selector) = req.points {
            match selector.points_selector_one_of {
                Some(points_selector::PointsSelectorOneOf::Points(ids)) => {
                    for point_id in ids.ids {
                        let id = match point_id.point_id_options {
                            Some(point_id::PointIdOptions::Num(n)) => n.to_string(),
                            Some(point_id::PointIdOptions::Uuid(u)) => u,
                            None => continue,
                        };

                        if let Ok(vec) = collection.get_vector(&id) {
                            let updated = Vector::new(id, vec.data.clone());
                            let _ = collection.update_vector(updated);
                        }
                    }
                }
                Some(points_selector::PointsSelectorOneOf::Filter(_)) => {
                    info!("Filter-based payload clear not fully implemented in gRPC");
                }
                None => {}
            }
        }

        Ok(Response::new(PointsOperationResponse {
            result: Some(UpdateResult {
                operation_id: Some(0),
                status: UpdateStatus::Completed as i32,
            }),
            time: start.elapsed().as_secs_f64(),
            usage: None,
        }))
    }

    async fn create_field_index(
        &self,
        request: Request<CreateFieldIndexCollection>,
    ) -> Result<Response<PointsOperationResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, field = %req.field_name, "Qdrant gRPC: Create field index");

        let _collection = self
            .store
            .get_collection(&req.collection_name)
            .map_err(|e| Status::not_found(format!("Collection not found: {}", e)))?;

        Ok(Response::new(PointsOperationResponse {
            result: Some(UpdateResult {
                operation_id: Some(0),
                status: UpdateStatus::Completed as i32,
            }),
            time: start.elapsed().as_secs_f64(),
            usage: None,
        }))
    }

    async fn delete_field_index(
        &self,
        request: Request<DeleteFieldIndexCollection>,
    ) -> Result<Response<PointsOperationResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, field = %req.field_name, "Qdrant gRPC: Delete field index");

        let _collection = self
            .store
            .get_collection(&req.collection_name)
            .map_err(|e| Status::not_found(format!("Collection not found: {}", e)))?;

        Ok(Response::new(PointsOperationResponse {
            result: Some(UpdateResult {
                operation_id: Some(0),
                status: UpdateStatus::Completed as i32,
            }),
            time: start.elapsed().as_secs_f64(),
            usage: None,
        }))
    }

    async fn search(
        &self,
        request: Request<SearchPoints>,
    ) -> Result<Response<SearchResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Search points");

        let collection = self
            .store
            .get_collection(&req.collection_name)
            .map_err(|e| Status::not_found(format!("Collection not found: {}", e)))?;

        let limit = req.limit as usize;
        let with_payload = req
            .with_payload
            .map(|w| w.selector_options.is_some())
            .unwrap_or(true);

        let results = collection
            .search(&req.vector, limit)
            .map_err(|e| Status::internal(format!("Search failed: {}", e)))?;

        let scored_points: Vec<ScoredPoint> = results
            .into_iter()
            .map(|r| ScoredPoint {
                id: Some(PointId {
                    point_id_options: Some(point_id::PointIdOptions::Uuid(r.id.clone())),
                }),
                payload: if with_payload {
                    r.payload
                        .as_ref()
                        .map(|p| convert_json_to_payload(&p.data))
                        .unwrap_or_default()
                } else {
                    std::collections::HashMap::new()
                },
                score: r.score,
                vectors: None,
                version: 0,
                shard_key: None,
                order_value: None,
            })
            .collect();

        Ok(Response::new(SearchResponse {
            result: scored_points,
            time: start.elapsed().as_secs_f64(),
            usage: None,
        }))
    }

    async fn search_batch(
        &self,
        request: Request<SearchBatchPoints>,
    ) -> Result<Response<SearchBatchResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Search batch");

        let collection = self
            .store
            .get_collection(&req.collection_name)
            .map_err(|e| Status::not_found(format!("Collection not found: {}", e)))?;

        let mut batch_results = vec![];

        for search_req in req.search_points {
            let limit = search_req.limit as usize;
            let with_payload = search_req
                .with_payload
                .map(|w| w.selector_options.is_some())
                .unwrap_or(true);

            let results = collection
                .search(&search_req.vector, limit)
                .map_err(|e| Status::internal(format!("Search failed: {}", e)))?;

            let scored_points: Vec<ScoredPoint> = results
                .into_iter()
                .map(|r| ScoredPoint {
                    id: Some(PointId {
                        point_id_options: Some(point_id::PointIdOptions::Uuid(r.id.clone())),
                    }),
                    payload: if with_payload {
                        r.payload
                            .as_ref()
                            .map(|p| convert_json_to_payload(&p.data))
                            .unwrap_or_default()
                    } else {
                        std::collections::HashMap::new()
                    },
                    score: r.score,
                    vectors: None,
                    version: 0,
                    shard_key: None,
                    order_value: None,
                })
                .collect();

            batch_results.push(BatchResult {
                result: scored_points,
            });
        }

        Ok(Response::new(SearchBatchResponse {
            result: batch_results,
            time: start.elapsed().as_secs_f64(),
            usage: None,
        }))
    }

    async fn search_groups(
        &self,
        request: Request<SearchPointGroups>,
    ) -> Result<Response<SearchGroupsResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Search groups");

        let collection = self
            .store
            .get_collection(&req.collection_name)
            .map_err(|e| Status::not_found(format!("Collection not found: {}", e)))?;

        let limit = req.limit as usize;
        let group_by = req.group_by;
        let group_size = req.group_size as usize;

        let results = collection
            .search(&req.vector, limit * group_size)
            .map_err(|e| Status::internal(format!("Search failed: {}", e)))?;

        let mut groups: std::collections::HashMap<String, Vec<ScoredPoint>> =
            std::collections::HashMap::new();

        for r in results {
            let group_key = r
                .payload
                .as_ref()
                .and_then(|p| p.data.get(&group_by))
                .and_then(|v| v.as_str())
                .unwrap_or("default")
                .to_string();

            let entry = groups.entry(group_key).or_default();
            if entry.len() < group_size {
                entry.push(ScoredPoint {
                    id: Some(PointId {
                        point_id_options: Some(point_id::PointIdOptions::Uuid(r.id.clone())),
                    }),
                    payload: r
                        .payload
                        .as_ref()
                        .map(|p| convert_json_to_payload(&p.data))
                        .unwrap_or_default(),
                    score: r.score,
                    vectors: None,
                    version: 0,
                    shard_key: None,
                    order_value: None,
                });
            }
        }

        let point_groups: Vec<PointGroup> = groups
            .into_iter()
            .take(limit)
            .map(|(key, hits)| PointGroup {
                id: Some(GroupId {
                    kind: Some(group_id::Kind::StringValue(key)),
                }),
                hits,
                lookup: None,
            })
            .collect();

        Ok(Response::new(SearchGroupsResponse {
            result: Some(GroupsResult {
                groups: point_groups,
            }),
            time: start.elapsed().as_secs_f64(),
            usage: None,
        }))
    }

    async fn scroll(
        &self,
        request: Request<ScrollPoints>,
    ) -> Result<Response<ScrollResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Scroll points");

        let collection = self
            .store
            .get_collection(&req.collection_name)
            .map_err(|e| Status::not_found(format!("Collection not found: {}", e)))?;

        let limit = req.limit.unwrap_or(10) as usize;
        let with_payload = req
            .with_payload
            .map(|w| w.selector_options.is_some())
            .unwrap_or(true);
        let with_vectors = req
            .with_vectors
            .map(|w| w.selector_options.is_some())
            .unwrap_or(false);

        let all_vectors = collection.get_all_vectors();

        let points: Vec<RetrievedPoint> = all_vectors
            .into_iter()
            .take(limit)
            .map(|v| {
                let payload_map = v
                    .payload
                    .as_ref()
                    .map(|p| convert_json_to_payload(&p.data))
                    .unwrap_or_default();

                RetrievedPoint {
                    id: Some(PointId {
                        point_id_options: Some(point_id::PointIdOptions::Uuid(v.id.clone())),
                    }),
                    payload: if with_payload {
                        payload_map
                    } else {
                        std::collections::HashMap::new()
                    },
                    vectors: if with_vectors {
                        Some(VectorsOutput {
                            vectors_options: Some(vectors_output::VectorsOptions::Vector(
                                VectorOutput {
                                    data: v.data.clone(),
                                    indices: None,
                                    vectors_count: None,
                                    vector: Some(vector_output::Vector::Dense(DenseVector {
                                        data: v.data.clone(),
                                    })),
                                },
                            )),
                        })
                    } else {
                        None
                    },
                    shard_key: None,
                    order_value: None,
                }
            })
            .collect();

        let next_page_offset = if points.len() >= limit {
            points.last().and_then(|p| p.id.clone())
        } else {
            None
        };

        Ok(Response::new(ScrollResponse {
            result: points,
            next_page_offset,
            time: start.elapsed().as_secs_f64(),
            usage: None,
        }))
    }

    async fn recommend(
        &self,
        request: Request<RecommendPoints>,
    ) -> Result<Response<RecommendResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Recommend points");

        let collection = self
            .store
            .get_collection(&req.collection_name)
            .map_err(|e| Status::not_found(format!("Collection not found: {}", e)))?;

        let limit = req.limit as usize;

        let mut positive_vectors: Vec<Vec<f32>> = vec![];
        for pos in &req.positive {
            if let Some(point_id::PointIdOptions::Uuid(id)) = pos.point_id_options.as_ref() {
                if let Ok(vec) = collection.get_vector(id) {
                    positive_vectors.push(vec.data.clone());
                }
            }
        }

        if positive_vectors.is_empty() {
            return Ok(Response::new(RecommendResponse {
                result: vec![],
                time: start.elapsed().as_secs_f64(),
                usage: None,
            }));
        }

        let dim = positive_vectors[0].len();
        let mut avg_vector = vec![0.0f32; dim];
        for vec in &positive_vectors {
            for (i, v) in vec.iter().enumerate() {
                avg_vector[i] += v;
            }
        }
        for v in &mut avg_vector {
            *v /= positive_vectors.len() as f32;
        }

        let results = collection
            .search(&avg_vector, limit)
            .map_err(|e| Status::internal(format!("Search failed: {}", e)))?;

        let positive_ids: std::collections::HashSet<String> = req
            .positive
            .iter()
            .filter_map(|p| match p.point_id_options.as_ref() {
                Some(point_id::PointIdOptions::Uuid(id)) => Some(id.clone()),
                Some(point_id::PointIdOptions::Num(n)) => Some(n.to_string()),
                None => None,
            })
            .collect();

        let scored_points: Vec<ScoredPoint> = results
            .into_iter()
            .filter(|r| !positive_ids.contains(&r.id))
            .map(|r| ScoredPoint {
                id: Some(PointId {
                    point_id_options: Some(point_id::PointIdOptions::Uuid(r.id.clone())),
                }),
                payload: r
                    .payload
                    .as_ref()
                    .map(|p| convert_json_to_payload(&p.data))
                    .unwrap_or_default(),
                score: r.score,
                vectors: None,
                version: 0,
                shard_key: None,
                order_value: None,
            })
            .collect();

        Ok(Response::new(RecommendResponse {
            result: scored_points,
            time: start.elapsed().as_secs_f64(),
            usage: None,
        }))
    }

    async fn recommend_batch(
        &self,
        request: Request<RecommendBatchPoints>,
    ) -> Result<Response<RecommendBatchResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Recommend batch");

        let mut batch_results = vec![];
        for _recommend_req in req.recommend_points {
            batch_results.push(BatchResult { result: vec![] });
        }

        Ok(Response::new(RecommendBatchResponse {
            result: batch_results,
            time: start.elapsed().as_secs_f64(),
            usage: None,
        }))
    }

    async fn recommend_groups(
        &self,
        request: Request<RecommendPointGroups>,
    ) -> Result<Response<RecommendGroupsResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Recommend groups");

        Ok(Response::new(RecommendGroupsResponse {
            result: Some(GroupsResult { groups: vec![] }),
            time: start.elapsed().as_secs_f64(),
            usage: None,
        }))
    }

    async fn discover(
        &self,
        request: Request<DiscoverPoints>,
    ) -> Result<Response<DiscoverResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Discover points");

        Ok(Response::new(DiscoverResponse {
            result: vec![],
            time: start.elapsed().as_secs_f64(),
            usage: None,
        }))
    }

    async fn discover_batch(
        &self,
        request: Request<DiscoverBatchPoints>,
    ) -> Result<Response<DiscoverBatchResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Discover batch");

        Ok(Response::new(DiscoverBatchResponse {
            result: vec![],
            time: start.elapsed().as_secs_f64(),
            usage: None,
        }))
    }

    async fn count(
        &self,
        request: Request<CountPoints>,
    ) -> Result<Response<CountResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Count points");

        let collection = self
            .store
            .get_collection(&req.collection_name)
            .map_err(|e| Status::not_found(format!("Collection not found: {}", e)))?;

        let count = collection.vector_count() as u64;

        Ok(Response::new(CountResponse {
            result: Some(CountResult { count }),
            time: start.elapsed().as_secs_f64(),
            usage: None,
        }))
    }

    async fn update_batch(
        &self,
        request: Request<UpdateBatchPoints>,
    ) -> Result<Response<UpdateBatchResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Update batch");

        let mut statuses = vec![];
        for _op in req.operations {
            statuses.push(UpdateResult {
                operation_id: Some(0),
                status: UpdateStatus::Completed as i32,
            });
        }

        Ok(Response::new(UpdateBatchResponse {
            result: statuses,
            time: start.elapsed().as_secs_f64(),
            usage: None,
        }))
    }

    async fn query(
        &self,
        request: Request<QueryPoints>,
    ) -> Result<Response<QueryResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Query points");

        let collection = self
            .store
            .get_collection(&req.collection_name)
            .map_err(|e| Status::not_found(format!("Collection not found: {}", e)))?;

        let limit = req.limit.unwrap_or(10) as usize;

        let query_vector: Option<Vec<f32>> =
            req.query.and_then(|q| q.variant).and_then(|v| match v {
                query::Variant::Nearest(vi) => vi.variant.and_then(|vv| match vv {
                    vector_input::Variant::Dense(d) => Some(d.data),
                    _ => None,
                }),
                _ => None,
            });

        let scored_points = if let Some(vector) = query_vector {
            let results = collection
                .search(&vector, limit)
                .map_err(|e| Status::internal(format!("Search failed: {}", e)))?;

            results
                .into_iter()
                .map(|r| ScoredPoint {
                    id: Some(PointId {
                        point_id_options: Some(point_id::PointIdOptions::Uuid(r.id.clone())),
                    }),
                    payload: r
                        .payload
                        .as_ref()
                        .map(|p| convert_json_to_payload(&p.data))
                        .unwrap_or_default(),
                    score: r.score,
                    vectors: None,
                    version: 0,
                    shard_key: None,
                    order_value: None,
                })
                .collect()
        } else {
            vec![]
        };

        Ok(Response::new(QueryResponse {
            result: scored_points,
            time: start.elapsed().as_secs_f64(),
            usage: None,
        }))
    }

    async fn query_batch(
        &self,
        request: Request<QueryBatchPoints>,
    ) -> Result<Response<QueryBatchResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Query batch");

        let mut batch_results = vec![];
        for _query in req.query_points {
            batch_results.push(BatchResult { result: vec![] });
        }

        Ok(Response::new(QueryBatchResponse {
            result: batch_results,
            time: start.elapsed().as_secs_f64(),
            usage: None,
        }))
    }

    async fn query_groups(
        &self,
        request: Request<QueryPointGroups>,
    ) -> Result<Response<QueryGroupsResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Query groups");

        Ok(Response::new(QueryGroupsResponse {
            result: Some(GroupsResult { groups: vec![] }),
            time: start.elapsed().as_secs_f64(),
            usage: None,
        }))
    }

    async fn facet(
        &self,
        request: Request<FacetCounts>,
    ) -> Result<Response<FacetResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Facet counts");

        Ok(Response::new(FacetResponse {
            hits: vec![],
            time: start.elapsed().as_secs_f64(),
            usage: None,
        }))
    }

    async fn search_matrix_pairs(
        &self,
        request: Request<SearchMatrixPoints>,
    ) -> Result<Response<SearchMatrixPairsResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Search matrix pairs");

        Ok(Response::new(SearchMatrixPairsResponse {
            result: Some(SearchMatrixPairs { pairs: vec![] }),
            time: start.elapsed().as_secs_f64(),
            usage: None,
        }))
    }

    async fn search_matrix_offsets(
        &self,
        request: Request<SearchMatrixPoints>,
    ) -> Result<Response<SearchMatrixOffsetsResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Search matrix offsets");

        Ok(Response::new(SearchMatrixOffsetsResponse {
            result: Some(SearchMatrixOffsets {
                offsets_row: vec![],
                offsets_col: vec![],
                scores: vec![],
                ids: vec![],
            }),
            time: start.elapsed().as_secs_f64(),
            usage: None,
        }))
    }
}

// ============================================================================
// Snapshots Service Implementation
// ============================================================================

#[tonic::async_trait]
impl Snapshots for QdrantGrpcService {
    async fn create(
        &self,
        request: Request<CreateSnapshotRequest>,
    ) -> Result<Response<CreateSnapshotResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: Create snapshot");

        let snapshot_manager = self
            .snapshot_manager
            .as_ref()
            .ok_or_else(|| Status::unavailable("Snapshot manager not initialized"))?;

        let snapshot = snapshot_manager
            .create_snapshot()
            .map_err(|e| Status::internal(format!("Failed to create snapshot: {}", e)))?;

        Ok(Response::new(CreateSnapshotResponse {
            snapshot_description: Some(SnapshotDescription {
                name: snapshot.id,
                creation_time: Some(prost_types::Timestamp {
                    seconds: snapshot.created_at.timestamp(),
                    nanos: snapshot.created_at.timestamp_subsec_nanos() as i32,
                }),
                size: snapshot.size_bytes as i64,
                checksum: None,
            }),
            time: start.elapsed().as_secs_f64(),
        }))
    }

    async fn list(
        &self,
        request: Request<ListSnapshotsRequest>,
    ) -> Result<Response<ListSnapshotsResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, "Qdrant gRPC: List snapshots");

        let snapshot_manager = self
            .snapshot_manager
            .as_ref()
            .ok_or_else(|| Status::unavailable("Snapshot manager not initialized"))?;

        let snapshots = snapshot_manager
            .list_snapshots()
            .map_err(|e| Status::internal(format!("Failed to list snapshots: {}", e)))?;

        let descriptions: Vec<SnapshotDescription> = snapshots
            .into_iter()
            .map(|s| SnapshotDescription {
                name: s.id,
                creation_time: Some(prost_types::Timestamp {
                    seconds: s.created_at.timestamp(),
                    nanos: s.created_at.timestamp_subsec_nanos() as i32,
                }),
                size: s.size_bytes as i64,
                checksum: None,
            })
            .collect();

        Ok(Response::new(ListSnapshotsResponse {
            snapshot_descriptions: descriptions,
            time: start.elapsed().as_secs_f64(),
        }))
    }

    async fn delete(
        &self,
        request: Request<DeleteSnapshotRequest>,
    ) -> Result<Response<DeleteSnapshotResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(collection = %req.collection_name, snapshot = %req.snapshot_name, "Qdrant gRPC: Delete snapshot");

        let snapshot_manager = self
            .snapshot_manager
            .as_ref()
            .ok_or_else(|| Status::unavailable("Snapshot manager not initialized"))?;

        snapshot_manager
            .delete_snapshot(&req.snapshot_name)
            .map_err(|e| Status::internal(format!("Failed to delete snapshot: {}", e)))?;

        Ok(Response::new(DeleteSnapshotResponse {
            time: start.elapsed().as_secs_f64(),
        }))
    }

    async fn create_full(
        &self,
        _request: Request<CreateFullSnapshotRequest>,
    ) -> Result<Response<CreateSnapshotResponse>, Status> {
        let start = Instant::now();
        info!("Qdrant gRPC: Create full snapshot");

        let snapshot_manager = self
            .snapshot_manager
            .as_ref()
            .ok_or_else(|| Status::unavailable("Snapshot manager not initialized"))?;

        let snapshot = snapshot_manager
            .create_snapshot()
            .map_err(|e| Status::internal(format!("Failed to create snapshot: {}", e)))?;

        Ok(Response::new(CreateSnapshotResponse {
            snapshot_description: Some(SnapshotDescription {
                name: snapshot.id,
                creation_time: Some(prost_types::Timestamp {
                    seconds: snapshot.created_at.timestamp(),
                    nanos: snapshot.created_at.timestamp_subsec_nanos() as i32,
                }),
                size: snapshot.size_bytes as i64,
                checksum: None,
            }),
            time: start.elapsed().as_secs_f64(),
        }))
    }

    async fn list_full(
        &self,
        _request: Request<ListFullSnapshotsRequest>,
    ) -> Result<Response<ListSnapshotsResponse>, Status> {
        let start = Instant::now();
        info!("Qdrant gRPC: List full snapshots");

        let snapshot_manager = self
            .snapshot_manager
            .as_ref()
            .ok_or_else(|| Status::unavailable("Snapshot manager not initialized"))?;

        let snapshots = snapshot_manager
            .list_snapshots()
            .map_err(|e| Status::internal(format!("Failed to list snapshots: {}", e)))?;

        let descriptions: Vec<SnapshotDescription> = snapshots
            .into_iter()
            .map(|s| SnapshotDescription {
                name: s.id,
                creation_time: Some(prost_types::Timestamp {
                    seconds: s.created_at.timestamp(),
                    nanos: s.created_at.timestamp_subsec_nanos() as i32,
                }),
                size: s.size_bytes as i64,
                checksum: None,
            })
            .collect();

        Ok(Response::new(ListSnapshotsResponse {
            snapshot_descriptions: descriptions,
            time: start.elapsed().as_secs_f64(),
        }))
    }

    async fn delete_full(
        &self,
        request: Request<DeleteFullSnapshotRequest>,
    ) -> Result<Response<DeleteSnapshotResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        info!(snapshot = %req.snapshot_name, "Qdrant gRPC: Delete full snapshot");

        let snapshot_manager = self
            .snapshot_manager
            .as_ref()
            .ok_or_else(|| Status::unavailable("Snapshot manager not initialized"))?;

        snapshot_manager
            .delete_snapshot(&req.snapshot_name)
            .map_err(|e| Status::internal(format!("Failed to delete snapshot: {}", e)))?;

        Ok(Response::new(DeleteSnapshotResponse {
            time: start.elapsed().as_secs_f64(),
        }))
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn convert_payload_to_json(
    payload: &std::collections::HashMap<String, Value>,
) -> serde_json::Value {
    let map: serde_json::Map<String, serde_json::Value> = payload
        .iter()
        .map(|(k, v)| (k.clone(), convert_value_to_json(v)))
        .collect();
    serde_json::Value::Object(map)
}

fn convert_value_to_json(value: &Value) -> serde_json::Value {
    match &value.kind {
        Some(value::Kind::NullValue(_)) => serde_json::Value::Null,
        Some(value::Kind::DoubleValue(d)) => serde_json::json!(*d),
        Some(value::Kind::IntegerValue(i)) => serde_json::json!(*i),
        Some(value::Kind::StringValue(s)) => serde_json::json!(s),
        Some(value::Kind::BoolValue(b)) => serde_json::json!(*b),
        Some(value::Kind::StructValue(s)) => {
            let map: serde_json::Map<String, serde_json::Value> = s
                .fields
                .iter()
                .map(|(k, v)| (k.clone(), convert_value_to_json(v)))
                .collect();
            serde_json::Value::Object(map)
        }
        Some(value::Kind::ListValue(l)) => {
            serde_json::Value::Array(l.values.iter().map(convert_value_to_json).collect())
        }
        None => serde_json::Value::Null,
    }
}

fn convert_json_to_payload(json: &serde_json::Value) -> std::collections::HashMap<String, Value> {
    match json {
        serde_json::Value::Object(map) => map
            .iter()
            .map(|(k, v)| (k.clone(), convert_json_to_value(v)))
            .collect(),
        _ => std::collections::HashMap::new(),
    }
}

fn convert_json_to_value(json: &serde_json::Value) -> Value {
    Value {
        kind: Some(match json {
            serde_json::Value::Null => value::Kind::NullValue(0),
            serde_json::Value::Bool(b) => value::Kind::BoolValue(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    value::Kind::IntegerValue(i)
                } else if let Some(f) = n.as_f64() {
                    value::Kind::DoubleValue(f)
                } else {
                    value::Kind::NullValue(0)
                }
            }
            serde_json::Value::String(s) => value::Kind::StringValue(s.clone()),
            serde_json::Value::Array(arr) => value::Kind::ListValue(ListValue {
                values: arr.iter().map(convert_json_to_value).collect(),
            }),
            serde_json::Value::Object(map) => value::Kind::StructValue(Struct {
                fields: map
                    .iter()
                    .map(|(k, v)| (k.clone(), convert_json_to_value(v)))
                    .collect(),
            }),
        }),
    }
}
