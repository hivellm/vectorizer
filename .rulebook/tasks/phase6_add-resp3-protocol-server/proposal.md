# Proposal: phase6_add-resp3-protocol-server

## Why

RESP3 (Redis Serialization Protocol v3) is what every Redis client speaks. Supporting RESP3 on Vectorizer means:

- Any existing Redis-aware tool (redis-cli, RedisInsight, language-idiomatic Redis clients) can inspect or drive Vectorizer without a dedicated SDK.
- A huge ecosystem of Redis tooling (dashboards, replication test tools, chaos tools, tracing) becomes free.
- Low-level ops (`PING`, `CLIENT LIST`, `CLIENT KILL`, `INFO`, `MEMORY USAGE`) match what operators already use for Redis.

Synap already implements RESP3 at `../Synap/synap-server/src/protocol/resp3/` (~915 LOC). We port it, map commands to our capability registry, and add vector-specific commands (`VECTORIZE`, `SEARCH`, `RECOMMEND`) following the same naming convention Synap uses.

## What Changes

New module `src/protocol/resp3/`:

- `parser.rs` — incremental RESP3 parser (simple strings, bulk strings, arrays, maps, sets, pushes, doubles, booleans, big numbers, nil, verbatim strings, attributes). Ported from Synap.
- `writer.rs` — RESP3 serializer.
- `server.rs` — tokio TCP listener, connection loop, pipelining support, PUB/SUB plumbing.
- `command/mod.rs` — per-command handlers. At minimum: PING, AUTH, HELLO, CLIENT, INFO, SELECT (no-op), QUIT, plus Vectorizer-specific commands (`VCREATE`, `VINSERT`, `VSEARCH`, `VDELETE`, etc.).

Commands map to the same capability registry. Any command not in the registry returns RESP3 error `ERR unknown command`.

Default port: 15504 (following Synap's +1 convention from RPC 15503).

## Impact

- Affected specs: `/.rulebook/specs/RPC.md` (RESP3 section), `docs/specs/VECTORIZER_RESP3.md`
- Affected code: new `src/protocol/resp3/`, `src/server/bootstrap.rs`, `src/config/`
- Breaking change: NO
- User benefit: Redis-tooling compatibility; ops ergonomics; friction-free adoption.
