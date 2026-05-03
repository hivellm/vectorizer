//! Typed wrappers around the v1 RPC command catalog.
//!
//! Each method in this module corresponds to one entry in the wire
//! spec's command catalog (§ 6). The wrapper:
//!
//! 1. Builds the positional `args` array per the spec.
//! 2. Calls [`RpcClient::call`].
//! 3. Decodes the [`VectorizerValue`] response into a typed Rust
//!    value with explicit field handling (no `serde_json::from_value`
//!    detour — the wire is MessagePack, not JSON).
//!
//! Adding a new typed wrapper for a v1 command landed on the server
//! is mechanical: a new method on `RpcClient` here, an entry in the
//! README, and (ideally) a test in `tests/rpc_integration.rs`.

use super::client::{Result, RpcClient, RpcClientError};
use super::types::VectorizerValue;

// ── Shared helpers ────────────────────────────────────────────────────────────

/// Decode a `VectorizerValue::Map` field as a `String`, returning
/// `RpcClientError::Server` when the field is absent or non-string.
fn need_str(v: &VectorizerValue, cmd: &str, key: &str) -> Result<String> {
    v.map_get(key)
        .and_then(|x| x.as_str().map(str::to_owned))
        .ok_or_else(|| RpcClientError::Server(format!("{cmd}: missing string field '{key}'")))
}

/// Decode a `VectorizerValue::Map` field as `i64`.
fn need_int(v: &VectorizerValue, cmd: &str, key: &str) -> Result<i64> {
    v.map_get(key)
        .and_then(|x| x.as_int())
        .ok_or_else(|| RpcClientError::Server(format!("{cmd}: missing int field '{key}'")))
}

/// Decode a `VectorizerValue::Map` field as `bool`.
fn need_bool(v: &VectorizerValue, cmd: &str, key: &str) -> Result<bool> {
    v.map_get(key)
        .and_then(|x| x.as_bool())
        .ok_or_else(|| RpcClientError::Server(format!("{cmd}: missing bool field '{key}'")))
}

/// Decode a top-level `Array` response into a `Vec<String>`.
fn decode_string_array(v: VectorizerValue, cmd: &str) -> Result<Vec<String>> {
    let arr = v
        .as_array()
        .ok_or_else(|| RpcClientError::Server(format!("{cmd}: expected Array")))?;
    Ok(arr
        .iter()
        .filter_map(|x| x.as_str().map(str::to_owned))
        .collect())
}

// ── Return types ──────────────────────────────────────────────────────────────

/// Collection metadata returned by `collections.get_info`.
#[derive(Debug, Clone)]
pub struct CollectionInfo {
    /// Collection name as registered on the server.
    pub name: String,
    /// Number of vectors currently stored.
    pub vector_count: i64,
    /// Number of source documents represented by those vectors.
    pub document_count: i64,
    /// Vector dimension.
    pub dimension: i64,
    /// Distance metric the collection's index uses (e.g. `"Cosine"`).
    pub metric: String,
    /// ISO-8601 timestamp of when the collection was created.
    pub created_at: String,
    /// ISO-8601 timestamp of the last mutation.
    pub updated_at: String,
}

/// One result from `search.basic` or `search.by_text`.
#[derive(Debug, Clone)]
pub struct SearchHit {
    /// Vector ID inside the collection.
    pub id: String,
    /// Similarity score in `[0.0, 1.0]` for cosine; backend-defined otherwise.
    pub score: f64,
    /// Optional payload serialised as a JSON string. Decode with
    /// `serde_json::from_str` for structured access.
    pub payload: Option<String>,
}

/// Response from `collections.create`.
#[derive(Debug, Clone)]
pub struct CreateCollectionResult {
    pub name: String,
    pub dimension: i64,
    pub metric: String,
    pub success: bool,
}

/// Response from `collections.cleanup_empty` (admin-gated on server).
#[derive(Debug, Clone)]
pub struct CleanupEmptyResult {
    pub removed: i64,
    pub dry_run: bool,
}

/// Response from `vectors.insert` / `vectors.insert_text` / `vectors.update`.
#[derive(Debug, Clone)]
pub struct VectorWriteResult {
    pub id: String,
    pub success: bool,
}

/// Per-item result inside batch responses.
#[derive(Debug, Clone)]
pub struct BatchItemResult {
    pub index: i64,
    pub id: Option<String>,
    pub status: String,
    pub error: Option<String>,
}

/// Response from `vectors.batch_insert` / `vectors.batch_insert_texts`.
#[derive(Debug, Clone)]
pub struct BatchInsertResult {
    pub inserted: i64,
    pub failed: i64,
    pub results: Vec<BatchItemResult>,
}

/// Response from `vectors.batch_update`.
#[derive(Debug, Clone)]
pub struct BatchUpdateResult {
    pub updated: i64,
    pub failed: i64,
    pub results: Vec<BatchItemResult>,
}

/// Response from `vectors.batch_delete`.
#[derive(Debug, Clone)]
pub struct BatchDeleteResult {
    pub deleted: i64,
    pub failed: i64,
    pub results: Vec<BatchItemResult>,
}

/// One per-query result from `vectors.batch_search`.
#[derive(Debug, Clone)]
pub struct BatchSearchResult {
    pub index: i64,
    pub status: String,
    pub results: Vec<SearchHit>,
    pub error: Option<String>,
}

/// Response from `vectors.move`.
#[derive(Debug, Clone)]
pub struct MoveRpcResult {
    pub src: String,
    pub dst: String,
    pub moved: i64,
    pub failed: i64,
}

/// Response from `vectors.copy`.
#[derive(Debug, Clone)]
pub struct CopyRpcResult {
    pub src: String,
    pub dst: String,
    pub copied: i64,
    pub failed: i64,
}

/// Response from `vectors.delete_by_filter`.
#[derive(Debug, Clone)]
pub struct DeleteByFilterRpcResult {
    pub scanned: i64,
    pub matched: i64,
    pub deleted: i64,
}

/// Response from `vectors.bulk_update_metadata`.
#[derive(Debug, Clone)]
pub struct BulkUpdateMetadataRpcResult {
    pub scanned: i64,
    pub matched: i64,
    pub updated: i64,
}

/// Response from `vectors.set_expiry`.
#[derive(Debug, Clone)]
pub struct SetExpiryResult {
    pub id: String,
    pub expires_at: i64,
    pub success: bool,
}

/// Response from `vectors.embed`.
#[derive(Debug, Clone)]
pub struct EmbedResult {
    pub embedding: Vec<f64>,
    pub model: String,
    pub dimension: i64,
}

/// Response from `vectors.list`.
#[derive(Debug, Clone)]
pub struct VectorListResult {
    pub items: Vec<VectorizerValue>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
}

/// Paginated list of vector IDs and data from `vectors.list`.
/// Items are raw `VectorizerValue::Map` entries containing `id`, `data`, etc.

/// Response from `search.explain`.
#[derive(Debug, Clone)]
pub struct SearchExplainResult {
    pub hits: Vec<SearchHit>,
    pub collection: String,
    pub k: i64,
    pub trace: SearchTrace,
}

/// HNSW traversal trace from `search.explain`.
#[derive(Debug, Clone)]
pub struct SearchTrace {
    pub visited_nodes: i64,
    pub ef_search: i64,
    pub hnsw_search_ms: f64,
    pub total_ms: f64,
}

/// Summary response from `discovery.discover`.
#[derive(Debug, Clone)]
pub struct DiscoverResult {
    pub answer_prompt: String,
    pub sections: i64,
    pub bullets: i64,
    pub chunks: i64,
}

/// One scored collection from `discovery.score_collections`.
#[derive(Debug, Clone)]
pub struct ScoredCollection {
    pub name: String,
    pub score: f64,
    pub vector_count: i64,
}

/// Response from `discovery.expand_queries`.
#[derive(Debug, Clone)]
pub struct ExpandQueriesResult {
    pub original_query: String,
    pub expanded_queries: Vec<String>,
    pub count: i64,
}

/// One chunk from `discovery.broad_discovery` / `discovery.semantic_focus`.
#[derive(Debug, Clone)]
pub struct DiscoveryChunk {
    pub collection: String,
    pub score: f64,
    pub content_preview: String,
}

/// Response from `discovery.compress_evidence`.
#[derive(Debug, Clone)]
pub struct CompressBullet {
    pub text: String,
    pub source_id: String,
    pub score: f64,
}

/// Response from `discovery.build_answer_plan`.
#[derive(Debug, Clone)]
pub struct AnswerPlanResult {
    pub sections: Vec<AnswerPlanSection>,
    pub total_bullets: i64,
}

/// One section inside an answer plan.
#[derive(Debug, Clone)]
pub struct AnswerPlanSection {
    pub title: String,
    pub bullets_count: i64,
}

/// Response from `discovery.render_llm_prompt`.
#[derive(Debug, Clone)]
pub struct RenderPromptResult {
    pub prompt: String,
    pub length: i64,
    pub estimated_tokens: i64,
}

/// Response from graph discovery stats (`graph.discovery_status`).
#[derive(Debug, Clone)]
pub struct GraphDiscoveryStatus {
    pub total_nodes: i64,
    pub nodes_with_edges: i64,
    pub total_edges: i64,
    pub progress_percentage: f64,
}

/// Response from `graph.discover_edges`.
#[derive(Debug, Clone)]
pub struct DiscoverEdgesResult {
    pub success: bool,
    pub total_nodes: i64,
    pub nodes_processed: i64,
    pub nodes_with_edges: i64,
    pub total_edges_created: i64,
}

/// Response from `graph.discover_edges_for_node`.
#[derive(Debug, Clone)]
pub struct DiscoverEdgesForNodeResult {
    pub success: bool,
    pub node_id: String,
    pub edges_created: i64,
}

/// Admin stats response from `admin.stats`.
#[derive(Debug, Clone)]
pub struct AdminStats {
    pub collections_count: i64,
    pub total_vectors: i64,
    pub version: String,
}

/// Admin status response from `admin.status`.
#[derive(Debug, Clone)]
pub struct AdminStatus {
    pub ready: bool,
    pub collections_count: i64,
    pub version: String,
}

/// Slow query config from `admin.slow_queries_config`.
#[derive(Debug, Clone)]
pub struct SlowQueryConfigResult {
    pub threshold_ms: i64,
    pub capacity: i64,
    pub status: String,
}

/// Response from `auth.me`.
#[derive(Debug, Clone)]
pub struct AuthMeResult {
    pub username: String,
    pub authenticated: bool,
}

/// Response from `auth.refresh_token`.
#[derive(Debug, Clone)]
pub struct RefreshTokenResult {
    pub access_token: String,
    pub token_type: String,
}

/// Response from `auth.validate_password`.
#[derive(Debug, Clone)]
pub struct ValidatePasswordResult {
    pub valid: bool,
    pub errors: Vec<String>,
}

/// Response from `auth.api_keys_create` / `auth.api_keys_create_scoped`.
#[derive(Debug, Clone)]
pub struct ApiKeyCreated {
    pub api_key: String,
    pub id: String,
    pub name: String,
}

/// Response from `auth.api_keys_rotate`.
#[derive(Debug, Clone)]
pub struct RotatedApiKey {
    pub old_key_id: String,
    pub new_key_id: String,
    pub new_token: String,
    pub grace_until: Option<String>,
}

/// Response from `replication.configure`.
#[derive(Debug, Clone)]
pub struct ReplicationConfigureResult {
    pub success: bool,
    pub role: String,
    pub message: String,
}

/// Response from `cluster.rebalance_status` — may be idle or active.
#[derive(Debug, Clone)]
pub struct RebalanceStatus {
    /// `"idle"` when no rebalance is running.
    pub status: Option<String>,
    pub message: Option<String>,
}

// ── Helper: decode batch item results ────────────────────────────────────────

fn decode_batch_items(arr: &[VectorizerValue]) -> Vec<BatchItemResult> {
    arr.iter()
        .map(|entry| {
            let index = entry.map_get("index").and_then(|v| v.as_int()).unwrap_or(0);
            let id = entry
                .map_get("id")
                .and_then(|v| v.as_str())
                .map(str::to_owned);
            let status = entry
                .map_get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_owned();
            let error = entry
                .map_get("error")
                .and_then(|v| v.as_str())
                .map(str::to_owned);
            BatchItemResult {
                index,
                id,
                status,
                error,
            }
        })
        .collect()
}

fn decode_search_hits(arr: &[VectorizerValue]) -> Vec<SearchHit> {
    arr.iter()
        .filter_map(|entry| {
            let id = entry.map_get("id")?.as_str().map(str::to_owned)?;
            let score = entry
                .map_get("score")
                .and_then(|v| v.as_float())
                .unwrap_or(0.0);
            let payload = entry
                .map_get("payload")
                .and_then(|v| v.as_str())
                .map(str::to_owned);
            Some(SearchHit { id, score, payload })
        })
        .collect()
}

// ═════════════════════════════════════════════════════════════════════════════
// Collections
// ═════════════════════════════════════════════════════════════════════════════

impl RpcClient {
    /// `collections.list` — return every collection name visible to
    /// the authenticated principal.
    pub async fn list_collections(&self) -> Result<Vec<String>> {
        let v = self.call("collections.list", vec![]).await?;
        decode_string_array(v, "collections.list")
    }

    /// `collections.get_info` — return metadata for one collection.
    pub async fn get_collection_info(&self, name: &str) -> Result<CollectionInfo> {
        let v = self
            .call(
                "collections.get_info",
                vec![VectorizerValue::Str(name.to_owned())],
            )
            .await?;
        Ok(CollectionInfo {
            name: need_str(&v, "collections.get_info", "name")?,
            vector_count: need_int(&v, "collections.get_info", "vector_count")?,
            document_count: need_int(&v, "collections.get_info", "document_count")?,
            dimension: need_int(&v, "collections.get_info", "dimension")?,
            metric: need_str(&v, "collections.get_info", "metric")?,
            created_at: need_str(&v, "collections.get_info", "created_at")?,
            updated_at: need_str(&v, "collections.get_info", "updated_at")?,
        })
    }

    /// `collections.create` — create a new collection.
    ///
    /// `config` is a `Map` with optional keys `dimension` (`Int`) and
    /// `metric` (`Str`: `"cosine"` | `"euclidean"` | `"dot"`).
    pub async fn create_collection(
        &self,
        name: &str,
        config: VectorizerValue,
    ) -> Result<CreateCollectionResult> {
        let v = self
            .call(
                "collections.create",
                vec![VectorizerValue::Str(name.to_owned()), config],
            )
            .await?;
        Ok(CreateCollectionResult {
            name: need_str(&v, "collections.create", "name")?,
            dimension: need_int(&v, "collections.create", "dimension")?,
            metric: need_str(&v, "collections.create", "metric")?,
            success: need_bool(&v, "collections.create", "success")?,
        })
    }

    /// `collections.delete` — delete a collection (admin-gated on server).
    pub async fn delete_collection(&self, name: &str) -> Result<bool> {
        let v = self
            .call(
                "collections.delete",
                vec![VectorizerValue::Str(name.to_owned())],
            )
            .await?;
        need_bool(&v, "collections.delete", "success")
    }

    /// `collections.list_empty` — list collections that contain zero vectors.
    pub async fn list_empty_collections(&self) -> Result<Vec<String>> {
        let v = self.call("collections.list_empty", vec![]).await?;
        decode_string_array(v, "collections.list_empty")
    }

    /// `collections.cleanup_empty` — remove empty collections.
    ///
    /// Pass `dry_run: true` to preview which collections would be removed
    /// without actually deleting them.
    pub async fn cleanup_empty_collections(&self, dry_run: bool) -> Result<CleanupEmptyResult> {
        let config = VectorizerValue::Map(vec![(
            VectorizerValue::Str("dry_run".into()),
            VectorizerValue::Bool(dry_run),
        )]);
        let v = self.call("collections.cleanup_empty", vec![config]).await?;
        Ok(CleanupEmptyResult {
            removed: need_int(&v, "collections.cleanup_empty", "removed")?,
            dry_run: need_bool(&v, "collections.cleanup_empty", "dry_run")?,
        })
    }

    /// `collections.force_save` — flush a collection's in-memory state to disk.
    pub async fn force_save_collection(&self, name: &str) -> Result<bool> {
        let v = self
            .call(
                "collections.force_save",
                vec![VectorizerValue::Str(name.to_owned())],
            )
            .await?;
        need_bool(&v, "collections.force_save", "success")
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Vectors
// ═════════════════════════════════════════════════════════════════════════════

impl RpcClient {
    /// `vectors.get` — fetch one vector by id. Returns the raw
    /// `VectorizerValue::Map` so callers can read whichever fields
    /// they care about (`id`, `data`, `payload`, `document_id`).
    pub async fn get_vector(&self, collection: &str, vector_id: &str) -> Result<VectorizerValue> {
        self.call(
            "vectors.get",
            vec![
                VectorizerValue::Str(collection.to_owned()),
                VectorizerValue::Str(vector_id.to_owned()),
            ],
        )
        .await
    }

    /// `vectors.insert` — insert one pre-computed vector.
    ///
    /// `data` must match the collection's configured dimension.
    /// `payload` is an optional `VectorizerValue::Map` of metadata.
    pub async fn insert_vector(
        &self,
        collection: &str,
        id: Option<&str>,
        data: Vec<f32>,
        payload: Option<VectorizerValue>,
    ) -> Result<VectorWriteResult> {
        let id_val = id
            .map(|s| VectorizerValue::Str(s.to_owned()))
            .unwrap_or(VectorizerValue::Null);
        let data_val = VectorizerValue::Array(
            data.into_iter()
                .map(|f| VectorizerValue::Float(f as f64))
                .collect(),
        );
        let mut args = vec![
            VectorizerValue::Str(collection.to_owned()),
            id_val,
            data_val,
        ];
        if let Some(p) = payload {
            args.push(p);
        }
        let v = self.call("vectors.insert", args).await?;
        Ok(VectorWriteResult {
            id: need_str(&v, "vectors.insert", "id")?,
            success: need_bool(&v, "vectors.insert", "success")?,
        })
    }

    /// `vectors.insert_text` — embed `text` server-side and insert.
    ///
    /// The server auto-creates the collection if it does not exist.
    pub async fn insert_text_vector(
        &self,
        collection: &str,
        id: Option<&str>,
        text: &str,
        payload: Option<VectorizerValue>,
    ) -> Result<VectorWriteResult> {
        let id_val = id
            .map(|s| VectorizerValue::Str(s.to_owned()))
            .unwrap_or(VectorizerValue::Null);
        let mut args = vec![
            VectorizerValue::Str(collection.to_owned()),
            id_val,
            VectorizerValue::Str(text.to_owned()),
        ];
        if let Some(p) = payload {
            args.push(p);
        }
        let v = self.call("vectors.insert_text", args).await?;
        Ok(VectorWriteResult {
            id: need_str(&v, "vectors.insert_text", "id")?,
            success: need_bool(&v, "vectors.insert_text", "success")?,
        })
    }

    /// `vectors.update` — replace a vector's data and/or payload.
    pub async fn update_vector(
        &self,
        collection: &str,
        id: &str,
        data: Vec<f32>,
        payload: Option<VectorizerValue>,
    ) -> Result<VectorWriteResult> {
        let data_val = VectorizerValue::Array(
            data.into_iter()
                .map(|f| VectorizerValue::Float(f as f64))
                .collect(),
        );
        let mut args = vec![
            VectorizerValue::Str(collection.to_owned()),
            VectorizerValue::Str(id.to_owned()),
            data_val,
        ];
        if let Some(p) = payload {
            args.push(p);
        }
        let v = self.call("vectors.update", args).await?;
        Ok(VectorWriteResult {
            id: need_str(&v, "vectors.update", "id")?,
            success: need_bool(&v, "vectors.update", "success")?,
        })
    }

    /// `vectors.delete` — delete one vector by id.
    pub async fn delete_vector_rpc(&self, collection: &str, id: &str) -> Result<bool> {
        let v = self
            .call(
                "vectors.delete",
                vec![
                    VectorizerValue::Str(collection.to_owned()),
                    VectorizerValue::Str(id.to_owned()),
                ],
            )
            .await?;
        need_bool(&v, "vectors.delete", "success")
    }

    /// `vectors.list` — page through vectors in a collection.
    ///
    /// `page` is zero-based; `limit` is capped at 50 by the server.
    pub async fn list_vectors(
        &self,
        collection: &str,
        page: i64,
        limit: i64,
    ) -> Result<VectorListResult> {
        let v = self
            .call(
                "vectors.list",
                vec![
                    VectorizerValue::Str(collection.to_owned()),
                    VectorizerValue::Int(page),
                    VectorizerValue::Int(limit),
                ],
            )
            .await?;
        let items = v
            .map_get("items")
            .and_then(|x| x.as_array())
            .map(|a| a.to_vec())
            .unwrap_or_default();
        Ok(VectorListResult {
            items,
            total: need_int(&v, "vectors.list", "total")?,
            page: need_int(&v, "vectors.list", "page")?,
            limit: need_int(&v, "vectors.list", "limit")?,
        })
    }

    /// `vectors.embed` — embed `text` server-side and return the embedding.
    pub async fn embed_text(&self, text: &str, model: Option<&str>) -> Result<EmbedResult> {
        let mut args = vec![VectorizerValue::Str(text.to_owned())];
        if let Some(m) = model {
            args.push(VectorizerValue::Str(m.to_owned()));
        }
        let v = self.call("vectors.embed", args).await?;
        let embedding = v
            .map_get("embedding")
            .and_then(|x| x.as_array())
            .map(|arr| arr.iter().filter_map(|x| x.as_float()).collect())
            .unwrap_or_default();
        Ok(EmbedResult {
            embedding,
            model: v
                .map_get("model")
                .and_then(|x| x.as_str())
                .unwrap_or("bm25")
                .to_owned(),
            dimension: v.map_get("dimension").and_then(|x| x.as_int()).unwrap_or(0),
        })
    }

    /// `vectors.batch_insert` — insert multiple pre-computed vectors.
    ///
    /// Each item in `items` is a `VectorizerValue::Map` with at least
    /// `data` (`Array<Float>`) and optionally `id` (`Str`) and `payload`
    /// (`Map`).
    pub async fn batch_insert_vectors(
        &self,
        collection: &str,
        items: Vec<VectorizerValue>,
    ) -> Result<BatchInsertResult> {
        let v = self
            .call(
                "vectors.batch_insert",
                vec![
                    VectorizerValue::Str(collection.to_owned()),
                    VectorizerValue::Array(items),
                ],
            )
            .await?;
        let results = v
            .map_get("results")
            .and_then(|x| x.as_array())
            .map(|a| decode_batch_items(a))
            .unwrap_or_default();
        Ok(BatchInsertResult {
            inserted: v.map_get("inserted").and_then(|x| x.as_int()).unwrap_or(0),
            failed: v.map_get("failed").and_then(|x| x.as_int()).unwrap_or(0),
            results,
        })
    }

    /// `vectors.batch_insert_texts` — embed and insert multiple text items.
    ///
    /// Each item in `items` is a `VectorizerValue::Map` with at least
    /// `text` (`Str`) and optionally `id` (`Str`) and `payload` (`Map`).
    pub async fn batch_insert_texts(
        &self,
        collection: &str,
        items: Vec<VectorizerValue>,
    ) -> Result<BatchInsertResult> {
        let v = self
            .call(
                "vectors.batch_insert_texts",
                vec![
                    VectorizerValue::Str(collection.to_owned()),
                    VectorizerValue::Array(items),
                ],
            )
            .await?;
        let results = v
            .map_get("results")
            .and_then(|x| x.as_array())
            .map(|a| decode_batch_items(a))
            .unwrap_or_default();
        Ok(BatchInsertResult {
            inserted: v.map_get("inserted").and_then(|x| x.as_int()).unwrap_or(0),
            failed: v.map_get("failed").and_then(|x| x.as_int()).unwrap_or(0),
            results,
        })
    }

    /// `vectors.batch_search` — run multiple searches in one round-trip.
    ///
    /// Each request in `requests` is a `VectorizerValue::Map` with
    /// `collection` (`Str`), `query` (`Str`), and optional `limit` (`Int`).
    pub async fn batch_search(
        &self,
        requests: Vec<VectorizerValue>,
    ) -> Result<Vec<BatchSearchResult>> {
        let v = self
            .call(
                "vectors.batch_search",
                vec![VectorizerValue::Array(requests)],
            )
            .await?;
        let arr = v
            .as_array()
            .ok_or_else(|| RpcClientError::Server("vectors.batch_search: expected Array".into()))?;
        Ok(arr
            .iter()
            .map(|entry| {
                let index = entry.map_get("index").and_then(|x| x.as_int()).unwrap_or(0);
                let status = entry
                    .map_get("status")
                    .and_then(|x| x.as_str())
                    .unwrap_or("unknown")
                    .to_owned();
                let error = entry
                    .map_get("error")
                    .and_then(|x| x.as_str())
                    .map(str::to_owned);
                let results = entry
                    .map_get("results")
                    .and_then(|x| x.as_array())
                    .map(|a| decode_search_hits(a))
                    .unwrap_or_default();
                BatchSearchResult {
                    index,
                    status,
                    results,
                    error,
                }
            })
            .collect())
    }

    /// `vectors.batch_update` — update multiple vectors' data and/or payload.
    ///
    /// Each item in `updates` is a `VectorizerValue::Map` with `id` (`Str`)
    /// and optionally `data` (`Array<Float>`) and `payload` (`Map`).
    pub async fn batch_update_vectors(
        &self,
        collection: &str,
        updates: Vec<VectorizerValue>,
    ) -> Result<BatchUpdateResult> {
        let v = self
            .call(
                "vectors.batch_update",
                vec![
                    VectorizerValue::Str(collection.to_owned()),
                    VectorizerValue::Array(updates),
                ],
            )
            .await?;
        let results = v
            .map_get("results")
            .and_then(|x| x.as_array())
            .map(|a| decode_batch_items(a))
            .unwrap_or_default();
        Ok(BatchUpdateResult {
            updated: v.map_get("updated").and_then(|x| x.as_int()).unwrap_or(0),
            failed: v.map_get("failed").and_then(|x| x.as_int()).unwrap_or(0),
            results,
        })
    }

    /// `vectors.batch_delete` — delete multiple vectors by id.
    pub async fn batch_delete_vectors(
        &self,
        collection: &str,
        ids: Vec<String>,
    ) -> Result<BatchDeleteResult> {
        let ids_val = VectorizerValue::Array(ids.into_iter().map(VectorizerValue::Str).collect());
        let v = self
            .call(
                "vectors.batch_delete",
                vec![VectorizerValue::Str(collection.to_owned()), ids_val],
            )
            .await?;
        let results = v
            .map_get("results")
            .and_then(|x| x.as_array())
            .map(|a| decode_batch_items(a))
            .unwrap_or_default();
        Ok(BatchDeleteResult {
            deleted: v.map_get("deleted").and_then(|x| x.as_int()).unwrap_or(0),
            failed: v.map_get("failed").and_then(|x| x.as_int()).unwrap_or(0),
            results,
        })
    }

    /// `vectors.move` — move vectors from `src` to `dst` collection.
    ///
    /// Named `move_vectors_rpc` to avoid collision with the REST SDK's
    /// `move_to_collection`.
    pub async fn move_vectors_rpc(
        &self,
        src: &str,
        dst: &str,
        ids: Vec<String>,
    ) -> Result<MoveRpcResult> {
        let ids_val = VectorizerValue::Array(ids.into_iter().map(VectorizerValue::Str).collect());
        let v = self
            .call(
                "vectors.move",
                vec![
                    VectorizerValue::Str(src.to_owned()),
                    VectorizerValue::Str(dst.to_owned()),
                    ids_val,
                ],
            )
            .await?;
        Ok(MoveRpcResult {
            src: need_str(&v, "vectors.move", "src")?,
            dst: need_str(&v, "vectors.move", "dst")?,
            moved: v.map_get("moved").and_then(|x| x.as_int()).unwrap_or(0),
            failed: v.map_get("failed").and_then(|x| x.as_int()).unwrap_or(0),
        })
    }

    /// `vectors.copy` — copy vectors from `src` to `dst` without deleting.
    pub async fn copy_vectors_rpc(
        &self,
        src: &str,
        dst: &str,
        ids: Vec<String>,
    ) -> Result<CopyRpcResult> {
        let ids_val = VectorizerValue::Array(ids.into_iter().map(VectorizerValue::Str).collect());
        let v = self
            .call(
                "vectors.copy",
                vec![
                    VectorizerValue::Str(src.to_owned()),
                    VectorizerValue::Str(dst.to_owned()),
                    ids_val,
                ],
            )
            .await?;
        Ok(CopyRpcResult {
            src: need_str(&v, "vectors.copy", "src")?,
            dst: need_str(&v, "vectors.copy", "dst")?,
            copied: v.map_get("copied").and_then(|x| x.as_int()).unwrap_or(0),
            failed: v.map_get("failed").and_then(|x| x.as_int()).unwrap_or(0),
        })
    }

    /// `vectors.delete_by_filter` — delete all vectors matching a Qdrant-style
    /// filter predicate.
    ///
    /// `filter` is a `VectorizerValue::Map` matching the Qdrant filter schema.
    pub async fn delete_by_filter_rpc(
        &self,
        collection: &str,
        filter: VectorizerValue,
    ) -> Result<DeleteByFilterRpcResult> {
        let v = self
            .call(
                "vectors.delete_by_filter",
                vec![VectorizerValue::Str(collection.to_owned()), filter],
            )
            .await?;
        Ok(DeleteByFilterRpcResult {
            scanned: v.map_get("scanned").and_then(|x| x.as_int()).unwrap_or(0),
            matched: v.map_get("matched").and_then(|x| x.as_int()).unwrap_or(0),
            deleted: v.map_get("deleted").and_then(|x| x.as_int()).unwrap_or(0),
        })
    }

    /// `vectors.bulk_update_metadata` — apply a JSON-merge-patch to all
    /// vectors matching `filter`.
    ///
    /// `filter` selects the target vectors; `patch` is applied via RFC 7396
    /// merge-patch.
    pub async fn bulk_update_metadata_rpc(
        &self,
        collection: &str,
        filter: VectorizerValue,
        patch: VectorizerValue,
    ) -> Result<BulkUpdateMetadataRpcResult> {
        let v = self
            .call(
                "vectors.bulk_update_metadata",
                vec![VectorizerValue::Str(collection.to_owned()), filter, patch],
            )
            .await?;
        Ok(BulkUpdateMetadataRpcResult {
            scanned: v.map_get("scanned").and_then(|x| x.as_int()).unwrap_or(0),
            matched: v.map_get("matched").and_then(|x| x.as_int()).unwrap_or(0),
            updated: v.map_get("updated").and_then(|x| x.as_int()).unwrap_or(0),
        })
    }

    /// `vectors.set_expiry` — attach a TTL to one vector.
    ///
    /// `expires_at` may be a Unix millisecond timestamp or an RFC3339 string.
    pub async fn set_vector_expiry(
        &self,
        collection: &str,
        id: &str,
        expires_at: &str,
    ) -> Result<SetExpiryResult> {
        let v = self
            .call(
                "vectors.set_expiry",
                vec![
                    VectorizerValue::Str(collection.to_owned()),
                    VectorizerValue::Str(id.to_owned()),
                    VectorizerValue::Str(expires_at.to_owned()),
                ],
            )
            .await?;
        Ok(SetExpiryResult {
            id: need_str(&v, "vectors.set_expiry", "id")?,
            expires_at: need_int(&v, "vectors.set_expiry", "expires_at")?,
            success: need_bool(&v, "vectors.set_expiry", "success")?,
        })
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Search
// ═════════════════════════════════════════════════════════════════════════════

impl RpcClient {
    /// `search.basic` — search `collection` for `query` and return up
    /// to `limit` hits sorted by descending similarity.
    pub async fn search_basic(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchHit>> {
        let args = vec![
            VectorizerValue::Str(collection.to_owned()),
            VectorizerValue::Str(query.to_owned()),
            VectorizerValue::Int(limit as i64),
        ];
        let v = self.call("search.basic", args).await?;
        let arr = v
            .as_array()
            .ok_or_else(|| RpcClientError::Server("search.basic: expected Array".into()))?;
        Ok(decode_search_hits(arr))
    }

    /// `search.intelligent` — multi-collection intelligent search.
    ///
    /// `request` is a `VectorizerValue::Map` with at minimum `query` (`Str`)
    /// and optionally `collections` (`Array<Str>`), `max_results` (`Int`),
    /// `domain_expansion` (`Bool`).
    pub async fn search_intelligent(&self, request: VectorizerValue) -> Result<VectorizerValue> {
        self.call("search.intelligent", vec![request]).await
    }

    /// `search.by_text` — search one collection by text query.
    pub async fn search_by_text(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchHit>> {
        let v = self
            .call(
                "search.by_text",
                vec![
                    VectorizerValue::Str(collection.to_owned()),
                    VectorizerValue::Str(query.to_owned()),
                    VectorizerValue::Int(limit as i64),
                ],
            )
            .await?;
        let arr = v
            .map_get("results")
            .and_then(|x| x.as_array())
            .ok_or_else(|| {
                RpcClientError::Server("search.by_text: missing results array".into())
            })?;
        Ok(decode_search_hits(arr))
    }

    /// `search.by_file` — file-content-based search.
    ///
    /// `request` is a `VectorizerValue::Map` describing the file query.
    /// The server currently returns an empty result set (stub surface);
    /// this method is provided for forward-compatibility.
    pub async fn search_by_file(
        &self,
        collection: &str,
        request: VectorizerValue,
    ) -> Result<Vec<SearchHit>> {
        let v = self
            .call(
                "search.by_file",
                vec![VectorizerValue::Str(collection.to_owned()), request],
            )
            .await?;
        let arr = v
            .map_get("results")
            .and_then(|x| x.as_array())
            .ok_or_else(|| {
                RpcClientError::Server("search.by_file: missing results array".into())
            })?;
        Ok(decode_search_hits(arr))
    }

    /// `search.hybrid` — RRF / weighted-combination hybrid dense+sparse search.
    ///
    /// `request` is a `VectorizerValue::Map` with at minimum `query` (`Str`)
    /// and optional keys `alpha`, `dense_k`, `sparse_k`, `final_k`,
    /// `algorithm` (`"rrf"` | `"weighted"` | `"alpha"`).
    pub async fn search_hybrid(
        &self,
        collection: &str,
        request: VectorizerValue,
    ) -> Result<Vec<SearchHit>> {
        let v = self
            .call(
                "search.hybrid",
                vec![VectorizerValue::Str(collection.to_owned()), request],
            )
            .await?;
        let arr = v
            .map_get("results")
            .and_then(|x| x.as_array())
            .ok_or_else(|| RpcClientError::Server("search.hybrid: missing results array".into()))?;
        Ok(decode_search_hits(arr))
    }

    /// `search.semantic` — semantic re-ranking search.
    ///
    /// `request` is a `VectorizerValue::Map` with `query` (`Str`),
    /// `collection` (`Str`), and optional `max_results`, `semantic_reranking`,
    /// `cross_encoder_reranking`, `similarity_threshold`.
    pub async fn search_semantic(&self, request: VectorizerValue) -> Result<VectorizerValue> {
        self.call("search.semantic", vec![request]).await
    }

    /// `search.contextual` — context-filtered semantic search.
    ///
    /// `request` is a `VectorizerValue::Map` with `query` (`Str`),
    /// `collection` (`Str`), and optional `context_filters` (`Map`),
    /// `max_results`, `context_weight`, `context_reranking`.
    pub async fn search_contextual(&self, request: VectorizerValue) -> Result<VectorizerValue> {
        self.call("search.contextual", vec![request]).await
    }

    /// `search.multi_collection` — fan-out search across multiple collections.
    ///
    /// `request` is a `VectorizerValue::Map` with `query` (`Str`),
    /// `collections` (`Array<Str>`), and optional `max_per_collection`,
    /// `max_total_results`, `cross_collection_reranking`.
    pub async fn search_multi_collection(
        &self,
        request: VectorizerValue,
    ) -> Result<VectorizerValue> {
        self.call("search.multi_collection", vec![request]).await
    }

    /// `search.explain` — run a vector search and return HNSW traversal trace.
    ///
    /// `collection` is the target collection; `request` must contain
    /// `vector` (`Array<Float>`) and optionally `k` (`Int`).
    pub async fn search_explain(
        &self,
        collection: &str,
        request: VectorizerValue,
    ) -> Result<SearchExplainResult> {
        let v = self
            .call(
                "search.explain",
                vec![VectorizerValue::Str(collection.to_owned()), request],
            )
            .await?;
        let hits = v
            .map_get("hits")
            .and_then(|x| x.as_array())
            .map(|a| decode_search_hits(a))
            .unwrap_or_default();
        let trace_val = v.map_get("trace").cloned().unwrap_or(VectorizerValue::Null);
        let trace = SearchTrace {
            visited_nodes: trace_val
                .map_get("visited_nodes")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
            ef_search: trace_val
                .map_get("ef_search")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
            hnsw_search_ms: trace_val
                .map_get("hnsw_search_ms")
                .and_then(|x| x.as_float())
                .unwrap_or(0.0),
            total_ms: trace_val
                .map_get("total_ms")
                .and_then(|x| x.as_float())
                .unwrap_or(0.0),
        };
        Ok(SearchExplainResult {
            hits,
            collection: v
                .map_get("collection")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_owned(),
            k: v.map_get("k").and_then(|x| x.as_int()).unwrap_or(0),
            trace,
        })
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Discovery
// ═════════════════════════════════════════════════════════════════════════════

impl RpcClient {
    /// `discovery.discover` — full discovery pipeline: embed → search →
    /// compress → build plan → render prompt.
    ///
    /// `request` must contain `query` (`Str`) and optionally
    /// `include_collections`, `exclude_collections` (`Array<Str>`),
    /// `max_bullets` (`Int`).
    pub async fn discover(&self, request: VectorizerValue) -> Result<DiscoverResult> {
        let v = self.call("discovery.discover", vec![request]).await?;
        Ok(DiscoverResult {
            answer_prompt: need_str(&v, "discovery.discover", "answer_prompt")?,
            sections: v.map_get("sections").and_then(|x| x.as_int()).unwrap_or(0),
            bullets: v.map_get("bullets").and_then(|x| x.as_int()).unwrap_or(0),
            chunks: v.map_get("chunks").and_then(|x| x.as_int()).unwrap_or(0),
        })
    }

    /// `discovery.filter_collections` — filter collection list by query
    /// relevance.
    ///
    /// `request` must contain `query` (`Str`) and optionally `include` /
    /// `exclude` (`Array<Str>`).
    pub async fn filter_collections(&self, request: VectorizerValue) -> Result<Vec<String>> {
        let v = self
            .call("discovery.filter_collections", vec![request])
            .await?;
        let arr = v
            .map_get("filtered_collections")
            .and_then(|x| x.as_array())
            .ok_or_else(|| {
                RpcClientError::Server(
                    "discovery.filter_collections: missing filtered_collections".into(),
                )
            })?;
        Ok(arr
            .iter()
            .filter_map(|entry| entry.map_get("name")?.as_str().map(str::to_owned))
            .collect())
    }

    /// `discovery.score_collections` — score all collections for a query.
    ///
    /// `request` must contain `query` (`Str`).
    pub async fn score_collections(
        &self,
        request: VectorizerValue,
    ) -> Result<Vec<ScoredCollection>> {
        let v = self
            .call("discovery.score_collections", vec![request])
            .await?;
        let arr = v
            .map_get("scored_collections")
            .and_then(|x| x.as_array())
            .ok_or_else(|| {
                RpcClientError::Server(
                    "discovery.score_collections: missing scored_collections".into(),
                )
            })?;
        Ok(arr
            .iter()
            .map(|entry| ScoredCollection {
                name: entry
                    .map_get("name")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_owned(),
                score: entry
                    .map_get("score")
                    .and_then(|x| x.as_float())
                    .unwrap_or(0.0),
                vector_count: entry
                    .map_get("vector_count")
                    .and_then(|x| x.as_int())
                    .unwrap_or(0),
            })
            .collect())
    }

    /// `discovery.expand_queries` — generate query variants via baseline expansion.
    ///
    /// `request` must contain `query` (`Str`) and optionally
    /// `max_expansions` (`Int`), `include_definition`, `include_features`,
    /// `include_architecture` (`Bool`).
    pub async fn expand_queries(&self, request: VectorizerValue) -> Result<ExpandQueriesResult> {
        let v = self.call("discovery.expand_queries", vec![request]).await?;
        let expanded = v
            .map_get("expanded_queries")
            .and_then(|x| x.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default();
        Ok(ExpandQueriesResult {
            original_query: need_str(&v, "discovery.expand_queries", "original_query")?,
            count: v.map_get("count").and_then(|x| x.as_int()).unwrap_or(0),
            expanded_queries: expanded,
        })
    }

    /// `discovery.broad_discovery` — multi-query broad search across all
    /// collections.
    ///
    /// `request` must contain `queries` (`Array<Str>`) and optionally
    /// `k` (`Int`).
    pub async fn broad_discovery(&self, request: VectorizerValue) -> Result<Vec<DiscoveryChunk>> {
        let v = self
            .call("discovery.broad_discovery", vec![request])
            .await?;
        let arr = v
            .map_get("chunks")
            .and_then(|x| x.as_array())
            .ok_or_else(|| {
                RpcClientError::Server("discovery.broad_discovery: missing chunks".into())
            })?;
        Ok(arr
            .iter()
            .map(|entry| DiscoveryChunk {
                collection: entry
                    .map_get("collection")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_owned(),
                score: entry
                    .map_get("score")
                    .and_then(|x| x.as_float())
                    .unwrap_or(0.0),
                content_preview: entry
                    .map_get("content_preview")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_owned(),
            })
            .collect())
    }

    /// `discovery.semantic_focus` — deep semantic search within one collection.
    ///
    /// `request` must contain `collection` (`Str`), `queries`
    /// (`Array<Str>`), and optionally `k` (`Int`).
    pub async fn semantic_focus(&self, request: VectorizerValue) -> Result<Vec<DiscoveryChunk>> {
        let v = self.call("discovery.semantic_focus", vec![request]).await?;
        let arr = v
            .map_get("chunks")
            .and_then(|x| x.as_array())
            .ok_or_else(|| {
                RpcClientError::Server("discovery.semantic_focus: missing chunks".into())
            })?;
        Ok(arr
            .iter()
            .map(|entry| DiscoveryChunk {
                collection: entry
                    .map_get("collection")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_owned(),
                score: entry
                    .map_get("score")
                    .and_then(|x| x.as_float())
                    .unwrap_or(0.0),
                content_preview: entry
                    .map_get("content_preview")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_owned(),
            })
            .collect())
    }

    /// `discovery.promote_readme` — promote README chunks to the top of a
    /// chunk set.
    ///
    /// `request` must contain `chunks` (`Array<Map>`) where each map has
    /// `collection`, `doc_id`, `content`, `score`, `file_path`,
    /// `chunk_index`, `file_extension`.
    pub async fn promote_readme(&self, request: VectorizerValue) -> Result<VectorizerValue> {
        self.call("discovery.promote_readme", vec![request]).await
    }

    /// `discovery.compress_evidence` — compress a chunk set into ranked bullets.
    ///
    /// `request` must contain `chunks` (`Array<Map>`) and optionally
    /// `max_bullets` (`Int`), `max_per_doc` (`Int`).
    pub async fn compress_evidence(&self, request: VectorizerValue) -> Result<Vec<CompressBullet>> {
        let v = self
            .call("discovery.compress_evidence", vec![request])
            .await?;
        let arr = v
            .map_get("bullets")
            .and_then(|x| x.as_array())
            .ok_or_else(|| {
                RpcClientError::Server("discovery.compress_evidence: missing bullets".into())
            })?;
        Ok(arr
            .iter()
            .map(|entry| CompressBullet {
                text: entry
                    .map_get("text")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_owned(),
                source_id: entry
                    .map_get("source_id")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_owned(),
                score: entry
                    .map_get("score")
                    .and_then(|x| x.as_float())
                    .unwrap_or(0.0),
            })
            .collect())
    }

    /// `discovery.build_answer_plan` — organise bullets into a structured
    /// answer plan.
    ///
    /// `request` must contain `bullets` (`Array<Map>`), each with `text`,
    /// `source_id`, `score`, `category`.
    pub async fn build_answer_plan(&self, request: VectorizerValue) -> Result<AnswerPlanResult> {
        let v = self
            .call("discovery.build_answer_plan", vec![request])
            .await?;
        let sections = v
            .map_get("sections")
            .and_then(|x| x.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|entry| AnswerPlanSection {
                        title: entry
                            .map_get("title")
                            .and_then(|x| x.as_str())
                            .unwrap_or("")
                            .to_owned(),
                        bullets_count: entry
                            .map_get("bullets_count")
                            .and_then(|x| x.as_int())
                            .unwrap_or(0),
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(AnswerPlanResult {
            sections,
            total_bullets: v
                .map_get("total_bullets")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
        })
    }

    /// `discovery.render_llm_prompt` — render an answer plan into an LLM
    /// prompt string.
    ///
    /// `request` must contain `plan` (`Map`) with `sections` (`Array<Map>`)
    /// and optionally `total_bullets`, `sources`.
    pub async fn render_llm_prompt(&self, request: VectorizerValue) -> Result<RenderPromptResult> {
        let v = self
            .call("discovery.render_llm_prompt", vec![request])
            .await?;
        Ok(RenderPromptResult {
            prompt: need_str(&v, "discovery.render_llm_prompt", "prompt")?,
            length: v.map_get("length").and_then(|x| x.as_int()).unwrap_or(0),
            estimated_tokens: v
                .map_get("estimated_tokens")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
        })
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// File ops
// ═════════════════════════════════════════════════════════════════════════════

impl RpcClient {
    /// `file.content` — retrieve raw file content stored in a collection.
    ///
    /// `request` must contain `collection` (`Str`) and `file_path` (`Str`),
    /// and optionally `max_size_kb` (`Int`).
    pub async fn file_content(&self, request: VectorizerValue) -> Result<VectorizerValue> {
        self.call("file.content", vec![request]).await
    }

    /// `file.list` — list files indexed in a collection.
    ///
    /// `request` must contain `collection` (`Str`) and optionally
    /// `filter_by_type` (`Array<Str>`), `min_chunks` (`Int`),
    /// `max_results` (`Int`), `sort_by` (`Str`).
    pub async fn file_list(&self, request: VectorizerValue) -> Result<VectorizerValue> {
        self.call("file.list", vec![request]).await
    }

    /// `file.summary` — extractive or structural summary of one file.
    ///
    /// `request` must contain `collection` (`Str`), `file_path` (`Str`), and
    /// optionally `summary_type` (`"extractive"` | `"structural"` | `"both"`),
    /// `max_sentences` (`Int`).
    pub async fn file_summary(&self, request: VectorizerValue) -> Result<VectorizerValue> {
        self.call("file.summary", vec![request]).await
    }

    /// `file.chunks` — retrieve ordered chunks for one file.
    ///
    /// `request` must contain `collection` (`Str`), `file_path` (`Str`), and
    /// optionally `start_chunk` (`Int`), `limit` (`Int`),
    /// `include_context` (`Bool`).
    pub async fn file_chunks(&self, request: VectorizerValue) -> Result<VectorizerValue> {
        self.call("file.chunks", vec![request]).await
    }

    /// `file.outline` — directory-tree outline of a collection's files.
    ///
    /// `request` must contain `collection` (`Str`) and optionally
    /// `max_depth` (`Int`), `include_summaries` (`Bool`),
    /// `highlight_key_files` (`Bool`).
    pub async fn file_outline(&self, request: VectorizerValue) -> Result<VectorizerValue> {
        self.call("file.outline", vec![request]).await
    }

    /// `file.related` — find files semantically related to a given file.
    ///
    /// `request` must contain `collection` (`Str`), `file_path` (`Str`), and
    /// optionally `limit` (`Int`), `similarity_threshold` (`Float`),
    /// `include_reason` (`Bool`).
    pub async fn file_related(&self, request: VectorizerValue) -> Result<VectorizerValue> {
        self.call("file.related", vec![request]).await
    }

    /// `file.search_by_type` — search within files of specific extension types.
    ///
    /// `request` must contain `collection` (`Str`), `query` (`Str`),
    /// `file_types` (`Array<Str>`), and optionally `limit` (`Int`),
    /// `return_full_files` (`Bool`).
    pub async fn file_search_by_type(&self, request: VectorizerValue) -> Result<VectorizerValue> {
        self.call("file.search_by_type", vec![request]).await
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Graph
// ═════════════════════════════════════════════════════════════════════════════

impl RpcClient {
    /// `graph.list_nodes` — list all graph nodes in a collection.
    pub async fn graph_list_nodes(&self, collection: &str) -> Result<VectorizerValue> {
        self.call(
            "graph.list_nodes",
            vec![VectorizerValue::Str(collection.to_owned())],
        )
        .await
    }

    /// `graph.neighbors` — fetch direct neighbors of a graph node.
    pub async fn graph_neighbors(
        &self,
        collection: &str,
        node_id: &str,
    ) -> Result<VectorizerValue> {
        self.call(
            "graph.neighbors",
            vec![
                VectorizerValue::Str(collection.to_owned()),
                VectorizerValue::Str(node_id.to_owned()),
            ],
        )
        .await
    }

    /// `graph.find_related` — find nodes reachable within `max_hops` of a node.
    pub async fn graph_find_related(
        &self,
        collection: &str,
        node_id: &str,
        max_hops: i64,
    ) -> Result<VectorizerValue> {
        self.call(
            "graph.find_related",
            vec![
                VectorizerValue::Str(collection.to_owned()),
                VectorizerValue::Str(node_id.to_owned()),
                VectorizerValue::Int(max_hops),
            ],
        )
        .await
    }

    /// `graph.find_path` — shortest path between two graph nodes.
    pub async fn graph_find_path(
        &self,
        collection: &str,
        from: &str,
        to: &str,
    ) -> Result<VectorizerValue> {
        self.call(
            "graph.find_path",
            vec![
                VectorizerValue::Str(collection.to_owned()),
                VectorizerValue::Str(from.to_owned()),
                VectorizerValue::Str(to.to_owned()),
            ],
        )
        .await
    }

    /// `graph.create_edge` — create a directed edge between two nodes.
    ///
    /// `edge` is a `VectorizerValue::Map` with `source` (`Str`), `target`
    /// (`Str`), `relationship_type` (`Str`), and optionally `weight`
    /// (`Float`).
    pub async fn graph_create_edge(
        &self,
        collection: &str,
        edge: VectorizerValue,
    ) -> Result<VectorizerValue> {
        self.call(
            "graph.create_edge",
            vec![VectorizerValue::Str(collection.to_owned()), edge],
        )
        .await
    }

    /// `graph.delete_edge` — remove an edge by its id.
    pub async fn graph_delete_edge(
        &self,
        collection: &str,
        edge_id: &str,
    ) -> Result<VectorizerValue> {
        self.call(
            "graph.delete_edge",
            vec![
                VectorizerValue::Str(collection.to_owned()),
                VectorizerValue::Str(edge_id.to_owned()),
            ],
        )
        .await
    }

    /// `graph.list_edges` — list all edges in a collection's graph.
    pub async fn graph_list_edges(&self, collection: &str) -> Result<VectorizerValue> {
        self.call(
            "graph.list_edges",
            vec![VectorizerValue::Str(collection.to_owned())],
        )
        .await
    }

    /// `graph.discover_edges` — auto-discover edges by vector similarity
    /// across the whole collection.
    ///
    /// `request` is an optional `VectorizerValue::Map` with
    /// `similarity_threshold` (`Float`) and `max_per_node` (`Int`).
    pub async fn graph_discover_edges(
        &self,
        collection: &str,
        request: VectorizerValue,
    ) -> Result<DiscoverEdgesResult> {
        let v = self
            .call(
                "graph.discover_edges",
                vec![VectorizerValue::Str(collection.to_owned()), request],
            )
            .await?;
        Ok(DiscoverEdgesResult {
            success: v
                .map_get("success")
                .and_then(|x| x.as_bool())
                .unwrap_or(false),
            total_nodes: v
                .map_get("total_nodes")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
            nodes_processed: v
                .map_get("nodes_processed")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
            nodes_with_edges: v
                .map_get("nodes_with_edges")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
            total_edges_created: v
                .map_get("total_edges_created")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
        })
    }

    /// `graph.discover_edges_for_node` — auto-discover edges for one node.
    ///
    /// `request` is an optional `VectorizerValue::Map` with
    /// `similarity_threshold` (`Float`) and `max_per_node` (`Int`).
    pub async fn graph_discover_edges_for_node(
        &self,
        collection: &str,
        node_id: &str,
        request: VectorizerValue,
    ) -> Result<DiscoverEdgesForNodeResult> {
        let v = self
            .call(
                "graph.discover_edges_for_node",
                vec![
                    VectorizerValue::Str(collection.to_owned()),
                    VectorizerValue::Str(node_id.to_owned()),
                    request,
                ],
            )
            .await?;
        Ok(DiscoverEdgesForNodeResult {
            success: v
                .map_get("success")
                .and_then(|x| x.as_bool())
                .unwrap_or(false),
            node_id: v
                .map_get("node_id")
                .and_then(|x| x.as_str())
                .unwrap_or(node_id)
                .to_owned(),
            edges_created: v
                .map_get("edges_created")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
        })
    }

    /// `graph.discovery_status` — percentage of nodes that have edges.
    pub async fn graph_discovery_status(&self, collection: &str) -> Result<GraphDiscoveryStatus> {
        let v = self
            .call(
                "graph.discovery_status",
                vec![VectorizerValue::Str(collection.to_owned())],
            )
            .await?;
        Ok(GraphDiscoveryStatus {
            total_nodes: v
                .map_get("total_nodes")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
            nodes_with_edges: v
                .map_get("nodes_with_edges")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
            total_edges: v
                .map_get("total_edges")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
            progress_percentage: v
                .map_get("progress_percentage")
                .and_then(|x| x.as_float())
                .unwrap_or(0.0),
        })
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Admin
// ═════════════════════════════════════════════════════════════════════════════

impl RpcClient {
    /// `admin.stats` — aggregate vector/collection counts.
    pub async fn admin_stats(&self) -> Result<AdminStats> {
        let v = self.call("admin.stats", vec![]).await?;
        Ok(AdminStats {
            collections_count: v
                .map_get("collections_count")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
            total_vectors: v
                .map_get("total_vectors")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
            version: v
                .map_get("version")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_owned(),
        })
    }

    /// `admin.status` — readiness probe and basic counts.
    pub async fn admin_status(&self) -> Result<AdminStatus> {
        let v = self.call("admin.status", vec![]).await?;
        Ok(AdminStatus {
            ready: v
                .map_get("ready")
                .and_then(|x| x.as_bool())
                .unwrap_or(false),
            collections_count: v
                .map_get("collections_count")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
            version: v
                .map_get("version")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_owned(),
        })
    }

    /// `admin.logs` — in-process log entries (the server currently returns
    /// empty; use the REST `/logs` endpoint for live streaming).
    pub async fn admin_logs(&self, request: Option<VectorizerValue>) -> Result<VectorizerValue> {
        let args = request.map(|r| vec![r]).unwrap_or_default();
        self.call("admin.logs", args).await
    }

    /// `admin.indexing_progress` — how many collections have been indexed.
    pub async fn admin_indexing_progress(&self) -> Result<VectorizerValue> {
        self.call("admin.indexing_progress", vec![]).await
    }

    /// `admin.config_get` — read the server's `config.yml`.
    pub async fn admin_config_get(&self) -> Result<VectorizerValue> {
        self.call("admin.config_get", vec![]).await
    }

    /// `admin.config_update` — write a patch map to `config.yml` (admin).
    ///
    /// `patch` is a `VectorizerValue::Map` of config keys to new values.
    pub async fn admin_config_update(&self, patch: VectorizerValue) -> Result<bool> {
        let v = self.call("admin.config_update", vec![patch]).await?;
        need_bool(&v, "admin.config_update", "success")
    }

    /// `admin.backups_list` — list available backup files.
    pub async fn admin_backups_list(&self) -> Result<VectorizerValue> {
        self.call("admin.backups_list", vec![]).await
    }

    /// `admin.backups_create` — create a backup (admin).
    ///
    /// `request` must contain `name` (`Str`) and optionally `collections`
    /// (`Array<Str>`).
    pub async fn admin_backups_create(&self, request: VectorizerValue) -> Result<String> {
        let v = self.call("admin.backups_create", vec![request]).await?;
        need_str(&v, "admin.backups_create", "backup_id")
    }

    /// `admin.backups_restore` — restore a backup by id (admin).
    ///
    /// `request` must contain `backup_id` (`Str`).
    pub async fn admin_backups_restore(&self, request: VectorizerValue) -> Result<bool> {
        let v = self.call("admin.backups_restore", vec![request]).await?;
        need_bool(&v, "admin.backups_restore", "success")
    }

    /// `admin.workspaces_list` — list configured workspaces.
    pub async fn admin_workspaces_list(&self) -> Result<VectorizerValue> {
        self.call("admin.workspaces_list", vec![]).await
    }

    /// `admin.workspace_get` — read `workspace.yml`.
    pub async fn admin_workspace_get(&self) -> Result<VectorizerValue> {
        self.call("admin.workspace_get", vec![]).await
    }

    /// `admin.workspace_add` — register a new workspace directory (admin).
    ///
    /// `request` must contain `path` (`Str`) and `collection_name` (`Str`).
    pub async fn admin_workspace_add(&self, request: VectorizerValue) -> Result<VectorizerValue> {
        self.call("admin.workspace_add", vec![request]).await
    }

    /// `admin.workspace_remove` — remove a workspace by name (admin).
    pub async fn admin_workspace_remove(&self, name: &str) -> Result<bool> {
        let v = self
            .call(
                "admin.workspace_remove",
                vec![VectorizerValue::Str(name.to_owned())],
            )
            .await?;
        need_bool(&v, "admin.workspace_remove", "success")
    }

    /// `admin.restart` — schedule a server restart (admin).
    pub async fn admin_restart(&self) -> Result<bool> {
        let v = self.call("admin.restart", vec![]).await?;
        need_bool(&v, "admin.restart", "success")
    }

    /// `admin.slow_queries_list` — retrieve the slow-query ring buffer.
    pub async fn admin_slow_queries_list(&self) -> Result<VectorizerValue> {
        self.call("admin.slow_queries_list", vec![]).await
    }

    /// `admin.slow_queries_config` — configure slow-query threshold and
    /// ring-buffer capacity.
    ///
    /// `config` must contain `threshold_ms` (`Int`) and optionally
    /// `capacity` (`Int`).
    pub async fn admin_slow_queries_config(
        &self,
        config: VectorizerValue,
    ) -> Result<SlowQueryConfigResult> {
        let v = self.call("admin.slow_queries_config", vec![config]).await?;
        Ok(SlowQueryConfigResult {
            threshold_ms: v
                .map_get("threshold_ms")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
            capacity: v.map_get("capacity").and_then(|x| x.as_int()).unwrap_or(0),
            status: v
                .map_get("status")
                .and_then(|x| x.as_str())
                .unwrap_or("ok")
                .to_owned(),
        })
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Auth / RBAC
// ═════════════════════════════════════════════════════════════════════════════

impl RpcClient {
    /// `auth.me` — return the authenticated principal's identity.
    pub async fn auth_me(&self) -> Result<AuthMeResult> {
        let v = self.call("auth.me", vec![]).await?;
        Ok(AuthMeResult {
            username: v
                .map_get("username")
                .and_then(|x| x.as_str())
                .unwrap_or("unknown")
                .to_owned(),
            authenticated: v
                .map_get("authenticated")
                .and_then(|x| x.as_bool())
                .unwrap_or(false),
        })
    }

    /// `auth.logout` — blacklist the supplied JWT so it cannot be reused.
    pub async fn auth_logout(&self, token: &str) -> Result<VectorizerValue> {
        self.call("auth.logout", vec![VectorizerValue::Str(token.to_owned())])
            .await
    }

    /// `auth.refresh_token` — exchange a valid JWT for a fresh one.
    pub async fn auth_refresh_token(&self, token: &str) -> Result<RefreshTokenResult> {
        let v = self
            .call(
                "auth.refresh_token",
                vec![VectorizerValue::Str(token.to_owned())],
            )
            .await?;
        Ok(RefreshTokenResult {
            access_token: need_str(&v, "auth.refresh_token", "access_token")?,
            token_type: v
                .map_get("token_type")
                .and_then(|x| x.as_str())
                .unwrap_or("Bearer")
                .to_owned(),
        })
    }

    /// `auth.validate_password` — check a plaintext password against the
    /// server's password policy.
    pub async fn auth_validate_password(&self, password: &str) -> Result<ValidatePasswordResult> {
        let v = self
            .call(
                "auth.validate_password",
                vec![VectorizerValue::Str(password.to_owned())],
            )
            .await?;
        let errors = v
            .map_get("errors")
            .and_then(|x| x.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default();
        Ok(ValidatePasswordResult {
            valid: v
                .map_get("valid")
                .and_then(|x| x.as_bool())
                .unwrap_or(false),
            errors,
        })
    }

    /// `auth.api_keys_create` — create a new API key.
    ///
    /// `request` must contain `name` (`Str`) and optionally `expires_in`
    /// (`Int`, seconds) and `permissions` (`Array<Str>`).
    pub async fn auth_api_keys_create(&self, request: VectorizerValue) -> Result<ApiKeyCreated> {
        let v = self.call("auth.api_keys_create", vec![request]).await?;
        Ok(ApiKeyCreated {
            api_key: need_str(&v, "auth.api_keys_create", "api_key")?,
            id: need_str(&v, "auth.api_keys_create", "id")?,
            name: need_str(&v, "auth.api_keys_create", "name")?,
        })
    }

    /// `auth.api_keys_list` — list API keys for the current principal.
    pub async fn auth_api_keys_list(&self) -> Result<VectorizerValue> {
        self.call("auth.api_keys_list", vec![]).await
    }

    /// `auth.api_keys_revoke` — permanently revoke an API key by id.
    pub async fn auth_api_keys_revoke(&self, key_id: &str) -> Result<bool> {
        let v = self
            .call(
                "auth.api_keys_revoke",
                vec![VectorizerValue::Str(key_id.to_owned())],
            )
            .await?;
        need_bool(&v, "auth.api_keys_revoke", "success")
    }

    /// `auth.api_keys_rotate` — rotate an API key (5-minute grace period).
    ///
    /// Named `rotate_api_key_rpc` to avoid collision with the REST SDK's
    /// `rotate_api_key`.
    pub async fn rotate_api_key_rpc(&self, key_id: &str) -> Result<RotatedApiKey> {
        let v = self
            .call(
                "auth.api_keys_rotate",
                vec![VectorizerValue::Str(key_id.to_owned())],
            )
            .await?;
        Ok(RotatedApiKey {
            old_key_id: need_str(&v, "auth.api_keys_rotate", "old_key_id")?,
            new_key_id: need_str(&v, "auth.api_keys_rotate", "new_key_id")?,
            new_token: need_str(&v, "auth.api_keys_rotate", "new_token")?,
            grace_until: v
                .map_get("grace_until")
                .and_then(|x| x.as_str())
                .map(str::to_owned),
        })
    }

    /// `auth.api_keys_create_scoped` — create a collection-scoped API key.
    ///
    /// `request` must contain `name` (`Str`) and optionally `expires_in`
    /// (`Int`), `permissions` (`Array<Str>`), `scopes` (`Array<Map>`).
    pub async fn auth_api_keys_create_scoped(
        &self,
        request: VectorizerValue,
    ) -> Result<ApiKeyCreated> {
        let v = self
            .call("auth.api_keys_create_scoped", vec![request])
            .await?;
        Ok(ApiKeyCreated {
            api_key: need_str(&v, "auth.api_keys_create_scoped", "api_key")?,
            id: need_str(&v, "auth.api_keys_create_scoped", "id")?,
            name: need_str(&v, "auth.api_keys_create_scoped", "name")?,
        })
    }

    /// `auth.users_create` — create a user (admin; returns server error in
    /// v1 — RpcState does not carry AuthHandlerState; use the REST endpoint).
    pub async fn auth_users_create(&self, request: VectorizerValue) -> Result<VectorizerValue> {
        self.call("auth.users_create", vec![request]).await
    }

    /// `auth.users_list` — list users (admin; returns server error in v1).
    pub async fn auth_users_list(&self) -> Result<VectorizerValue> {
        self.call("auth.users_list", vec![]).await
    }

    /// `auth.users_delete` — delete a user (admin; returns server error in v1).
    pub async fn auth_users_delete(&self, request: VectorizerValue) -> Result<VectorizerValue> {
        self.call("auth.users_delete", vec![request]).await
    }

    /// `auth.users_change_password` — change a user's password (returns
    /// server error in v1 — use REST).
    pub async fn auth_users_change_password(
        &self,
        request: VectorizerValue,
    ) -> Result<VectorizerValue> {
        self.call("auth.users_change_password", vec![request]).await
    }

    /// `auth.introspect` — inspect a token's claims and blacklist status.
    pub async fn auth_introspect(&self, token: &str) -> Result<VectorizerValue> {
        self.call(
            "auth.introspect",
            vec![VectorizerValue::Str(token.to_owned())],
        )
        .await
    }

    /// `auth.audit` — query the auth audit log.
    ///
    /// `request` is an optional `VectorizerValue::Map` with `from` (`Str`),
    /// `to` (`Str`), `actor` (`Str`), `action` (`Str`), `limit` (`Int`).
    pub async fn auth_audit(&self, request: VectorizerValue) -> Result<VectorizerValue> {
        self.call("auth.audit", vec![request]).await
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Replication
// ═════════════════════════════════════════════════════════════════════════════

impl RpcClient {
    /// `replication.status` — current replication role and replica list.
    pub async fn replication_status(&self) -> Result<VectorizerValue> {
        self.call("replication.status", vec![]).await
    }

    /// `replication.configure` — set the replication role for this node.
    ///
    /// `config` must contain `role` (`Str`: `"master"` | `"replica"` |
    /// `"standalone"`) and optionally `bind_address`, `master_address`
    /// (`Str`). A server restart is required for the change to take effect.
    pub async fn replication_configure(
        &self,
        config: VectorizerValue,
    ) -> Result<ReplicationConfigureResult> {
        let v = self.call("replication.configure", vec![config]).await?;
        Ok(ReplicationConfigureResult {
            success: need_bool(&v, "replication.configure", "success")?,
            role: need_str(&v, "replication.configure", "role")?,
            message: v
                .map_get("message")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_owned(),
        })
    }

    /// `replication.stats` — replication throughput and lag statistics.
    pub async fn replication_stats(&self) -> Result<VectorizerValue> {
        self.call("replication.stats", vec![]).await
    }

    /// `replication.replicas_list` — list connected replicas (master only).
    pub async fn replication_replicas_list(&self) -> Result<VectorizerValue> {
        self.call("replication.replicas_list", vec![]).await
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Cluster
// ═════════════════════════════════════════════════════════════════════════════

impl RpcClient {
    /// `cluster.failover` — promote a replica to master (admin).
    pub async fn cluster_failover(&self, replica_id: &str) -> Result<VectorizerValue> {
        self.call(
            "cluster.failover",
            vec![VectorizerValue::Str(replica_id.to_owned())],
        )
        .await
    }

    /// `cluster.replica_resync` — force a replica to resync from master (admin).
    pub async fn cluster_replica_resync(&self, replica_id: &str) -> Result<VectorizerValue> {
        self.call(
            "cluster.replica_resync",
            vec![VectorizerValue::Str(replica_id.to_owned())],
        )
        .await
    }

    /// `cluster.peer_add` — add a new peer to the cluster (admin).
    ///
    /// `request` must contain `address` (`Str`) and optionally `role`
    /// (`Str`: `"member"` | `"observer"`).
    pub async fn cluster_peer_add(&self, request: VectorizerValue) -> Result<VectorizerValue> {
        self.call("cluster.peer_add", vec![request]).await
    }

    /// `cluster.rebalance` — trigger a shard rebalance across peers (admin).
    pub async fn cluster_rebalance(&self) -> Result<VectorizerValue> {
        self.call("cluster.rebalance", vec![]).await
    }

    /// `cluster.rebalance_status` — check the status of an in-progress
    /// rebalance (or confirm idle).
    pub async fn cluster_rebalance_status(&self) -> Result<RebalanceStatus> {
        let v = self.call("cluster.rebalance_status", vec![]).await?;
        Ok(RebalanceStatus {
            status: v
                .map_get("status")
                .and_then(|x| x.as_str())
                .map(str::to_owned),
            message: v
                .map_get("message")
                .and_then(|x| x.as_str())
                .map(str::to_owned),
        })
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Tests
// ═════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Collections ──────────────────────────────────────────────────────────

    #[test]
    fn collection_info_fields_present() {
        let map = VectorizerValue::Map(vec![
            (
                VectorizerValue::Str("name".into()),
                VectorizerValue::Str("test".into()),
            ),
            (
                VectorizerValue::Str("vector_count".into()),
                VectorizerValue::Int(42),
            ),
            (
                VectorizerValue::Str("document_count".into()),
                VectorizerValue::Int(10),
            ),
            (
                VectorizerValue::Str("dimension".into()),
                VectorizerValue::Int(512),
            ),
            (
                VectorizerValue::Str("metric".into()),
                VectorizerValue::Str("Cosine".into()),
            ),
            (
                VectorizerValue::Str("created_at".into()),
                VectorizerValue::Str("2024-01-01T00:00:00Z".into()),
            ),
            (
                VectorizerValue::Str("updated_at".into()),
                VectorizerValue::Str("2024-01-02T00:00:00Z".into()),
            ),
        ]);
        let info = CollectionInfo {
            name: need_str(&map, "test", "name").unwrap(),
            vector_count: need_int(&map, "test", "vector_count").unwrap(),
            document_count: need_int(&map, "test", "document_count").unwrap(),
            dimension: need_int(&map, "test", "dimension").unwrap(),
            metric: need_str(&map, "test", "metric").unwrap(),
            created_at: need_str(&map, "test", "created_at").unwrap(),
            updated_at: need_str(&map, "test", "updated_at").unwrap(),
        };
        assert_eq!(info.name, "test");
        assert_eq!(info.vector_count, 42);
        assert_eq!(info.dimension, 512);
    }

    #[test]
    fn create_collection_result_decodes() {
        let map = VectorizerValue::Map(vec![
            (
                VectorizerValue::Str("name".into()),
                VectorizerValue::Str("myc".into()),
            ),
            (
                VectorizerValue::Str("dimension".into()),
                VectorizerValue::Int(128),
            ),
            (
                VectorizerValue::Str("metric".into()),
                VectorizerValue::Str("cosine".into()),
            ),
            (
                VectorizerValue::Str("success".into()),
                VectorizerValue::Bool(true),
            ),
        ]);
        let r = CreateCollectionResult {
            name: need_str(&map, "c", "name").unwrap(),
            dimension: need_int(&map, "c", "dimension").unwrap(),
            metric: need_str(&map, "c", "metric").unwrap(),
            success: need_bool(&map, "c", "success").unwrap(),
        };
        assert!(r.success);
        assert_eq!(r.dimension, 128);
    }

    #[test]
    fn cleanup_empty_result_decodes() {
        let map = VectorizerValue::Map(vec![
            (
                VectorizerValue::Str("removed".into()),
                VectorizerValue::Int(3),
            ),
            (
                VectorizerValue::Str("dry_run".into()),
                VectorizerValue::Bool(false),
            ),
        ]);
        let r = CleanupEmptyResult {
            removed: need_int(&map, "c", "removed").unwrap(),
            dry_run: need_bool(&map, "c", "dry_run").unwrap(),
        };
        assert_eq!(r.removed, 3);
        assert!(!r.dry_run);
    }

    // ── Vectors ──────────────────────────────────────────────────────────────

    #[test]
    fn vector_write_result_decodes() {
        let map = VectorizerValue::Map(vec![
            (
                VectorizerValue::Str("id".into()),
                VectorizerValue::Str("abc-123".into()),
            ),
            (
                VectorizerValue::Str("success".into()),
                VectorizerValue::Bool(true),
            ),
        ]);
        let r = VectorWriteResult {
            id: need_str(&map, "v", "id").unwrap(),
            success: need_bool(&map, "v", "success").unwrap(),
        };
        assert_eq!(r.id, "abc-123");
        assert!(r.success);
    }

    #[test]
    fn batch_insert_result_decodes() {
        let item = VectorizerValue::Map(vec![
            (
                VectorizerValue::Str("index".into()),
                VectorizerValue::Int(0),
            ),
            (
                VectorizerValue::Str("id".into()),
                VectorizerValue::Str("x".into()),
            ),
            (
                VectorizerValue::Str("status".into()),
                VectorizerValue::Str("ok".into()),
            ),
        ]);
        let items = &[item];
        let results = decode_batch_items(items);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, "ok");
        assert_eq!(results[0].id.as_deref(), Some("x"));
    }

    #[test]
    fn batch_search_result_decodes() {
        let hit = VectorizerValue::Map(vec![
            (
                VectorizerValue::Str("id".into()),
                VectorizerValue::Str("v1".into()),
            ),
            (
                VectorizerValue::Str("score".into()),
                VectorizerValue::Float(0.95),
            ),
        ]);
        let hits = &[hit];
        let decoded = decode_search_hits(hits);
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].id, "v1");
        assert!((decoded[0].score - 0.95).abs() < 1e-6);
    }

    #[test]
    fn move_rpc_result_decodes() {
        let map = VectorizerValue::Map(vec![
            (
                VectorizerValue::Str("src".into()),
                VectorizerValue::Str("col_a".into()),
            ),
            (
                VectorizerValue::Str("dst".into()),
                VectorizerValue::Str("col_b".into()),
            ),
            (
                VectorizerValue::Str("moved".into()),
                VectorizerValue::Int(5),
            ),
            (
                VectorizerValue::Str("failed".into()),
                VectorizerValue::Int(1),
            ),
        ]);
        let r = MoveRpcResult {
            src: need_str(&map, "m", "src").unwrap(),
            dst: need_str(&map, "m", "dst").unwrap(),
            moved: map.map_get("moved").and_then(|x| x.as_int()).unwrap_or(0),
            failed: map.map_get("failed").and_then(|x| x.as_int()).unwrap_or(0),
        };
        assert_eq!(r.src, "col_a");
        assert_eq!(r.moved, 5);
    }

    #[test]
    fn set_expiry_result_decodes() {
        let map = VectorizerValue::Map(vec![
            (
                VectorizerValue::Str("id".into()),
                VectorizerValue::Str("v99".into()),
            ),
            (
                VectorizerValue::Str("expires_at".into()),
                VectorizerValue::Int(9_999_999),
            ),
            (
                VectorizerValue::Str("success".into()),
                VectorizerValue::Bool(true),
            ),
        ]);
        let r = SetExpiryResult {
            id: need_str(&map, "se", "id").unwrap(),
            expires_at: need_int(&map, "se", "expires_at").unwrap(),
            success: need_bool(&map, "se", "success").unwrap(),
        };
        assert_eq!(r.expires_at, 9_999_999);
    }

    // ── Search ───────────────────────────────────────────────────────────────

    #[test]
    fn search_explain_trace_decodes() {
        let trace_val = VectorizerValue::Map(vec![
            (
                VectorizerValue::Str("visited_nodes".into()),
                VectorizerValue::Int(50),
            ),
            (
                VectorizerValue::Str("ef_search".into()),
                VectorizerValue::Int(100),
            ),
            (
                VectorizerValue::Str("hnsw_search_ms".into()),
                VectorizerValue::Float(1.5),
            ),
            (
                VectorizerValue::Str("total_ms".into()),
                VectorizerValue::Float(2.0),
            ),
        ]);
        let trace = SearchTrace {
            visited_nodes: trace_val
                .map_get("visited_nodes")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
            ef_search: trace_val
                .map_get("ef_search")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
            hnsw_search_ms: trace_val
                .map_get("hnsw_search_ms")
                .and_then(|x| x.as_float())
                .unwrap_or(0.0),
            total_ms: trace_val
                .map_get("total_ms")
                .and_then(|x| x.as_float())
                .unwrap_or(0.0),
        };
        assert_eq!(trace.visited_nodes, 50);
        assert!((trace.hnsw_search_ms - 1.5).abs() < 1e-6);
    }

    // ── Discovery ────────────────────────────────────────────────────────────

    #[test]
    fn discover_result_decodes() {
        let map = VectorizerValue::Map(vec![
            (
                VectorizerValue::Str("answer_prompt".into()),
                VectorizerValue::Str("Here is ...".into()),
            ),
            (
                VectorizerValue::Str("sections".into()),
                VectorizerValue::Int(3),
            ),
            (
                VectorizerValue::Str("bullets".into()),
                VectorizerValue::Int(12),
            ),
            (
                VectorizerValue::Str("chunks".into()),
                VectorizerValue::Int(8),
            ),
        ]);
        let r = DiscoverResult {
            answer_prompt: need_str(&map, "d", "answer_prompt").unwrap(),
            sections: map
                .map_get("sections")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
            bullets: map.map_get("bullets").and_then(|x| x.as_int()).unwrap_or(0),
            chunks: map.map_get("chunks").and_then(|x| x.as_int()).unwrap_or(0),
        };
        assert_eq!(r.bullets, 12);
    }

    #[test]
    fn expand_queries_result_decodes() {
        let map = VectorizerValue::Map(vec![
            (
                VectorizerValue::Str("original_query".into()),
                VectorizerValue::Str("rust".into()),
            ),
            (
                VectorizerValue::Str("expanded_queries".into()),
                VectorizerValue::Array(vec![
                    VectorizerValue::Str("rust programming".into()),
                    VectorizerValue::Str("rust language".into()),
                ]),
            ),
            (
                VectorizerValue::Str("count".into()),
                VectorizerValue::Int(2),
            ),
        ]);
        let expanded: Vec<String> = map
            .map_get("expanded_queries")
            .and_then(|x| x.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default();
        assert_eq!(expanded.len(), 2);
    }

    // ── Graph ────────────────────────────────────────────────────────────────

    #[test]
    fn graph_discovery_status_decodes() {
        let map = VectorizerValue::Map(vec![
            (
                VectorizerValue::Str("total_nodes".into()),
                VectorizerValue::Int(100),
            ),
            (
                VectorizerValue::Str("nodes_with_edges".into()),
                VectorizerValue::Int(75),
            ),
            (
                VectorizerValue::Str("total_edges".into()),
                VectorizerValue::Int(200),
            ),
            (
                VectorizerValue::Str("progress_percentage".into()),
                VectorizerValue::Float(75.0),
            ),
        ]);
        let r = GraphDiscoveryStatus {
            total_nodes: map
                .map_get("total_nodes")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
            nodes_with_edges: map
                .map_get("nodes_with_edges")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
            total_edges: map
                .map_get("total_edges")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
            progress_percentage: map
                .map_get("progress_percentage")
                .and_then(|x| x.as_float())
                .unwrap_or(0.0),
        };
        assert_eq!(r.total_nodes, 100);
        assert!((r.progress_percentage - 75.0).abs() < 1e-6);
    }

    #[test]
    fn discover_edges_result_decodes() {
        let map = VectorizerValue::Map(vec![
            (
                VectorizerValue::Str("success".into()),
                VectorizerValue::Bool(true),
            ),
            (
                VectorizerValue::Str("total_nodes".into()),
                VectorizerValue::Int(50),
            ),
            (
                VectorizerValue::Str("nodes_processed".into()),
                VectorizerValue::Int(50),
            ),
            (
                VectorizerValue::Str("nodes_with_edges".into()),
                VectorizerValue::Int(40),
            ),
            (
                VectorizerValue::Str("total_edges_created".into()),
                VectorizerValue::Int(120),
            ),
        ]);
        let r = DiscoverEdgesResult {
            success: map
                .map_get("success")
                .and_then(|x| x.as_bool())
                .unwrap_or(false),
            total_nodes: map
                .map_get("total_nodes")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
            nodes_processed: map
                .map_get("nodes_processed")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
            nodes_with_edges: map
                .map_get("nodes_with_edges")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
            total_edges_created: map
                .map_get("total_edges_created")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
        };
        assert!(r.success);
        assert_eq!(r.total_edges_created, 120);
    }

    // ── Admin ────────────────────────────────────────────────────────────────

    #[test]
    fn admin_stats_decodes() {
        let map = VectorizerValue::Map(vec![
            (
                VectorizerValue::Str("collections_count".into()),
                VectorizerValue::Int(5),
            ),
            (
                VectorizerValue::Str("total_vectors".into()),
                VectorizerValue::Int(1000),
            ),
            (
                VectorizerValue::Str("version".into()),
                VectorizerValue::Str("3.8.0".into()),
            ),
        ]);
        let r = AdminStats {
            collections_count: map
                .map_get("collections_count")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
            total_vectors: map
                .map_get("total_vectors")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
            version: map
                .map_get("version")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_owned(),
        };
        assert_eq!(r.collections_count, 5);
        assert_eq!(r.version, "3.8.0");
    }

    #[test]
    fn slow_query_config_decodes() {
        let map = VectorizerValue::Map(vec![
            (
                VectorizerValue::Str("threshold_ms".into()),
                VectorizerValue::Int(100),
            ),
            (
                VectorizerValue::Str("capacity".into()),
                VectorizerValue::Int(500),
            ),
            (
                VectorizerValue::Str("status".into()),
                VectorizerValue::Str("ok".into()),
            ),
        ]);
        let r = SlowQueryConfigResult {
            threshold_ms: map
                .map_get("threshold_ms")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
            capacity: map
                .map_get("capacity")
                .and_then(|x| x.as_int())
                .unwrap_or(0),
            status: map
                .map_get("status")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_owned(),
        };
        assert_eq!(r.threshold_ms, 100);
        assert_eq!(r.status, "ok");
    }

    // ── Auth ─────────────────────────────────────────────────────────────────

    #[test]
    fn api_key_created_decodes() {
        let map = VectorizerValue::Map(vec![
            (
                VectorizerValue::Str("api_key".into()),
                VectorizerValue::Str("vz_abc123".into()),
            ),
            (
                VectorizerValue::Str("id".into()),
                VectorizerValue::Str("key-id-1".into()),
            ),
            (
                VectorizerValue::Str("name".into()),
                VectorizerValue::Str("ci-key".into()),
            ),
        ]);
        let r = ApiKeyCreated {
            api_key: need_str(&map, "a", "api_key").unwrap(),
            id: need_str(&map, "a", "id").unwrap(),
            name: need_str(&map, "a", "name").unwrap(),
        };
        assert_eq!(r.api_key, "vz_abc123");
        assert_eq!(r.name, "ci-key");
    }

    #[test]
    fn rotated_api_key_decodes() {
        let map = VectorizerValue::Map(vec![
            (
                VectorizerValue::Str("old_key_id".into()),
                VectorizerValue::Str("old".into()),
            ),
            (
                VectorizerValue::Str("new_key_id".into()),
                VectorizerValue::Str("new".into()),
            ),
            (
                VectorizerValue::Str("new_token".into()),
                VectorizerValue::Str("vz_new_xxx".into()),
            ),
        ]);
        let r = RotatedApiKey {
            old_key_id: need_str(&map, "r", "old_key_id").unwrap(),
            new_key_id: need_str(&map, "r", "new_key_id").unwrap(),
            new_token: need_str(&map, "r", "new_token").unwrap(),
            grace_until: map
                .map_get("grace_until")
                .and_then(|x| x.as_str())
                .map(str::to_owned),
        };
        assert_eq!(r.old_key_id, "old");
        assert_eq!(r.new_token, "vz_new_xxx");
        assert!(r.grace_until.is_none());
    }

    #[test]
    fn validate_password_result_decodes() {
        let map = VectorizerValue::Map(vec![
            (
                VectorizerValue::Str("valid".into()),
                VectorizerValue::Bool(false),
            ),
            (
                VectorizerValue::Str("errors".into()),
                VectorizerValue::Array(vec![VectorizerValue::Str("too short".into())]),
            ),
        ]);
        let errors: Vec<String> = map
            .map_get("errors")
            .and_then(|x| x.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0], "too short");
    }

    // ── Replication / Cluster ─────────────────────────────────────────────────

    #[test]
    fn replication_configure_result_decodes() {
        let map = VectorizerValue::Map(vec![
            (
                VectorizerValue::Str("success".into()),
                VectorizerValue::Bool(true),
            ),
            (
                VectorizerValue::Str("role".into()),
                VectorizerValue::Str("master".into()),
            ),
            (
                VectorizerValue::Str("message".into()),
                VectorizerValue::Str("restart required".into()),
            ),
        ]);
        let r = ReplicationConfigureResult {
            success: need_bool(&map, "rc", "success").unwrap(),
            role: need_str(&map, "rc", "role").unwrap(),
            message: map
                .map_get("message")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_owned(),
        };
        assert!(r.success);
        assert_eq!(r.role, "master");
    }

    #[test]
    fn rebalance_status_idle_decodes() {
        let map = VectorizerValue::Map(vec![
            (
                VectorizerValue::Str("status".into()),
                VectorizerValue::Str("idle".into()),
            ),
            (
                VectorizerValue::Str("message".into()),
                VectorizerValue::Str("No rebalance".into()),
            ),
        ]);
        let r = RebalanceStatus {
            status: map
                .map_get("status")
                .and_then(|x| x.as_str())
                .map(str::to_owned),
            message: map
                .map_get("message")
                .and_then(|x| x.as_str())
                .map(str::to_owned),
        };
        assert_eq!(r.status.as_deref(), Some("idle"));
    }

    // ── Need-* helpers error on missing fields ────────────────────────────────

    #[test]
    fn need_str_errors_on_missing() {
        let map = VectorizerValue::Map(vec![]);
        assert!(need_str(&map, "cmd", "missing_field").is_err());
    }

    #[test]
    fn need_int_errors_on_missing() {
        let map = VectorizerValue::Map(vec![]);
        assert!(need_int(&map, "cmd", "missing_field").is_err());
    }

    #[test]
    fn decode_string_array_errors_on_non_array() {
        let v = VectorizerValue::Str("not_an_array".into());
        assert!(decode_string_array(v, "cmd").is_err());
    }
}
