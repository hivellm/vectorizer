//! Qdrant-compatible REST surface (`/qdrant/*` endpoints).
//!
//! 25 methods spanning core CRUD, snapshots, sharding, cluster
//! management, metadata, and the Qdrant 1.7+ Query API.
//!
//! Every method returns `serde_json::Value` because the Qdrant
//! response shapes evolve faster than we want to chase with typed
//! structs — the server's `/qdrant/*` translation layer keeps us
//! source-compatible with Qdrant clients without locking the SDK
//! to a specific Qdrant minor version.

use crate::error::{Result, VectorizerError};

use super::VectorizerClient;

/// Build the standard "parse-or-fail" wrapper used by every method
/// in this module — keeps each method to one logical statement.
macro_rules! parse_qdrant {
    ($response:expr, $what:literal) => {{
        serde_json::from_str(&$response).map_err(|e| {
            VectorizerError::server(format!(concat!("Failed to parse Qdrant ", $what, " response: {}"), e))
        })
    }};
}

impl VectorizerClient {
    /// List all collections (Qdrant-compatible).
    pub async fn qdrant_list_collections(&self) -> Result<serde_json::Value> {
        let response = self
            .make_request("GET", "/qdrant/collections", None)
            .await?;
        parse_qdrant!(response, "collections")
    }

    /// Get one collection's metadata (Qdrant-compatible).
    pub async fn qdrant_get_collection(&self, name: &str) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{name}");
        let response = self.make_request("GET", &url, None).await?;
        parse_qdrant!(response, "collection")
    }

    /// Create a collection (Qdrant-compatible).
    pub async fn qdrant_create_collection(
        &self,
        name: &str,
        config: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{name}");
        let payload = serde_json::json!({ "config": config });
        let response = self.make_request("PUT", &url, Some(payload)).await?;
        parse_qdrant!(response, "create collection")
    }

    /// Upsert points into a collection (Qdrant-compatible).
    pub async fn qdrant_upsert_points(
        &self,
        collection: &str,
        points: &serde_json::Value,
        wait: bool,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/points");
        let payload = serde_json::json!({ "points": points, "wait": wait });
        let response = self.make_request("PUT", &url, Some(payload)).await?;
        parse_qdrant!(response, "upsert points")
    }

    /// Search points (Qdrant-compatible).
    pub async fn qdrant_search_points(
        &self,
        collection: &str,
        vector: &[f32],
        limit: Option<usize>,
        filter: Option<&serde_json::Value>,
        with_payload: bool,
        with_vector: bool,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/points/search");
        let mut payload = serde_json::json!({
            "vector": vector,
            "limit": limit.unwrap_or(10),
            "with_payload": with_payload,
            "with_vector": with_vector,
        });
        if let Some(filter) = filter {
            payload["filter"] = filter.clone();
        }
        let response = self.make_request("POST", &url, Some(payload)).await?;
        parse_qdrant!(response, "search")
    }

    /// Delete points by id (Qdrant-compatible).
    pub async fn qdrant_delete_points(
        &self,
        collection: &str,
        point_ids: &[serde_json::Value],
        wait: bool,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/points/delete");
        let payload = serde_json::json!({ "points": point_ids, "wait": wait });
        let response = self.make_request("POST", &url, Some(payload)).await?;
        parse_qdrant!(response, "delete points")
    }

    /// Retrieve points by id (Qdrant-compatible).
    pub async fn qdrant_retrieve_points(
        &self,
        collection: &str,
        point_ids: &[serde_json::Value],
        with_payload: bool,
        with_vector: bool,
    ) -> Result<serde_json::Value> {
        let ids_str = point_ids
            .iter()
            .map(|id| match id {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                _ => serde_json::to_string(id).unwrap_or_default(),
            })
            .collect::<Vec<_>>()
            .join(",");
        let url = format!(
            "/qdrant/collections/{collection}/points?ids={ids_str}\
             &with_payload={with_payload}&with_vector={with_vector}",
        );
        let response = self.make_request("GET", &url, None).await?;
        parse_qdrant!(response, "retrieve points")
    }

    /// Count points (Qdrant-compatible).
    pub async fn qdrant_count_points(
        &self,
        collection: &str,
        filter: Option<&serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/points/count");
        let payload = if let Some(filter) = filter {
            serde_json::json!({ "filter": filter })
        } else {
            serde_json::json!({})
        };
        let response = self.make_request("POST", &url, Some(payload)).await?;
        parse_qdrant!(response, "count points")
    }

    // ── Snapshots ───────────────────────────────────────────────

    /// List snapshots for a collection (Qdrant-compatible).
    pub async fn qdrant_list_collection_snapshots(
        &self,
        collection: &str,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/snapshots");
        let response = self.make_request("GET", &url, None).await?;
        parse_qdrant!(response, "list snapshots")
    }

    /// Create a snapshot for a collection (Qdrant-compatible).
    pub async fn qdrant_create_collection_snapshot(
        &self,
        collection: &str,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/snapshots");
        let response = self.make_request("POST", &url, None).await?;
        parse_qdrant!(response, "create snapshot")
    }

    /// Delete a snapshot (Qdrant-compatible).
    pub async fn qdrant_delete_collection_snapshot(
        &self,
        collection: &str,
        snapshot_name: &str,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/snapshots/{snapshot_name}");
        let response = self.make_request("DELETE", &url, None).await?;
        parse_qdrant!(response, "delete snapshot")
    }

    /// Recover a collection from a snapshot location
    /// (Qdrant-compatible).
    pub async fn qdrant_recover_collection_snapshot(
        &self,
        collection: &str,
        location: &str,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/snapshots/recover");
        let payload = serde_json::json!({ "location": location });
        let response = self.make_request("POST", &url, Some(payload)).await?;
        parse_qdrant!(response, "recover snapshot")
    }

    /// List all snapshots across collections (Qdrant-compatible).
    pub async fn qdrant_list_all_snapshots(&self) -> Result<serde_json::Value> {
        let response = self.make_request("GET", "/qdrant/snapshots", None).await?;
        parse_qdrant!(response, "list all snapshots")
    }

    /// Create a full-cluster snapshot (Qdrant-compatible).
    pub async fn qdrant_create_full_snapshot(&self) -> Result<serde_json::Value> {
        let response = self
            .make_request("POST", "/qdrant/snapshots", None)
            .await?;
        parse_qdrant!(response, "create full snapshot")
    }

    // ── Sharding ────────────────────────────────────────────────

    /// List shard keys for a collection (Qdrant-compatible).
    pub async fn qdrant_list_shard_keys(&self, collection: &str) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/shards");
        let response = self.make_request("GET", &url, None).await?;
        parse_qdrant!(response, "list shard keys")
    }

    /// Create a shard key (Qdrant-compatible).
    pub async fn qdrant_create_shard_key(
        &self,
        collection: &str,
        shard_key: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/shards");
        let payload = serde_json::json!({ "shard_key": shard_key });
        let response = self.make_request("PUT", &url, Some(payload)).await?;
        parse_qdrant!(response, "create shard key")
    }

    /// Delete a shard key (Qdrant-compatible).
    pub async fn qdrant_delete_shard_key(
        &self,
        collection: &str,
        shard_key: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/shards/delete");
        let payload = serde_json::json!({ "shard_key": shard_key });
        let response = self.make_request("POST", &url, Some(payload)).await?;
        parse_qdrant!(response, "delete shard key")
    }

    // ── Cluster + metadata ──────────────────────────────────────

    /// Get cluster status (Qdrant-compatible).
    pub async fn qdrant_get_cluster_status(&self) -> Result<serde_json::Value> {
        let response = self.make_request("GET", "/qdrant/cluster", None).await?;
        parse_qdrant!(response, "cluster status")
    }

    /// Trigger a cluster recovery on the current peer
    /// (Qdrant-compatible).
    pub async fn qdrant_cluster_recover(&self) -> Result<serde_json::Value> {
        let response = self
            .make_request("POST", "/qdrant/cluster/recover", None)
            .await?;
        parse_qdrant!(response, "cluster recover")
    }

    /// Remove a peer from the cluster (Qdrant-compatible).
    pub async fn qdrant_remove_peer(&self, peer_id: &str) -> Result<serde_json::Value> {
        let url = format!("/qdrant/cluster/peer/{peer_id}");
        let response = self.make_request("DELETE", &url, None).await?;
        parse_qdrant!(response, "remove peer")
    }

    /// List metadata keys (Qdrant-compatible).
    pub async fn qdrant_list_metadata_keys(&self) -> Result<serde_json::Value> {
        let response = self
            .make_request("GET", "/qdrant/cluster/metadata/keys", None)
            .await?;
        parse_qdrant!(response, "list metadata keys")
    }

    /// Get one metadata key (Qdrant-compatible).
    pub async fn qdrant_get_metadata_key(&self, key: &str) -> Result<serde_json::Value> {
        let url = format!("/qdrant/cluster/metadata/keys/{key}");
        let response = self.make_request("GET", &url, None).await?;
        parse_qdrant!(response, "get metadata key")
    }

    /// Update one metadata key (Qdrant-compatible).
    pub async fn qdrant_update_metadata_key(
        &self,
        key: &str,
        value: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/cluster/metadata/keys/{key}");
        let payload = serde_json::json!({ "value": value });
        let response = self.make_request("PUT", &url, Some(payload)).await?;
        parse_qdrant!(response, "update metadata key")
    }

    // ── Query API (Qdrant 1.7+) ─────────────────────────────────

    /// Query points using the Qdrant 1.7+ Query API.
    pub async fn qdrant_query_points(
        &self,
        collection: &str,
        request: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/points/query");
        let response = self
            .make_request("POST", &url, Some(request.clone()))
            .await?;
        parse_qdrant!(response, "query points")
    }

    /// Batch-query points using the Qdrant 1.7+ Query API.
    pub async fn qdrant_batch_query_points(
        &self,
        collection: &str,
        request: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/points/query/batch");
        let response = self
            .make_request("POST", &url, Some(request.clone()))
            .await?;
        parse_qdrant!(response, "batch query points")
    }

    /// Query points with grouping (Qdrant 1.7+ Query API).
    pub async fn qdrant_query_points_groups(
        &self,
        collection: &str,
        request: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/points/query/groups");
        let response = self
            .make_request("POST", &url, Some(request.clone()))
            .await?;
        parse_qdrant!(response, "query points groups")
    }

    /// Search points with grouping (Qdrant Search Groups API).
    pub async fn qdrant_search_points_groups(
        &self,
        collection: &str,
        request: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/points/search/groups");
        let response = self
            .make_request("POST", &url, Some(request.clone()))
            .await?;
        parse_qdrant!(response, "search points groups")
    }

    /// Search matrix pairs (Qdrant Search Matrix API).
    pub async fn qdrant_search_matrix_pairs(
        &self,
        collection: &str,
        request: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/points/search/matrix/pairs");
        let response = self
            .make_request("POST", &url, Some(request.clone()))
            .await?;
        parse_qdrant!(response, "search matrix pairs")
    }

    /// Search matrix offsets (Qdrant Search Matrix API).
    pub async fn qdrant_search_matrix_offsets(
        &self,
        collection: &str,
        request: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/points/search/matrix/offsets");
        let response = self
            .make_request("POST", &url, Some(request.clone()))
            .await?;
        parse_qdrant!(response, "search matrix offsets")
    }
}
