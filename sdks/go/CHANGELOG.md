# Changelog

All notable changes to the Hive Vectorizer Go SDK will be documented in this file.

## [3.3.0] - 2026-05-03

> Note: phantom entries 3.4.0–3.8.0 (released 2026-05-02) consolidated into 3.3.0 to align with the server release. See `fb8ddb89` for the same operation on the server CHANGELOG.

### Added

REST control surface parity with Rust/TypeScript/Python SDKs (phase20). The
Go SDK now exposes ~79 new REST methods covering every endpoint shipped in
phases 12-15. No RPC dependency required.

- **Admin/observability (17)** — `GetServerStats`, `GetStatus`, `GetLogs`,
  `GetIndexingProgress`, `ForceSaveCollection`, `ListEmptyCollections`,
  `CleanupEmptyCollections`, `GetConfig`, `UpdateConfig`, `ListBackups`,
  `CreateBackup`, `RestoreBackup`, `RestartServer`, `ListWorkspaces`,
  `GetWorkspaceConfig`, `AddWorkspace`, `RemoveWorkspace`.
- **Auth/RBAC (11)** — `Me`, `Logout`, `RefreshToken`, `ValidatePassword`,
  `CreateApiKey`, `ListApiKeys`, `RevokeApiKey`, `CreateUser`, `ListUsers`,
  `DeleteUser`, `ChangePassword`.
- **Replication (4)** — `GetReplicationStatus`, `ConfigureReplication`,
  `GetReplicationStats`, `ListReplicas`.
- **Hub (10)** — `ListUserBackups`, `CreateUserBackup`, `RestoreUserBackup`,
  `UploadUserBackup`, `GetUserBackup`, `DeleteUserBackup`,
  `DownloadUserBackup` (raw bytes), `GetUsageStatistics`, `GetQuotaInfo`,
  `ValidateHubAPIKey`.
- **Discovery pipeline (6)** — `BroadDiscovery`, `SemanticFocus`,
  `PromoteReadme`, `CompressEvidence`, `BuildAnswerPlan`, `RenderLlmPrompt`.
- **Vectors single+batch+search (9)** — `UpdateVectorPayload`,
  `InsertTextWithID`, `ListVectors`, `BatchInsertTexts`, `InsertVectors`,
  `BatchSearchQueries`, `BatchUpdateVectors`, `SearchByText`, `SearchByFile`.
- **Tier-control (6)** — `DeleteByFilter`, `BulkUpdateMetadata`,
  `CopyVectors`, `ReencodeCollection`, `SetCollectionTTL`, `SetVectorExpiry`.
  Empty-filter validation rejected client-side before HTTP for
  `DeleteByFilter` and `BulkUpdateMetadata`.
- **Schema evolution + explain + slow queries (8)** — `RenameCollection`,
  `ReindexCollection`, `SnapshotCollectionNative`,
  `ListCollectionSnapshotsNative`, `RestoreCollectionSnapshotNative`,
  `ExplainSearch`, `ListSlowQueries`, `SetSlowQueryConfig`.
- **Cluster + auth admin (9)** — `ClusterFailover`, `ClusterResyncReplica`,
  `ClusterAddPeer`, `ClusterRebalance`, `ClusterRebalanceStatus` (returns
  `nil` when idle), `RotateApiKey`, `CreateScopedApiKey`, `IntrospectToken`,
  `ListAuditLog`.
- **Models** — ~50 new types in `models.go` mirroring Rust source-of-truth
  shapes (Stats, ServerStatus, LogEntry, IndexingProgress, ConfigSnapshot,
  BackupInfo, WorkspaceConfig, User, JwtToken, ApiKey, PasswordPolicyReport,
  ReplicationStatus, ReplicaInfo, UserBackup, BroadDiscoveryRequest/Response,
  SemanticFocusRequest/Response, PromoteReadmeRequest/Response,
  CompressEvidenceRequest/Response, AnswerPlan, LlmPrompt, VectorPage,
  BatchInsertReport, BatchUpdateReport, DeleteByFilterReport,
  BulkUpdateReport, CopyReport, ReencodeJob, ReindexJob, NativeSnapshotInfo,
  ExplainResponse, SlowQueryEntry, SlowQueryConfig, FailoverReport,
  ResyncJob, AddPeerRequest, PeerInfo, RebalanceJob, RotatedKey,
  TokenIntrospection, AuditEntry, etc.).
- **Phase 16 full RPC command catalog.** Typed Go methods on `*rpc.Client`
  covering every command in `rpc_capability_names()` (95 commands across 8
  domain groups). New response struct types in `rpc/types_phase16.go`.
  Methods follow existing PascalCase + `Rpc` suffix convention to avoid
  collision with REST SDK names (e.g. `MoveVectorsRpc`, `RotateApiKeyRpc`).
  - Collections (5 new): `CreateCollectionRpc`, `DeleteCollectionRpc`,
    `ListEmptyCollections`, `CleanupEmptyCollections`, `ForceSaveCollection`.
  - Vectors (15 new): `InsertVectorRpc`, `InsertTextVectorRpc`,
    `UpdateVectorRpc`, `DeleteVectorRpc`, `ListVectors`, `EmbedText`,
    `BatchInsertVectors`, `BatchInsertTexts`, `BatchSearch`,
    `BatchUpdateVectors`, `BatchDeleteVectors`, `MoveVectorsRpc`,
    `CopyVectorsRpc`, `DeleteByFilterRpc`, `BulkUpdateMetadataRpc`,
    `SetVectorExpiry`.
  - Search (7 new): `SearchIntelligent`, `SearchByText`, `SearchByFile`,
    `SearchHybrid`, `SearchSemantic`, `SearchContextual`,
    `SearchMultiCollection`, `SearchExplain`.
  - Discovery (10 new): `Discover`, `FilterCollections`, `ScoreCollections`,
    `ExpandQueries`, `BroadDiscovery`, `SemanticFocus`, `PromoteReadme`,
    `CompressEvidence`, `BuildAnswerPlan`, `RenderLlmPrompt`.
  - File ops (7 new): `FileContent`, `FileList`, `FileSummary`, `FileChunks`,
    `FileOutline`, `FileRelated`, `FileSearchByType`.
  - Graph (10 new): `GraphListNodes`, `GraphNeighbors`, `GraphFindRelated`,
    `GraphFindPath`, `GraphCreateEdge`, `GraphDeleteEdge`, `GraphListEdges`,
    `GraphDiscoverEdges`, `GraphDiscoverEdgesForNode`, `GraphDiscoveryStatus`.
  - Admin (16 new): `AdminStats`, `AdminStatus`, `AdminLogs`,
    `AdminIndexingProgress`, `AdminConfigGet`, `AdminConfigUpdate`,
    `AdminBackupsList`, `AdminBackupsCreate`, `AdminBackupsRestore`,
    `AdminWorkspacesList`, `AdminWorkspaceGet`, `AdminWorkspaceAdd`,
    `AdminWorkspaceRemove`, `AdminRestart`, `AdminSlowQueriesList`,
    `AdminSlowQueriesConfig`.
  - Auth (11 new): `AuthMe`, `AuthLogout`, `AuthRefreshToken`,
    `AuthValidatePassword`, `AuthApiKeysCreate`, `AuthApiKeysList`,
    `AuthApiKeysRevoke`, `RotateApiKeyRpc`, `AuthApiKeysCreateScoped`,
    `AuthIntrospect`, `AuthAudit`.
  - Replication (4 new): `ReplicationStatus`, `ReplicationConfigure`,
    `ReplicationStats`, `ReplicationReplicasList`.
  - Cluster (5 new): `ClusterFailover`, `ClusterReplicaResync`,
    `ClusterPeerAdd`, `ClusterRebalance`, `ClusterRebalanceStatus`.

### Tests

- 9 new wire-shape test files exercise every new REST method via
  `httptest.NewServer`. All hermetic — `go test ./...` does not dial port
  15002 or 15503 (per spec scenario "Default Go test run is hermetic").
- 10 wire-shape unit tests in `rpc/commands_phase16_test.go` (one per
  domain group) using the in-process fake-server pattern.

### Changed

- `sdks/go/version.go` — `Version` constant bumped to `3.3.0`.

## [3.2.0] - 2026-05-01

### Added

- **Backpressure-aware HTTP client.** Honors the server-side bulk-
  upsert backpressure shipped in Vectorizer 3.2.0
  ([#263](https://github.com/hivellm/vectorizer/issues/263)). On HTTP
  `429 Too Many Requests` the client parses `Retry-After` (seconds
  form), sleeps, and retries — bounded by the same 3-attempt /
  30 s-cap / 1 s-default policy used by every other first-party SDK.
  After retry exhaustion a typed `*Error` carrying HTTP `Status: 429`
  is returned. Pre-3.2.0 clients bounced 429s into a generic 5xx and
  lost the retry budget. Implementation in `client.go::request` /
  `parseRetryAfterSeconds`; lock-in tests at `retry_after_test.go`.

### Changed

- Version bumped to 3.2.0 to track the server release.

## [3.1.0] - 2026-04-29

### Added

- **`InsertVectors(...)`** — bulk-insert pre-computed embeddings
  with caller-supplied vector ids. Skips the embedding pipeline
  entirely. Useful when the client already has its own embedder or
  wants idempotent re-ingest by stable id.
- **`Insert` / `InsertText` / `InsertTexts` accept `ID`** as the
  stored `Vector.ID`. Non-chunked inputs use the client `ID`
  verbatim; chunked inputs derive `<id>#<chunk_index>` (e.g.
  `doc:42#0`, `doc:42#1`). Re-running the same payload now upserts
  in place instead of duplicating.
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
separator). Violations return HTTP 400 with
`error_type: "validation_error"`.

## [3.0.0] - 2026-04-19

### Added

- **VectorizerRPC client** (new default transport in v3.x) at
  `rpc/`. Binary, length-prefixed MessagePack over raw TCP (port
  15503), spec at `docs/specs/VECTORIZER_RPC.md` in the parent
  repo. Polyglot parity with the Rust, Python, TypeScript, and C#
  SDKs.
  - `rpc.Client` (uses `net.TCPConn` + `vmihailenco/msgpack`)
    multiplexes calls on a single TCP connection by `Request.ID`.
  - `rpc.ConnectURL(ctx, url, opts)` — canonical URL parser shared
    with every other Vectorizer SDK. Accepts `vectorizer://host:port`,
    `vectorizer://host` (default port 15503), bare `host:port`, and
    `http(s)://host:port`. Rejects userinfo credentials.
  - `rpc.HelloPayload` / `rpc.HelloResponse` — sticky per-connection
    auth handshake.
  - `rpc.Pool` — minimal bounded connection pool with an RAII-style
    guard.
  - Typed wrappers: `ListCollections`, `GetCollectionInfo`,
    `GetVector`, `SearchBasic`. Match the polyglot SDK shapes.
  - New runnable example at `examples/rpc_quickstart/main.go`.
- New runtime dependency: `github.com/vmihailenco/msgpack/v5`.

### Changed

- Bumped to v3.0.0 to mark the new default transport. The legacy
  REST `vectorizer.Client` (over `net/http`) stays available
  unchanged.
- README rewritten with an RPC-first quickstart and a "Switching
  transports" matrix.

### Note

The package surface is **additive** for existing 2.x callers:
`vectorizer.Client` and every model still import from the same
paths. The 3.0 marker reflects that the recommended transport
changes — there is no forced migration of existing code.
