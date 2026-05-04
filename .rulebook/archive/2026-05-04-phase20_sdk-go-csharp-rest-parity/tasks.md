## 1. Inventory + shared types

- [x] 1.1 Lock the route inventory in `specs/sdk-go-csharp-rest-parity/spec.md` — one Requirement per route group, mirroring phase12-15 specs
- [x] 1.2 Cross-check `sdks/rust/src/client/` and `sdks/typescript/src/client/` to enumerate every REST method shipped in 3.4 → 3.7
- [x] 1.3 Record the canonical request/response shapes from the Rust SDK (source of truth) for every method in scope

## 2. Go SDK — shared models

- [x] 2.1 Add types to `sdks/go/models.go`: `Stats`, `ServerStatus`, `LogEntry`, `IndexingProgress`, `ConfigSnapshot`, `BackupInfo`, `ReplicationStatus`, `ReplicaInfo`, `ApiKey`, `User`, `WorkspaceConfig`
- [x] 2.2 Add tier-control types: `DeleteByFilterReport`, `BulkUpdateReport`, `CopyReport`, `ReencodeJob`, `TtlConfig`, `VectorExpiryRequest`
- [x] 2.3 Add schema-evolution types: `ReindexJob`, `SnapshotInfo`, `ExplainResponse`, `SlowQueryEntry`, `SlowQueryConfig`
- [x] 2.4 Add cluster + auth types: `FailoverReport`, `ResyncJob`, `PeerInfo`, `RebalanceJob`, `RebalanceStatus`, `RotatedKey`, `TokenIntrospection`, `AuditEntry`

## 3. Go SDK — admin / observability (`sdks/go/admin.go`)

- [x] 3.1 `GetStats(ctx) (Stats, error)`
- [x] 3.2 `GetStatus(ctx) (ServerStatus, error)`
- [x] 3.3 `GetLogs(ctx, params) ([]LogEntry, error)`
- [x] 3.4 `GetIndexingProgress(ctx) (IndexingProgress, error)`
- [x] 3.5 `ForceSaveCollection(ctx, name) error`
- [x] 3.6 `ListEmptyCollections(ctx) ([]string, error)`
- [x] 3.7 `CleanupEmptyCollections(ctx) (CleanupReport, error)`
- [x] 3.8 `GetConfig(ctx) (ConfigSnapshot, error)`
- [x] 3.9 `UpdateConfig(ctx, patch) (ConfigSnapshot, error)`
- [x] 3.10 `ListBackups(ctx) ([]BackupInfo, error)`
- [x] 3.11 `CreateBackup(ctx, req) (BackupInfo, error)`
- [x] 3.12 `RestoreBackup(ctx, req) error`
- [x] 3.13 `RestartServer(ctx) error`
- [x] 3.14 `ListWorkspaces(ctx) ([]WorkspaceConfig, error)`
- [x] 3.15 `GetWorkspaceConfig(ctx) (WorkspaceConfig, error)`
- [x] 3.16 `AddWorkspace(ctx, req) error`
- [x] 3.17 `RemoveWorkspace(ctx, name) error`

## 4. Go SDK — auth surface (`sdks/go/auth.go`)

- [x] 4.1 `Me(ctx) (User, error)`
- [x] 4.2 `Logout(ctx) error`
- [x] 4.3 `RefreshToken(ctx) (JwtToken, error)`
- [x] 4.4 `ValidatePassword(ctx, password) (PasswordPolicyReport, error)`
- [x] 4.5 `CreateApiKey(ctx, req) (ApiKey, error)`
- [x] 4.6 `ListApiKeys(ctx) ([]ApiKey, error)`
- [x] 4.7 `RevokeApiKey(ctx, id) error`
- [x] 4.8 `CreateUser(ctx, req) (User, error)`
- [x] 4.9 `ListUsers(ctx) ([]User, error)`
- [x] 4.10 `DeleteUser(ctx, username) error`
- [x] 4.11 `ChangePassword(ctx, username, newPassword) error`

## 5. Go SDK — replication (`sdks/go/replication.go`)

- [x] 5.1 `GetReplicationStatus(ctx) (ReplicationStatus, error)`
- [x] 5.2 `ConfigureReplication(ctx, config) error`
- [x] 5.3 `GetReplicationStats(ctx) (ReplicationStats, error)`
- [x] 5.4 `ListReplicas(ctx) ([]ReplicaInfo, error)`

## 6. Go SDK — hub (`sdks/go/hub.go`)

- [x] 6.1 `ListUserBackups`, `CreateUserBackup`, `RestoreUserBackup`, `UploadUserBackup`, `GetUserBackup`, `DeleteUserBackup`, `DownloadUserBackup`
- [x] 6.2 `GetUsageStatistics`, `GetQuotaInfo`, `ValidateApiKey`

## 7. Go SDK — discovery pipeline (`sdks/go/discovery_pipeline.go`)

- [x] 7.1 `BroadDiscovery(ctx, req) (BroadDiscoveryResponse, error)`
- [x] 7.2 `SemanticFocus(ctx, req) (SemanticFocusResponse, error)`
- [x] 7.3 `PromoteReadme(ctx, req) (PromoteReadmeResponse, error)`
- [x] 7.4 `CompressEvidence(ctx, req) (CompressEvidenceResponse, error)`
- [x] 7.5 `BuildAnswerPlan(ctx, req) (AnswerPlan, error)`
- [x] 7.6 `RenderLlmPrompt(ctx, req) (LlmPrompt, error)`

## 8. Go SDK — vectors single + batch + search variants

- [x] 8.1 `UpdateVector(ctx, collection, id, req)` — POST /update
- [x] 8.2 `InsertText(ctx, collection, id, text, metadata)` — POST /insert
- [x] 8.3 `ListVectors(ctx, collection, page, limit)` — GET /collections/{name}/vectors
- [x] 8.4 `GetVectorByPath(ctx, collection, id)` — GET /collections/{name}/vectors/{id}
- [x] 8.5 `BatchInsertTexts(ctx, collection, items)` — POST /batch_insert
- [x] 8.6 `InsertVectors(ctx, collection, vectors)` — POST /insert_vectors
- [x] 8.7 `BatchSearch(ctx, requests)` — POST /batch_search
- [x] 8.8 `BatchUpdateVectors(ctx, collection, updates)` — POST /batch_update
- [x] 8.9 `SearchVectorsByText(ctx, collection, query, limit)` — POST /collections/{n}/search/text
- [x] 8.10 `SearchByFile(ctx, collection, req)` — POST /collections/{n}/search/file

## 9. Go SDK — tier-control (`sdks/go/tier_control.go`)

- [x] 9.1 `DeleteByFilter(ctx, collection, filter) (DeleteByFilterReport, error)`
- [x] 9.2 `BulkUpdateMetadata(ctx, collection, filter, patch) (BulkUpdateReport, error)`
- [x] 9.3 `CopyVectors(ctx, src, dst, ids) (CopyReport, error)`
- [x] 9.4 `ReencodeCollection(ctx, name, targetEncoding) (ReencodeJob, error)`
- [x] 9.5 `SetCollectionTtl(ctx, name, ttlSecs) error`
- [x] 9.6 `SetVectorExpiry(ctx, collection, id, expiresAt) error`

## 10. Go SDK — schema evolution + explain + slow queries (`sdks/go/schema_evolution.go`)

- [x] 10.1 `RenameCollection(ctx, old, new) error`
- [x] 10.2 `ReindexCollection(ctx, name, params) (ReindexJob, error)`
- [x] 10.3 `SnapshotCollectionNative(ctx, name, req) (SnapshotInfo, error)`
- [x] 10.4 `ListCollectionSnapshotsNative(ctx, name) ([]SnapshotInfo, error)`
- [x] 10.5 `RestoreCollectionSnapshotNative(ctx, name, id) error`
- [x] 10.6 `ExplainSearch(ctx, collection, req) (ExplainResponse, error)`
- [x] 10.7 `ListSlowQueries(ctx, params) ([]SlowQueryEntry, error)`
- [x] 10.8 `SetSlowQueryConfig(ctx, config) (SlowQueryConfig, error)`

## 11. Go SDK — cluster + auth admin (`sdks/go/cluster_admin.go`)

- [x] 11.1 `ClusterFailover(ctx, replicaId) (FailoverReport, error)`
- [x] 11.2 `ClusterResyncReplica(ctx, replicaId) (ResyncJob, error)`
- [x] 11.3 `ClusterAddPeer(ctx, req) (PeerInfo, error)`
- [x] 11.4 `ClusterRebalance(ctx) (RebalanceJob, error)`
- [x] 11.5 `ClusterRebalanceStatus(ctx) (RebalanceStatus, error)`
- [x] 11.6 `RotateApiKey(ctx, id) (RotatedKey, error)`
- [x] 11.7 `CreateScopedApiKey(ctx, req) (ApiKey, error)`
- [x] 11.8 `IntrospectToken(ctx, token) (TokenIntrospection, error)`
- [x] 11.9 `ListAuditLog(ctx, params) ([]AuditEntry, error)`

## 12. Go SDK — version + tests

- [x] 12.1 Bump `sdks/go/version.go` constant `Version` to `3.9.0`
- [x] 12.2 Bump `sdks/go/go.mod` module path version comment if present
- [x] 12.3 Wire-shape unit tests per method file (`*_test.go`) using `httptest.NewServer` and the existing pattern from `qdrant_test.go`
- [x] 12.4 Live-server integration tests under build tag `//go:build s2s` mirroring the Rust `s2s-tests` feature
- [x] 12.5 `go build ./...` and `go test ./...` exit 0

## 13. C# SDK — shared DTOs (`sdks/csharp/src/Models/`)

- [x] 13.1 Add admin/observability DTOs: `Stats`, `ServerStatus`, `LogEntry`, `IndexingProgress`, `ConfigSnapshot`, `BackupInfo`, `WorkspaceConfig`
- [x] 13.2 Add auth DTOs: `ApiKey`, `User`, `JwtToken`, `PasswordPolicyReport`, `RotatedKey`, `TokenIntrospection`, `AuditEntry`
- [x] 13.3 Add replication DTOs: `ReplicationStatus`, `ReplicaInfo`, `ReplicationStats`
- [x] 13.4 Add tier-control DTOs: `DeleteByFilterReport`, `BulkUpdateReport`, `CopyReport`, `ReencodeJob`, `TtlConfig`, `VectorExpiryRequest`
- [x] 13.5 Add schema-evolution DTOs: `ReindexJob`, `SnapshotInfo`, `ExplainResponse`, `SlowQueryEntry`, `SlowQueryConfig`
- [x] 13.6 Add cluster DTOs: `FailoverReport`, `ResyncJob`, `PeerInfo`, `RebalanceJob`, `RebalanceStatus`

## 14. C# SDK — admin (`sdks/csharp/src/Admin.cs`)

- [x] 14.1 Mirror Go §3 (17 methods) on `HttpVectorizerClient` as `async Task<T>`
- [x] 14.2 Expose via `IVectorizerClient` interface
- [x] 14.3 XML-doc each public member

## 15. C# SDK — auth (`sdks/csharp/src/Auth.cs`)

- [x] 15.1 Mirror Go §4 (11 methods)
- [x] 15.2 Expose via `IVectorizerClient`
- [x] 15.3 XML-doc each public member

## 16. C# SDK — replication (`sdks/csharp/src/Replication.cs`)

- [x] 16.1 Mirror Go §5 (4 methods)
- [x] 16.2 Expose via `IVectorizerClient`
- [x] 16.3 XML-doc each public member

## 17. C# SDK — hub (`sdks/csharp/src/Hub.cs`)

- [x] 17.1 Mirror Go §6 (10 methods)
- [x] 17.2 Expose via `IVectorizerClient`
- [x] 17.3 XML-doc each public member

## 18. C# SDK — discovery pipeline (`sdks/csharp/src/DiscoveryPipeline.cs`)

- [x] 18.1 Mirror Go §7 (6 methods)
- [x] 18.2 Expose via `IVectorizerClient`
- [x] 18.3 XML-doc each public member

## 19. C# SDK — vectors single + batch + search variants

- [x] 19.1 Mirror Go §8 (10 methods) on `HttpVectorizerClient` and `BatchOperations.cs`
- [x] 19.2 Expose via `IVectorizerClient`
- [x] 19.3 XML-doc each public member

## 20. C# SDK — tier-control (`sdks/csharp/src/TierControl.cs`)

- [x] 20.1 Mirror Go §9 (6 methods)
- [x] 20.2 Expose via `IVectorizerClient`
- [x] 20.3 XML-doc each public member

## 21. C# SDK — schema evolution + explain + slow queries (`sdks/csharp/src/SchemaEvolution.cs`)

- [x] 21.1 Mirror Go §10 (8 methods)
- [x] 21.2 Expose via `IVectorizerClient`
- [x] 21.3 XML-doc each public member

## 22. C# SDK — cluster + auth admin (`sdks/csharp/src/ClusterAdmin.cs`)

- [x] 22.1 Mirror Go §11 (9 methods)
- [x] 22.2 Expose via `IVectorizerClient`
- [x] 22.3 XML-doc each public member

## 23. C# SDK — version + tests

- [x] 23.1 Bump `sdks/csharp/Vectorizer.csproj` `<Version>` to `3.9.0`
- [x] 23.2 Bump `sdks/csharp/src/Vectorizer.Rpc/Vectorizer.Rpc.csproj` `<Version>` to `3.9.0`
- [x] 23.3 xUnit wire-shape tests per surface file in `sdks/csharp/Vectorizer.Tests/`
- [x] 23.4 Live-server integration tests under `[Trait("Category","s2s")]`
- [x] 23.5 `dotnet build` and `dotnet test --filter "Category!=s2s"` exit 0

## 24. Documentation

- [x] 24.1 Extend `docs/users/api/API_REFERENCE.md` "SDK 3.4-3.7 control surface" tables with Go and C# columns
- [x] 24.2 Add per-domain examples (admin, auth, replication, tier-control, schema-evolution, cluster) to `sdks/go/README.md`
- [x] 24.3 Add per-domain examples to `sdks/csharp/README.md`
- [x] 24.4 CHANGELOG entry in `sdks/go/CHANGELOG.md` under `## [3.9.0]` listing every new method
- [x] 24.5 CHANGELOG entry in `sdks/csharp/CHANGELOG.md` under `## 3.9.0` listing every new method
- [x] 24.6 Update `sdks/COVERAGE_REPORT.md` to reflect Go and C# REST parity (remove the `🚧 partial` markers once tests land)

## 25. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 25.1 Update or create documentation covering the implementation
- [x] 25.2 Write tests covering the new behavior
- [x] 25.3 Run tests and confirm they pass
