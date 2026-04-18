## 1. Layout

- [ ] 1.1 Create `src/db/collection/` directory; move the existing file to `src/db/collection/mod.rs` initially.
- [ ] 1.2 Decide the five sub-files (data / index / persistence / graph / quantization) and sketch which `impl Collection` methods go where.

## 2. Extraction

- [ ] 2.1 Move the `data` methods to `src/db/collection/data.rs`.
- [ ] 2.2 Move the `index` methods to `src/db/collection/index.rs`.
- [ ] 2.3 Move the `persistence` methods to `src/db/collection/persistence.rs`.
- [ ] 2.4 Move the `graph` methods to `src/db/collection/graph.rs`.
- [ ] 2.5 Move the `quantization` methods to `src/db/collection/quantization.rs`.
- [ ] 2.6 Move the `#[cfg(test)]` block to `src/db/collection/tests.rs`.

## 3. Verification

- [ ] 3.1 `cargo check --all-features` clean.
- [ ] 3.2 Existing tests still pass (no test rewriting).
- [ ] 3.3 Every new sub-file is under 600 lines.

## 4. Tail (mandatory)

- [ ] 4.1 Update the module-level doc comment in `collection/mod.rs` explaining the split.
- [ ] 4.2 Tests already exist; no new tests required — the refactor is layout-only.
- [ ] 4.3 Run `cargo test --all-features` and confirm no drift.
