## 1. Analysis

- [ ] 1.1 Inspect `src/db/graph/` to determine whether `Edge`/`Node`/`RelationshipType` carry db-state or are pure data; record in `design.md`
- [ ] 1.2 Choose Option A (move to models) or Option B (remove re-export) via `rulebook_decision_create`

## 2. Option A — Move to models

- [ ] 2.1 Create `src/models/graph.rs` with the moved struct/enum definitions
- [ ] 2.2 Change `src/db/graph/` to import from `crate::models::graph`
- [ ] 2.3 Delete the re-export in `src/models/mod.rs`; it becomes redundant

## 3. Option B — Remove re-export only

- [ ] 3.1 Delete the `pub use crate::db::graph::{...};` line from `src/models/mod.rs`
- [ ] 3.2 Update all consumers to import from `crate::db::graph::...` directly

## 4. Enforcement

- [ ] 4.1 Add an architectural check (e.g., `cargo-modules` or a custom grep) in CI that rejects `use crate::db::` from within `src/models/`

## 5. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 5.1 Update `docs/architecture/layering.md` with the enforced rule and the rationale for the chosen option
- [ ] 5.2 Ensure existing tests still pass after the import-path shuffle
- [ ] 5.3 Run `cargo test --all-features` and confirm all tests pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
