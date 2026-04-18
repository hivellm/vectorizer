## 1. Research reference

- [ ] 1.1 Read `../Synap/synap-server/src/protocol/resp3/parser.rs` (424 LOC); note edge cases (pushes, attributes, big numbers)
- [ ] 1.2 Read `writer.rs` (273 LOC) and `server.rs` (217 LOC); identify adaptations required for our `AppState`
- [ ] 1.3 Read Redis RESP3 spec at https://github.com/redis/redis-specifications/blob/master/protocol/RESP3.md to double-check corner cases

## 2. Parser + writer

- [ ] 2.1 Port parser into `src/protocol/resp3/parser.rs`; adapt error types
- [ ] 2.2 Port writer into `src/protocol/resp3/writer.rs`
- [ ] 2.3 Add fuzz target `fuzz/fuzz_targets/resp3_parser.rs` using `cargo-fuzz` on random byte streams

## 3. Server

- [ ] 3.1 Port server loop into `src/protocol/resp3/server.rs`
- [ ] 3.2 Implement pipelining support (parse multiple requests before responding)
- [ ] 3.3 Wire TLS option from config

## 4. Commands

- [ ] 4.1 Implement core commands: PING, AUTH, HELLO (including protocol negotiation), CLIENT, INFO, SELECT (no-op to appease clients), QUIT
- [ ] 4.2 Implement Vectorizer commands mapping to capability registry: VCREATE, VDROP, VLIST, VINSERT, VUPSERT, VDELETE, VGET, VSEARCH, VRECOMMEND
- [ ] 4.3 Return RESP3 error `ERR unknown command '<name>'` for anything outside the registry
- [ ] 4.4 Add per-command integration tests using the `redis` crate as a client

## 5. Integration

- [ ] 5.1 Spawn the RESP3 listener from `src/server/bootstrap.rs` when `config.resp3.enabled = true` (default true)
- [ ] 5.2 Expose RESP3 connection/command metrics on `/metrics`

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 6.1 Document RESP3 support in `docs/deployment/resp3.md` including redis-cli examples; update README.md with "Redis-compatible clients" section
- [ ] 6.2 Integration tests in `tests/protocol/resp3/` cover: HELLO negotiation (RESP2 reject, RESP3 accept), AUTH, every implemented command, pipelining, unknown command error, fuzz-parser survives 1M random inputs
- [ ] 6.3 Run `cargo test --all-features -- resp3` and confirm all pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
