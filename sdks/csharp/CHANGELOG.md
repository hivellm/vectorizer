# Changelog

All notable changes to the Hive Vectorizer C# SDK will be documented in this file.

Two NuGet packages share this changelog:

- `Vectorizer.Sdk.Rpc` — RPC-first client (recommended).
- `Vectorizer.Sdk` — legacy REST-only client.

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
