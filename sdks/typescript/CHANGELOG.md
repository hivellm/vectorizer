# Changelog

All notable changes to the Hive Vectorizer TypeScript Client SDK will be documented in this file.

## [3.3.0] - 2026-05-02

### Added

- **Tier-demotion API ([#265](https://github.com/hivellm/vectorizer/issues/265)).** Three new methods on `VectorsClient`:
  - `deleteVector(collection, vectorId): Promise<void>` calling `DELETE /collections/{c}/vectors/{id}`.
  - `moveToCollection(src, dst, ids): Promise<MoveReport>` calling `POST /collections/{src}/vectors/move`. Server invariant: dst-insert-before-src-delete; a mid-batch crash leaves a recoverable duplicate, never data loss. Per-id outcomes (`ok | missing_in_src | dst_insert_failed | src_delete_failed`) populate `MoveReport.results` without aborting the batch.
- New types under `@hivehub/vectorizer-sdk`: `DeleteReport`, `MoveReport`, `VectorOpResult`, `VectorOpStatus`.

### Changed

- **`deleteVectors` now returns `DeleteReport`.** The 3.2 implementation posted to a non-existent `/collections/{c}/vectors/delete` route and returned `{ deleted: number }`. The 3.3 contract aligns with the real server route (`POST /batch_delete`) and surfaces the full per-id status array via `DeleteReport`. Callers asserting on `result.deleted` keep working; callers destructuring the old shape need to read from `report.deleted` / `report.results`.

## [3.2.0] - 2026-05-01

### Added

- **Backpressure-aware HTTP client.** Honors the new server-side
  bulk-upsert backpressure shipped in Vectorizer 3.2.0
  ([#263](https://github.com/hivellm/vectorizer/issues/263)). On HTTP
  `429 Too Many Requests` the client parses `Retry-After` (seconds
  form), sleeps, and retries — bounded by the same 3-attempt / 30 s-
  cap / 1 s-default policy used by every other first-party SDK. After
  retry exhaustion a `VectorizerError` carrying `status: 429` is
  surfaced. Pre-3.2.0 clients bounced 429s into a generic 5xx and
  lost the retry budget.
- New tests at `tests/retry-after.test.ts` lock the parser semantics
  and the 3-attempt budget.

### Changed

- Version bumped to 3.2.0 to track the server release.

## [3.1.0] - 2026-04-29

### Added

- **`insertVectors(collection, vectors, publicKey?)`** — bulk-insert
  pre-computed embeddings with caller-supplied vector ids. Skips the
  embedding pipeline entirely. Useful when the client already has its
  own embedder or wants idempotent re-ingest by stable id.
- **`insert` / `insertTexts` accept `id`** as the stored
  `Vector.id`. Non-chunked inputs use the client `id` verbatim;
  chunked inputs derive `<id>#<chunkIndex>` (e.g. `doc:42#0`,
  `doc:42#1`). Re-running the same payload now upserts in place
  instead of duplicating, and `delete` round-trips by client id work
  without a UUID lookup.
- **`payload.parent_id` on chunked vectors** links chunks back to
  the source document. Set to the request `id` when provided;
  otherwise a single freshly-minted UUID v4 is shared across every
  chunk of the same `insertTexts` entry.

### Changed

- **Chunked-payload layout flipped from nested to flat — BREAKING for
  clients reading `payload.metadata.<field>` directly.** Pre-3.1.0
  chunks landed as `{content, metadata: {file_path, chunk_index, ...}}`.
  3.1.0 emits `{content, file_path, chunk_index, parent_id, ...}`
  with every key at the root. Server-provided keys take precedence
  over user metadata. Readers tolerate both shapes during the
  deprecation window. See the parent-repo CHANGELOG for the
  migration matrix.

### Note

Client-id contract: non-empty, length ≤ 256, no leading/trailing
whitespace, must not contain `#` (reserved as the chunk-id
separator). Violations return HTTP 400 with
`error_type: "validation_error"`.

## [3.0.0] - 2026-04-19

### Added

- **VectorizerRPC client** (new default transport in v3.x). Binary,
  length-prefixed MessagePack over raw TCP (port 15503), spec at
  `docs/specs/VECTORIZER_RPC.md`. Polyglot parity with the Rust SDK
  at `sdks/rust/src/rpc/` and the Python SDK at `sdks/python/rpc/`.
  - `RpcClient` (Node-only, uses `node:net`) multiplexes calls on a
    single TCP connection by `Request.id` into per-call promises.
  - `parseEndpoint` — canonical URL parser shared with every other
    Vectorizer SDK. Accepts `vectorizer://host:port`,
    `vectorizer://host` (default port 15503), bare `host:port`, and
    `http(s)://host:port`. Rejects userinfo credentials.
  - `HelloPayload` / `HelloResponse` — sticky per-connection auth
    handshake.
  - `RpcPool` — minimal bounded connection pool with an RAII-style
    `PooledClient` guard.
  - Typed wrappers: `listCollections`, `getCollectionInfo`,
    `getVector`, `searchBasic`. Match the Rust + Python SDK shapes.
  - Top-level `import { RpcClient } from '@hivehub/vectorizer-sdk'`
    convenience export, plus the per-namespace `import { ... } from '@hivehub/vectorizer-sdk/rpc'`.
- New runtime dependency: `@msgpack/msgpack@^3.0.0`.
- 39 new tests under `tests/rpc/` (unit + integration with an
  in-test fake server using Node's `net` module). Includes wire-spec
  golden vectors that bit-exactly match the hex dumps in the
  protocol spec § 11.

### Changed

- Bumped to v3.0.0 to mark the new default transport. The legacy
  `VectorizerClient` REST client stays available unchanged.
- README rewritten with an RPC-first quickstart and a "Switching
  transports" matrix.

### Note

The package surface is **additive** for existing v2.x callers:
`VectorizerClient` and every model class still import from the same
paths. The "breaking" v3.0 marker reflects that the recommended
transport changes — there is no forced migration of existing code.

The standalone `@hivehub/vectorizer-sdk-js` package was retired at
the same time; this TypeScript SDK ships compiled CommonJS + ESM and
is fully usable from plain JavaScript.

## [1.3.0] - 2025-11-15

### Added

- **Hybrid Search Support**: Complete TypeScript implementation of hybrid search

  - `SparseVector`: Interface for sparse vector representation
  - `HybridSearchRequest`: Request interface with full type safety
  - `HybridSearchResponse` and `HybridSearchResult`: Response interfaces
  - `hybridSearch()`: Method in VectorizerClient with full type checking
  - `validateHybridSearchRequest()`: Validation function for request data
  - `validateSparseVector()`: Validation function for sparse vectors

- **Qdrant Compatibility**: Full Qdrant REST API compatibility methods
  - `qdrantListCollections()`: List all collections
  - `qdrantGetCollection()`: Get collection information
  - `qdrantCreateCollection()`: Create collection with Qdrant config
  - `qdrantUpsertPoints()`: Upsert points to collection
  - `qdrantSearchPoints()`: Search points in collection
  - `qdrantDeletePoints()`: Delete points from collection
  - `qdrantRetrievePoints()`: Retrieve points by IDs
  - `qdrantCountPoints()`: Count points in collection

### Changed

- **Version Sync**: Updated to v1.3.0 to match Vectorizer server release
- **Server Compatibility**: Compatible with Vectorizer v1.3.0 (hybrid search and Qdrant compatibility)
- **Type Safety**: Full TypeScript type safety for all new hybrid search and Qdrant methods

### Note

This release adds hybrid search and Qdrant compatibility features. All existing functionality remains unchanged and backward compatible.

## [1.2.0] - 2025-10-25

### Added

- **Replication Models**: New data models and types for replication monitoring
  - `ReplicaStatus`: Enum for replica node status (Connected, Syncing, Lagging, Disconnected)
  - `ReplicaInfo`: Interface for replica node details (host, port, status, heartbeat, operations synced)
  - `ReplicationStats`: Enhanced interface with new v1.2.0 fields:
    - `role`: Node role (Master or Replica)
    - `bytes_sent`: Total bytes sent to replicas
    - `bytes_received`: Total bytes received from master
    - `last_sync`: Timestamp of last synchronization
    - `operations_pending`: Number of operations waiting to be replicated
    - `snapshot_size`: Size of snapshot data
    - `connected_replicas`: Number of connected replica nodes (Master only)
  - `ReplicationStatusResponse`: Interface for `/replication/status` endpoint response
  - `ReplicaListResponse`: Interface for `/replication/replicas` endpoint response
  - Validation functions: `isReplicaStatus()`, `validateReplicaInfo()`, `validateReplicationStats()`

### Changed

- **Backwards Compatible**: All new replication fields are optional to maintain compatibility with older servers
- **Legacy Fields Maintained**: Existing replication fields (`master_offset`, `replica_offset`, `lag_operations`, `total_replicated`) continue to work

### Technical

- Added comprehensive type guards and validation functions for replication models
- Enhanced TypeScript interfaces with proper optional types for new fields
- Maintained strict type safety for all new models

## [1.0.0] - 2025-10-21

### Changed

- **Version Sync**: Updated to v1.0.0 to match Vectorizer server release
- **Server Compatibility**: Compatible with Vectorizer v1.3.0 (hybrid search and Qdrant compatibility)
- **No Breaking Changes**: REST API remains unchanged, full backward compatibility
- **Production Ready**: Stable release aligned with server v1.0.0

### Note

This release maintains full compatibility with Vectorizer REST API. The MCP refactoring in server v1.0.0 does not affect SDK functionality as SDKs use REST/UMICP protocols.

## [0.4.2] - 2025-10-15

### Added

- **GUI-Specific API Endpoints** for server management and monitoring:
  - `getStatus()` - Get server status, version, uptime, and statistics
  - `getLogs(params?)` - Retrieve recent server logs with optional filtering
  - `forceSaveCollection(name)` - Force immediate save of a specific collection
  - `addWorkspace(params)` - Add a new workspace configuration
  - `removeWorkspace(params)` - Remove an existing workspace
  - `listWorkspaces()` - List all configured workspaces
  - `getConfig()` - Get current server configuration
  - `updateConfig(config)` - Update server configuration
  - `restartServer()` - Admin endpoint to restart the server
  - `listBackups()` - List available backup files
  - `createBackup(params?)` - Create a new backup
  - `restoreBackup(params)` - Restore from a backup file
  - `getBackupDirectory()` - Get the backup directory path

## [0.4.1] - 2025-10-15

### Changed

- Updated `@hivellm/umicp` dependency to `^0.1.5`
- Installation no longer requires C++ build tools
- Faster and more reliable installation process

### Fixed

- Fixed installation failures on systems without build tools
- Removed build errors during package installation

## [0.4.0] - 2025-10-12

### Added

- **UMICP Protocol Support**: Added support for the UMICP (Universal Messaging and Inter-process Communication Protocol)
  - New `UMICPClient` for high-performance communication
  - Transport abstraction layer supporting multiple protocols (HTTP/HTTPS and UMICP)
  - Connection string support for easy protocol switching (e.g., `umicp://localhost:15003`)
  - `TransportFactory` for creating protocol-specific clients
  - `parseConnectionString` utility for parsing connection URIs

### Changed

- Refactored `VectorizerClient` to use transport abstraction instead of direct HTTP client
- Updated `VectorizerClientConfig` to support multiple protocols:
  - Added `protocol` field to specify transport protocol
  - Added `connectionString` field for URI-based configuration
  - Added `umicp` field for UMICP-specific options
- Updated `setApiKey()` method to reinitialize transport with new API key

### New API

- `client.getProtocol()`: Get the current transport protocol being used
- Multiple transport options:
  - HTTP/HTTPS (default)
  - UMICP (via `@hivellm/umicp` package)

### Dependencies

- Added `@hivellm/umicp@^0.1.3` as a dependency

### Documentation

- Updated README with UMICP configuration examples
- Added protocol comparison table
- Added examples for using UMICP transport
- Created `examples/umicp-usage.ts` demonstrating UMICP usage

### Technical

- Implemented `ITransport` interface for protocol abstraction
- Created separate transport implementations:
  - `HttpClient` for HTTP/HTTPS
  - `UMICPClient` for UMICP protocol
- Added comprehensive error handling for both protocols
- Maintained backward compatibility with existing HTTP-only configurations

## [0.3.4] - Previous Version

- (Previous changes...)
