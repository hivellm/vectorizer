## 1. Inventory + shared types

- [ ] 1.1 Lock the route inventory in `specs/sdk-go-csharp-rest-parity/spec.md` — one Requirement per route group, mirroring phase12-15 specs
- [ ] 1.2 Cross-check `sdks/rust/src/client/` and `sdks/typescript/src/client/` to enumerate every REST method shipped in 3.4 → 3.7
- [ ] 1.3 Record the canonical request/response shapes from the Rust SDK (source of truth) for every method in scope

## 2. Go SDK — shared models

- [ ] 2.1 Add types to `sdks/go/models.go`: `Stats`, `ServerStatus`, `LogEntry`, `IndexingProgress`, `ConfigSnapshot`, `BackupInfo`, `ReplicationStatus`, `ReplicaInfo`, `ApiKey`, `User`, `WorkspaceConfig`
- [ ] 2.2 Add tier-control types: `DeleteByFilterReport`, `BulkUpdateReport`, `CopyReport`, `ReencodeJob`, `TtlConfig`, `VectorExpiryRequest`
- [ ] 2.3 Add schema-evolution types: `ReindexJob`, `SnapshotInfo`, `ExplainResponse`, `SlowQueryEntry`, `SlowQueryConfig`
- [ ] 2.4 Add cluster + auth types: `FailoverReport`, `ResyncJob`, `PeerInfo`, `RebalanceJob`, `RebalanceStatus`, `RotatedKey`, `TokenIntrospection`, `AuditEntry`

## 3. Go SDK — admin / observability (`sdks/go/admin.go`)

- [ ] 3.1 `GetStats(ctx) (Stats, error)`
- [ ] 3.2 `GetStatus(ctx) (ServerStatus, error)`
- [ ] 3.3 `GetLogs(ctx, params) ([]LogEntry, error)`
- [ ] 3.4 `GetIndexingProgress(ctx) (IndexingProgress, error)`
- [ ] 3.5 `ForceSaveCollection(ctx, name) error`
- [ ] 3.6 `ListEmptyCollections(ctx) ([]string, error)`
- [ ] 3.7 `CleanupEmptyCollections(ctx) (CleanupReport, error)`
- [ ] 3.8 `GetConfig(ctx) (ConfigSnapshot, error)`
- [ ] 3.9 `UpdateConfig(ctx, patch) (ConfigSnapshot, error)`
- [ ] 3.10 `ListBackups(ctx) ([]BackupInfo, error)`
- [ ] 3.11 `CreateBackup(ctx, req) (BackupInfo, error)`
- [ ] 3.12 `RestoreBackup(ctx, req) error`
- [ ] 3.13 `RestartServer(ctx) error`
- [ ] 3.14 `ListWorkspaces(ctx) ([]WorkspaceConfig, error)`
- [ ] 3.15 `GetWorkspaceConfig(ctx) (WorkspaceConfig, error)`
- [ ] 3.16 `AddWorkspace(ctx, req) error`
- [ ] 3.17 `RemoveWorkspace(ctx, name) error`

## 4. Go SDK — auth surface (`sdks/go/auth.go`)

- [ ] 4.1 `Me(ctx) (User, error)`
- [ ] 4.2 `Logout(ctx) error`
- [ ] 4.3 `RefreshToken(ctx) (JwtToken, error)`
- [ ] 4.4 `ValidatePassword(ctx, password) (PasswordPolicyReport, error)`
- [ ] 4.5 `CreateApiKey(ctx, req) (ApiKey, error)`
- [ ] 4.6 `ListApiKeys(ctx) ([]ApiKey, error)`
- [ ] 4.7 `RevokeApiKey(ctx, id) error`
- [ ] 4.8 `CreateUser(ctx, req) (User, error)`
- [ ] 4.9 `ListUsers(ctx) ([]User, error)`
- [ ] 4.10 `DeleteUser(ctx, username) error`
- [ ] 4.11 `ChangePassword(ctx, username, newPassword) error`

## 5. Go SDK — replication (`sdks/go/replication.go`)

- [ ] 5.1 `GetReplicationStatus(ctx) (ReplicationStatus, error)`
- [ ] 5.2 `ConfigureReplication(ctx, config) error`
- [ ] 5.3 `GetReplicationStats(ctx) (ReplicationStats, error)`
- [ ] 5.4 `ListReplicas(ctx) ([]ReplicaInfo, error)`

## 6. Go SDK — hub (`sdks/go/hub.go`)

- [ ] 6.1 `ListUserBackups`, `CreateUserBackup`, `RestoreUserBackup`, `UploadUserBackup`, `GetUserBackup`, `DeleteUserBackup`, `DownloadUserBackup`
- [ ] 6.2 `GetUsageStatistics`, `GetQuotaInfo`, `ValidateApiKey`

## 7. Go SDK — discovery pipeline (`sdks/go/discovery_pipeline.go`)

- [ ] 7.1 `BroadDiscovery(ctx, req) (BroadDiscoveryResponse, error)`
- [ ] 7.2 `SemanticFocus(ctx, req) (SemanticFocusResponse, error)`
- [ ] 7.3 `PromoteReadme(ctx, req) (PromoteReadmeResponse, error)`
- [ ] 7.4 `CompressEvidence(ctx, req) (CompressEvidenceResponse, error)`
- [ ] 7.5 `BuildAnswerPlan(ctx, req) (AnswerPlan, error)`
- [ ] 7.6 `RenderLlmPrompt(ctx, req) (LlmPrompt, error)`

## 8. Go SDK — vectors single + batch + search variants

- [ ] 8.1 `UpdateVector(ctx, collection, id, req)` — POST /update
- [ ] 8.2 `InsertText(ctx, collection, id, text, metadata)` — POST /insert
- [ ] 8.3 `ListVectors(ctx, collection, page, limit)` — GET /collections/{name}/vectors
- [ ] 8.4 `GetVectorByPath(ctx, collection, id)` — GET /collections/{name}/vectors/{id}
- [ ] 8.5 `BatchInsertTexts(ctx, collection, items)` — POST /batch_insert
- [ ] 8.6 `InsertVectors(ctx, collection, vectors)` — POST /insert_vectors
- [ ] 8.7 `BatchSearch(ctx, requests)` — POST /batch_search
- [ ] 8.8 `BatchUpdateVectors(ctx, collection, updates)` — POST /batch_update
- [ ] 8.9 `SearchVectorsByText(ctx, collection, query, limit)` — POST /collections/{n}/search/text
- [ ] 8.10 `SearchByFile(ctx, collection, req)` — POST /collections/{n}/search/file

## 9. Go SDK — tier-control (`sdks/go/tier_control.go`)

- [ ] 9.1 `DeleteByFilter(ctx, collection, filter) (DeleteByFilterReport, error)`
- [ ] 9.2 `BulkUpdateMetadata(ctx, collection, filter, patch) (BulkUpdateReport, error)`
- [ ] 9.3 `CopyVectors(ctx, src, dst, ids) (CopyReport, error)`
- [ ] 9.4 `ReencodeCollection(ctx, name, targetEncoding) (ReencodeJob, error)`
- [ ] 9.5 `SetCollectionTtl(ctx, name, ttlSecs) error`
- [ ] 9.6 `SetVectorExpiry(ctx, collection, id, expiresAt) error`

## 10. Go SDK — schema evolution + explain + slow queries (`sdks/go/schema_evolution.go`)

- [ ] 10.1 `RenameCollection(ctx, old, new) error`
- [ ] 10.2 `ReindexCollection(ctx, name, params) (ReindexJob, error)`
- [ ] 10.3 `SnapshotCollectionNative(ctx, name, req) (SnapshotInfo, error)`
- [ ] 10.4 `ListCollectionSnapshotsNative(ctx, name) ([]SnapshotInfo, error)`
- [ ] 10.5 `RestoreCollectionSnapshotNative(ctx, name, id) error`
- [ ] 10.6 `ExplainSearch(ctx, collection, req) (ExplainResponse, error)`
- [ ] 10.7 `ListSlowQueries(ctx, params) ([]SlowQueryEntry, error)`
- [ ] 10.8 `SetSlowQueryConfig(ctx, config) (SlowQueryConfig, error)`

## 11. Go SDK — cluster + auth admin (`sdks/go/cluster_admin.go`)

- [ ] 11.1 `ClusterFailover(ctx, replicaId) (FailoverReport, error)`
- [ ] 11.2 `ClusterResyncReplica(ctx, replicaId) (ResyncJob, error)`
- [ ] 11.3 `ClusterAddPeer(ctx, req) (PeerInfo, error)`
- [ ] 11.4 `ClusterRebalance(ctx) (RebalanceJob, error)`
- [ ] 11.5 `ClusterRebalanceStatus(ctx) (RebalanceStatus, error)`
- [ ] 11.6 `RotateApiKey(ctx, id) (RotatedKey, error)`
- [ ] 11.7 `CreateScopedApiKey(ctx, req) (ApiKey, error)`
- [ ] 11.8 `IntrospectToken(ctx, token) (TokenIntrospection, error)`
- [ ] 11.9 `ListAuditLog(ctx, params) ([]AuditEntry, error)`

## 12. Go SDK — version + tests

- [ ] 12.1 Bump `sdks/go/version.go` constant `Version` to `3.9.0`
- [ ] 12.2 Bump `sdks/go/go.mod` module path version comment if present
- [ ] 12.3 Wire-shape unit tests per method file (`*_test.go`) using `httptest.NewServer` and the existing pattern from `qdrant_test.go`
- [ ] 12.4 Live-server integration tests under build tag `//go:build s2s` mirroring the Rust `s2s-tests` feature
- [ ] 12.5 `go build ./...` and `go test ./...` exit 0

## 13. C# SDK — shared DTOs (`sdks/csharp/src/Models/`)

- [ ] 13.1 Add admin/observability DTOs: `Stats`, `ServerStatus`, `LogEntry`, `IndexingProgress`, `ConfigSnapshot`, `BackupInfo`, `WorkspaceConfig`
- [ ] 13.2 Add auth DTOs: `ApiKey`, `User`, `JwtToken`, `PasswordPolicyReport`, `RotatedKey`, `TokenIntrospection`, `AuditEntry`
- [ ] 13.3 Add replication DTOs: `ReplicationStatus`, `ReplicaInfo`, `ReplicationStats`
- [ ] 13.4 Add tier-control DTOs: `DeleteByFilterReport`, `BulkUpdateReport`, `CopyReport`, `ReencodeJob`, `TtlConfig`, `VectorExpiryRequest`
- [ ] 13.5 Add schema-evolution DTOs: `ReindexJob`, `SnapshotInfo`, `ExplainResponse`, `SlowQueryEntry`, `SlowQueryConfig`
- [ ] 13.6 Add cluster DTOs: `FailoverReport`, `ResyncJob`, `PeerInfo`, `RebalanceJob`, `RebalanceStatus`

## 14. C# SDK — admin (`sdks/csharp/src/Admin.cs`)

- [ ] 14.1 Mirror Go §3 (17 methods) on `HttpVectorizerClient` as `async Task<T>`
- [ ] 14.2 Expose via `IVectorizerClient` interface
- [ ] 14.3 XML-doc each public member

## 15. C# SDK — auth (`sdks/csharp/src/Auth.cs`)

- [ ] 15.1 Mirror Go §4 (11 methods)
- [ ] 15.2 Expose via `IVectorizerClient`
- [ ] 15.3 XML-doc each public member

## 16. C# SDK — replication (`sdks/csharp/src/Replication.cs`)

- [ ] 16.1 Mirror Go §5 (4 methods)
- [ ] 16.2 Expose via `IVectorizerClient`
- [ ] 16.3 XML-doc each public member

## 17. C# SDK — hub (`sdks/csharp/src/Hub.cs`)

- [ ] 17.1 Mirror Go §6 (10 methods)
- [ ] 17.2 Expose via `IVectorizerClient`
- [ ] 17.3 XML-doc each public member

## 18. C# SDK — discovery pipeline (`sdks/csharp/src/DiscoveryPipeline.cs`)

- [ ] 18.1 Mirror Go §7 (6 methods)
- [ ] 18.2 Expose via `IVectorizerClient`
- [ ] 18.3 XML-doc each public member

## 19. C# SDK — vectors single + batch + search variants

- [ ] 19.1 Mirror Go §8 (10 methods) on `HttpVectorizerClient` and `BatchOperations.cs`
- [ ] 19.2 Expose via `IVectorizerClient`
- [ ] 19.3 XML-doc each public member

## 20. C# SDK — tier-control (`sdks/csharp/src/TierControl.cs`)

- [ ] 20.1 Mirror Go §9 (6 methods)
- [ ] 20.2 Expose via `IVectorizerClient`
- [ ] 20.3 XML-doc each public member

## 21. C# SDK — schema evolution + explain + slow queries (`sdks/csharp/src/SchemaEvolution.cs`)

- [ ] 21.1 Mirror Go §10 (8 methods)
- [ ] 21.2 Expose via `IVectorizerClient`
- [ ] 21.3 XML-doc each public member

## 22. C# SDK — cluster + auth admin (`sdks/csharp/src/ClusterAdmin.cs`)

- [ ] 22.1 Mirror Go §11 (9 methods)
- [ ] 22.2 Expose via `IVectorizerClient`
- [ ] 22.3 XML-doc each public member

## 23. C# SDK — version + tests

- [ ] 23.1 Bump `sdks/csharp/Vectorizer.csproj` `<Version>` to `3.9.0`
- [ ] 23.2 Bump `sdks/csharp/src/Vectorizer.Rpc/Vectorizer.Rpc.csproj` `<Version>` to `3.9.0`
- [ ] 23.3 xUnit wire-shape tests per surface file in `sdks/csharp/Vectorizer.Tests/`
- [ ] 23.4 Live-server integration tests under `[Trait("Category","s2s")]`
- [ ] 23.5 `dotnet build` and `dotnet test --filter "Category!=s2s"` exit 0

## 24. Documentation

- [ ] 24.1 Extend `docs/users/api/API_REFERENCE.md` "SDK 3.4-3.7 control surface" tables with Go and C# columns
- [ ] 24.2 Add per-domain examples (admin, auth, replication, tier-control, schema-evolution, cluster) to `sdks/go/README.md`
- [ ] 24.3 Add per-domain examples to `sdks/csharp/README.md`
- [ ] 24.4 CHANGELOG entry in `sdks/go/CHANGELOG.md` under `## [3.9.0]` listing every new method
- [ ] 24.5 CHANGELOG entry in `sdks/csharp/CHANGELOG.md` under `## 3.9.0` listing every new method
- [ ] 24.6 Update `sdks/COVERAGE_REPORT.md` to reflect Go and C# REST parity (remove the `🚧 partial` markers once tests land)

## 25. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 25.1 Update or create documentation covering the implementation
- [ ] 25.2 Write tests covering the new behavior
- [ ] 25.3 Run tests and confirm they pass
