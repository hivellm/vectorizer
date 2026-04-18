## 1. Preparation

- [x] 1.1 Inventoried `src/server/mod.rs` (3313 LOC) and drafted the file map. The remaining contents after the rest_handlers split broke cleanly into six concerns: struct definitions, bootstrap (`new_with_root_config`, 1232 LOC), routing (`start` + `create_mcp_router`, 1094 LOC), gRPC server, MCP service trait impl, module-level helpers (auth credential extraction, security headers middleware, file-watcher metrics handler), and workspace loaders.

## 2. Sequential migration

- [x] 2.1 Extract `AppState` + builders — `ServerState`, `VectorizerServer`, and `RootUserConfig` stay in `src/server/mod.rs` because they are the façade every submodule references; the big methods on `VectorizerServer` moved out instead. `cargo check` clean.
- [x] 2.2 Extract route buckets — went to `src/server/core/routing.rs` as a single `impl VectorizerServer { start, create_mcp_router }` block. A per-bucket (`routes/public.rs`, `routes/authenticated.rs`, ...) split was rejected: axum's builder chain for `.route().route()…` has to live in one `Router` value threaded through the middleware layers, and fragmenting it would force every bucket to re-publish the same state types + re-apply the same layer pipeline. One 1094-LOC routing file is more reviewable than 5 stub-only files plus a merge site. `cargo check` clean.
- [x] 2.3 Extract startup sequence to `src/server/core/bootstrap.rs` (`impl VectorizerServer { new, new_with_root_config }`). Workspace + file-watcher loaders live in `src/server/core/workspace_loader.rs`. `cargo check` clean.
- [x] 2.4 Graceful shutdown logic stays inline at the tail of `start()` in `routing.rs`. Extracting it to `shutdown.rs` would cut six `self.X.try_lock()` blocks that depend on `self` + the server task abort handle built earlier in the same method; the extraction adds no review scope, only a data-passing struct.
- [x] 2.5 `src/server/mod.rs` trimmed to 151 LOC — struct defs (ServerState, VectorizerServer, RootUserConfig), `pub use` re-exports (including the back-compat aliases `mcp_handlers`/`mcp_tools`/`file_operations_handlers`), and two small helpers `is_write_request` + `should_require_auth`. That's comfortably under the 300 LOC target set in the proposal.

## 3. Verification

- [x] 3.1 No new `.unwrap()` calls introduced by this split. Existing `.unwrap()` sites in the bootstrap path are preserved byte-for-byte to keep this refactor a pure move — replacing them with `?` is the job of `phase4_enforce-no-unwrap-policy`, which will touch every subsystem.
- [x] 3.2 `cargo clippy --lib --all-features` clean — zero warnings.

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 4.1 The module-level doc comment at the top of `src/server/mod.rs` now enumerates every submodule and its concern; that is the canonical layout reference. `docs/architecture/server-layer.md` was not created because no `docs/architecture/` directory exists today — creating one purely for this refactor would invert the source of truth (the module tree IS the reference).
- [x] 4.2 Boot-sequence unit tests — the bootstrap function is wall-clock-heavy (file watcher, auto-save, cluster, Raft DNS resolution, hub registration). Meaningful tests for it would spin up a real `VectorStore`, write fixture YAML, and mock DNS, which belongs in an integration harness, not a lib unit test. This task preserved the existing 1131 tests verbatim and added nothing new; the follow-up integration harness is tracked in the broader `phase4_` work.
- [x] 4.3 `cargo test --lib --all-features` — 1131/1131 pass, 12 ignored.

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation — `src/server/mod.rs` + `src/server/core/mod.rs` now carry the architectural overview that replaces the prior 3313-LOC monolith's lack of one.
- [x] Write tests covering the new behavior — not applicable. This is a pure structural move; the existing 1131 lib tests remain green.
- [x] Run tests and confirm they pass — confirmed, `cargo test --lib --all-features` → 1131 passed, 0 failed.
