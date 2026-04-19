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

/// One result from `search.basic`.
#[derive(Debug, Clone)]
pub struct SearchHit {
    /// Vector ID inside the collection.
    pub id: String,
    /// Similarity score in `[0.0, 1.0]` for cosine; backend-defined
    /// otherwise.
    pub score: f64,
    /// Optional payload as a JSON string. The server stores payloads
    /// as `serde_json::Value`; the RPC layer ships them as a string
    /// because the wire `VectorizerValue` enum doesn't model JSON
    /// directly. Decode with `serde_json::from_str` if you need
    /// structured access.
    pub payload: Option<String>,
}

impl RpcClient {
    /// `collections.list` — return every collection name visible to
    /// the authenticated principal.
    pub async fn list_collections(&self) -> Result<Vec<String>> {
        let v = self.call("collections.list", vec![]).await?;
        let arr = v
            .as_array()
            .ok_or_else(|| RpcClientError::Server("collections.list: expected Array".into()))?;
        Ok(arr
            .iter()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect())
    }

    /// `collections.get_info` — return metadata for one collection.
    pub async fn get_collection_info(&self, name: &str) -> Result<CollectionInfo> {
        let v = self
            .call(
                "collections.get_info",
                vec![VectorizerValue::Str(name.to_owned())],
            )
            .await?;
        let need_str = |key: &str| -> Result<String> {
            v.map_get(key)
                .and_then(|x| x.as_str().map(str::to_owned))
                .ok_or_else(|| {
                    RpcClientError::Server(format!(
                        "collections.get_info: missing string field '{key}'"
                    ))
                })
        };
        let need_int = |key: &str| -> Result<i64> {
            v.map_get(key).and_then(|x| x.as_int()).ok_or_else(|| {
                RpcClientError::Server(format!("collections.get_info: missing int field '{key}'"))
            })
        };

        Ok(CollectionInfo {
            name: need_str("name")?,
            vector_count: need_int("vector_count")?,
            document_count: need_int("document_count")?,
            dimension: need_int("dimension")?,
            metric: need_str("metric")?,
            created_at: need_str("created_at")?,
            updated_at: need_str("updated_at")?,
        })
    }

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
        let mut hits = Vec::with_capacity(arr.len());
        for entry in arr {
            let id = entry
                .map_get("id")
                .and_then(|v| v.as_str())
                .map(str::to_owned)
                .ok_or_else(|| RpcClientError::Server("search.basic: hit missing 'id'".into()))?;
            let score = entry
                .map_get("score")
                .and_then(|v| v.as_float())
                .ok_or_else(|| {
                    RpcClientError::Server("search.basic: hit missing 'score'".into())
                })?;
            let payload = entry
                .map_get("payload")
                .and_then(|v| v.as_str())
                .map(str::to_owned);
            hits.push(SearchHit { id, score, payload });
        }
        Ok(hits)
    }
}
