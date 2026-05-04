//! Admin / observability surface.
//!
//! Covers statistics, status, logs, indexing progress, collection
//! maintenance (force-save, empty-collection cleanup), server config,
//! backup management, admin restart, and workspace management.
//!
//! All methods call `self.make_request` via the shared dispatcher in
//! [`super`] so the transport abstraction is preserved.

use super::VectorizerClient;
use crate::error::{Result, VectorizerError};
use crate::models::{
    AddWorkspaceRequest, BackupInfo, CleanupReport, ConfigPatch, ConfigSnapshot,
    CreateBackupRequest, IndexingProgress, LogEntry, LogsQuery, RestoreBackupRequest,
    RuntimeMetrics, ServerStatus, SlowQueryConfig, SlowQueryEntry, Stats, WorkspaceConfig,
};

impl VectorizerClient {
    /// Aggregate collection + vector counts.
    ///
    /// Calls `GET /stats`.
    pub async fn get_stats(&self) -> Result<Stats> {
        let response = self.make_request("GET", "/stats", None).await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse get_stats response: {e}"))
        })
    }

    /// Runtime metrics snapshot for the dashboard (phase25).
    ///
    /// Calls `GET /metrics/runtime`. Returns CPU, memory, active
    /// connections, rolling 60-second QPS, per-route p50/p99,
    /// 5xx error rate, and the WAL state. Requires admin auth.
    pub async fn get_runtime_metrics(&self) -> Result<RuntimeMetrics> {
        let response = self.make_request("GET", "/metrics/runtime", None).await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse get_runtime_metrics response: {e}"))
        })
    }

    /// Server liveness / version / uptime.
    ///
    /// Calls `GET /status`.
    pub async fn get_status(&self) -> Result<ServerStatus> {
        let response = self.make_request("GET", "/status", None).await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse get_status response: {e}"))
        })
    }

    /// Tail recent log lines.
    ///
    /// Calls `GET /logs?lines=N&level=LEVEL`.
    pub async fn get_logs(&self, params: LogsQuery) -> Result<Vec<LogEntry>> {
        let mut qs = String::new();
        if let Some(lines) = params.lines {
            qs.push_str(&format!("lines={lines}"));
        }
        if let Some(level) = &params.level {
            if !qs.is_empty() {
                qs.push('&');
            }
            qs.push_str(&format!("level={level}"));
        }
        let endpoint = if qs.is_empty() {
            "/logs".to_string()
        } else {
            format!("/logs?{qs}")
        };
        let response = self.make_request("GET", &endpoint, None).await?;
        let val: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse get_logs response: {e}"))
        })?;
        let logs = val
            .get("logs")
            .and_then(|l| l.as_array())
            .cloned()
            .unwrap_or_default();
        let entries: Result<Vec<LogEntry>> = logs
            .into_iter()
            .map(|v| {
                serde_json::from_value(v)
                    .map_err(|e| VectorizerError::server(format!("Failed to parse log entry: {e}")))
            })
            .collect();
        entries
    }

    /// Per-collection indexing progress.
    ///
    /// Calls `GET /indexing/progress`.
    pub async fn get_indexing_progress(&self) -> Result<IndexingProgress> {
        let response = self.make_request("GET", "/indexing/progress", None).await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse get_indexing_progress response: {e}"
            ))
        })
    }

    /// Flush one collection to disk immediately.
    ///
    /// Calls `POST /collections/{name}/force-save`.
    pub async fn force_save_collection(&self, collection: &str) -> Result<()> {
        self.make_request(
            "POST",
            &format!("/collections/{collection}/force-save"),
            None,
        )
        .await?;
        Ok(())
    }

    /// List collections that contain zero vectors.
    ///
    /// Calls `GET /collections/empty`.
    pub async fn list_empty_collections(&self) -> Result<Vec<String>> {
        let response = self.make_request("GET", "/collections/empty", None).await?;
        let val: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse list_empty_collections response: {e}"
            ))
        })?;
        // Server returns either an array directly or {collections: [...]}
        let arr = if val.is_array() {
            val.as_array().cloned().unwrap_or_default()
        } else {
            val.get("collections")
                .and_then(|c| c.as_array())
                .cloned()
                .unwrap_or_default()
        };
        Ok(arr
            .into_iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect())
    }

    /// Delete all empty collections in one call.
    ///
    /// Calls `DELETE /collections/cleanup`.
    pub async fn cleanup_empty_collections(&self) -> Result<CleanupReport> {
        let response = self
            .make_request("DELETE", "/collections/cleanup", None)
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse cleanup_empty_collections response: {e}"
            ))
        })
    }

    /// Read the server's current `config.yml`.
    ///
    /// Calls `GET /config`.
    pub async fn get_config(&self) -> Result<ConfigSnapshot> {
        let response = self.make_request("GET", "/config", None).await?;
        let val: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse get_config response: {e}"))
        })?;
        Ok(ConfigSnapshot(val))
    }

    /// Overwrite the server's `config.yml` (admin).
    ///
    /// Calls `POST /config` with the full config object.
    /// Returns the config as echoed back by the server (free-form JSON).
    pub async fn update_config(&self, patch: ConfigPatch) -> Result<ConfigSnapshot> {
        let response = self.make_request("POST", "/config", Some(patch.0)).await?;
        let val: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse update_config response: {e}"))
        })?;
        Ok(ConfigSnapshot(val))
    }

    /// List all server-side backup files.
    ///
    /// Calls `GET /backups`.
    pub async fn list_backups(&self) -> Result<Vec<BackupInfo>> {
        let response = self.make_request("GET", "/backups", None).await?;
        let val: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse list_backups response: {e}"))
        })?;
        let arr = val
            .get("backups")
            .and_then(|b| b.as_array())
            .cloned()
            .unwrap_or_default();
        arr.into_iter()
            .map(|v| {
                serde_json::from_value(v).map_err(|e| {
                    VectorizerError::server(format!("Failed to parse backup entry: {e}"))
                })
            })
            .collect()
    }

    /// Create a new backup (admin).
    ///
    /// Calls `POST /backups/create` with `{name, collections}`.
    pub async fn create_backup(&self, request: CreateBackupRequest) -> Result<BackupInfo> {
        let payload = serde_json::to_value(&request).map_err(|e| {
            VectorizerError::server(format!("Failed to serialize create_backup request: {e}"))
        })?;
        let response = self
            .make_request("POST", "/backups/create", Some(payload))
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse create_backup response: {e}"))
        })
    }

    /// Restore a backup from the server's backup directory (admin).
    ///
    /// Calls `POST /backups/restore` with `{backup_id}`.
    pub async fn restore_backup(&self, request: RestoreBackupRequest) -> Result<()> {
        let payload = serde_json::to_value(&request).map_err(|e| {
            VectorizerError::server(format!("Failed to serialize restore_backup request: {e}"))
        })?;
        self.make_request("POST", "/backups/restore", Some(payload))
            .await?;
        Ok(())
    }

    /// Initiate a graceful server restart (admin).
    ///
    /// Calls `POST /admin/restart`. The server responds before the
    /// process actually restarts; callers should poll `/health` until
    /// the server is back.
    pub async fn restart_server(&self) -> Result<()> {
        self.make_request("POST", "/admin/restart", None).await?;
        Ok(())
    }

    /// List configured workspace directories.
    ///
    /// Calls `GET /workspace/list`.
    pub async fn list_workspaces(&self) -> Result<Vec<WorkspaceConfig>> {
        let response = self.make_request("GET", "/workspace/list", None).await?;
        let val: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse list_workspaces response: {e}"))
        })?;
        let arr = val
            .get("workspaces")
            .and_then(|w| w.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(arr.into_iter().map(WorkspaceConfig).collect())
    }

    /// Read the workspace configuration file.
    ///
    /// Calls `GET /workspace/config`.
    pub async fn get_workspace_config(&self) -> Result<WorkspaceConfig> {
        let response = self.make_request("GET", "/workspace/config", None).await?;
        let val: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse get_workspace_config response: {e}"
            ))
        })?;
        Ok(WorkspaceConfig(val))
    }

    /// Register a new workspace directory (admin).
    ///
    /// Calls `POST /workspace/add` with `{path, collection_name}`.
    pub async fn add_workspace(&self, request: AddWorkspaceRequest) -> Result<()> {
        let payload = serde_json::to_value(&request).map_err(|e| {
            VectorizerError::server(format!("Failed to serialize add_workspace request: {e}"))
        })?;
        self.make_request("POST", "/workspace/add", Some(payload))
            .await?;
        Ok(())
    }

    /// Remove a registered workspace directory (admin).
    ///
    /// Calls `POST /workspace/remove` with `{path}`.
    pub async fn remove_workspace(&self, name: &str) -> Result<()> {
        let payload = serde_json::json!({ "path": name });
        self.make_request("POST", "/workspace/remove", Some(payload))
            .await?;
        Ok(())
    }

    // ── Phase-14: observability ────────────────────────────────────────────────

    /// List slow-query ring-buffer entries (phase14).
    ///
    /// Calls `GET /slow_queries`. Returns entries in the order they were
    /// recorded (oldest first). The response also carries the current
    /// ring-buffer configuration, but this method returns only the entries.
    ///
    /// Use [`set_slow_query_config`] to tune the threshold and capacity.
    pub async fn list_slow_queries(&self) -> Result<Vec<SlowQueryEntry>> {
        let response = self.make_request("GET", "/slow_queries", None).await?;
        let val: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse list_slow_queries response: {e}"))
        })?;
        let arr = val
            .get("entries")
            .and_then(|e| e.as_array())
            .cloned()
            .unwrap_or_default();
        arr.into_iter()
            .map(|v| {
                serde_json::from_value(v).map_err(|e| {
                    VectorizerError::server(format!("Failed to parse slow-query entry: {e}"))
                })
            })
            .collect()
    }

    /// Reconfigure the slow-query ring buffer (phase14).
    ///
    /// Calls `POST /slow_queries/config` with
    /// `{"threshold_ms": <u64>, "capacity": <usize>}`.
    ///
    /// Existing entries are retained. If the new capacity is smaller than
    /// the current entry count the oldest entries are evicted by the server.
    pub async fn set_slow_query_config(&self, config: SlowQueryConfig) -> Result<SlowQueryConfig> {
        let payload = serde_json::json!({
            "threshold_ms": config.threshold_ms,
            "capacity": config.capacity,
        });
        let response = self
            .make_request("POST", "/slow_queries/config", Some(payload))
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse set_slow_query_config response: {e}"
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use serde_json::json;

    use crate::models::{
        AddWorkspaceRequest, BackupInfo, CleanupReport, ConfigPatch, ConfigSnapshot,
        CreateBackupRequest, IndexingProgress, LogEntry, LogsQuery, RestoreBackupRequest,
        RuntimeMetrics, ServerStatus, SlowQueryConfig, SlowQueryEntry, Stats, WorkspaceConfig,
    };

    #[test]
    fn stats_deserializes() {
        let raw = json!({
            "collections": 5,
            "total_vectors": 1000,
            "uptime_seconds": 3600,
            "version": "3.4.0"
        });
        let s: Stats = serde_json::from_value(raw).unwrap();
        assert_eq!(s.collections, 5);
        assert_eq!(s.total_vectors, 1000);
        assert_eq!(s.version, "3.4.0");
        // Older servers without phase25 §5 fields fall back to ("none", 1.0).
        assert_eq!(s.default_quantization, "none");
        assert!((s.compression_ratio - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn stats_deserializes_phase25_quantization_fields() {
        let raw = json!({
            "collections": 3,
            "total_vectors": 12_000,
            "uptime_seconds": 60,
            "version": "3.4.0",
            "default_quantization": "sq-8bit",
            "compression_ratio": 4.0,
        });
        let s: Stats = serde_json::from_value(raw).unwrap();
        assert_eq!(s.default_quantization, "sq-8bit");
        assert!((s.compression_ratio - 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn runtime_metrics_deserializes_full_snapshot() {
        let raw = json!({
            "cpu_percent": 12.4,
            "memory_rss_bytes": 124_857_600u64,
            "memory_total_bytes": 17_179_869_184u64,
            "memory_percent": 0.73,
            "active_connections": 8,
            "uptime_seconds": 3712,
            "qps_window_60s": 142.3,
            "error_rate_5xx_60s": 0.001,
            "throughput_by_route": [
                {"route": "/insert_texts", "qps": 12.0, "p50_ms": 8.2, "p99_ms": 41.0}
            ],
            "wal": {
                "current_seq": 482919u64,
                "size_bytes": 12_582_912u64,
                "last_checkpoint_at": 1_714_828_800u64,
                "last_checkpoint_seq": 482_800u64,
            }
        });
        let m: RuntimeMetrics = serde_json::from_value(raw).unwrap();
        assert!((m.cpu_percent - 12.4).abs() < f64::EPSILON);
        assert_eq!(m.active_connections, 8);
        assert_eq!(m.throughput_by_route.len(), 1);
        assert_eq!(m.throughput_by_route[0].route, "/insert_texts");
        assert!((m.throughput_by_route[0].p99_ms - 41.0).abs() < f64::EPSILON);
        assert_eq!(m.wal.current_seq, 482919);
        assert_eq!(m.wal.last_checkpoint_seq, 482_800);
    }

    #[test]
    fn runtime_metrics_tolerates_missing_fields() {
        // Standalone server without WAL or routes: every field is
        // marked default so partial payloads still deserialize.
        let raw = json!({
            "cpu_percent": 1.0,
            "memory_total_bytes": 8_000_000_000u64,
        });
        let m: RuntimeMetrics = serde_json::from_value(raw).unwrap();
        assert!((m.cpu_percent - 1.0).abs() < f64::EPSILON);
        assert_eq!(m.active_connections, 0);
        assert!(m.throughput_by_route.is_empty());
        assert_eq!(m.wal.current_seq, 0);
    }

    #[test]
    fn server_status_deserializes() {
        let raw = json!({
            "online": true,
            "version": "3.4.0",
            "uptime_seconds": 120,
            "collections_count": 3
        });
        let ss: ServerStatus = serde_json::from_value(raw).unwrap();
        assert!(ss.online);
        assert_eq!(ss.collections_count, 3);
    }

    #[test]
    fn log_entry_deserializes() {
        let raw = json!({
            "timestamp": "2026-05-02T00:00:00Z",
            "level": "INFO",
            "message": "Server started",
            "source": "vectorizer"
        });
        let le: LogEntry = serde_json::from_value(raw).unwrap();
        assert_eq!(le.level, "INFO");
        assert_eq!(le.source, "vectorizer");
    }

    #[test]
    fn logs_query_default_serializes() {
        let q = LogsQuery::default();
        let v = serde_json::to_value(&q).unwrap();
        assert_eq!(v, json!({}));
    }

    #[test]
    fn logs_query_with_params_serializes() {
        let q = LogsQuery {
            lines: Some(50),
            level: Some("ERROR".into()),
        };
        let v = serde_json::to_value(&q).unwrap();
        assert_eq!(v["lines"], 50);
        assert_eq!(v["level"], "ERROR");
    }

    #[test]
    fn indexing_progress_deserializes() {
        let raw = json!({
            "overall_status": "completed",
            "collections": [],
            "is_indexing": false
        });
        let ip: IndexingProgress = serde_json::from_value(raw).unwrap();
        assert_eq!(ip.overall_status, "completed");
    }

    #[test]
    fn cleanup_report_deserializes() {
        let raw = json!({
            "success": true,
            "removed": 2,
            "collections": ["empty1", "empty2"],
            "message": "Done"
        });
        let cr: CleanupReport = serde_json::from_value(raw).unwrap();
        assert!(cr.success);
        assert_eq!(cr.removed, 2);
        assert_eq!(cr.collections.len(), 2);
    }

    #[test]
    fn config_snapshot_round_trips() {
        let val = json!({ "server": { "port": 15002 } });
        let cs = ConfigSnapshot(val.clone());
        let serialized = serde_json::to_value(&cs).unwrap();
        assert_eq!(serialized, val);
    }

    #[test]
    fn config_patch_round_trips() {
        let val = json!({ "embedding": { "provider": "fastembed" } });
        let cp = ConfigPatch(val.clone());
        let serialized = serde_json::to_value(&cp).unwrap();
        assert_eq!(serialized, val);
    }

    #[test]
    fn backup_info_deserializes() {
        let raw = json!({
            "id": "abc-123",
            "name": "weekly",
            "date": "2026-05-02T00:00:00Z",
            "size": 4096,
            "collections": ["docs"]
        });
        let bi: BackupInfo = serde_json::from_value(raw).unwrap();
        assert_eq!(bi.id, "abc-123");
        assert_eq!(bi.collections, vec!["docs"]);
    }

    #[test]
    fn create_backup_request_serializes() {
        let req = CreateBackupRequest {
            name: "nightly".into(),
            collections: vec!["code".into()],
        };
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["name"], "nightly");
        assert_eq!(v["collections"][0], "code");
    }

    #[test]
    fn restore_backup_request_serializes() {
        let req = RestoreBackupRequest {
            backup_id: "xyz-789".into(),
        };
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["backup_id"], "xyz-789");
    }

    #[test]
    fn workspace_config_round_trips() {
        let val = json!({ "projects": [], "global_settings": {} });
        let wc = WorkspaceConfig(val.clone());
        let serialized = serde_json::to_value(&wc).unwrap();
        assert_eq!(serialized, val);
    }

    #[test]
    fn add_workspace_request_serializes() {
        let req = AddWorkspaceRequest {
            path: "/home/user/project".into(),
            collection_name: "project_docs".into(),
        };
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["path"], "/home/user/project");
        assert_eq!(v["collection_name"], "project_docs");
    }

    // ── Phase-14 slow-query round-trip tests ──────────────────────────────────

    #[test]
    fn slow_query_entry_wire_shape() {
        // Mirror of one item in `GET /slow_queries` → entries[].
        let raw = json!({
            "timestamp": "2026-05-02T00:01:00Z",
            "collection": "docs",
            "k": 10,
            "duration_ms": 312.5,
        });
        let e: SlowQueryEntry = serde_json::from_value(raw).unwrap();
        assert_eq!(e.collection, "docs");
        assert_eq!(e.k, 10);
        assert!((e.duration_ms - 312.5).abs() < f64::EPSILON);
    }

    #[test]
    fn slow_query_config_round_trips() {
        // Mirror of `POST /slow_queries/config` response.
        let raw = json!({
            "threshold_ms": 200u64,
            "capacity": 500usize,
            "status": "ok",
        });
        let cfg: SlowQueryConfig = serde_json::from_value(raw).unwrap();
        assert_eq!(cfg.threshold_ms, 200);
        assert_eq!(cfg.capacity, 500);

        // Serialize back — status is server-only, not echoed by the struct.
        let v = serde_json::to_value(&cfg).unwrap();
        assert_eq!(v["threshold_ms"], 200);
        assert_eq!(v["capacity"], 500);
    }

    #[test]
    fn slow_query_config_payload_shape() {
        // Verify the request body for `POST /slow_queries/config`.
        let cfg = SlowQueryConfig {
            threshold_ms: 150,
            capacity: 1000,
        };
        let payload = json!({
            "threshold_ms": cfg.threshold_ms,
            "capacity": cfg.capacity,
        });
        assert_eq!(payload["threshold_ms"], 150);
        assert_eq!(payload["capacity"], 1000);
    }

    #[test]
    fn list_slow_queries_response_parses_entries() {
        // Full response shape from `GET /slow_queries`.
        let raw = json!({
            "entries": [
                {
                    "timestamp": "2026-05-02T00:01:00Z",
                    "collection": "docs",
                    "k": 5,
                    "duration_ms": 450.0,
                },
                {
                    "timestamp": "2026-05-02T00:02:00Z",
                    "collection": "logs",
                    "k": 20,
                    "duration_ms": 800.0,
                }
            ],
            "total": 2,
            "config": {
                "threshold_ms": 200,
                "capacity": 1000,
            }
        });
        let entries = raw["entries"].as_array().unwrap();
        let parsed: Vec<SlowQueryEntry> = entries
            .iter()
            .map(|v| serde_json::from_value(v.clone()).unwrap())
            .collect();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].collection, "docs");
        assert_eq!(parsed[1].k, 20);
    }
}
