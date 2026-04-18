## 1. Preparation

- [ ] 1.1 Convert `src/db/vector_store.rs` to `src/db/vector_store/mod.rs` (moving file into a directory)
- [ ] 1.2 Sketch the split boundaries in `design.md` with method-to-file mapping

## 2. Sequential migration (1 concern per sub-step, verify compilation each time)

- [ ] 2.1 Extract collection CRUD to `vector_store/collections.rs`; `cargo check`
- [ ] 2.2 Extract vector CRUD to `vector_store/vectors.rs`; `cargo check`
- [ ] 2.3 Extract search dispatch to `vector_store/search.rs`; `cargo check`
- [ ] 2.4 Extract alias management to `vector_store/aliases.rs`; `cargo check`
- [ ] 2.5 Extract snapshot logic to `vector_store/snapshots.rs`; `cargo check`
- [ ] 2.6 Extract persistence orchestration to `vector_store/persistence.rs`; `cargo check`
- [ ] 2.7 Extract HNSW operations to `vector_store/hnsw_ops.rs`; `cargo check`

## 3. Verification

- [ ] 3.1 `cargo clippy --all-targets -- -D warnings` — zero warnings
- [ ] 3.2 Each file ≤700 LOC (or justify in design.md)
- [ ] 3.3 Delete `vector_store.rs.bak` if still present in the repo

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 4.1 Update `docs/architecture/db-layer.md` (or create) with the new module layout
- [ ] 4.2 Existing unit + integration tests continue to pass unchanged; add a module-size regression test
- [ ] 4.3 Run `cargo test --all-features` and confirm all tests pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
