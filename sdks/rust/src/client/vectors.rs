//! Vector-level surface: get, batch-insert texts, embed.
//!
//! Single-vector retrieval, batch text insertion, and on-server
//! embedding generation. Search lives in [`super::search`];
//! collection-level CRUD in [`super::collections`].

use super::VectorizerClient;
use crate::error::{Result, VectorizerError};
use crate::models::*;

impl VectorizerClient {
    /// Fetch one vector by id.
    ///
    /// **Server caveat (observed on `hivehub/vectorizer:3.0.x`):** the
    /// `GET /collections/{c}/vectors/{id}` endpoint currently returns
    /// HTTP 200 with a synthetic uniform-vector payload
    /// (`[0.1, 0.1, …]`) even for ids that don't exist. Callers that
    /// need real miss detection should probe via
    /// [`VectorizerClient::list_vectors`] or search and not trust an
    /// `Ok(Vector)` as proof of existence until the server fix ships.
    pub async fn get_vector(&self, collection: &str, vector_id: &str) -> Result<Vector> {
        let response = self
            .make_request(
                "GET",
                &format!("/collections/{collection}/vectors/{vector_id}"),
                None,
            )
            .await?;
        let vector: Vector = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse get vector response: {e}"))
        })?;
        Ok(vector)
    }

    /// Insert a batch of texts into a collection. The server embeds
    /// each entry with the collection's configured provider (BM25 by
    /// default; FastEmbed ONNX when selected in `config.yml`).
    ///
    /// Wire contract: the server's `POST /insert_texts` handler
    /// expects `{ "collection": "<name>", "texts": [...] }` — the
    /// collection is a top-level field in the JSON body, not a path
    /// segment. The earlier `POST /collections/{c}/documents` path
    /// this method used was never served (the 3.0.x server returns
    /// 404 for it) and has been removed.
    ///
    /// Per-entry `id` field: the server **reassigns** every inserted
    /// vector a server-generated UUID regardless of what the caller
    /// sent. The original client id is stashed as `client_id` on the
    /// response entry. Callers that need idempotency by client id
    /// should key off the `client_id` round-trip, not the
    /// server-assigned UUID.
    pub async fn insert_texts(
        &self,
        collection: &str,
        texts: Vec<BatchTextRequest>,
    ) -> Result<BatchResponse> {
        let payload = serde_json::json!({
            "collection": collection,
            "texts": texts,
        });
        let response = self
            .make_request("POST", "/insert_texts", Some(payload))
            .await?;
        let mut batch_response: BatchResponse = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse insert texts response: {e}"))
        })?;
        // v3 omits the pre-v3 `success` field and instead emits
        // `inserted` / `failed` counts (aliased onto
        // `successful_operations` / `failed_operations`). The struct
        // doc-comment tells callers to derive the flag themselves; do
        // that here once so existing consumers (and the SDK integration
        // suite) keep working across the shape change.
        if !batch_response.success
            && batch_response.failed_operations == 0
            && batch_response.successful_operations > 0
        {
            batch_response.success = true;
        }
        // v3 also drops the pre-v3 `operation` tag. The call site here
        // unambiguously *is* an insert, so fill it in if the server
        // didn't — callers that assert on the tag keep working.
        if batch_response.operation.is_empty() {
            batch_response.operation = "insert".to_string();
        }
        Ok(batch_response)
    }

    /// Delete a single vector by id from a collection.
    ///
    /// Calls `DELETE /collections/{collection}/vectors/{vector_id}`.
    /// Returns `Ok(())` on 2xx; the server treats "not found" as a
    /// 4xx that surfaces as a `VectorizerError::NotFound`-class error
    /// via the shared error mapper.
    ///
    /// Companion to [`Self::delete_vectors`] (batch) and
    /// [`Self::move_to_collection`] (cross-collection move). See
    /// issue #265 for the tier-demotion use case.
    pub async fn delete_vector(&self, collection: &str, vector_id: &str) -> Result<()> {
        self.make_request(
            "DELETE",
            &format!("/collections/{collection}/vectors/{vector_id}"),
            None,
        )
        .await?;
        Ok(())
    }

    /// Delete a batch of vectors from a single collection. Per-id
    /// failures (e.g. not-found) are captured in
    /// [`DeleteReport::results`] without aborting the batch.
    ///
    /// Calls `POST /batch_delete` with `{"collection": ..., "ids": [...]}`.
    pub async fn delete_vectors(&self, collection: &str, ids: &[String]) -> Result<DeleteReport> {
        let payload = serde_json::json!({
            "collection": collection,
            "ids": ids,
        });
        let response = self
            .make_request("POST", "/batch_delete", Some(payload))
            .await?;
        let report: DeleteReport = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse delete_vectors response: {e}"))
        })?;
        Ok(report)
    }

    /// Move vectors from `src` to `dst` without re-embedding (issue #265).
    ///
    /// Calls `POST /collections/{src}/vectors/move` with
    /// `{"destination": dst, "ids": [...]}`. Server invariant: the
    /// destination insert lands BEFORE the source delete, so a
    /// mid-batch crash leaves a recoverable duplicate (never data
    /// loss). Per-id outcomes (`ok`, `missing_in_src`,
    /// `dst_insert_failed`, `src_delete_failed`) populate
    /// [`MoveReport::results`] without aborting the batch.
    ///
    /// Typical use: tier-demotion pruner that walks a hot collection
    /// and relocates aged vectors to a warm/cold collection.
    pub async fn move_to_collection(
        &self,
        src: &str,
        dst: &str,
        ids: &[String],
    ) -> Result<MoveReport> {
        let payload = serde_json::json!({
            "destination": dst,
            "ids": ids,
        });
        let response = self
            .make_request(
                "POST",
                &format!("/collections/{src}/vectors/move"),
                Some(payload),
            )
            .await?;
        let report: MoveReport = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse move_to_collection response: {e}"))
        })?;
        Ok(report)
    }

    /// Update a vector's metadata in-place.
    ///
    /// Calls `POST /update` with `{collection, id, ...metadata}`.
    /// Returns the server's confirmation as a synthetic [`Vector`].
    ///
    /// Server contract (`PUT /vectors`): the handler accepts `id` and
    /// `collection` from the JSON body, invalidates the cache, and
    /// returns `{message}`. The SDK synthesises a minimal `Vector`
    /// from the request parameters because the server does not echo
    /// back the full vector payload.
    pub async fn update_vector(
        &self,
        collection: &str,
        id: &str,
        request: UpdateVectorRequest,
    ) -> Result<Vector> {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "collection".into(),
            serde_json::Value::String(collection.to_string()),
        );
        payload.insert("id".into(), serde_json::Value::String(id.to_string()));
        if let Some(meta) = request.metadata {
            payload.insert("metadata".into(), meta);
        }
        self.make_request("POST", "/update", Some(serde_json::Value::Object(payload)))
            .await?;
        Ok(Vector {
            id: id.to_string(),
            data: vec![],
            metadata: None,
            public_key: None,
        })
    }

    /// Insert a single text document into a collection (auto-chunking when
    /// the text is long).
    ///
    /// Calls `POST /insert` with `{collection, id?, text, metadata?}`.
    /// Returns the first vector id created as a synthetic `Vector`.
    ///
    /// Server response: `{message, vectors_created, vector_ids, collection, chunked}`.
    pub async fn insert_text(
        &self,
        collection: &str,
        id: &str,
        text: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<Vector> {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "collection".into(),
            serde_json::Value::String(collection.to_string()),
        );
        payload.insert("id".into(), serde_json::Value::String(id.to_string()));
        payload.insert("text".into(), serde_json::Value::String(text.to_string()));
        if let Some(meta) = metadata {
            payload.insert("metadata".into(), meta);
        }
        let response = self
            .make_request("POST", "/insert", Some(serde_json::Value::Object(payload)))
            .await?;
        let val: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse insert_text response: {e}"))
        })?;
        let assigned_id = val
            .get("vector_ids")
            .and_then(|a| a.as_array())
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
            .unwrap_or(id)
            .to_string();
        Ok(Vector {
            id: assigned_id,
            data: vec![],
            metadata: None,
            public_key: None,
        })
    }

    /// List vectors in a collection with pagination.
    ///
    /// Calls `GET /collections/{name}/vectors?page=&limit=`.
    ///
    /// Note: the server handler uses `offset` (not `page`) as the query
    /// parameter. `page` is translated to `offset = page * limit` by the
    /// SDK. Pass `page=None` and `limit=None` for the server defaults
    /// (limit=10, offset=0).
    pub async fn list_vectors(
        &self,
        collection: &str,
        page: Option<u32>,
        limit: Option<u32>,
    ) -> Result<VectorPage> {
        let limit_val = limit.unwrap_or(10);
        let offset_val = page.unwrap_or(0) * limit_val;
        let endpoint =
            format!("/collections/{collection}/vectors?limit={limit_val}&offset={offset_val}");
        let response = self.make_request("GET", &endpoint, None).await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse list_vectors response: {e}"))
        })
    }

    /// Fetch a single vector by id via the path-based `GET` endpoint.
    ///
    /// Calls `GET /collections/{name}/vectors/{id}`.
    ///
    /// This is distinct from `get_vector` which uses the older `POST /vector`
    /// shape. The path-based handler currently returns a synthetic
    /// uniform-vector payload — see the server handler doc for the caveat.
    pub async fn get_vector_by_path(&self, collection: &str, id: &str) -> Result<Vector> {
        let response = self
            .make_request(
                "GET",
                &format!("/collections/{collection}/vectors/{id}"),
                None,
            )
            .await?;
        let val: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse get_vector_by_path response: {e}"))
        })?;
        let vec_id = val
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or(id)
            .to_string();
        let data: Vec<f32> = val
            .get("vector")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_f64().map(|f| f as f32))
                    .collect()
            })
            .unwrap_or_default();
        let metadata: Option<std::collections::HashMap<String, serde_json::Value>> = val
            .get("payload")
            .and_then(|p| serde_json::from_value(p.clone()).ok());
        Ok(Vector {
            id: vec_id,
            data,
            metadata,
            public_key: None,
        })
    }

    /// Batch-insert multiple text documents into a collection.
    ///
    /// Calls `POST /batch_insert` with `{collection, texts: [...]}`.
    /// Returns aggregate insert counts in [`BatchInsertReport`].
    pub async fn batch_insert_texts(
        &self,
        collection: &str,
        items: Vec<BatchInsertItem>,
    ) -> Result<BatchInsertReport> {
        let texts: Vec<serde_json::Value> = items
            .into_iter()
            .map(|item| {
                let mut obj = serde_json::Map::new();
                obj.insert("text".into(), serde_json::Value::String(item.text));
                if let Some(id) = item.id {
                    obj.insert("id".into(), serde_json::Value::String(id));
                }
                if let Some(meta) = item.metadata {
                    obj.insert("metadata".into(), meta);
                }
                serde_json::Value::Object(obj)
            })
            .collect();
        let payload = serde_json::json!({ "collection": collection, "texts": texts });
        let response = self
            .make_request("POST", "/batch_insert", Some(payload))
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse batch_insert_texts response: {e}"))
        })
    }

    /// Bulk-insert pre-computed embeddings.
    ///
    /// Calls `POST /insert_vectors` with `{collection, vectors: [...]}`.
    /// Skips the server-side embedding pipeline entirely; the caller
    /// supplies raw `Vec<f32>` embeddings.
    pub async fn insert_vectors(
        &self,
        collection: &str,
        vectors: Vec<RawVectorInsert>,
    ) -> Result<BatchInsertReport> {
        let payload = serde_json::json!({ "collection": collection, "vectors": vectors });
        let response = self
            .make_request("POST", "/insert_vectors", Some(payload))
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse insert_vectors response: {e}"))
        })
    }

    /// Run multiple search queries against one collection in a single
    /// round-trip.
    ///
    /// Calls `POST /batch_search` with `{collection, queries: [...]}`.
    /// Each query may carry either a text `query` (embedded server-side)
    /// or a raw `vector`. Returns one [`SearchResponse`] per query.
    pub async fn batch_search(
        &self,
        collection: &str,
        requests: Vec<BatchSearchQuery>,
    ) -> Result<Vec<SearchResponse>> {
        let payload = serde_json::json!({ "collection": collection, "queries": requests });
        let response = self
            .make_request("POST", "/batch_search", Some(payload))
            .await?;
        let val: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse batch_search response: {e}"))
        })?;
        // Server returns {collection, count, succeeded, failed, results: [...]}
        // where each element is a per-query result object.
        let results_arr = val
            .get("results")
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();
        let mut out = Vec::with_capacity(results_arr.len());
        for entry in results_arr {
            let sr: SearchResponse = serde_json::from_value(entry).map_err(|e| {
                VectorizerError::server(format!("Failed to parse batch_search entry: {e}"))
            })?;
            out.push(sr);
        }
        Ok(out)
    }

    /// Batch-update vector payloads (and optionally dense vectors).
    ///
    /// Calls `POST /batch_update` with `{collection, updates: [...]}`.
    pub async fn batch_update_vectors(
        &self,
        collection: &str,
        updates: Vec<VectorUpdate>,
    ) -> Result<BatchUpdateReport> {
        let payload = serde_json::json!({ "collection": collection, "updates": updates });
        let response = self
            .make_request("POST", "/batch_update", Some(payload))
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse batch_update_vectors response: {e}"
            ))
        })
    }

    /// Delete every vector in a collection that matches a Qdrant-style
    /// metadata filter (phase13).
    ///
    /// Calls `POST /collections/{name}/vectors/delete_by_filter` with
    /// `{"filter": <filter>}`. An empty filter is rejected by the server
    /// with 400 to prevent accidental full-collection wipes.
    ///
    /// Response: `{scanned, matched, deleted, results}`.
    pub async fn delete_by_filter(
        &self,
        collection: &str,
        filter: serde_json::Value,
    ) -> Result<DeleteByFilterReport> {
        let payload = serde_json::json!({ "filter": filter });
        let response = self
            .make_request(
                "POST",
                &format!("/collections/{collection}/vectors/delete_by_filter"),
                Some(payload),
            )
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse delete_by_filter response: {e}"))
        })
    }

    /// Apply a JSON-merge-patch to the payload of every vector matching a
    /// filter (phase13).
    ///
    /// Calls `POST /collections/{name}/vectors/bulk_update_metadata` with
    /// `{"filter": <filter>, "patch": <patch>}`. Patch is applied with
    /// RFC 7396 semantics: keys in `patch` overwrite existing payload values;
    /// `null` values remove keys.
    ///
    /// Response: `{scanned, matched, updated, results}`.
    pub async fn bulk_update_metadata(
        &self,
        collection: &str,
        filter: serde_json::Value,
        patch: serde_json::Value,
    ) -> Result<BulkUpdateReport> {
        let payload = serde_json::json!({ "filter": filter, "patch": patch });
        let response = self
            .make_request(
                "POST",
                &format!("/collections/{collection}/vectors/bulk_update_metadata"),
                Some(payload),
            )
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse bulk_update_metadata response: {e}"
            ))
        })
    }

    /// Copy vectors from `src` to `dst` without re-embedding (phase13).
    ///
    /// Unlike `move_to_collection`, the source vectors are NOT deleted.
    /// Calls `POST /collections/{src}/vectors/copy` with
    /// `{"destination": dst, "ids": [...]}`.
    ///
    /// Per-id status: `ok | missing_in_src | dst_insert_failed`.
    /// Response: `{src, dst, requested, copied, failed, results}`.
    pub async fn copy_vectors(&self, src: &str, dst: &str, ids: &[String]) -> Result<CopyReport> {
        let payload = serde_json::json!({ "destination": dst, "ids": ids });
        let response = self
            .make_request(
                "POST",
                &format!("/collections/{src}/vectors/copy"),
                Some(payload),
            )
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse copy_vectors response: {e}"))
        })
    }

    /// Set or clear a per-vector expiry timestamp (phase13).
    ///
    /// Calls `PATCH /collections/{name}/vectors/{id}/expiry` with
    /// `{"expires_at": <unix_ms>}`. Pass `None` to clear an existing expiry.
    /// The timestamp is stored as `__expires_at` inside the vector payload and
    /// is read by the per-collection TTL reaper.
    pub async fn set_vector_expiry(
        &self,
        collection: &str,
        vector_id: &str,
        expires_at: Option<i64>,
    ) -> Result<()> {
        let payload = serde_json::json!({ "expires_at": expires_at });
        self.make_request(
            "PATCH",
            &format!("/collections/{collection}/vectors/{vector_id}/expiry"),
            Some(payload),
        )
        .await?;
        Ok(())
    }

    /// Generate an embedding for `text` using either the supplied
    /// `model` name or the server default.
    pub async fn embed_text(&self, text: &str, model: Option<&str>) -> Result<EmbeddingResponse> {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "text".to_string(),
            serde_json::Value::String(text.to_string()),
        );
        if let Some(model) = model {
            payload.insert(
                "model".to_string(),
                serde_json::Value::String(model.to_string()),
            );
        }
        let response = self
            .make_request("POST", "/embed", Some(serde_json::Value::Object(payload)))
            .await?;
        let embedding_response: EmbeddingResponse =
            serde_json::from_str(&response).map_err(|e| {
                VectorizerError::server(format!("Failed to parse embedding response: {e}"))
            })?;
        Ok(embedding_response)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::models::{
        BulkUpdateReport, CopyReport, DeleteByFilterReport, ReencodeJob, VectorOpResult,
    };

    #[test]
    fn delete_by_filter_report_deserializes_server_contract() {
        let raw = json!({
            "scanned": 100,
            "matched": 3,
            "deleted": 2,
            "results": [
                {"id": "vec-1", "status": "deleted"},
                {"id": "vec-2", "status": "deleted"},
                {"id": "vec-3", "status": "error", "error": "not found"},
            ],
        });
        let report: DeleteByFilterReport = serde_json::from_value(raw).unwrap();
        assert_eq!(report.scanned, 100);
        assert_eq!(report.matched, 3);
        assert_eq!(report.deleted, 2);
        assert_eq!(report.results.len(), 3);
    }

    #[test]
    fn bulk_update_report_deserializes_server_contract() {
        let raw = json!({
            "scanned": 50,
            "matched": 5,
            "updated": 5,
            "results": [
                {"id": "vec-1", "status": "updated"},
            ],
        });
        let report: BulkUpdateReport = serde_json::from_value(raw).unwrap();
        assert_eq!(report.scanned, 50);
        assert_eq!(report.matched, 5);
        assert_eq!(report.updated, 5);
    }

    #[test]
    fn copy_report_deserializes_server_contract() {
        let raw = json!({
            "src": "hot",
            "dst": "cold",
            "requested": 3,
            "copied": 2,
            "failed": 1,
            "results": [
                {"id": "v1", "status": "ok"},
                {"id": "v2", "status": "ok"},
                {"id": "v3", "status": "missing_in_src", "error": "not found"},
            ],
        });
        let report: CopyReport = serde_json::from_value(raw).unwrap();
        assert_eq!(report.src, "hot");
        assert_eq!(report.dst, "cold");
        assert_eq!(report.copied, 2);
        assert_eq!(report.failed, 1);
        let statuses: Vec<&str> = report.results.iter().map(|r| r.status.as_str()).collect();
        assert_eq!(statuses, vec!["ok", "ok", "missing_in_src"]);
    }

    #[test]
    fn copy_report_round_trips_through_serde() {
        let report = CopyReport {
            src: "src".into(),
            dst: "dst".into(),
            requested: 1,
            copied: 1,
            failed: 0,
            results: vec![VectorOpResult {
                id: Some("v1".into()),
                status: "ok".into(),
                error: None,
                index: None,
            }],
        };
        let serialized = serde_json::to_value(&report).unwrap();
        let parsed: CopyReport = serde_json::from_value(serialized).unwrap();
        assert_eq!(parsed, report);
    }

    #[test]
    fn reencode_job_deserializes_server_contract() {
        let raw = json!({
            "job_id": "reencode-mycol-1234567890",
            "collection": "mycol",
            "state": "completed",
            "target_encoding": "sq8",
            "progress": 1.0,
        });
        let job: ReencodeJob = serde_json::from_value(raw).unwrap();
        assert_eq!(job.collection, "mycol");
        assert_eq!(job.state, "completed");
        assert_eq!(job.target_encoding, "sq8");
        assert!((job.progress - 1.0).abs() < f64::EPSILON);
    }
}
