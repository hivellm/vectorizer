## 1. Server endpoint — POST /collections/{src}/vectors/move
- [ ] 1.1 Define request struct `MoveVectorsRequest { destination, ids }` in `crates/vectorizer-server/src/server/rest_handlers/vectors.rs`
- [ ] 1.2 Define response structs `MoveVectorsResponse` + `MoveVectorResult { id, status }` with status enum `ok | missing_in_src | dst_insert_failed | src_delete_failed`
- [ ] 1.3 Implement handler `move_vectors`: load src vector by id, insert into dst (carrying payload + raw vector data — NOT re-embed), then delete from src; capture per-id status without aborting the batch
- [ ] 1.4 Register route in `crates/vectorizer-server/src/server/core/routing.rs` as `POST /collections/{name}/vectors/move`
- [ ] 1.5 Verify auth + RBAC layer applies same as existing `delete_vector` / `batch_delete_vectors` routes

## 2. Server tests
- [ ] 2.1 Integration test: happy path moves N vectors, response matches contract
- [ ] 2.2 Integration test: missing id in src yields `missing_in_src` and does NOT touch dst
- [ ] 2.3 Integration test: dim mismatch between src and dst yields `dst_insert_failed` and src vector is NOT deleted (insert-before-delete invariant)
- [ ] 2.4 Integration test: partial-batch behavior — a single failure does not abort the batch; remaining ids still process

## 3. Rust SDK (sdks/rust)
- [ ] 3.1 Add `DeleteReport`, `MoveReport`, `VectorOpResult { id, status }` types under `sdks/rust/src/models/`
- [ ] 3.2 Implement `delete_vector(&self, collection, vector_id) -> Result<()>` calling `DELETE /collections/{name}/vectors/{id}`
- [ ] 3.3 Implement `delete_vectors(&self, collection, ids) -> Result<DeleteReport>` calling `POST /batch_delete`
- [ ] 3.4 Implement `move_to_collection(&self, src, dst, ids) -> Result<MoveReport>` calling `POST /collections/{src}/vectors/move`
- [ ] 3.5 Re-export types from `sdks/rust/src/lib.rs` so consumers can pattern-match per-id statuses
- [ ] 3.6 Bump `sdks/rust/Cargo.toml` version 3.2 → 3.3

## 4. Rust SDK tests
- [ ] 4.1 Unit test: each method serializes + deserializes the wire payload
- [ ] 4.2 Integration test against a live vectorizer-server instance (mirror `s2s-tests` style)

## 5. TypeScript SDK (sdks/typescript)
- [ ] 5.1 Add `DeleteReport`, `MoveReport`, `VectorOpResult` interfaces under `sdks/typescript/src/models/`
- [ ] 5.2 Implement `deleteVector(collection, vectorId): Promise<void>` in `client/vectors.ts`
- [ ] 5.3 Implement `deleteVectors(collection, ids): Promise<DeleteReport>` in `client/vectors.ts`
- [ ] 5.4 Implement `moveToCollection(src, dst, ids): Promise<MoveReport>` in `client/vectors.ts`
- [ ] 5.5 Export new types from `sdks/typescript/src/index.ts`
- [ ] 5.6 Bump `sdks/typescript/package.json` version 3.2 → 3.3

## 6. TypeScript SDK tests
- [ ] 6.1 Unit tests for each method's request shape (vitest, matching existing patterns)
- [ ] 6.2 Integration test against a live vectorizer-server instance

## 7. Python SDK (sdks/python)
- [ ] 7.1 Add `DeleteReport`, `MoveReport`, `VectorOpResult` dataclasses
- [ ] 7.2 Implement `delete_vector(collection, vector_id)` on the client
- [ ] 7.3 Implement `delete_vectors(collection, ids) -> DeleteReport`
- [ ] 7.4 Implement `move_to_collection(src, dst, ids) -> MoveReport`
- [ ] 7.5 Bump `sdks/python/pyproject.toml` version 3.2 → 3.3

## 8. Python SDK tests
- [ ] 8.1 pytest unit tests for each method's request shape
- [ ] 8.2 pytest integration test against a live vectorizer-server instance

## 9. Documentation
- [ ] 9.1 Update server API reference under `docs/` to document `POST /collections/{src}/vectors/move`
- [ ] 9.2 Update `sdks/rust/README.md` with examples for the three new methods
- [ ] 9.3 Update `sdks/typescript/README.md` with examples
- [ ] 9.4 Update `sdks/python/README.md` with examples
- [ ] 9.5 Add a "tier demotion" section to one SDK README showing the cortex pruner pattern (get → move → delete loop)
- [ ] 9.6 Update `CHANGELOG.md` Unreleased entries (server + each SDK)

## 10. Tail (mandatory — enforced by rulebook v5.3.0)
- [ ] 10.1 Update or create documentation covering the implementation
- [ ] 10.2 Write tests covering the new behavior
- [ ] 10.3 Run tests and confirm they pass
