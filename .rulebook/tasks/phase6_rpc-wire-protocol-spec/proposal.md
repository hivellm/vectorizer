# Proposal: phase6_rpc-wire-protocol-spec

## Why

The project currently exposes three transports (REST on Axum, MCP via WebSocket, gRPC via tonic). Each has high per-request overhead:

- REST: HTTP framing, JSON parsing, TLS handshake per connection.
- gRPC: HTTP/2 + protobuf is fast but heavy codegen + language-specific generated code.
- MCP: JSON-RPC over WebSocket, fine for interactive tools, not for high-throughput ingest.

Synap (sibling project at `../Synap/`) solves the same problem with a simple binary RPC: **length-prefixed MessagePack frames over raw TCP** — ~1500 lines in `../Synap/synap-server/src/protocol/`. It's 10x lighter than gRPC, supports arbitrary `Value` (null/bool/int/float/bytes/str/array/map), and lets every SDK bring its own standard MessagePack library without codegen.

Before implementing the server (`phase6_add-rpc-protocol-server`) and RESP3 compat (`phase6_add-resp3-protocol-server`), we need a **wire spec document** so all 6 SDK tasks (`phase6_sdk-*-rpc`) can be developed in parallel against a frozen reference.

## What Changes

Produce `docs/specs/VECTORIZER_RPC.md` (and `/.rulebook/specs/RPC.md`) documenting:

1. **Framing**: `[len: u32 LE][body: MessagePack bytes]`, matching Synap's `synap_rpc/codec.rs` verbatim. Max body size (e.g., 64 MiB) with rationale.
2. **Request/Response envelope**: `Request { command: String, request_id: String, payload: Value }` / `Response { success: bool, request_id: String, payload: Option<Value>, error: Option<Error> }`.
3. **Error shape**: `{ code: u16, message: String, details: Option<Value> }`; mapping to existing `VectorizerError` kinds (coordinate with `phase3_unify-error-enums`).
4. **Command catalog**: one-to-one mapping with the capability registry from `phase4_rest-mcp-parity-tests`. Names: `collections.create`, `vectors.insert`, `search.query`, etc.
5. **Authentication**: bearer token in the first handshake `Request` or as part of every envelope; pick one and justify.
6. **Streaming**: how search results that return 1000s of vectors are chunked (single frame vs multi-frame with `last: bool`).
7. **Versioning**: wire-level version byte before the first frame, or a `HELLO` command for version negotiation (Synap uses the latter).
8. **Comparison table**: SynapRPC vs VectorizerRPC — note any divergence (if we need a wire change vs pure reuse, call it out).

## Impact

- Affected specs: new `/.rulebook/specs/RPC.md`, new `docs/specs/VECTORIZER_RPC.md`
- Affected code: none yet (this task is spec-only, unblocks implementation)
- Breaking change: NO (new capability; existing transports untouched)
- User benefit: frozen reference for SDK parallelization; captures all design decisions before implementation starts; makes `phase6_make-rpc-default-transport` a conscious cutover.
