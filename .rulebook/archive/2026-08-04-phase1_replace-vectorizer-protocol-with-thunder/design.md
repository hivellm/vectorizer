# Design: replace vectorizer-protocol with thunder-rpc

Resolves the two open questions in `proposal.md`, grounded in the Synap + Nexus
thunder migrations and a full inventory of `vectorizer-protocol`'s surface.

## Research findings (reference implementations)

### Thunder is the packaged form of the wire we already hand-copied
`crates/vectorizer-protocol/src/rpc_wire/types.rs:4` states the wire types were
"Ported from `../Synap/synap-server/src/protocol/synap_rpc/types.rs` … the
on-wire representation is identical to SynapRPC's." SynapRPC is now the
`thunder-rpc` crate (lib `thunder`, wire **v1, frozen**). So our bespoke
`rpc_wire` is a manual copy of exactly what Thunder now ships and maintains.

- **Wire**: 4-byte little-endian length prefix + MessagePack body (rmp-serde).
  Identical to our `codec.rs` (`[u32 LE len][MessagePack]`, 64 MiB cap).
- **Server** (Synap `crates/synap-server/src/protocol/synap_rpc/server.rs`):
  implement `thunder::server::Dispatch { dispatch(session, command, args) ->
  Result<Value,String>; authenticate(creds) -> Principal }`, then
  `thunder::server::spawn_listener(dispatch, config, info, listener_config)`.
- **Config** (`thunder::Config::standard().scheme(..).port(..)
  .handshake(Handshake::AuthCommand).push(PushPolicy::Enabled)
  .error_codes(ErrorConvention::Resp3Prefixes).max_frame_bytes(..)`).
- **Rust client** (Synap `sdks/rust/src/transport/mod.rs`):
  `thunder::Client::connect_with(endpoint, config, client_config)`.

### Thunder has official client libraries in every SDK language
Synap's non-Rust SDKs do **not** hand-implement the wire — they depend on the
per-language Thunder client:

| Lang | Thunder client package | Evidence |
|------|------------------------|----------|
| Rust | `thunder-rpc` (crates.io) | Synap `sdks/rust/Cargo.toml` |
| Python | `hivellm-thunder` → `import thunder_rpc` | Synap `sdks/python/synap_sdk/transport_rpc.py` |
| TypeScript | `@hivehub/thunder` (npm) | Synap `sdks/typescript/package.json` |
| Go | `github.com/hivellm/thunder-go` | Synap `sdks/go/go.mod` |
| C# | `HiveLLM.Thunder` (NuGet) | Synap `sdks/csharp/src/Synap.SDK/Synap.SDK.csproj` |

### Synap and Nexus have no gRPC
Zero `tonic`/`prost`/`grpc` in either workspace. Thunder replaced their native
RPC entirely; REST/HTTP remains only as a fallback for commands not mapped to
the binary protocol. Neither project informs a gRPC-to-thunder port, because
neither ever had gRPC.

## Open question 1 — cross-language SDK transport → RESOLVED

**Each SDK adopts the official Thunder client library for its language** and
drops its hand-rolled VectorizerRPC binary transport. `VectorizerValue`
(8 variants: Null/Bool/Int/Float/Bytes/Str/Array/Map) maps 1:1 onto
`thunder::Value`; `Request{id,command,args}` / `Response{id,result}` are Thunder
frames. The command catalog (the ~75 `command` strings) is unchanged — only the
transport under it changes. REST stays as the documented fallback (as in Synap).

## Open question 2 — gRPC / cluster disposition → RESOLVED

**Do not port gRPC to Thunder.** Thunder is a MessagePack RPC transport, not
gRPC; and `proto/qdrant/*` exists specifically to serve external Qdrant clients
over gRPC — that surface MUST stay tonic/prost. The three proto trees are
independent of the RPC-wire:

- `proto/vectorizer.proto` — first-party `VectorizerService` (13 methods)
- `proto/cluster.proto` — `ClusterService` (27 methods incl. Raft), used by
  `crates/vectorizer/src/cluster/{grpc_service,raft_node,server_client,state_sync}.rs`
  + `grpc_conversions.rs`
- `proto/qdrant/*.proto` — Qdrant-compatible `Collections/Points/Snapshots`

**Decision: relocate the tonic/prost gRPC generation into a dedicated
`crates/vectorizer-grpc` crate** (proto/ + build.rs + generated `grpc_gen/`),
carrying the `tonic`/`prost`/`tonic-prost`/`protoc-bin-vendored` deps with it.
`crates/vectorizer` and `crates/vectorizer-server` depend on `vectorizer-grpc`
for the gRPC types. This keeps the cluster/Qdrant gRPC exactly as-is (no
behavior change) while freeing `vectorizer-protocol` of everything except
`rpc_wire` — which then moves to Thunder, leaving the crate empty to delete.

## Migration plan (maps to tasks.md phases 2–5)

1. **Relocate gRPC (unblocks deletion).** New `crates/vectorizer-grpc` = the
   current `proto/`, `build.rs`, and `grpc_gen/` verbatim. Repoint
   `vectorizer` + `vectorizer-server` imports `vectorizer_protocol::grpc_gen`
   → `vectorizer_grpc`. `cargo check` green. No wire/behavior change.
2. **Server RPC → Thunder.** Add `thunder-rpc` (server) to `vectorizer-server`.
   Rewrite `protocol/rpc/server.rs` (TCP loop + framing) as a
   `thunder::server::Dispatch` + `spawn_listener`. `protocol/rpc/dispatch.rs`
   keeps its ~75-arm `match req.command`; swap `VectorizerValue` → `thunder::Value`
   and the auth hook onto `Dispatch::authenticate`. Delete `rpc_wire` usage.
3. **Rust SDK → Thunder.** `sdks/rust`: replace the `vectorizer_protocol::rpc_wire`
   re-exports in `src/rpc/{types,codec}.rs` with a `thunder::Client` transport
   (follow Synap `sdks/rust/src/transport/mod.rs`).
4. **Non-Rust SDKs → Thunder.** TS `@hivehub/thunder`, Python `hivellm-thunder`,
   Go `thunder-go`, C# `HiveLLM.Thunder` — each replaces its binary transport
   module with the official client, keeping the command surface. (The GUI
   follow-up is tracked by `phase2_update-gui-to-thunder-sdk`.)
5. **Delete `vectorizer-protocol`.** Once nothing imports it, remove the crate
   from `members`; `cargo check` + clippy clean.

## Wire compatibility / breaking-change note

Our `rpc_wire` is byte-identical to SynapRPC's frozen v1 wire, so the framing +
MessagePack encoding are already Thunder-compatible. The observable risks are
the handshake/auth negotiation (`Handshake::AuthCommand`) and the error-code
convention (`Resp3Prefixes`) — the server config MUST match what the Thunder
client libs expect (Synap's `synap_config()` is the template). A server built on
Thunder and old hand-rolled clients that predate `AuthCommand` may not
interoperate; the SDKs ship in lockstep (all at 3.6.x), so clients and server
upgrade together, and REST remains the compatibility fallback. This is a
transport-internal change: the REST + MCP surfaces are untouched.

## Deferred-scope guard

Phases 2–5 are a large, wire-touching migration across the server, the cluster
gRPC relocation, and five SDKs. Each phase is independently `cargo check`-able
and lands as its own commit in the order above; none is dropped.
