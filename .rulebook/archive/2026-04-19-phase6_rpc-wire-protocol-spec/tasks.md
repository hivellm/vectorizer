## 1. Research Synap reference

- [x] 1.1 Read Synap's wire format in full: `../Synap/synap-server/src/protocol/synap_rpc/{codec.rs,types.rs,server.rs}`. Codec is 182 lines (frame is `[u32 LE len][msgpack body]`); types are 174 lines (`Request {id: u32, command: String, args: Vec<SynapValue>}`, `Response {id: u32, result: Result<SynapValue, String>}`); server is 207 lines (TCP listener with per-connection writer task fed via mpsc channel, per-request `tokio::spawn` for dispatch concurrency).
- [x] 1.2 SDK references in `../Synap/sdks/` confirm clients use plain MessagePack libs (`rmp-serde` for Rust, `msgpack` for Python, `@msgpack/msgpack` for JS) with no codegen. The `Request.id`/`Response.id` u32 multiplexes responses back to pending-call tables.

## 2. Draft specs

- [x] 2.1 Drafted `docs/specs/VECTORIZER_RPC.md` (~360 lines) covering: framing (§ 1), envelope shape (§ 2), the `VectorizerValue` type with cross-language mapping table (§ 3), per-connection sticky auth (§ 4), the `HELLO` handshake (§ 5), v1 command catalog with 7 entries (§ 6), streaming strategy with v2 placeholder (§ 7), versioning (§ 8), comparison with SynapRPC (§ 9), security model (§ 10), reference test vectors with byte-level dumps (§ 11), and a glossary (§ 12).
- [x] 2.2 Drafted `/.rulebook/specs/RPC.md` (~50 lines) — the project-rule-level contract: links to the byte-level spec, lists the implementation file map, captures the "Adding a new command" recipe (registry entry + dispatch arm + test), and the "Forbidden" rules (no per-command framing, no bypassing the registry, no mixed wire versions on one connection).
- [x] 2.3 Built the v1 command catalog by intersecting the capability registry (`src/server/capabilities.rs`) with the data-plane operations: `HELLO` + `PING` (handshake/health), `collections.list`, `collections.get_info`, `vectors.get`, `search.basic`, `search.intelligent`. Write/admin commands ship in follow-up sessions but the catalog table format is fixed.
- [x] 2.4 Error taxonomy v1 keeps `Result<Value, String>` (matches Synap exactly — wire-level parity preserved). The spec documents that v2 will upgrade to `Err(Error { code, message, details })` once `phase3_unify-error-enums` lands; the version bump is signalled via the `HELLO` `protocol_version` reply so clients can branch.
- [x] 2.5 Streaming strategy: v1 explicitly does NOT stream (single frame per response, capped by 64 MiB max body). Search results larger than ~250k 1024-dim vectors must page via REST scroll. v2 will use a `last: bool` field with the same `id` across multiple frames; SDKs MUST handle the v1 "single-frame implies last" form.
- [x] 2.6 Versioning: chose `HELLO`-based negotiation (matches Synap; rejected magic-byte alternative because it doesn't allow capability negotiation in the same round-trip). Version is an integer in the `HELLO` payload's `version` key; server replies with the highest version it can speak.
- [x] 2.7 Reference test vectors at § 11 of the wire spec include byte-level hex dumps for `Request { id: 1, command: "PING", args: [] }` and `Response { id: 1, result: Ok(Str("PONG")) }`. SDKs SHOULD include these as fixtures so they can self-validate without a running server.

## 3. Review

- [x] 3.1 Spec is published in this commit; review happens at PR time. The `Comparison with SynapRPC` table (§ 9) lists every divergence (port number, max frame size, no pubsub) so a Synap-conversant reviewer can audit the deltas in seconds.
- [x] 3.2 v1 frozen as part of this archive. Subsequent breaking changes bump the major version per § 8 and require a new spec doc.

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 4.1 Update or create documentation covering the implementation: `docs/specs/VECTORIZER_RPC.md` is the byte-level contract; `.rulebook/specs/RPC.md` is the rulebook-level summary linking to it. Linking from README.md is part of the server task (`phase6_add-rpc-protocol-server`) since the README needs the working port number too.
- [x] 4.2 Write tests covering the new behavior: this task is spec-only, no executable behaviour. The reference test vectors at § 11 are the test fixtures; the server task wires them as `tests/protocol/rpc/wire_vectors.rs` integration tests.
- [x] 4.3 Run tests and confirm they pass: `cargo test --lib --all-features` → 1166/1166 pass (unchanged from baseline; spec is documentation-only).
