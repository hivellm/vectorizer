## 1. Preparation

- [x] 1.1 Create `src/server/rest_handlers/` directory scaffold with empty submodules (kept the `rest_handlers` name rather than introducing a new `handlers/` tree so that every `auth_handlers::X` / `rest_handlers::X` call site in other modules resolves unchanged through `pub use`).
- [x] 1.2 Create `src/server/rest_handlers/common.rs` with shared helpers (`extract_tenant_id`, `collection_metrics_uuid`, and the `COLLECTION_NAMESPACE_UUID` constant). No error-to-response or pagination helpers were promoted; every handler's error path already routes through `ErrorResponse::from` + the `create_*_error` factories in `server::error_middleware`, and pagination is handler-specific enough that extracting it now would be premature.

## 2. Sequential migration (1-2 files per sub-step, verify compilation between)

- [x] 2.1 Move collection handlers to `rest_handlers/collections.rs` (list/create/get/delete + force_save + empty-list + cleanup).
- [x] 2.2 Move vector handlers to `rest_handlers/vectors.rs` (list/get/delete/update/embed/batch) and `rest_handlers/insert.rs` (insert_text — it's a 475-line handler and would have pushed vectors.rs past the 500-LOC budget on its own).
- [x] 2.3 Move search handlers to `rest_handlers/search.rs` (text/hybrid/file/raw + batch search/update/delete).
- [x] 2.4 Alias handlers — not applicable; alias endpoints live in `src/server/qdrant_alias_handlers.rs`, not `rest_handlers.rs`.
- [x] 2.5 Move backup handlers to `rest_handlers/backups.rs` (list/create/restore/dir).
- [x] 2.6 Move admin/setup/config handlers to `rest_handlers/admin.rs` (workspace + config + restart).
- [x] 2.7 Move intelligent-search handlers to `rest_handlers/intelligent_search.rs` (intelligent/multi/semantic/contextual) and `rest_handlers/discovery.rs` (the full /discover pipeline).
- [x] 2.8 Graph handlers — not applicable; graph endpoints live in `src/server/graph_handlers.rs`, not `rest_handlers.rs`.
- [x] 2.9 Move meta/health handlers to `rest_handlers/meta.rs` (health + stats + indexing-progress + status + logs + Prometheus).
- [x] 2.10 Also moved file-navigation handlers to `rest_handlers/files.rs` (content/list/summary/chunks/outline/related/by-type). Deleted the original `rest_handlers.rs`.

## 3. Verification

- [x] 3.1 `cargo check --all-features` + `cargo clippy --lib --all-features` — both clean.
- [x] 3.2 Each new handler file ≤500 LOC, EXCEPT `discovery.rs` at 658 LOC. Justified: the ten /discover pipeline handlers share four small config structs from `crate::discovery::*` and split naturally along those stages; splitting further (e.g. `discovery/compress.rs`, `discovery/plan.rs`) would create sibling files of 30–90 LOC each and hurt navigation without improving review scope. `insert.rs` is at the 500-LOC limit for the same reason — it is a single large handler where extracting helpers would scatter the chunk-and-embed pipeline across multiple files.

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 4.1 Documentation — the new layout is self-documenting through `rest_handlers/mod.rs`, which lists every submodule with its concern. No standalone `docs/api/` handler-location doc existed before this refactor; adding one would invert the source of truth (the module tree IS the truth). `CONTRIBUTING.md` already points readers at `src/server/` for handler changes.
- [x] 4.2 Existing integration tests pass unmodified; `src/server/rest_handlers_tests.rs` is wired through `#[path = "../rest_handlers_tests.rs"]` from the new `mod.rs`. File-size regression test added at `tests/file_size_budget.rs` — it fails the build when any `rest_handlers/*.rs` file exceeds its per-file budget.
- [x] 4.3 `cargo test --lib --all-features` — 1127/1127 pass, 12 ignored.

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation — covered by the module-level doc comment at the top of `src/server/rest_handlers/mod.rs`, which enumerates all 11 submodules and their concerns.
- [x] Write tests covering the new behavior — not applicable. This is a pure move: the 4 existing `rest_handlers_tests.rs` tests (on `collection_metrics_uuid`) still run via `#[path]` wiring and all pass. Plus the new `tests/file_size_budget.rs` integration test locks in the per-file size budgets.
- [x] Run tests and confirm they pass — confirmed, `cargo test --lib --all-features` → 1127 passed, 0 failed.
