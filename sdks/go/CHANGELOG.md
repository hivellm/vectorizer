# Changelog

All notable changes to the Hive Vectorizer Go SDK will be documented in this file.

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
