# Proposal: phase6_sdk-rust-rpc

## Why

`sdks/rust/` must expose the new Vectorizer RPC transport so Rust consumers get the fast path by default. The server side is implemented by `phase6_add-rpc-protocol-server`; this task builds the client.

Reference: Synap's Rust SDK at `../Synap/sdks/rust/` provides a working template — shares the same wire spec, so a large fraction of the client code can be ported directly.

## What Changes

Inside `sdks/rust/`:

1. Add `vectorizer-rpc` module with `RpcClient` struct (`connect(addr) -> Result<Self>`, `call(command, payload) -> Result<Value>`, plus typed wrappers for each capability).
2. Internally uses `tokio::net::TcpStream` + `rmp-serde` + the shared codec (either duplicated or re-exported from the server crate as a `vectorizer-protocol` workspace member).
3. Connection pooling (`RpcPool`) to amortize TCP + auth handshakes.
4. Auto-reconnect with exponential backoff.
5. Typed convenience methods: `client.collections().create(...)`, `client.vectors().insert(...)`, `client.search().query(...)`.
6. Keep existing `HttpClient` available; make `Client::new(...)` default to RPC, with `Client::with_http(...)` as the opt-in HTTP variant.
7. Feature flags: `rpc` (default), `http`, `grpc`, `mcp` — each transport gated so slim builds can pick only what they need.
8. Update `sdks/rust/README.md` quickstart to show the RPC example first.

## Impact

- Affected specs: SDK spec
- Affected code: `sdks/rust/src/` (new `rpc/` module), `sdks/rust/Cargo.toml`, `sdks/rust/README.md`, `sdks/rust/examples/`
- Breaking change: YES (default changes from HTTP to RPC) — documented + switchable via constructor
- User benefit: faster default path for Rust users; feature-gated transports keep binary size sane.

## Default URL scheme

The constructor accepts a URL string. Scheme drives transport:

- `vectorizer://host:15503` → RPC (binary MessagePack, see
  `docs/specs/VECTORIZER_RPC.md`).
- `vectorizer://host` → RPC on default port 15503.
- `host:15503` (no scheme) → RPC.
- `http://host:15002` / `https://host` → REST (legacy fallback).

`vectorizer://` is the canonical default per
`phase6_make-rpc-default-transport`. Examples in `README.md` and
`sdks/rust/examples/` use `vectorizer://` first; the REST form is
documented as the "legacy" path under a "Switching transports" header.
