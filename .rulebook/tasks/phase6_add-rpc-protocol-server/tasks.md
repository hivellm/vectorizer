## 1. Prerequisites

- [ ] 1.1 Confirm `phase6_rpc-wire-protocol-spec` is merged and the spec is frozen
- [ ] 1.2 Confirm `phase4_rest-mcp-parity-tests` capability registry is in place (or coordinate ordering)

## 2. Codec + types

- [ ] 2.1 Port `../Synap/synap-server/src/protocol/synap_rpc/codec.rs` into `src/protocol/rpc/codec.rs`; adapt imports and error types
- [ ] 2.2 Port `types.rs` into `src/protocol/rpc/types.rs`; replace `SynapValue` with `VectorizerValue`; keep the wire shape identical
- [ ] 2.3 Add `rmp-serde = "1"` to `Cargo.toml`
- [ ] 2.4 Add unit tests for round-trip encode/decode on every `VectorizerValue` variant

## 3. Server

- [ ] 3.1 Port `../Synap/synap-server/src/protocol/synap_rpc/server.rs` into `src/protocol/rpc/server.rs`; adapt bootstrap/state types
- [ ] 3.2 Add graceful-shutdown hook wired into `src/server/shutdown.rs`
- [ ] 3.3 Add configurable port (default 15503), bind address, max-frame-size, and per-connection timeouts to `src/config/`
- [ ] 3.4 Wire authentication: bearer token via first `HELLO` request, stored in connection state; reject subsequent commands if not authenticated per the spec

## 4. Dispatch

- [ ] 4.1 Create `src/protocol/rpc/dispatch/mod.rs` with a command-name-to-handler table driven by the capability registry
- [ ] 4.2 Implement dispatch for read commands: `collections.list`, `collections.get`, `vectors.get`, `search.query`, `search.recommend`, `search.scroll`
- [ ] 4.3 Implement dispatch for write commands: `collections.create`, `collections.delete`, `vectors.insert`, `vectors.upsert`, `vectors.delete`
- [ ] 4.4 Implement dispatch for admin commands (gated by admin-role auth): `snapshots.create`, `backups.restore`, `workspace.add`
- [ ] 4.5 Implement streaming responses for large search results per the spec decision

## 5. Integration

- [ ] 5.1 Spawn the RPC listener from `src/server/bootstrap.rs` when `config.rpc.enabled = true` (default true)
- [ ] 5.2 Expose RPC connection/command metrics on the existing `/metrics` endpoint

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 6.1 Document the RPC server in `docs/deployment/rpc.md` + update `README.md` with the new port and example client snippet
- [ ] 6.2 Integration tests in `tests/protocol/rpc/`: handshake, auth success/failure, every dispatched command, streaming, graceful shutdown, backpressure
- [ ] 6.3 Run `cargo test --all-features -- rpc` and confirm all pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
