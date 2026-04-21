## 1. Layout

- [ ] 1.1 Create `src/db/collection/` directory; move the existing file to `src/db/collection/mod.rs` initially — not performed. A single `impl Collection` block spans L78-1817 of the current file; mechanically splitting a single impl across multiple files requires restructuring it into multiple `impl Collection { ... }` blocks first. Tackled incrementally: the test module is now a separate sibling file (see 2.6), which alone takes the main file from 2,636 → 1,821 lines.
- [ ] 1.2 Decide the five sub-files (data / index / persistence / graph / quantization) and sketch which `impl Collection` methods go where — captured in the task proposal; the detailed per-method assignment remains for a follow-up pass. The `collection.rs` inherent-impl layout (explicit per-concern `impl Collection` blocks) is prerequisite for moving any chunk to its own file.

## 2. Extraction

- [ ] 2.1 Move the `data` methods to `src/db/collection/data.rs` — gated on 1.1.
- [ ] 2.2 Move the `index` methods to `src/db/collection/index.rs` — gated on 1.1.
- [ ] 2.3 Move the `persistence` methods to `src/db/collection/persistence.rs` — gated on 1.1.
- [ ] 2.4 Move the `graph` methods to `src/db/collection/graph.rs` — gated on 1.1.
- [ ] 2.5 Move the `quantization` methods to `src/db/collection/quantization.rs` — gated on 1.1.
- [x] 2.6 Move the `#[cfg(test)]` block to `src/db/collection/tests.rs` — delivered via `#[path = "collection_tests.rs"] mod tests;` at the bottom of `collection.rs`. 819 lines of tests now live in a dedicated sibling file; the impl-only file shrinks to 1,821 lines.

## 3. Verification

- [x] 3.1 `cargo check --all-features` clean.
- [x] 3.2 Existing tests still pass (no test rewriting) — 28 collection tests pass unchanged.
- [ ] 3.3 Every new sub-file is under 600 lines — partially met. `collection_tests.rs` is 819 lines (test code; high threshold is reasonable); `collection.rs` is 1,821 lines, still over. The 600-line target requires the impl-split pass covered by items 1.1-2.5.

## 4. Tail (mandatory)

- [x] 4.1 Module-level doc comment — `collection_tests.rs` has a `//!` header explaining the `#[path]` wiring; the main `collection.rs` retains its existing doc.
- [x] 4.2 Tests already exist; no new tests required — the 28 unit tests move with the extracted file.
- [x] 4.3 Run `cargo test --all-features` and confirm no drift — 1127/1127 lib, 790/790 integration.

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation
- [x] Write tests covering the new behavior
- [x] Run tests and confirm they pass
