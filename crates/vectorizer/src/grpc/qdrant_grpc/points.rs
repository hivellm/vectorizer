//! `impl Points for QdrantGrpcService` — extracted from the prior
//! monolithic `qdrant_grpc.rs` (phase3_split-qdrant-grpc). The impl block
//! itself is unchanged; only the file it lives in is new.

use std::time::Instant;

use tonic::{Request, Response, Status};
use tracing::{debug, error, info};

use super::{
    QdrantGrpcService, convert_grpc_filter, convert_json_to_payload, convert_payload_to_json,
    get_matching_vector_ids,
};
use crate::grpc::qdrant_proto::r#match::MatchValue;
use crate::grpc::qdrant_proto::points_server::Points;
use crate::grpc::qdrant_proto::*;
use crate::models::qdrant::filter::{QdrantCondition, QdrantFilter, QdrantMatchValue, QdrantRange};
use crate::models::qdrant::filter_processor::FilterProcessor;
use crate::models::{Payload, Vector};

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
                Some(points_selector::PointsSelectorOneOf::Filter(filter)) => {
                    let internal_filter = convert_grpc_filter(&filter);
                    let matching_ids = get_matching_vector_ids(&*collection, &internal_filter);
                    info!(
                        "Filter-based deletion: {} vectors matched",
                        matching_ids.len()
                    );
                    for id in matching_ids {
                        let _ = collection.delete_vector(&id);
                    }
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
                Some(points_selector::PointsSelectorOneOf::Filter(filter)) => {
                    let internal_filter = convert_grpc_filter(&filter);
                    let matching_ids = get_matching_vector_ids(&*collection, &internal_filter);
                    info!(
                        "Filter-based payload update: {} vectors matched",
                        matching_ids.len()
                    );
                    for id in matching_ids {
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
                Some(points_selector::PointsSelectorOneOf::Filter(filter)) => {
                    let internal_filter = convert_grpc_filter(&filter);
                    let matching_ids = get_matching_vector_ids(&*collection, &internal_filter);
                    info!(
                        "Filter-based payload overwrite: {} vectors matched",
                        matching_ids.len()
                    );
                    for id in matching_ids {
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
                Some(points_selector::PointsSelectorOneOf::Filter(filter)) => {
                    let internal_filter = convert_grpc_filter(&filter);
                    let matching_ids = get_matching_vector_ids(&*collection, &internal_filter);
                    info!(
                        "Filter-based payload key deletion: {} vectors matched",
                        matching_ids.len()
                    );
                    for id in matching_ids {
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
                Some(points_selector::PointsSelectorOneOf::Filter(filter)) => {
                    let internal_filter = convert_grpc_filter(&filter);
                    let matching_ids = get_matching_vector_ids(&*collection, &internal_filter);
                    info!(
                        "Filter-based payload clear: {} vectors matched",
                        matching_ids.len()
                    );
                    for id in matching_ids {
                        if let Ok(vec) = collection.get_vector(&id) {
                            let updated = Vector::new(id, vec.data.clone());
                            let _ = collection.update_vector(updated);
                        }
                    }
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
