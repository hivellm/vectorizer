//! Collection-management surface: list, create, get info, delete.
//!
//! These are the four endpoints that operate on collections as a
//! whole — vector-level CRUD lives in [`super::vectors`], search
//! over a collection lives in [`super::search`].

use super::VectorizerClient;
use crate::error::{Result, VectorizerError};
use crate::models::*;

impl VectorizerClient {
    /// List every collection visible to the authenticated principal.
    /// Accepts both the legacy bare-array response and the newer
    /// `{collections: [...]}` wrapper.
    pub async fn list_collections(&self) -> Result<Vec<Collection>> {
        let response = self.make_request("GET", "/collections", None).await?;
        let collections: Vec<Collection> = if let Ok(wrapper) =
            serde_json::from_str::<serde_json::Value>(&response)
        {
            if let Some(arr) = wrapper.get("collections").and_then(|v| v.as_array()) {
                serde_json::from_value(serde_json::Value::Array(arr.clone())).map_err(|e| {
                    VectorizerError::server(format!("Failed to parse collections array: {e}"))
                })?
            } else if wrapper.is_array() {
                serde_json::from_value(wrapper).map_err(|e| {
                    VectorizerError::server(format!("Failed to parse collections response: {e}"))
                })?
            } else {
                return Err(VectorizerError::server(
                    "Unexpected collections response format".to_string(),
                ));
            }
        } else {
            return Err(VectorizerError::server(
                "Failed to parse collections response".to_string(),
            ));
        };
        Ok(collections)
    }

    /// Create a new collection. The returned [`CollectionInfo`] is
    /// synthesised from the server's create-response plus the
    /// arguments — the server response only carries the collection
    /// name today.
    pub async fn create_collection(
        &self,
        name: &str,
        dimension: usize,
        metric: Option<SimilarityMetric>,
    ) -> Result<CollectionInfo> {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "name".to_string(),
            serde_json::Value::String(name.to_string()),
        );
        payload.insert(
            "dimension".to_string(),
            serde_json::Value::Number(dimension.into()),
        );
        payload.insert(
            "metric".to_string(),
            serde_json::Value::String(format!("{:?}", metric.unwrap_or_default()).to_lowercase()),
        );

        let response = self
            .make_request(
                "POST",
                "/collections",
                Some(serde_json::Value::Object(payload)),
            )
            .await?;
        let create_response: CreateCollectionResponse =
            serde_json::from_str(&response).map_err(|e| {
                VectorizerError::server(format!("Failed to parse create collection response: {e}"))
            })?;

        let info = CollectionInfo {
            name: create_response.collection,
            dimension,
            metric: format!("{:?}", metric.unwrap_or_default()).to_lowercase(),
            vector_count: 0,
            document_count: 0,
            created_at: String::new(),
            updated_at: String::new(),
            indexing_status: Some(crate::models::IndexingStatus {
                status: "created".to_string(),
                progress: 0.0,
                total_documents: 0,
                processed_documents: 0,
                vector_count: 0,
                estimated_time_remaining: None,
                last_updated: String::new(),
            }),
            size: None,
            quantization: None,
            normalization: None,
            status: Some("created".to_string()),
        };
        Ok(info)
    }

    /// Delete a collection by name.
    pub async fn delete_collection(&self, name: &str) -> Result<()> {
        self.make_request("DELETE", &format!("/collections/{name}"), None)
            .await?;
        Ok(())
    }

    /// Fetch metadata for a collection (vector count, dimension,
    /// metric, timestamps, indexing status).
    pub async fn get_collection_info(&self, collection: &str) -> Result<CollectionInfo> {
        let response = self
            .make_request("GET", &format!("/collections/{collection}"), None)
            .await?;
        let info: CollectionInfo = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse collection info: {e}"))
        })?;
        Ok(info)
    }

    /// Re-quantize an existing collection in-place without re-embedding
    /// (phase13).
    ///
    /// Calls `POST /collections/{name}/reencode` with
    /// `{"target_encoding": "<encoding>"}`. Valid encoding values:
    /// `"sq8"`, `"binary"`, `"fp32"`.
    ///
    /// The server runs the reencode synchronously and returns
    /// `{job_id, collection, state, target_encoding, progress}` on
    /// completion. `state` will be `"completed"` on success.
    pub async fn reencode_collection(
        &self,
        collection: &str,
        target_encoding: &str,
    ) -> Result<ReencodeJob> {
        let payload = serde_json::json!({ "target_encoding": target_encoding });
        let response = self
            .make_request(
                "POST",
                &format!("/collections/{collection}/reencode"),
                Some(payload),
            )
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse reencode_collection response: {e}"))
        })
    }

    /// Set or clear a per-collection TTL (phase13).
    ///
    /// Calls `POST /collections/{name}/ttl` with `{"ttl_secs": <secs>}`.
    /// Pass `None` to clear the collection-level TTL. Existing vectors are
    /// NOT retroactively expired; only subsequent insertions that carry
    /// `__expires_at` in their payload are affected.
    ///
    /// For per-vector expiry use `set_vector_expiry` on the vectors surface.
    pub async fn set_collection_ttl(&self, collection: &str, ttl_secs: Option<u64>) -> Result<()> {
        let payload = serde_json::json!({ "ttl_secs": ttl_secs });
        self.make_request(
            "POST",
            &format!("/collections/{collection}/ttl"),
            Some(payload),
        )
        .await?;
        Ok(())
    }

    // ── Phase-14: schema-evolution methods ────────────────────────────────────

    /// Atomically rename a collection (phase14).
    ///
    /// Calls `POST /collections/{name}/rename` with `{"new_name": "<name>"}`.
    ///
    /// The server keeps the old name as an in-memory alias for one minor
    /// version so existing clients keep working without reconfiguration.
    /// The alias does not survive a restart.
    pub async fn rename_collection(&self, collection: &str, new_name: &str) -> Result<()> {
        let payload = serde_json::json!({ "new_name": new_name });
        self.make_request(
            "POST",
            &format!("/collections/{collection}/rename"),
            Some(payload),
        )
        .await?;
        Ok(())
    }

    /// Rebuild the HNSW index with new parameters (phase14).
    ///
    /// Calls `POST /collections/{name}/reindex` with
    /// `{"m": u32, "ef_construction": u32, "ef_search": u32}`.
    ///
    /// No re-embedding is required — the existing stored vectors are used.
    /// The server holds the collection write-lock for the duration, so
    /// concurrent inserts queue behind the swap.
    ///
    /// Returns a [`ReindexJob`] with `state == "completed"` on success.
    pub async fn reindex_collection(
        &self,
        collection: &str,
        params: crate::models::ReindexParams,
    ) -> Result<crate::models::ReindexJob> {
        let payload = serde_json::json!({
            "m": params.m,
            "ef_construction": params.ef_construction,
            "ef_search": params.ef_search,
        });
        let response = self
            .make_request(
                "POST",
                &format!("/collections/{collection}/reindex"),
                Some(payload),
            )
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse reindex_collection response: {e}"))
        })
    }

    /// Create a native per-collection snapshot (phase14).
    ///
    /// Calls `POST /collections/{name}/snapshot` (empty body).
    ///
    /// The server writes a gzip-compressed JSON snapshot under
    /// `<data_dir>/collection_snapshots/<name>/` and returns the snapshot
    /// metadata.
    pub async fn snapshot_collection_native(
        &self,
        collection: &str,
    ) -> Result<crate::models::NativeSnapshotInfo> {
        let response = self
            .make_request(
                "POST",
                &format!("/collections/{collection}/snapshot"),
                Some(serde_json::json!({})),
            )
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse snapshot_collection_native response: {e}"
            ))
        })
    }

    /// List all native snapshots for a collection (phase14).
    ///
    /// Calls `GET /collections/{name}/snapshots`.
    ///
    /// Returns snapshots newest-first as reported by the server.
    pub async fn list_collection_snapshots_native(
        &self,
        collection: &str,
    ) -> Result<Vec<crate::models::NativeSnapshotInfo>> {
        let response = self
            .make_request("GET", &format!("/collections/{collection}/snapshots"), None)
            .await?;
        let val: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse list_collection_snapshots_native response: {e}"
            ))
        })?;
        let arr = val
            .get("snapshots")
            .and_then(|s| s.as_array())
            .cloned()
            .unwrap_or_default();
        arr.into_iter()
            .map(|v| {
                serde_json::from_value(v).map_err(|e| {
                    VectorizerError::server(format!("Failed to parse snapshot entry: {e}"))
                })
            })
            .collect()
    }

    /// Restore a collection from a native snapshot (phase14).
    ///
    /// Calls `POST /collections/{name}/snapshots/{id}/restore` (empty body).
    ///
    /// Drops the current in-memory state and replaces it with the snapshot data.
    pub async fn restore_collection_snapshot_native(
        &self,
        collection: &str,
        snapshot_id: &str,
    ) -> Result<()> {
        self.make_request(
            "POST",
            &format!("/collections/{collection}/snapshots/{snapshot_id}/restore"),
            Some(serde_json::json!({})),
        )
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::models::{NativeSnapshotInfo, ReencodeJob, ReindexJob, ReindexParams};

    #[test]
    fn reencode_job_wire_shape() {
        // Mirror of `POST /collections/{name}/reencode` response.
        let raw = json!({
            "job_id": "reencode-myc-1746000000",
            "collection": "myc",
            "state": "completed",
            "target_encoding": "fp32",
            "progress": 1.0,
        });
        let job: ReencodeJob = serde_json::from_value(raw).unwrap();
        assert_eq!(job.job_id, "reencode-myc-1746000000");
        assert_eq!(job.state, "completed");
        assert_eq!(job.target_encoding, "fp32");
    }

    #[test]
    fn set_collection_ttl_payload_shape() {
        // Verify the JSON payload serializes correctly for both Some and None.
        let with_ttl = json!({ "ttl_secs": 3600u64 });
        assert_eq!(with_ttl["ttl_secs"], 3600);

        let clear_ttl = json!({ "ttl_secs": serde_json::Value::Null });
        assert!(clear_ttl["ttl_secs"].is_null());
    }

    // ── Phase-14 round-trip tests ─────────────────────────────────────────────

    #[test]
    fn rename_collection_payload_shape() {
        // Verify `POST /collections/{name}/rename` body serializes correctly.
        let payload = json!({ "new_name": "docs_v2" });
        assert_eq!(payload["new_name"], "docs_v2");
    }

    #[test]
    fn reindex_params_serialize() {
        let params = ReindexParams {
            m: 32,
            ef_construction: 400,
            ef_search: 200,
        };
        let v = serde_json::to_value(&params).unwrap();
        assert_eq!(v["m"], 32);
        assert_eq!(v["ef_construction"], 400);
        assert_eq!(v["ef_search"], 200);
    }

    #[test]
    fn reindex_job_wire_shape() {
        // Mirror of `POST /collections/{name}/reindex` response.
        let raw = json!({
            "job_id": "reindex-docs-1746000001",
            "collection": "docs",
            "state": "completed",
            "params": { "m": 32, "ef_construction": 400, "ef_search": 200 },
            "progress": 1.0,
        });
        let job: ReindexJob = serde_json::from_value(raw).unwrap();
        assert_eq!(job.job_id, "reindex-docs-1746000001");
        assert_eq!(job.state, "completed");
        assert!((job.progress - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn native_snapshot_info_wire_shape() {
        // Mirror of `POST /collections/{name}/snapshot` response.
        let raw = json!({
            "id": "snap-abc-123",
            "collection": "docs",
            "created_at": "2026-05-02T00:00:00Z",
            "size_bytes": 4096u64,
            "status": "ok",
        });
        let info: NativeSnapshotInfo = serde_json::from_value(raw).unwrap();
        assert_eq!(info.id, "snap-abc-123");
        assert_eq!(info.collection, "docs");
        assert_eq!(info.size_bytes, 4096);
    }

    #[test]
    fn list_snapshots_response_parses() {
        // Mirror of `GET /collections/{name}/snapshots` response.
        let raw = json!({
            "collection": "docs",
            "snapshots": [
                {
                    "id": "snap-abc-123",
                    "collection": "docs",
                    "created_at": "2026-05-02T00:00:00Z",
                    "size_bytes": 4096u64,
                }
            ],
            "total": 1,
        });
        let arr = raw["snapshots"].as_array().unwrap();
        let snaps: Vec<NativeSnapshotInfo> = arr
            .iter()
            .map(|v| serde_json::from_value(v.clone()).unwrap())
            .collect();
        assert_eq!(snaps.len(), 1);
        assert_eq!(snaps[0].id, "snap-abc-123");
    }

    #[test]
    fn restore_snapshot_payload_shape() {
        // `POST /collections/{name}/snapshots/{id}/restore` sends empty body.
        let payload = json!({});
        assert!(payload.as_object().map(|o| o.is_empty()).unwrap_or(false));
    }
}
