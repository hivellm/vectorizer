# Proposal: phase3_unify-error-enums

## Why

The audit identified a central `VectorizerError` in `src/error.rs` but **9 parallel error enums** living independently alongside it:

- `BatchErrorType` — `src/batch/error.rs`
- `CompressionError` — `src/compression/mod.rs`
- `QuorumError` — cluster/replication
- `MigrationError` — persistence migrations
- `DiscoveryError` — file discovery
- ...and 4 more across `src/cache/`, `src/auth/`, `src/grpc/`, `src/embedding/`

Because they don't compose into `VectorizerError`, every layer boundary does ad-hoc `format!("{}: {}", context, inner)` conversion. In `src/server/mcp_handlers.rs:27-91`, this becomes 60+ lines of string-glued error construction that loses error kind/cause chains.

Consequence: the REST layer's HTTP status mapping, the MCP layer's error codes, and the gRPC status conversion all derive from stringified errors. A `NotFound` can become a `500` because the string lost its classification.

## What Changes

1. Keep `VectorizerError` as the **canonical outermost error**.
2. Convert each module enum into a `#[derive(thiserror::Error)]` error that implements `Into<VectorizerError>` via a new `VectorizerError::module(err)` variant or a `#[from]` attribute.
3. Replace string-glued conversion sites in MCP/REST/gRPC layers with `?` propagation.
4. Ensure each `VectorizerError` variant carries an `ErrorKind` enum suitable for mapping to HTTP status, gRPC status, and MCP error code — centralize the mapping in `src/error/mapping.rs`.
5. Enforce via policy: no new leaf error types allowed in modules without a `From` impl into `VectorizerError`.

## Impact

- Affected specs: error-handling spec
- Affected code: `src/error.rs`, all 9 leaf-error modules, MCP/REST/gRPC mappers (`src/server/mcp_handlers.rs`, handler error-to-response helpers, `src/grpc/conversions.rs`)
- Breaking change: error message strings may change (callers parsing strings will break; JSON error codes stay stable)
- User benefit: correct HTTP/gRPC status codes, richer error causes in logs, smaller LoC at boundaries.
