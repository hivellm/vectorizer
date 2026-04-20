//! `impl Snapshots for QdrantGrpcService` — extracted from the prior
//! monolithic `qdrant_grpc.rs` (phase3_split-qdrant-grpc). The impl block
//! itself is unchanged; only the file it lives in is new.

use std::time::Instant;

use tonic::{Request, Response, Status};
use tracing::{error, info};

use super::QdrantGrpcService;
use crate::grpc::qdrant_proto::snapshots_server::Snapshots;
use crate::grpc::qdrant_proto::*;

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
