## 1. Prerequisites

- [ ] 1.1 `phase6_rpc-wire-protocol-spec` frozen
- [ ] 1.2 `phase6_add-rpc-protocol-server` merged (can be tested against)
- [ ] 1.3 Read `../Synap/sdks/rust/` to understand existing patterns

## 2. Shared codec

- [ ] 2.1 Consider extracting `src/protocol/rpc/codec.rs` + `types.rs` into a workspace-member `vectorizer-protocol` crate so SDK and server reuse the same types
- [ ] 2.2 Add `vectorizer-protocol` as a dep in `sdks/rust/Cargo.toml`

## 3. Client core

- [ ] 3.1 Implement `RpcClient` with `connect`, `call`, `close`, `ping`
- [ ] 3.2 Implement handshake (HELLO + AUTH per spec)
- [ ] 3.3 Implement `RpcPool` with tokio `bb8` or custom pool using `Arc<Mutex<Vec<RpcClient>>>`
- [ ] 3.4 Implement reconnect with exponential backoff

## 4. Typed API

- [ ] 4.1 Generate typed method wrappers from the capability registry (macro or manual) for collections/vectors/search/admin
- [ ] 4.2 Ensure response types match `src/models::*` via shared crate

## 5. Feature flags + defaults

- [ ] 5.1 Add `[features]` section: `rpc = []` (default), `http = ["reqwest"]`, `grpc = ["tonic"]`, `mcp = ["tokio-tungstenite"]`
- [ ] 5.2 Make `Client::new(url)` default to RPC
- [ ] 5.3 Implement the canonical URL parser as a `parse_endpoint(url: &str) -> Result<Endpoint, ParseError>` helper: `vectorizer://host:port` → RPC on the given port; `vectorizer://host` (no port) → RPC on default port 15503; `host:port` (no scheme) → RPC; `http(s)://host:port` → REST. Reject any other scheme with `ParseError::UnsupportedScheme`. The `Client::new` constructor and any `connect_with_url` helpers route through this single parser.
- [ ] 5.4 Unit tests for `parse_endpoint` covering: each of the 4 valid forms, the default-port branch (15503), an invalid scheme (`ftp://`), an empty string, and a URL carrying credentials in the userinfo (which MUST be rejected — credentials go in HELLO, not the URL).

## 6. Examples + docs

- [ ] 6.1 Update `sdks/rust/examples/quickstart.rs` to RPC
- [ ] 6.2 Add `sdks/rust/examples/http_compat.rs` showing legacy HTTP path
- [ ] 6.3 Rewrite `sdks/rust/README.md` with RPC-first quickstart

## 7. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 7.1 Publish SDK docs via `cargo doc`; link from the project README
- [ ] 7.2 Integration tests in `sdks/rust/tests/rpc.rs`: connect, auth, full CRUD, search, streaming, reconnect-on-drop, pool exhaustion
- [ ] 7.3 Run `cargo test --all-features -p vectorizer-sdk` and confirm all pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
