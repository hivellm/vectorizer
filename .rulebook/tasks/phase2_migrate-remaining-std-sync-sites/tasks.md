## 1. Migration (one file per sub-step, `cargo check` between each)

- [ ] 1.1 `src/db/collection.rs` — primary core, highest blast radius. Do first.
- [ ] 1.2 `src/persistence/wal.rs`
- [ ] 1.3 `src/persistence/dynamic.rs`
- [ ] 1.4 `src/cache/advanced_cache.rs`
- [ ] 1.5 `src/quantization/storage.rs`
- [ ] 1.6 `src/quantization/hnsw_integration.rs`
- [ ] 1.7 `src/db/quantized_collection.rs`
- [ ] 1.8 `src/storage/advanced.rs`
- [ ] 1.9 `src/storage/reader.rs`
- [ ] 1.10 `src/search/advanced_search.rs`
- [ ] 1.11 `src/api/advanced_api.rs`
- [ ] 1.12 `src/processing/advanced_pipeline.rs`
- [ ] 1.13 `src/ml/advanced_ml.rs`
- [ ] 1.14 `src/security/enhanced_security.rs`

## 2. Enforcement

- [ ] 2.1 Add a CI grep gate in `.github/workflows/rust-lint.yml` that fails if any new `use std::sync::\{.*(Mutex|RwLock)` appears in `src/`
- [ ] 2.2 Run the gate locally; confirm zero hits

## 3. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 3.1 Update `/.rulebook/specs/RUST.md` clarifying allowed `std::sync` items (Arc, atomic, Once, OnceLock, Condvar, mpsc)
- [ ] 3.2 Run `cargo test --all-features` and confirm no regressions
- [ ] 3.3 Run `cargo clippy --all-targets -- -D warnings` and confirm zero new warnings

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
