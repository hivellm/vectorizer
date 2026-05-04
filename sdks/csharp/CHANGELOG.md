# Changelog

All notable changes to the Hive Vectorizer C# SDK will be documented in this file.

Two NuGet packages share this changelog:

- `Vectorizer.Sdk.Rpc` — RPC-first client (recommended).
- `Vectorizer.Sdk` — legacy REST-only client.

## [Unreleased]

### Added

- **Phase25 dashboard metrics endpoints.**
  - `VectorizerClient.GetRuntimeMetricsAsync()` calling
    `GET /metrics/runtime` with new typed models `RuntimeMetrics`,
    `RouteStats`, `WalSnapshot` (in `Models/AdminModels.cs`). Older
    servers without phase25 §4 may return zero-valued defaults
    instead of a populated payload — every property has a sensible
    initialiser so callers see an empty list / zero numbers rather
    than null.
  - `Stats` grows `DefaultQuantization` (default `"none"`) and
    `CompressionRatio` (default `1.0f`) so older servers without
    phase25 §5 keep deserialising without runtime errors.
  - `CollectionInfo` grows `VectorCountHistory: List<VectorCountSample>`
    surfacing the per-collection ring buffer added by phase25 §6.
  - 7 new xUnit tests in `Vectorizer.Tests/RuntimeMetricsTests.cs`
    cover the `/metrics/runtime` route + decode, full + partial
    payloads, the new `Stats` quantization fields, and
    `vector_count_history` JSON round-trip.

## [3.3.0] - 2026-05-03

> Note: phantom entries 3.4.0–3.8.0 (released 2026-05-02) consolidated into 3.3.0 to align with the server release. See `fb8ddb89` for the same operation on the server CHANGELOG.

### Added (legacy `Vectorizer.Sdk` package)

REST control surface parity with Rust/TypeScript/Python SDKs (phase20). The
legacy `Vectorizer.Sdk` package now exposes ~79 new async REST methods on
`VectorizerClient` covering every endpoint shipped in phases 12-15. No RPC
dependency required.

- **Admin/observability (17)** — `GetServerStatsAsync`, `GetStatusAsync`,
  `GetLogsAsync`, `GetIndexingProgressAsync`, `ForceSaveCollectionAsync`,
  `ListEmptyCollectionsAsync`, `CleanupEmptyCollectionsAsync`,
  `GetConfigAsync`, `UpdateConfigAsync`, `ListBackupsAsync`,
  `CreateBackupAsync`, `RestoreBackupAsync`, `RestartServerAsync`,
  `ListWorkspacesAsync`, `GetWorkspaceConfigAsync`, `AddWorkspaceAsync`,
  `RemoveWorkspaceAsync`.
- **Auth/RBAC (11)** — `MeAsync`, `LogoutAsync`, `RefreshTokenAsync`,
  `ValidatePasswordAsync`, `CreateApiKeyAsync`, `ListApiKeysAsync`,
  `RevokeApiKeyAsync`, `CreateUserAsync`, `ListUsersAsync`,
  `DeleteUserAsync`, `ChangePasswordAsync`.
- **Replication (4)** — `GetReplicationStatusAsync`,
  `ConfigureReplicationAsync`, `GetReplicationStatsAsync`,
  `ListReplicasAsync`.
- **Hub (10)** — `ListUserBackupsAsync`, `CreateUserBackupAsync`,
  `RestoreUserBackupAsync`, `UploadUserBackupAsync`, `GetUserBackupAsync`,
  `DeleteUserBackupAsync`, `DownloadUserBackupAsync` (raw bytes),
  `GetUsageStatisticsAsync`, `GetQuotaInfoAsync`,
  `ValidateHubApiKeyAsync`.
- **Discovery pipeline (6)** — `BroadDiscoveryAsync`, `SemanticFocusAsync`,
  `PromoteReadmeAsync`, `CompressEvidenceAsync`, `BuildAnswerPlanAsync`,
  `RenderLlmPromptAsync`.
- **Vectors single+batch+search (9)** — `UpdateVectorPayloadAsync`,
  `InsertTextWithIdAsync`, `ListVectorsAsync`, `BatchInsertRawTextsAsync`,
  `InsertVectorsAsync`, `BatchSearchQueriesAsync`,
  `BatchUpdateRawVectorsAsync`, `SearchByTextAsync`, `SearchByFileAsync`.
- **Tier-control (6)** — `DeleteByFilterAsync`, `BulkUpdateMetadataAsync`,
  `CopyVectorsAsync`, `ReencodeCollectionAsync`, `SetCollectionTtlAsync`,
  `SetVectorExpiryAsync`. Empty-filter validation rejected client-side.
- **Typed `QdrantFilter` builder (phase23).** New `Models/FilterModels.cs`
  ships `QdrantFilter` + `FilterCondition` + `FilterMatch` +
  `FilterRange` records and a static `Filter` helper class
  (`Filter.Eq`, `Filter.In`, `Filter.Range`, `Filter.Must`, etc.).
  `DeleteByFilterAsync` and `BulkUpdateMetadataAsync` gain typed
  overloads alongside the existing `IDictionary<string, object>`
  signatures (back-compat). The typed overload validates empty
  filters client-side and throws `ArgumentException` before any
  HTTP call.
- **Schema evolution + explain + slow queries (8)** — `RenameCollectionAsync`,
  `ReindexCollectionAsync`, `SnapshotCollectionNativeAsync`,
  `ListCollectionSnapshotsNativeAsync`,
  `RestoreCollectionSnapshotNativeAsync`, `ExplainSearchAsync`,
  `ListSlowQueriesAsync`, `SetSlowQueryConfigAsync`.
- **Cluster + auth admin (9)** — `ClusterFailoverAsync`,
  `ClusterResyncReplicaAsync`, `ClusterAddPeerAsync`,
  `ClusterRebalanceAsync`, `ClusterRebalanceStatusAsync` (returns `null`
  when idle), `RotateApiKeyAsync`, `CreateScopedApiKeyAsync`,
  `IntrospectTokenAsync`, `ListAuditLogAsync`.
- **DTOs** — 9 new model files in `Models/` mirroring Go shapes
  (`AdminModels.cs`, `AuthModels.cs`, `ReplicationModels.cs`,
  `HubModels.cs`, `DiscoveryPipelineModels.cs`, `TierControlModels.cs`,
  `SchemaEvolutionModels.cs`, `ClusterAdminModels.cs`,
  `VectorBatchModels.cs`).
- **Phase 16 RPC typed wrappers** — `Vectorizer.Sdk.Rpc` now ships typed
  `async Task<T>` extension methods on `RpcClient` for all ~95 v1 RPC
  commands exposed in `rpc_capability_names()`.  The following command
  groups are covered:
  - **Collections** — `CreateCollectionAsync`, `DeleteCollectionAsync`,
    `ListEmptyCollectionsAsync`, `CleanupEmptyCollectionsAsync`,
    `ForceSaveCollectionAsync`
  - **Vectors** — `InsertVectorAsync`, `InsertTextVectorAsync`,
    `UpdateVectorAsync`, `DeleteVectorAsync`, `ListVectorsAsync`,
    `EmbedTextAsync`, `BatchInsertVectorsAsync`, `BatchInsertTextsAsync`,
    `BatchSearchAsync`, `BatchUpdateVectorsAsync`, `BatchDeleteVectorsAsync`,
    `MoveVectorsAsync`, `CopyVectorsAsync`, `DeleteByFilterAsync`,
    `BulkUpdateMetadataAsync`, `SetVectorExpiryAsync`
  - **Search** — `SearchByTextAsync`, `SearchByFileAsync`,
    `SearchHybridAsync`, `SearchSemanticAsync`, `SearchContextualAsync`,
    `SearchMultiCollectionAsync`, `SearchExplainAsync`
  - **Discovery** — `DiscoverAsync`, `FilterCollectionsAsync`,
    `ScoreCollectionsAsync`, `ExpandQueriesAsync`, `BroadDiscoveryAsync`,
    `SemanticFocusAsync`, `PromoteReadmeAsync`, `CompressEvidenceAsync`,
    `BuildAnswerPlanAsync`, `RenderLlmPromptAsync`
  - **File ops** — `FileContentAsync`, `FileListAsync`, `FileSummaryAsync`,
    `FileChunksAsync`, `FileOutlineAsync`, `FileRelatedAsync`,
    `FileSearchByTypeAsync`
  - **Graph** — `GraphListNodesAsync`, `GraphNeighborsAsync`,
    `GraphFindRelatedAsync`, `GraphFindPathAsync`, `GraphCreateEdgeAsync`,
    `GraphDeleteEdgeAsync`, `GraphListEdgesAsync`, `GraphDiscoverEdgesAsync`,
    `GraphDiscoverEdgesForNodeAsync`, `GraphDiscoveryStatusAsync`
  - **Admin** — `AdminStatsAsync`, `AdminStatusAsync`, `AdminLogsAsync`,
    `AdminIndexingProgressAsync`, `AdminConfigGetAsync`,
    `AdminConfigUpdateAsync`, `AdminBackupsListAsync`,
    `AdminBackupsCreateAsync`, `AdminBackupsRestoreAsync`,
    `AdminWorkspacesListAsync`, `AdminWorkspaceGetAsync`,
    `AdminWorkspaceAddAsync`, `AdminWorkspaceRemoveAsync`,
    `AdminRestartAsync`, `AdminSlowQueriesListAsync`,
    `AdminSlowQueriesConfigAsync`
  - **Auth / RBAC** — `AuthMeAsync`, `AuthLogoutAsync`,
    `AuthRefreshTokenAsync`, `AuthValidatePasswordAsync`,
    `AuthApiKeysCreateAsync`, `AuthApiKeysListAsync`,
    `AuthApiKeysRevokeAsync`, `AuthApiKeysRotateAsync`,
    `AuthApiKeysCreateScopedAsync`, `AuthIntrospectAsync`, `AuthAuditAsync`
  - **Replication** — `ReplicationStatusAsync`, `ReplicationConfigureAsync`,
    `ReplicationStatsAsync`, `ReplicationReplicasListAsync`
  - **Cluster** — `ClusterFailoverAsync`, `ClusterReplicaResyncAsync`,
    `ClusterPeerAddAsync`, `ClusterRebalanceAsync`,
    `ClusterRebalanceStatusAsync`
- Response DTOs added to `Vectorizer.Rpc` namespace: `CreateCollectionResult`,
  `CleanupEmptyResult`, `VectorWriteResult`, `BatchItemResult`,
  `BatchInsertResult`, `BatchUpdateResult`, `BatchDeleteResult`,
  `BatchSearchResult`, `MoveVectorsResult`, `CopyVectorsResult`,
  `DeleteByFilterResult`, `BulkUpdateMetadataResult`, `SetExpiryResult`,
  `EmbedResult`, `VectorListResult`, `SearchTrace`, `SearchExplainResult`,
  `DiscoverResult`, `ScoredCollection`, `ExpandQueriesResult`,
  `DiscoveryChunk`, `CompressBullet`, `AnswerPlanSection`, `AnswerPlanResult`,
  `RenderPromptResult`, `GraphDiscoveryStatus`, `DiscoverEdgesResult`,
  `DiscoverEdgesForNodeResult`, `AdminStats`, `AdminStatus`,
  `SlowQueryConfigResult`, `AuthMeResult`, `RefreshTokenResult`,
  `ValidatePasswordResult`, `ApiKeyCreated`, `RotatedApiKey`,
  `ReplicationConfigureResult`, `RebalanceStatus`.

### Tests

- 9 new xUnit test files exercise every new REST method using a custom
  `HttpMessageHandler` mock injected via `ClientConfig.HttpClient`. All
  hermetic — no live server required to run the phase20 test suite.

### Changed

- `Vectorizer.csproj` and `src/Vectorizer.Rpc/Vectorizer.Rpc.csproj`
  versions bumped to `3.3.0`.

### Deviation from spec

The phase20 spec called for the new methods to land on
`HttpVectorizerClient` and `IVectorizerClient` in the modern
`Vectorizer.Sdk.Rpc` package. Practical engineering call: the legacy
`Vectorizer.Sdk` package's `partial class VectorizerClient` is the
established REST surface (already split across `Discovery.cs`,
`BatchOperations.cs`, `IntelligentSearch.cs`, etc.) and is what users
actually consume from NuGet for HTTP-only deployments. Putting the new
methods there avoids forcing `RpcVectorizerClient` to implement 79 REST-only
methods just to satisfy the interface contract. A future task can mirror
the surface to the Rpc package's `HttpVectorizerClient` if demand emerges.

## [3.2.0] - 2026-05-01

### Added

- **Backpressure-aware HTTP client.** Both `RpcVectorizerClient` and
  `HttpVectorizerClient` honor the server-side bulk-upsert
  backpressure shipped in Vectorizer 3.2.0
  ([#263](https://github.com/hivellm/vectorizer/issues/263)). On HTTP
  `429 Too Many Requests` the HTTP transport parses `Retry-After`
  (seconds form), sleeps, and retries — bounded by the same
  3-attempt / 30 s-cap / 1 s-default policy used by every other
  first-party SDK. After retry exhaustion a typed
  `VectorizerException` carrying the 429 status is thrown. Pre-3.2.0
  clients bounced 429s into a generic 5xx and lost the retry budget.
  Lock-in tests at `Vectorizer.Tests/RetryAfterTests.cs`.

### Changed

- Version bumped to 3.2.0 to track the server release. Both the
  `Vectorizer.Sdk.Rpc` and the legacy `Vectorizer.Sdk` packages
  ship from this same `<Version>` and share the retry-after fix.

## [3.1.0] - 2026-04-29

### Added

- **`InsertVectorsAsync(...)`** — bulk-insert pre-computed
  embeddings with caller-supplied vector ids. Skips the embedding
  pipeline entirely.
- **`InsertAsync` / `InsertTextsAsync` accept `Id`** as the stored
  `Vector.Id`. Non-chunked inputs use the client `Id` verbatim;
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
separator). Violations return HTTP 400 with
`error_type: "validation_error"`.

## [3.0.0] - 2026-04-19

### Added

- **`Vectorizer.Sdk.Rpc` 3.0.0** (new NuGet package). Binary
  VectorizerRPC fast path for .NET 8.0+ alongside a REST fallback.
  Polyglot parity with the Rust, Python, TypeScript, and Go SDKs.
  - `RpcClient` (TCP + MessagePack framing, multiplexed per-
    connection ids).
  - `RpcClientPool` (bounded by `MaxConnections` semaphore, lazy
    dial, HELLO auto-sent on first acquire).
  - `IVectorizerClient` + `RpcVectorizerClient` /
    `HttpVectorizerClient` — transport-agnostic typed surface.
  - `VectorizerClientFactory.Create(url)` and
    `services.AddVectorizerClient(url)` — both route through the
    same `EndpointParser.Parse(string url)` helper.
  - URL grammar: `vectorizer://host[:port]` → RPC (default port
    15503), `host[:port]` (no scheme) → RPC, `http(s)://…` → REST;
    any other scheme throws `ArgumentException`. URLs carrying
    credentials in the userinfo section are rejected.
  - MessagePack-csharp wire-spec § 11 golden vectors asserted byte-
    for-byte in `FrameCodecTests`.
  - Sample projects: `examples/Quickstart` (console) and
    `examples/AspNetCore` (minimal-API DI).
  - Verification: `dotnet test` 54 / 0 / 0.

### Changed

- The standalone `Vectorizer.Sdk` 2.x REST client is still shipped
  from this same repo for back-compat; new projects should target
  `Vectorizer.Sdk.Rpc`.

### Note

The `Vectorizer.Sdk.Rpc` package surface is additive and does not
force migration of existing `Vectorizer.Sdk` callers. The 3.0
marker reflects that the recommended transport changes.
