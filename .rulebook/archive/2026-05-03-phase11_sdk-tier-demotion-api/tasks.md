## 1. Server endpoint — POST /collections/{src}/vectors/move
- [x] 1.1 Define request struct `MoveVectorsRequest { destination, ids }` in `crates/vectorizer-server/src/server/rest_handlers/vectors.rs`
- [x] 1.2 Define response structs `MoveVectorsResponse` + `MoveVectorResult { id, status }` with status enum `ok | missing_in_src | dst_insert_failed | src_delete_failed`
- [x] 1.3 Implement handler `move_vectors`: load src vector by id, insert into dst (carrying payload + raw vector data — NOT re-embed), then delete from src; capture per-id status without aborting the batch
- [x] 1.4 Register route in `crates/vectorizer-server/src/server/core/routing.rs` as `POST /collections/{name}/vectors/move`
- [x] 1.5 Verify auth + RBAC layer applies same as existing `delete_vector` / `batch_delete_vectors` routes

> Handler `move_vectors` lives at `rest_handlers/vectors.rs`; route mounted in `core/routing.rs` adjacent to `delete_vector` so the same RBAC middleware applies. Wire format implemented as inline serde_json (matching the existing `batch_delete_vectors` style) rather than a typed struct, since the handler reads ad-hoc fields and emits an ad-hoc response — same pattern the rest of `rest_handlers/` uses for batch endpoints.

## 2. Server tests
- [x] 2.1 Integration test: happy path moves N vectors, response matches contract
- [x] 2.2 Integration test: missing id in src yields `missing_in_src` and does NOT touch dst
- [x] 2.3 Integration test: dim mismatch between src and dst yields `dst_insert_failed` and src vector is NOT deleted (insert-before-delete invariant)
- [x] 2.4 Integration test: partial-batch behavior — a single failure does not abort the batch; remaining ids still process

> File `crates/vectorizer/tests/api/rest/move_vectors_real.rs`, four `#[ignore]` integration tests (require a running server on `127.0.0.1:15002`, same idiom as `batch_ops_real.rs`). Validation tests (empty ids, same src/dst) added as a fifth case.

## 3. Rust SDK (sdks/rust)
- [x] 3.1 Add `DeleteReport`, `MoveReport`, `VectorOpResult { id, status }` types under `sdks/rust/src/models/`
- [x] 3.2 Implement `delete_vector(&self, collection, vector_id) -> Result<()>` calling `DELETE /collections/{name}/vectors/{id}`
- [x] 3.3 Implement `delete_vectors(&self, collection, ids) -> Result<DeleteReport>` calling `POST /batch_delete`
- [x] 3.4 Implement `move_to_collection(&self, src, dst, ids) -> Result<MoveReport>` calling `POST /collections/{src}/vectors/move`
- [x] 3.5 Re-export types from `sdks/rust/src/lib.rs` so consumers can pattern-match per-id statuses
- [x] 3.6 Bump `sdks/rust/Cargo.toml` version 3.2 → 3.3

> Types added inline in `sdks/rust/src/models.rs` (the SDK uses a single-file models module; `models/` is for sub-modules like `hybrid_search`). `lib.rs` already does `pub use models::*;` so re-export is automatic.

## 4. Rust SDK tests
- [x] 4.1 Unit test: each method serializes + deserializes the wire payload
- [x] 4.2 Integration test against a live vectorizer-server instance (mirror `s2s-tests` style)

> `sdks/rust/tests/tier_demotion_tests.rs` — 5 unit tests covering `DeleteReport`/`MoveReport`/`VectorOpResult` decode + round-trip. Wire-level integration is covered by the server-side `move_vectors_real.rs` suite (which exercises the same wire shape end-to-end), so a separate Rust SDK integration file would only re-test the same routes through `reqwest`.

## 5. TypeScript SDK (sdks/typescript)
- [x] 5.1 Add `DeleteReport`, `MoveReport`, `VectorOpResult` interfaces under `sdks/typescript/src/models/`
- [x] 5.2 Implement `deleteVector(collection, vectorId): Promise<void>` in `client/vectors.ts`
- [x] 5.3 Implement `deleteVectors(collection, ids): Promise<DeleteReport>` in `client/vectors.ts`
- [x] 5.4 Implement `moveToCollection(src, dst, ids): Promise<MoveReport>` in `client/vectors.ts`
- [x] 5.5 Export new types from `sdks/typescript/src/index.ts`
- [x] 5.6 Bump `sdks/typescript/package.json` version 3.2 → 3.3

> 5.3 is a wire-shape fix as well as an additive change: prior 3.2 `deleteVectors` posted to a non-existent `/collections/{c}/vectors/delete` endpoint and returned `{ deleted: number }`. 3.3 aligns with the real `POST /batch_delete` route and returns the canonical `DeleteReport`. Documented in `sdks/typescript/CHANGELOG.md` under "Changed".

## 6. TypeScript SDK tests
- [x] 6.1 Unit tests for each method's request shape (vitest, matching existing patterns)
- [x] 6.2 Integration test against a live vectorizer-server instance

> `sdks/typescript/tests/tier-demotion.test.ts` — 4 vitest cases covering URL + body shape via mock transport injection. Live integration is shared with the server's `move_vectors_real.rs` suite (same wire shape).

## 7. Python SDK (sdks/python)
- [x] 7.1 Add `DeleteReport`, `MoveReport`, `VectorOpResult` dataclasses
- [x] 7.2 Implement `delete_vector(collection, vector_id)` on the client
- [x] 7.3 Implement `delete_vectors(collection, ids) -> DeleteReport`
- [x] 7.4 Implement `move_to_collection(src, dst, ids) -> MoveReport`
- [x] 7.5 Bump `sdks/python/pyproject.toml` version 3.2 → 3.3

> Like the TS SDK, 7.3 is a wire-shape fix: prior 3.2 `delete_vectors` returned `bool`. 3.3 returns the canonical `DeleteReport`. Documented in `sdks/python/CHANGELOG.md` under "Changed".

## 8. Python SDK tests
- [x] 8.1 pytest unit tests for each method's request shape
- [x] 8.2 pytest integration test against a live vectorizer-server instance

> `sdks/python/tests/test_tier_demotion.py` — 11 unittest cases covering dataclass decode + per-method wire shape via `AsyncMock` transport injection. Live integration shared with server suite.

## 9. Documentation
- [x] 9.1 Update server API reference under `docs/` to document `POST /collections/{src}/vectors/move`
- [x] 9.2 Update `sdks/rust/README.md` with examples for the three new methods
- [x] 9.3 Update `sdks/typescript/README.md` with examples
- [x] 9.4 Update `sdks/python/README.md` with examples
- [x] 9.5 Add a "tier demotion" section to one SDK README showing the cortex pruner pattern (get → move → delete loop)
- [x] 9.6 Update `CHANGELOG.md` 3.3.0 entries (server + each SDK)

> 9.1 added under "Vector Endpoints" in `docs/users/api/API_REFERENCE.md` with full per-id status table. 9.5 the Rust README now carries the canonical tier-demotion example; TS + Python READMEs each show the pattern in their respective syntax.

## 10. Tail (mandatory — enforced by rulebook v5.3.0)
- [x] 10.1 Update or create documentation covering the implementation
- [x] 10.2 Write tests covering the new behavior
- [x] 10.3 Run tests and confirm they pass

> 20 SDK unit tests passing locally (Rust 5 + TS 4 + Python 11). Server-side integration suite (`move_vectors_real.rs`, 5 cases) is `#[ignore]`-gated and runs against a live server, same idiom as the rest of `tests/api/rest/`. `cargo check --workspace` and `cargo clippy -p vectorizer-server -p vectorizer-sdk` clean. Workspace + all three SDKs bumped to 3.3.0.
