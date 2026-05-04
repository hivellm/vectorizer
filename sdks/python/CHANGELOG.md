# Changelog

All notable changes to the Hive Vectorizer Python SDK will be documented in this file.

## [3.3.0] - 2026-05-02

> Note: phantom entries 3.4.0–3.8.0 (released 2026-05-02) consolidated into 3.3.0 to align with the server release. See `fb8ddb89` for the same operation on the server CHANGELOG.

### Added

- **Tier-demotion API ([#265](https://github.com/hivellm/vectorizer/issues/265)).** Three new methods on `VectorizerClient`:
  - `delete_vector(collection, vector_id) -> None` calling `DELETE /collections/{c}/vectors/{id}`.
  - `move_to_collection(src, dst, ids) -> MoveReport` calling `POST /collections/{src}/vectors/move`. Server invariant: dst-insert-before-src-delete; a mid-batch crash leaves a recoverable duplicate, never data loss. Per-id outcomes (`ok | missing_in_src | dst_insert_failed | src_delete_failed`) populate `MoveReport.results` without aborting the batch.
- New dataclasses under `vectorizer_sdk.models`: `DeleteReport`, `MoveReport`, `VectorOpResult`.
- **Phase12 control-surface parity** — Python SDK now matches the Rust SDK's full API surface.
- **Admin methods** (17 total, 15 new): `get_stats`, `get_status`, `get_logs`, `force_save_collection`,
  `list_empty_collections`, `cleanup_empty_collections`, `get_config`, `update_config`,
  `list_backups`, `create_backup`, `restore_backup`, `restart_server`, `list_workspaces`,
  `get_workspace_config`, `add_workspace`, `remove_workspace`.
- **Auth methods** (11 total, 10 new): `me`, `logout`, `refresh_token`, `validate_password`,
  `create_api_key`, `list_api_keys`, `revoke_api_key`, `create_user`, `list_users`,
  `delete_user`, `change_password`.
- **Replication module** (`vectorizer.replication.ReplicationClient`, 4 methods):
  `get_replication_status`, `configure_replication`, `get_replication_stats`, `list_replicas`.
- **HiveHub module** (`vectorizer.hub.HubClient`, 10 methods):
  `list_user_backups`, `create_user_backup`, `restore_user_backup`, `upload_user_backup`,
  `get_user_backup`, `delete_user_backup`, `download_user_backup`, `get_usage_statistics`,
  `get_quota_info`, `validate_hub_api_key`.
- **Discovery pipeline module** (`vectorizer.discovery.DiscoveryClient`, 6 methods):
  `broad_discovery`, `semantic_focus`, `promote_readme`, `compress_evidence`,
  `build_answer_plan`, `render_llm_prompt`.
- **Vector methods** (8 new): `update_vector`, `insert_text`, `list_vectors`,
  `get_vector_by_path`, `insert_vectors`, `batch_insert`, `batch_search`, `batch_update`.
- **Search method**: `search_by_file` — search by indexed file path.
- **New dataclasses** in `models.py`: `Stats`, `ServerStatus`, `LogsQuery`, `LogEntry`,
  `CleanupReport`, `BackupInfo`, `CreateBackupRequest`, `RestoreBackupRequest`,
  `AddWorkspaceRequest`, `User`, `JwtToken`, `PasswordPolicyReport`, `CreateApiKeyRequest`,
  `ApiKey`, `CreateUserRequest`, `ReplicationStatus`, `ReplicationConfig`, `VectorPage`,
  `UpdateVectorRequest`, `BatchInsertItem`, `BatchInsertReport`, `VectorUpdate`,
  `BatchUpdateReport`, `RawVectorInsert`, `BatchSearchQuery`, `SearchByFileRequest`,
  `BroadDiscoveryRequest`, `BroadDiscoveryResponse`, `SemanticFocusRequest`,
  `SemanticFocusResponse`, `PromoteReadmeRequest`, `PromoteReadmeResponse`,
  `CompressEvidenceRequest`, `CompressEvidenceResponse`, `AnswerPlanRequest`, `AnswerPlan`,
  `RenderPromptRequest`, `LlmPrompt`, `UserBackup`, `CreateUserBackupRequest`,
  `RestoreUserBackupRequest`, `UploadUserBackupRequest`, `UsageStatistics`, `QuotaInfo`,
  `HubApiKeyValidation`.
- **Phase13 tier-control methods** — Python SDK now matches the Rust SDK's phase13 surface.
- **Vector methods** (4 new): `delete_by_filter`, `bulk_update_metadata`, `copy_vectors`,
  `set_vector_expiry` on `VectorsClient`.
- **Collection methods** (2 new): `reencode_collection`, `set_collection_ttl`
  on `CollectionsClient`.
- **New dataclasses** in `models.py`: `DeleteByFilterReport`, `BulkUpdateReport`,
  `CopyReport`, `ReencodeJob`.
- `patch` method added to `Transport` ABC, `RestTransport`, and `HTTPClient`.
- All phase13 dataclasses re-exported from `vectorizer/__init__.py`.
- **Schema-evolution + observability API (phase14).** Eight new server routes exposed across three client modules:
  - **`vectorizer/collections.py`** — `rename_collection`, `reindex_collection`, `snapshot_collection_native`, `list_collection_snapshots_native`, `restore_collection_snapshot_native`.
  - **`vectorizer/search.py`** — `explain_search` (`POST /collections/{name}/explain`): returns search results plus a full HNSW execution trace (`visited_nodes`, `ef_search`, `hnsw_search_ms`, `payload_filter_evals`, `quantization_score_ms`, `total_ms`).
  - **`vectorizer/admin.py`** — `list_slow_queries` (`GET /slow_queries`), `set_slow_query_config` (`POST /slow_queries/config`).
- New dataclasses in `models.py`: `ReindexParams`, `ReindexJob`, `NativeSnapshotInfo`, `ExplainTrace`, `ExplainResponse`, `SlowQueryEntry`, `SlowQueryConfig`. All re-exported from `vectorizer/__init__.py`.
- **Cluster + auth admin API (phase15).** Nine new server routes exposed across two client modules:
  - **`vectorizer/replication.py`** — `cluster_failover`, `cluster_resync_replica`, `cluster_add_peer`, `cluster_rebalance`, `cluster_rebalance_status`.
  - **`vectorizer/auth.py`** — `rotate_api_key`, `create_scoped_api_key`, `introspect_token`, `list_audit_log`.
- New dataclasses in `models.py`: `FailoverReport`, `ResyncJob`, `PeerInfo`, `AddPeerRequest`, `RebalanceJob`, `RotatedKey`, `TokenScope`, `CreateScopedApiKeyRequest`, `TokenIntrospection`, `AuditEntry`, `AuditQuery`. All re-exported from `vectorizer/__init__.py`.
- **Phase16 RPC typed wrappers** — full coverage of all 95 server commands via `rpc/commands.py`. Every command has both a sync module-level function and an `async def` coroutine, monkey-patched onto `RpcClient` / `AsyncRpcClient`.
  - **collections** (7): `list_collections`, `get_collection_info`, `create_collection`, `delete_collection`, `list_empty_collections`, `cleanup_empty_collections`, `force_save_collection`.
  - **vectors** (17): `get_vector`, `insert_vector`, `insert_text_vector`, `update_vector`, `delete_vector_rpc`, `list_vectors`, `embed_text`, `batch_insert_vectors`, `batch_insert_texts`, `batch_search`, `batch_update_vectors`, `batch_delete_vectors`, `move_vectors_rpc`, `copy_vectors_rpc`, `delete_by_filter_rpc`, `bulk_update_metadata_rpc`, `set_vector_expiry`.
  - **search** (9): `search_basic`, `search_intelligent`, `search_by_text`, `search_by_file`, `search_hybrid`, `search_semantic`, `search_contextual`, `search_multi_collection`, `search_explain`.
  - **discovery** (10): `discover`, `filter_collections`, `score_collections`, `expand_queries`, `broad_discovery`, `semantic_focus`, `promote_readme`, `compress_evidence`, `build_answer_plan`, `render_llm_prompt`.
  - **file** (7): `file_content`, `file_list`, `file_summary`, `file_chunks`, `file_outline`, `file_related`, `file_search_by_type`.
  - **graph** (10): `graph_list_nodes`, `graph_neighbors`, `graph_find_related`, `graph_find_path`, `graph_create_edge`, `graph_delete_edge`, `graph_list_edges`, `graph_discover_edges`, `graph_discover_edges_for_node`, `graph_discovery_status`.
  - **admin** (16): `admin_stats`, `admin_status`, `admin_logs`, `admin_indexing_progress`, `admin_config_get`, `admin_config_update`, `admin_backups_list`, `admin_backups_create`, `admin_backups_restore`, `admin_workspaces_list`, `admin_workspace_get`, `admin_workspace_add`, `admin_workspace_remove`, `admin_restart`, `admin_slow_queries_list`, `admin_slow_queries_config`.
  - **auth** (11): `auth_me`, `auth_logout`, `auth_refresh_token`, `auth_validate_password`, `auth_api_keys_create`, `auth_api_keys_list`, `auth_api_keys_revoke`, `rotate_api_key_rpc`, `auth_api_keys_create_scoped`, `auth_introspect`, `auth_audit`.
  - **replication** (4): `replication_status`, `replication_configure`, `replication_stats`, `replication_replicas_list`.
  - **cluster** (5): `cluster_failover`, `cluster_replica_resync`, `cluster_peer_add`, `cluster_rebalance`, `cluster_rebalance_status`.
- New dataclasses re-exported from `rpc/__init__.py` and `__init__.py`: `AdminStats`, `AdminStatus`, `AnswerPlanResult`, `AnswerPlanSection`, `ApiKeyCreated`, `AuthMeResult`, `BatchDeleteResult`, `BatchInsertResult`, `BatchItemResult`, `BatchSearchResult`, `BatchUpdateResult`, `BulkUpdateMetadataRpcResult`, `CleanupEmptyResult`, `CompressBullet`, `CopyRpcResult`, `CreateCollectionResult`, `DeleteByFilterRpcResult`, `DiscoverEdgesForNodeResult`, `DiscoverEdgesResult`, `DiscoverResult`, `DiscoveryChunk`, `EmbedResult`, `ExpandQueriesResult`, `GraphDiscoveryStatus`, `MoveRpcResult`, `RebalanceStatus`, `RefreshTokenResult`, `RenderPromptResult`, `ReplicationConfigureResult`, `RotatedApiKey`, `ScoredCollection`, `SearchExplainResult`, `SearchTrace`, `SetExpiryResult`, `SlowQueryConfigResult`, `ValidatePasswordResult`, `VectorListResult`, `VectorWriteResult`.

### Tests

- pytest tests in `tests/test_cluster_auth_admin.py`.
- pytest tests in `tests/test_rpc_phase16.py` covering all 10 domain groups via `AsyncMock`.

### Fixed

- **`vectorizer/graph.py` NameError bug** — `delete_graph_edge` referenced undefined `collection`; fixed to `f"/graph/edges/{edge_id}"` with correct `bool` return type.

### Changed

- **`delete_vectors` now returns `DeleteReport`** (was `bool` in 3.2). The wire path is unchanged (`POST /batch_delete`), but the SDK now decodes the server's full per-id status array via the `DeleteReport` dataclass. Callers that previously checked `if result:` need to switch to `report.failed == 0` (or inspect `report.results`).

## [3.2.0] - 2026-05-01

### Added

- **Backpressure-aware HTTP client.** Honors the server-side bulk-
  upsert backpressure shipped in Vectorizer 3.2.0
  ([#263](https://github.com/hivellm/vectorizer/issues/263)). On HTTP
  `429 Too Many Requests` the client parses `Retry-After` (seconds
  form), sleeps, and retries — bounded by the same 3-attempt / 30 s-
  cap / 1 s-default policy used by every other first-party SDK. After
  retry exhaustion a typed `RateLimitError` is raised. Pre-3.2.0
  clients bounced 429s into a generic 5xx and lost the retry budget.
- New tests at `tests/test_retry_after_parse.py` lock the parser
  semantics and the 3-attempt budget.

### Changed

- Version bumped to 3.2.0 to track the server release.

## [3.1.0] - 2026-04-29

### Added

- **`insert_vectors(collection, vectors, public_key=None)`** — bulk-
  insert pre-computed embeddings with caller-supplied vector ids.
  Skips the embedding pipeline entirely. Useful when the client
  already has its own embedder or wants idempotent re-ingest by
  stable id.
- **`insert` / `insert_texts` accept `id`** as the stored
  `Vector.id`. Non-chunked inputs use the client `id` verbatim;
  chunked inputs derive `<id>#<chunk_index>` (e.g. `doc:42#0`,
  `doc:42#1`). Re-running the same payload now upserts in place
  instead of duplicating.
- **`payload.parent_id` on chunked vectors** links chunks back to
  the source document.

### Changed

- **Chunked-payload layout flipped from nested to flat — BREAKING
  for clients reading `payload["metadata"][field]` directly.** Pre-
  3.1.0 chunks landed as `{content, metadata: {file_path,
  chunk_index, ...}}`. 3.1.0 emits `{content, file_path,
  chunk_index, parent_id, ...}` with every key at the root. Server-
  provided keys take precedence over user metadata. Readers tolerate
  both shapes during the deprecation window. See the parent-repo
  CHANGELOG for the migration matrix.

### Note

Client-id contract: non-empty, length ≤ 256, no leading/trailing
whitespace, must not contain `#` (reserved as the chunk-id
separator).

## [3.0.0] - 2026-04-19

### Added

- **VectorizerRPC client** (new default transport in v3.x). Binary,
  length-prefixed MessagePack over raw TCP (port 15503), spec at
  `docs/specs/VECTORIZER_RPC.md`. Polyglot parity with the Rust SDK
  at `sdks/rust/src/rpc/`.
  - `rpc.RpcClient` (sync, stdlib `socket` + threading) and
    `rpc.AsyncRpcClient` (`asyncio.open_connection`-based). Both
    multiplex calls on a single TCP connection by `Request.id`.
  - `rpc.parse_endpoint` — canonical URL parser shared with every
    other Vectorizer SDK. Accepts `vectorizer://host:port`,
    `vectorizer://host` (default port 15503), bare `host:port`, and
    `http(s)://host:port`. Rejects userinfo credentials.
  - `rpc.HelloPayload` / `rpc.HelloResponse` — sticky per-connection
    auth handshake.
  - `rpc.RpcPool` and `rpc.AsyncRpcPool` — minimal bounded connection
    pools with an RAII-style guard.
  - Typed wrappers: `list_collections`, `get_collection_info`,
    `get_vector`, `search_basic`. Match the Rust SDK shape exactly.
  - Top-level `vectorizer_sdk.connect(url)` / `connect_async(url)`
    convenience functions.
- New runtime dependency: `msgpack>=1.0.0`.
- New example: `examples/rpc_quickstart.py`.
- 45 new tests under `tests/rpc/` (unit + integration with an in-test
  fake server). Includes wire-spec golden vectors that bit-exactly
  match the hex dumps in the protocol spec.

### Changed

- Bumped to v3.0.0 to mark the new default transport. The legacy
  `VectorizerClient` REST client stays available unchanged for
  callers that need HTTP.
- `README.md` rewritten with an RPC-first quickstart and a
  "Switching transports" matrix.

### Note

The package surface is **additive** for existing v2.x callers:
`VectorizerClient` and every model class still import from the same
paths. The "breaking" v3.0 marker reflects that the recommended
transport changes — there is no forced migration of existing code.

## [1.3.0] - 2025-11-15

### Added

- **Hybrid Search Support**: Complete implementation of hybrid search combining dense and sparse vectors

  - `SparseVector`: Model for sparse vector representation with indices and values
  - `HybridSearchRequest`: Request model with alpha, algorithm (rrf/weighted/alpha), and k parameters
  - `HybridSearchResponse` and `HybridSearchResult`: Response models for hybrid search results
  - `hybrid_search()`: Method in VectorizerClient for performing hybrid searches
  - Full validation and error handling

- **Qdrant Compatibility**: Full Qdrant REST API compatibility methods
  - `qdrant_list_collections()`: List all collections (Qdrant format)
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

### Note

This release adds hybrid search and Qdrant compatibility features. All existing functionality remains unchanged and backward compatible.

## [1.2.0] - 2025-10-25

### Added

- **Replication Models**: New data models for replication monitoring
  - `ReplicaStatus`: Status enum for replica nodes (Connected, Syncing, Lagging, Disconnected)
  - `ReplicaInfo`: Detailed information about replica nodes (host, port, status, heartbeat, operations synced)
  - `ReplicationStats`: Enhanced statistics with new v1.2.0 fields:
    - `role`: Node role (Master or Replica)
    - `bytes_sent`: Total bytes sent to replicas
    - `bytes_received`: Total bytes received from master
    - `last_sync`: Timestamp of last synchronization
    - `operations_pending`: Number of operations waiting to be replicated
    - `snapshot_size`: Size of snapshot data
    - `connected_replicas`: Number of connected replica nodes (Master only)
  - `ReplicationStatusResponse`: Response structure for `/replication/status` endpoint
  - `ReplicaListResponse`: Response structure for `/replication/replicas` endpoint

### Changed

- **Backwards Compatible**: All new replication fields are optional to maintain compatibility with older servers
- **Legacy Fields Maintained**: Existing replication fields (`master_offset`, `replica_offset`, `lag_operations`, `total_replicated`) continue to work

### Technical

- Added comprehensive validation for new replication models
- Enhanced type hints with proper Optional types for new fields
- Maintained strict dataclass validation for all new models

## [1.0.0] - 2025-10-21

### Changed

- **Version Sync**: Updated to v1.0.0 to match Vectorizer server release
- **Server Compatibility**: Compatible with Vectorizer v1.3.0 (hybrid search and Qdrant compatibility)
- **No Breaking Changes**: REST API remains unchanged, full backward compatibility
- **Production Ready**: Stable release aligned with server v1.0.0

### Note

This release maintains full compatibility with Vectorizer REST API. The MCP refactoring in server v1.0.0 does not affect SDK functionality as SDKs use REST/UMICP protocols.

## [0.4.0] - 2025-10-12

### 🎉 Published to PyPI

- **Package**: [hive-vectorizer](https://pypi.org/project/hive-vectorizer/0.4.0/)
- **Installation**: `pip install hive-vectorizer`
- Migrated from `setup.py` to modern `pyproject.toml` configuration
- Added `.gitignore` to prevent committing build artifacts and credentials

### Added

- **UMICP Protocol Support**: Added support for the UMICP (Universal Messaging and Inter-process Communication Protocol)
  - New `UMICPClient` using official `umicp-python` package
  - Transport abstraction layer supporting multiple protocols (HTTP/HTTPS and UMICP)
  - Connection string support for easy protocol switching (e.g., `umicp://localhost:15003`)
  - `TransportFactory` for creating protocol-specific clients
  - `parse_connection_string` utility for parsing connection URIs
  - `HTTPClient` module extracted for better separation of concerns
- Build and publish scripts (`build.sh`, `publish.sh`, `build.ps1`, `publish.ps1`)
- Comprehensive test suite for UMICP transport

### Changed

- Refactored `VectorizerClient` to use transport abstraction instead of direct aiohttp calls
- Updated `VectorizerClient` constructor to support multiple protocols:
  - Added `connection_string` parameter for URI-based configuration
  - Added `protocol` parameter to specify transport protocol
  - Added `umicp` parameter for UMICP-specific options
- Updated `connect()` and `close()` methods to handle multiple transport types
- Migrated package configuration from `setup.py` to `pyproject.toml` (PEP 517/518)

### New API

- `client.get_protocol()`: Get the current transport protocol being used
- Multiple transport options:
  - HTTP/HTTPS (default)
  - UMICP (via `umicp-python` package)

### Dependencies

- Added `umicp-python>=0.1.3` for UMICP protocol support
- Added `aiohttp>=3.8.0` as primary dependency

### Documentation

- Created `examples/umicp_usage.py` demonstrating UMICP usage
- Updated README.md with UMICP configuration examples
- Added protocol comparison table

### Technical

- Created transport abstraction for protocol independence
- Separated HTTP logic into `utils/http_client.py`
- Created `utils/umicp_client.py` wrapper around `umicp-python`
- Created `utils/transport.py` for transport factory and parsing
- Added comprehensive error handling for both protocols
- Maintained backward compatibility with existing HTTP-only configurations
- Translated all test comments and strings to English

## [0.3.4] - Previous Version

- (Previous changes...)
