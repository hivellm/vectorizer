# Proposal: phase1_replace-vectorizer-protocol-with-thunder

## Why
`vectorizer-protocol` is a bespoke wire-protocol crate: a hand-rolled RPC
binary transport (`src/rpc_wire/` — codec + types, ~526 LOC) plus a tonic/prost
gRPC surface (`src/grpc_gen/`). Maintaining a custom RPC codec + keeping every
SDK (Rust/TS/Python/Go/C#) byte-compatible with it is a recurring cost and a
drift risk. Synap and Nexus already retired their custom protocols in favor of
`thunder-rpc` (v0.2.2, lib name `thunder`) for RPC — including their Rust SDKs
(`synap/sdks/rust`, `nexus-cli`). Do the same here and delete
`vectorizer-protocol`.

## What Changes
Adopt `thunder-rpc` as the RPC transport and remove `vectorizer-protocol`,
mirroring the Synap/Nexus migrations (use those as the reference implementation):
`thunder-rpc = "0.2.2"` with `server` on the server crate and `client` on the
SDK/CLI, `tokio` where async.

- **Server**: replace `crates/vectorizer-server/src/protocol/rpc/{server,dispatch}.rs`
  (the VectorizerRPC transport) with a thunder server; map the existing
  command/dispatch table onto thunder's service/handler model.
- **Rust SDK**: `sdks/rust` swaps its `vectorizer-protocol` dep for a
  `thunder-rpc` client (follow synap/sdks/rust).
- **Wire types**: move the request/response types out of
  `vectorizer-protocol/rpc_wire/types.rs` into a small shared crate or the
  server, serialized via thunder's codec (drop the hand-rolled codec.rs).
- **gRPC / cluster**: `vectorizer-protocol` also generates the cluster gRPC
  (`grpc_gen/`, used by cluster/{grpc_service,raft_node,state_sync,server_client}
  + grpc_conversions + build.rs). Decide: port the cluster RPC to thunder too,
  or relocate the tonic/prost gRPC gen into the `vectorizer` crate's build.rs so
  `vectorizer-protocol` can be deleted. (Resolve in the spec.)
- **Cross-language SDKs**: TS/Python/Go/C# currently implement the
  VectorizerRPC binary wire. Determine thunder's on-the-wire format and how
  Synap/Nexus handle non-Rust clients (thunder client libs vs a documented wire)
  and migrate each SDK's transport accordingly. This is the largest sub-scope.
- Delete the `vectorizer-protocol` crate from the workspace once nothing
  references it.

## Open questions (resolve in the spec before coding)
- Does thunder-rpc have (or need) non-Rust client support, or a stable wire the
  TS/Python/Go/C# SDKs implement? How did Synap/Nexus solve multi-language?
- Is the cluster gRPC ported to thunder, or kept as tonic/prost relocated out of
  vectorizer-protocol?

## Impact
- Affected specs: rpc-transport, sdk-transport, cluster
- Affected code: crates/vectorizer-protocol/** (deleted), crates/vectorizer-server/
  src/protocol/rpc/**, crates/vectorizer/src/{protocol/rpc,cluster/**,grpc_conversions.rs,build.rs},
  sdks/rust + sdks/{typescript,python,go,csharp} RPC transports
- Breaking change: potentially YES on the wire (VectorizerRPC clients) — needs a
  compatibility/versioning plan; server REST/MCP surface is unaffected
- User benefit: one shared, maintained RPC stack (thunder) across HiveLLM
  (Vectorizer/Synap/Nexus); no bespoke codec to keep byte-compatible
