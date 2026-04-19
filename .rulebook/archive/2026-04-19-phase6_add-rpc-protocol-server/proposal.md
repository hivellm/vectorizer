# Proposal: phase6_add-rpc-protocol-server

## Why

Depends on `phase6_rpc-wire-protocol-spec` being finalized. This task implements the server side: a native TCP listener accepting length-prefixed MessagePack frames per the Vectorizer RPC spec, dispatching to the same capability registry used by REST/MCP/gRPC (from `phase4_rest-mcp-parity-tests`).

Why build this in addition to gRPC:
- Much thinner wire and zero codegen in client languages (any MessagePack library works).
- Synap already ships a production-quality reference at `../Synap/synap-server/src/protocol/synap_rpc/` (~390 LOC core). We can port + adapt rather than design from scratch.
- Enables `phase6_make-rpc-default-transport` once SDKs are ready.

## What Changes

New module `src/protocol/rpc/` mirroring Synap's layout:

- `src/protocol/rpc/codec.rs` — frame encode/decode (`u32 LE len` + MessagePack body); ported from Synap.
- `src/protocol/rpc/types.rs` — `Request`, `Response`, `Value`, `Error` structs; serde via `rmp-serde`.
- `src/protocol/rpc/server.rs` — tokio TCP listener, connection loop, timeouts, backpressure, TLS option.
- `src/protocol/rpc/dispatch/` — one handler per command; each handler looks up the capability from `src/server/capabilities.rs` (introduced in `phase4_rest-mcp-parity-tests`) and invokes the same service-layer function the REST handler uses.
- `src/protocol/rpc/auth.rs` — bearer-token handshake; reuses `src/auth/` primitives.

Add `rmp-serde = "1"` to `Cargo.toml`.

Extend `src/server/bootstrap.rs` (from `phase3_split-server-mod-monolith`) to optionally spawn the RPC listener on a dedicated port (default 15503, rationale matching Synap's 15500-range convention).

## Impact

- Affected specs: `/.rulebook/specs/RPC.md`, `docs/specs/VECTORIZER_RPC.md`
- Affected code: new `src/protocol/rpc/` module, `src/server/bootstrap.rs`, `Cargo.toml`, `src/server/capabilities.rs`, `config.example.yml`
- Breaking change: NO (adds a new port/transport; existing ones untouched)
- User benefit: high-throughput, low-overhead transport for bulk ingest + low-latency search; unblocks `phase6_make-rpc-default-transport`.
