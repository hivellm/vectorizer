# Changelog

All notable changes to the Hive Vectorizer Rust SDK will be documented in this file.

## [3.3.0] - 2026-05-02

> Note: phantom entries 3.4.0–3.8.0 (released 2026-05-02) consolidated into 3.3.0 to align with the server release. See `fb8ddb89` for the same operation on the server CHANGELOG. The phase25/27 dashboard metrics surface (originally drafted as `[Unreleased]`) is also rolled into this entry.

### Added

- **Phase25 dashboard metrics endpoints.**
  - `VectorizerClient::get_runtime_metrics()` calling `GET /metrics/runtime`
    with new typed `RuntimeMetrics` (cpu/memory/connections/uptime/QPS/5xx-rate),
    `RouteStats` (per-route p50/p99), and `WalSnapshot`
    (current_seq / size_bytes / last_checkpoint_at / last_checkpoint_seq).
    Defaults zero-valued so older servers and standalone-mode (no WAL)
    payloads parse unchanged.
  - `Stats` grows `default_quantization: String` (`none` / `binary` /
    `sq-4bit` / `sq-8bit` / `sq-16bit` / `sq` / `pq`) and
    `compression_ratio: f32`. Both default to `("none", 1.0)` so older
    servers keep deserialising.
  - `Collection` grows `vector_count_history: Vec<VectorCountSample>`
    (60-sample ring, one per minute) for the dashboard's per-collection
    sparkline. Empty array on older servers or for collections that
    have never been read.
  - 4 new unit tests cover full + partial `RuntimeMetrics` payloads and
    the new `Stats` quantization fields.

- **Typed `Filter` / `QdrantFilter` builder (phase23).** New `models/filter.rs` module ships typed filter types with full wire-shape coverage: `Filter`, `Condition`, `Match`, `Range`. Re-exported from the SDK's `models` module. Doc comments on `delete_by_filter` and `bulk_update_metadata` recommend the typed value over raw JSON. Server-side error messages for malformed filters now return `error_type: "parse_error"` with serde field paths instead of the misleading `"empty filter is not allowed"`. 8 unit tests cover wire-shape stability for every variant + nested compound filters + omitting absent clauses.
- **Tier-demotion API ([#265](https://github.com/hivellm/vectorizer/issues/265)).** Three new methods on `VectorizerClient`:
  - `delete_vector(collection, vector_id) -> Result<()>` calling `DELETE /collections/{c}/vectors/{id}`.
  - `delete_vectors(collection, ids) -> Result<DeleteReport>` calling `POST /batch_delete` with per-id status in `results`.
  - `move_to_collection(src, dst, ids) -> Result<MoveReport>` calling `POST /collections/{src}/vectors/move`. Server invariant: dst-insert-before-src-delete; a mid-batch crash leaves a recoverable duplicate, never data loss. Per-id outcomes (`ok | missing_in_src | dst_insert_failed | src_delete_failed`) populate `MoveReport.results` without aborting the batch.
- New report types under `vectorizer_sdk::models`: `DeleteReport`, `MoveReport`, `VectorOpResult`.
- **Control-surface parity (phase12).** Full REST coverage across all server surfaces:
  - **Vectors surface** — `update_vector`, `insert_text`, `list_vectors`, `get_vector_by_path`, `batch_insert_texts`, `insert_vectors` (pre-computed), `batch_search`, `batch_update_vectors`.
  - **Search surface** — `search_by_file` (`POST /collections/{name}/search/file`).
  - **Discovery pipeline** — `broad_discovery`, `semantic_focus`, `promote_readme`, `compress_evidence`, `build_answer_plan`, `render_llm_prompt`.
  - **Admin surface** (`client/admin.rs`, new) — `get_stats`, `get_status`, `get_logs`, `get_indexing_progress`, `force_save_collection`, `list_empty_collections`, `cleanup_empty_collections`, `get_config`, `update_config`, `list_backups`, `create_backup`, `restore_backup`, `restart_server`, `list_workspaces`, `get_workspace_config`, `add_workspace`, `remove_workspace`.
  - **Auth surface** (`client/auth.rs`, new) — `me`, `logout`, `refresh_token`, `validate_password`, `create_api_key`, `list_api_keys`, `revoke_api_key`, `create_user`, `list_users`, `delete_user`, `change_password`.
  - **Replication surface** (`client/replication.rs`, new) — `get_replication_status`, `configure_replication`, `get_replication_stats`, `list_replicas`.
  - **HiveHub surface** (`client/hub.rs`, new) — `list_user_backups`, `create_user_backup`, `restore_user_backup`, `upload_user_backup`, `get_user_backup`, `delete_user_backup`, `download_user_backup`, `get_usage_statistics`, `get_quota_info`, `validate_hub_api_key`.
- New model types in `vectorizer_sdk::models`: `VectorPage`, `UpdateVectorRequest`, `BatchInsertItem`, `BatchInsertReport`, `VectorUpdate`, `BatchUpdateReport`, `RawVectorInsert`, `BatchSearchQuery` (extended), `SearchByFileRequest`, `BroadDiscoveryRequest`, `BroadDiscoveryResponse`, `SemanticFocusRequest`, `SemanticFocusResponse`, `PromoteReadmeRequest`, `PromoteReadmeResponse`, `CompressEvidenceRequest`, `CompressEvidenceResponse`, `AnswerPlanRequest`, `AnswerPlan`, `RenderPromptRequest`, `LlmPrompt`, `Stats`, `ServerStatus`, `LogsQuery`, `LogEntry`, `CleanupReport`, `ConfigSnapshot`, `ConfigPatch`, `BackupInfo`, `CreateBackupRequest`, `RestoreBackupRequest`, `WorkspaceConfig`, `AddWorkspaceRequest`, `User`, `JwtToken`, `PasswordPolicyReport`, `CreateApiKeyRequest`, `ApiKey`, `CreateUserRequest`, `ReplicationStatus`, `ReplicationConfig`, `ReplicaInfo`, `ReplicationStats`, `UserBackup`, `CreateUserBackupRequest`, `RestoreUserBackupRequest`, `UploadUserBackupRequest`, `UsageStatistics`, `QuotaInfo`, `HubApiKeyValidation`.
- **Schema-evolution + observability API (phase14).** Eight new server routes exposed across three client files:
  - **`client/collections.rs`** — `rename_collection`, `reindex_collection`, `snapshot_collection_native`, `list_collection_snapshots_native`, `restore_collection_snapshot_native`.
  - **`client/search.rs`** — `explain_search` (`POST /collections/{name}/explain`): returns search results plus a full HNSW execution trace (`visited_nodes`, `ef_search`, `hnsw_search_ms`, `payload_filter_evals`, `quantization_score_ms`, `total_ms`).
  - **`client/admin.rs`** — `list_slow_queries` (`GET /slow_queries`), `set_slow_query_config` (`POST /slow_queries/config`).
- New model types in `src/models.rs`: `ReindexParams`, `ReindexJob`, `NativeSnapshotInfo`, `ExplainRequest`, `ExplainResponse`, `ExplainTrace`, `SlowQueryEntry`, `SlowQueryConfig`.
- **Cluster + auth admin API (phase15).** Nine new server routes exposed across two client files:
  - **`client/replication.rs`** — `cluster_failover` (`POST /cluster/failover`), `cluster_resync_replica` (`POST /cluster/replicas/{id}/resync`), `cluster_add_peer` (`POST /cluster/peers`), `cluster_rebalance` (`POST /cluster/rebalance`), `cluster_rebalance_status` (`GET /cluster/rebalance/status`).
  - **`client/auth.rs`** — `rotate_api_key` (`POST /auth/keys/{id}/rotate`), `create_scoped_api_key` (`POST /auth/keys`), `introspect_token` (`POST /auth/introspect`), `list_audit_log` (`GET /auth/audit`).
- New model types in `src/models.rs`: `FailoverReport`, `ResyncJob`, `PeerInfo`, `AddPeerRequest`, `RebalanceJob`, `RotatedKey`, `CreateScopedApiKeyRequest`, `TokenScope`, `TokenIntrospection`, `AuditEntry`, `AuditQuery`.
- **Phase16 RPC typed wrappers.** 96 new methods on `RpcClient` mirroring every command in `rpc_capability_names()` that was not previously wrapped:
  - **Collections (5):** `create_collection`, `delete_collection`, `list_empty_collections`, `cleanup_empty_collections`, `force_save_collection`.
  - **Vectors (15):** `insert_vector`, `insert_text_vector`, `update_vector`, `delete_vector_rpc`, `list_vectors`, `embed_text`, `batch_insert_vectors`, `batch_insert_texts`, `batch_search`, `batch_update_vectors`, `batch_delete_vectors`, `move_vectors_rpc`, `copy_vectors_rpc`, `delete_by_filter_rpc`, `bulk_update_metadata_rpc`, `set_vector_expiry`.
  - **Search (7):** `search_intelligent`, `search_by_text`, `search_by_file`, `search_hybrid`, `search_semantic`, `search_contextual`, `search_multi_collection`, `search_explain`.
  - **Discovery (10):** `discover`, `filter_collections`, `score_collections`, `expand_queries`, `broad_discovery`, `semantic_focus`, `promote_readme`, `compress_evidence`, `build_answer_plan`, `render_llm_prompt`.
  - **File ops (7):** `file_content`, `file_list`, `file_summary`, `file_chunks`, `file_outline`, `file_related`, `file_search_by_type`.
  - **Graph (10):** `graph_list_nodes`, `graph_neighbors`, `graph_find_related`, `graph_find_path`, `graph_create_edge`, `graph_delete_edge`, `graph_list_edges`, `graph_discover_edges`, `graph_discover_edges_for_node`, `graph_discovery_status`.
  - **Admin (16):** `admin_stats`, `admin_status`, `admin_logs`, `admin_indexing_progress`, `admin_config_get`, `admin_config_update`, `admin_backups_list`, `admin_backups_create`, `admin_backups_restore`, `admin_workspaces_list`, `admin_workspace_get`, `admin_workspace_add`, `admin_workspace_remove`, `admin_restart`, `admin_slow_queries_list`, `admin_slow_queries_config`.
  - **Auth (13):** `auth_me`, `auth_logout`, `auth_refresh_token`, `auth_validate_password`, `auth_api_keys_create`, `auth_api_keys_list`, `auth_api_keys_revoke`, `rotate_api_key_rpc`, `auth_api_keys_create_scoped`, `auth_users_create`, `auth_users_list`, `auth_users_delete`, `auth_users_change_password`, `auth_introspect`, `auth_audit`.
  - **Replication (4):** `replication_status`, `replication_configure`, `replication_stats`, `replication_replicas_list`.
  - **Cluster (5):** `cluster_failover`, `cluster_replica_resync`, `cluster_peer_add`, `cluster_rebalance`, `cluster_rebalance_status`.
- New RPC-specific return types re-exported from `vectorizer_sdk::rpc`: `CollectionInfo`, `CreateCollectionResult`, `CleanupEmptyResult`, `VectorWriteResult`, `BatchInsertResult`, `BatchUpdateResult`, `BatchDeleteResult`, `BatchItemResult`, `BatchSearchResult`, `MoveRpcResult`, `CopyRpcResult`, `DeleteByFilterRpcResult`, `BulkUpdateMetadataRpcResult`, `SetExpiryResult`, `EmbedResult`, `VectorListResult`, `SearchHit`, `SearchExplainResult`, `SearchTrace`, `DiscoverResult`, `ScoredCollection`, `ExpandQueriesResult`, `DiscoveryChunk`, `CompressBullet`, `AnswerPlanResult`, `AnswerPlanSection`, `RenderPromptResult`, `GraphDiscoveryStatus`, `DiscoverEdgesResult`, `DiscoverEdgesForNodeResult`, `AdminStats`, `AdminStatus`, `SlowQueryConfigResult`, `AuthMeResult`, `RefreshTokenResult`, `ValidatePasswordResult`, `ApiKeyCreated`, `RotatedApiKey`, `ReplicationConfigureResult`, `RebalanceStatus`.

### Tests

- Inline `#[cfg(test)]` round-trip tests covering all new wire shapes (13 new tests for phase15).
- 22 inline `#[cfg(test)]` unit tests covering request shape construction and response decoding for all domain groups (phase16).

## [3.2.0] - 2026-05-01

### Added

- **Backpressure-aware HTTP transport.** Honors the server-side
  bulk-upsert backpressure shipped in Vectorizer 3.2.0
  ([#263](https://github.com/hivellm/vectorizer/issues/263)). On HTTP
  `429 Too Many Requests` the client parses `Retry-After` (seconds
  form), sleeps via `tokio::time::sleep`, and retries — bounded by
  the same 3-attempt / 30 s-cap / 1 s-default policy used by every
  other first-party SDK. After retry exhaustion a typed
  `VectorizerError::RateLimit` is surfaced. Implementation in
  `src/http_transport.rs::parse_retry_after_secs`; lock-in tests at
  `tests/retry_after_parse.rs`.
- `vectorizer-protocol` path dep pinned to the matching server
  version so `cargo publish` resolves the registry version cleanly.

### Changed

- Version bumped to 3.2.0 to track the server release.

## [3.1.0] - 2026-04-29

### Added

- **`VectorizerClient::insert_vectors(...)`** — bulk-insert pre-
  computed embeddings with caller-supplied vector ids. Skips the
  embedding pipeline entirely.
- **`insert` / `insert_texts` accept `id`** as the stored
  `Vector.id`. Non-chunked inputs use the client `id` verbatim;
  chunked inputs derive `<id>#<chunk_index>` (e.g. `doc:42#0`,
  `doc:42#1`). Re-running the same payload upserts in place.
- **`payload.parent_id` on chunked vectors** links chunks back to
  the source document.

### Changed

- **Chunked-payload layout flipped from nested to flat — BREAKING
  for clients reading `payload.metadata.<field>` directly.** Pre-
  3.1.0 chunks landed as `{content, metadata: {file_path,
  chunk_index, ...}}`. 3.1.0 emits `{content, file_path,
  chunk_index, parent_id, ...}` with every key at the root. Server-
  provided keys take precedence over user metadata. Readers tolerate
  both shapes during the deprecation window. See the parent-repo
  CHANGELOG for the migration matrix.

### Note

Client-id contract: non-empty, length ≤ 256, no leading/trailing
whitespace, must not contain `#` (reserved as the chunk-id
separator). Violations return HTTP 400 with
`error_type: "validation_error"`.

## [3.0.0] - 2026-04-19

### Added

- **VectorizerRPC client** (new default transport in v3.x). Binary,
  length-prefixed MessagePack over raw TCP (port 15503), spec at
  `docs/specs/VECTORIZER_RPC.md`. Polyglot parity with the Python,
  TypeScript, Go, and C# SDKs.
  - `RpcClient` (`tokio::net::TcpStream`) multiplexes calls on a
    single TCP connection by `Request.id` into per-call oneshots.
  - `parse_endpoint_url` — canonical URL parser shared with every
    other Vectorizer SDK. Accepts `vectorizer://host:port`,
    `vectorizer://host` (default port 15503), bare `host:port`, and
    `http(s)://host:port`. Rejects userinfo credentials.
  - `HelloPayload` / `HelloResponse` — sticky per-connection auth
    handshake.
  - `RpcPool` with bounded `max_connections` and an RAII guard.
  - Typed wrappers: `list_collections`, `get_collection_info`,
    `get_vector`, `search_basic`. Match the polyglot SDK shapes.

### Changed

- Bumped to v3.0.0 to mark the new default transport. The legacy
  HTTP path stays available behind the default-on `http` Cargo
  feature.
- README rewritten with an RPC-first quickstart and a "Switching
  transports" matrix.

### Note

The package surface is **additive** for existing 1.x callers:
`VectorizerClient` and every model still import from the same paths.
The 3.0 marker reflects that the recommended transport changes —
there is no forced migration of existing code.

## [1.3.0] - 2025-11-15

### Added

- **Hybrid Search Support**: Complete Rust implementation with full type safety
  - `SparseVector`: Struct for sparse vector representation with validation
  - `HybridSearchRequest`: Request struct with serde serialization
  - `HybridSearchResponse` and `HybridSearchResult`: Response structs
  - `HybridScoringAlgorithm`: Enum for scoring algorithms (RRF, Weighted, Alpha)
  - `hybrid_search()`: Method in VectorizerClient with full error handling
  - Module `models::hybrid_search` for all hybrid search types

- **Qdrant Compatibility**: Full Qdrant REST API compatibility methods
  - `qdrant_list_collections()`: List all collections
  - `qdrant_get_collection()`: Get collection information
  - `qdrant_create_collection()`: Create collection with Qdrant config
  - `qdrant_upsert_points()`: Upsert points to collection
  - `qdrant_search_points()`: Search points in collection
  - `qdrant_delete_points()`: Delete points from collection
  - `qdrant_retrieve_points()`: Retrieve points by IDs
  - `qdrant_count_points()`: Count points in collection

### Changed

- **Version Sync**: Updated to v1.3.0 to match Vectorizer server release
- **Server Compatibility**: Compatible with Vectorizer v1.3.0 (hybrid search and Qdrant compatibility)
- **Type Safety**: Full Rust type safety with serde serialization for all new methods

### Note

This release adds hybrid search and Qdrant compatibility features. All existing functionality remains unchanged and backward compatible.

## [1.2.0] - 2025-10-25

### Added

- **Replication Models**: New data structures for replication monitoring
  - `ReplicaStatus`: Enum for replica node status (Connected, Syncing, Lagging, Disconnected)
  - `ReplicaInfo`: Struct for replica node details with all fields
  - `ReplicationStats`: Enhanced statistics struct with new v1.2.0 fields:
    - `role`: Node role (Master or Replica)
    - `bytes_sent`: Total bytes sent to replicas
    - `bytes_received`: Total bytes received from master
    - `last_sync`: Timestamp of last synchronization
    - `operations_pending`: Number of operations waiting to be replicated
    - `snapshot_size`: Size of snapshot data
    - `connected_replicas`: Number of connected replica nodes (Master only)
  - `ReplicationStatusResponse`: Response struct for `/replication/status` endpoint
  - `ReplicaListResponse`: Response struct for `/replication/replicas` endpoint

### Changed

- **Backwards Compatible**: All new replication fields are `Option<T>` to maintain compatibility with older servers
- **Legacy Fields Maintained**: Existing replication fields continue to work and are non-optional for stability

### Technical

- Used `#[serde(skip_serializing_if = "Option::is_none")]` for new optional fields
- Added comprehensive documentation for all new types
- Used `DateTime<Utc>` from chrono for timestamp fields
- Maintained strict typing and Rust best practices

## [1.0.0] - 2025-10-21

### Changed

- **Version Sync**: Updated to v1.2.0 to match Vectorizer server release
- **Server Compatibility**: Compatible with Vectorizer v1.3.0 (hybrid search and Qdrant compatibility)
- **No Breaking Changes**: REST API remains unchanged, full backward compatibility
- **Production Ready**: Stable release aligned with server v1.3.0

### Note

This release maintains full compatibility with Vectorizer REST API. Version 1.3.0 adds hybrid search support (search_hybrid tool) and Qdrant compatibility, but does not affect SDK functionality as SDKs use REST/UMICP protocols.

## [0.4.0] - 2025-10-12

### Added

- **UMICP Protocol Support**: Added support for the UMICP (Universal Messaging and Inter-process Communication Protocol)
  - New `UmicpTransport` using `umicp-core` crate (optional feature)
  - Transport abstraction layer supporting multiple protocols (HTTP/HTTPS and UMICP)
  - Connection string support for easy protocol switching (e.g., `umicp://localhost:15003`)
  - `parse_connection_string` utility for parsing connection URIs
  - `Protocol` enum for protocol selection

### Changed

- Refactored `VectorizerClient` to use transport abstraction instead of direct reqwest
- Updated `VectorizerClient` with new configuration options:
  - Added `ClientConfig` struct for flexible initialization
  - Added `protocol` field to specify transport protocol
  - Added `connection_string` field for URI-based configuration
  - Added `umicp` field for UMICP-specific options
- All HTTP requests now go through transport layer for protocol flexibility

### New API

- `VectorizerClient::new(config: ClientConfig)`: Create client with full configuration
- `VectorizerClient::from_connection_string(conn_str, api_key)`: Create from connection string
- `client.protocol()`: Get the current transport protocol being used
- Multiple transport options:
  - HTTP/HTTPS (default)
  - UMICP (optional feature, requires `--features umicp`)

### Dependencies

- Added `async-trait@0.1` for transport trait
- Added `umicp-core@0.1` as optional dependency (feature-gated)
- Updated `reqwest` to `0.11.24` for compatibility

### Features

- `umicp`: Enable UMICP protocol support (opt-in via cargo feature)

### Documentation

- Created `examples/umicp_usage.rs` demonstrating UMICP usage
- Created comprehensive UMICP tests

### Technical

- Implemented `Transport` trait for protocol abstraction
- Created separate transport implementations:
  - `HttpTransport` for HTTP/HTTPS
  - `UmicpTransport` for UMICP protocol (feature-gated)
- Added comprehensive error handling for both protocols
- Maintained backward compatibility with existing HTTP-only configurations

### Requirements

- **Minimum Rust Version**: 1.75.0 for HTTP transport only
- **For UMICP Feature**: Rust 1.82+ (due to transitive dependencies from reqwest 0.12)

## [0.3.4] - Previous Version

- (Previous changes...)
