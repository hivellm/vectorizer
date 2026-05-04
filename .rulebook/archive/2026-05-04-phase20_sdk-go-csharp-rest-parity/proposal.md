# Proposal: phase20_sdk-go-csharp-rest-parity

## Why

Phases 12-15 (archived 2026-05-03) shipped ~50 new server endpoints across
admin/observability, auth/RBAC, replication, hub backups, discovery
pipeline, tier-control primitives, schema evolution + explain, and
cluster + auth admin. Those phases targeted Rust, TypeScript, and
Python SDKs explicitly and left Go and C# behind on the REST surface.
Phase 16 then closed the gap on RPC (all 5 SDKs got 100 commands), so
today every server capability is reachable from Go and C# **only via
the RPC client**.

Verified by grep on 2026-05-03 (release/v3.3.0):
- `sdks/go/` REST helpers: zero matches for `DeleteByFilter`,
  `BulkUpdateMetadata`, `CopyVectors`, `ReencodeCollection`,
  `SetCollectionTtl`, `RenameCollection`, `ExplainSearch`,
  `ListSlowQueries`, `ClusterFailover`, `RotateApiKey`,
  `IntrospectToken`, `ListAuditLog` (all present only inside
  `sdks/go/rpc/commands_phase16.go`).
- `sdks/csharp/` REST helpers: same — phase12-15 surface lives only
  in `sdks/csharp/src/Vectorizer.Rpc/RpcCommands.cs`, not in the REST
  client (`HttpVectorizerClient.cs` / `VectorizerClient.cs`).

Without REST parity, Go and C# users who don't deploy RPC (port 15503,
binary protocol) cannot use tier control, schema evolution, day-2 ops,
or cluster admin from those languages. That breaks the project's
"REST-first, MCP/RPC equal-or-after" rule documented in CLAUDE.md.

## What Changes

Mirror the Rust/TS/Python REST control surface in Go and C# at the
**HTTP client layer** (no RPC dependency), reaching feature parity
with `sdks/rust/src/client/` and `sdks/typescript/src/client/`.

Scope per SDK:

1. **Go SDK** — add idiomatic REST methods on `*Client` (the existing
   `client.go` http client) covering:
   - admin/observability (~17 methods from phase12 §4)
   - auth/RBAC (~11 methods from phase12 §5)
   - replication (~4 methods from phase12 §6)
   - hub backups + usage (~10 methods from phase12 §7)
   - discovery pipeline (~6 methods from phase12 §3)
   - vectors single + batch + search variants (~8 methods from phase12 §2-3)
   - tier-control (~6 methods from phase13 §7)
   - schema evolution + explain + slow queries (~8 methods from phase14 §6)
   - cluster + auth admin (~9 methods from phase15 §8)
   Total: ~79 REST methods + matching response types in `models.go`.

2. **C# SDK** — same scope, mirrored on `HttpVectorizerClient` and
   `IVectorizerClient` in `sdks/csharp/src/`. Roughly 79 async
   methods + DTOs in `Models/`.

3. **Tests** — wire-shape unit tests per method (mirror the
   `sdks/typescript/tests/` and `sdks/rust/tests/` pattern). Live
   server integration tests deferred behind a build tag (Go
   `//go:build s2s`) and a test category attribute (C#
   `[Trait("Category","s2s")]`), matching the `s2s-tests` feature
   gate the other SDKs use.

4. **Docs** — extend `docs/users/api/API_REFERENCE.md` "SDK 3.4-3.7
   control surface" tables with Go and C# columns. Add usage examples
   to `sdks/go/README.md` and `sdks/csharp/README.md`.

5. **Versioning** — bump Go `version.go` and C# csproj to `3.9.0`,
   bump `Vectorizer.csproj` and `Vectorizer.Rpc.csproj` together so
   NuGet stays consistent. CHANGELOG entries for both SDKs.

## Impact

- Affected specs:
  - `.rulebook/tasks/phase20_sdk-go-csharp-rest-parity/specs/sdk-go-csharp-rest-parity/spec.md`
    (new — mirrors the phase12-15 spec deltas for Go + C#)
- Affected code:
  - `sdks/go/` — new `admin.go`, `auth.go`, `replication.go`,
    `hub.go`, `discovery_pipeline.go`, `tier_control.go`,
    `schema_evolution.go`, `cluster_admin.go`, plus extensions in
    `vectors.go`, `collections.go`, `models.go`
  - `sdks/csharp/src/` — new `Admin.cs`, `Auth.cs`,
    `Replication.cs`, `Hub.cs`, `DiscoveryPipeline.cs`,
    `TierControl.cs`, `SchemaEvolution.cs`, `ClusterAdmin.cs`,
    plus DTOs under `Models/`
  - `sdks/csharp/tests/` and `sdks/go/` test files for wire-shape
    coverage
  - `docs/users/api/API_REFERENCE.md`
  - `sdks/go/README.md`, `sdks/csharp/README.md`,
    `sdks/go/CHANGELOG.md`, `sdks/csharp/CHANGELOG.md`
- Breaking change: NO (additive — new public methods, no signature
  changes to existing ones)
- User benefit: Go and C# clients reach 100% REST parity with
  Rust/TS/Python. Users on those languages no longer need to
  bring up the RPC transport just to use tier control, schema
  evolution, day-2 ops, or cluster/auth admin endpoints.
