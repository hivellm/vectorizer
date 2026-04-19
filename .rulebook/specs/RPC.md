# RPC transport (rulebook spec)

This is the rulebook-level reference for the binary RPC transport.
The full byte-level spec lives at
[`docs/specs/VECTORIZER_RPC.md`](../../docs/specs/VECTORIZER_RPC.md).

## What it is

Length-prefixed MessagePack frames over raw TCP. Per-connection sticky
auth via a `HELLO` handshake. One handler per command; commands map
1:1 to entries in the capability registry
(`src/server/capabilities.rs`).

```
[u32 LE length][MessagePack body]
```

## Why

REST is universal and human-readable but pays an HTTP-framing tax per
request. gRPC is fast but pulls in protobuf codegen for every SDK
language. The capability registry from `phase4_rest-mcp-parity-tests`
made adding a fourth transport mechanical — same handlers, new framing.
RPC is the **default first-party SDK transport** starting with
`phase6_make-rpc-default-transport`; REST stays as the universal
fallback.

## Implementation

| Layer | Path |
|---|---|
| Wire spec | `docs/specs/VECTORIZER_RPC.md` |
| Codec | `src/protocol/rpc/codec.rs` |
| Types (Request, Response, Value) | `src/protocol/rpc/types.rs` |
| Server (TCP listen + connection loop) | `src/protocol/rpc/server.rs` |
| Dispatch (command name → handler) | `src/protocol/rpc/dispatch.rs` |
| Bootstrap wire-up | `src/server/core/bootstrap.rs` |
| Default port | 15503 (config: `rpc.port`) |

## Reference (Synap)

The implementation ports `../Synap/synap-server/src/protocol/synap_rpc/`
(production-tested, ~390 LOC core). Wire-level parity with SynapRPC is
intentional — see § 9 of the wire spec for the small list of
divergences.

## Adding a new command

1. Add the operation to `src/server/capabilities.rs::inventory()` with
   `Transport::Both` (so REST + RPC both expose it).
2. Add a dispatch arm in `src/protocol/rpc/dispatch.rs` that pulls the
   args out of `Request.args`, calls the same service-layer function
   the REST handler uses, and converts the result to `VectorizerValue`.
3. Add a test in `tests/protocol/rpc/` exercising the round-trip.
4. The capability registry's parity tests (`tests/api/parity.rs`) will
   automatically include the new command in their structural checks.

## Forbidden

- **Do not** invent new framing per command. The `Value` enum's `Map`
  variant is rich enough to model any structured payload.
- **Do not** bypass the capability registry — every dispatched command
  must have a registry entry. The dispatch table is a flat
  `match` on `Request.command`; reflection / dynamic loading is
  explicitly out of scope.
- **Do not** mix wire versions on the same connection. `HELLO` freezes
  the version for the connection's lifetime.
