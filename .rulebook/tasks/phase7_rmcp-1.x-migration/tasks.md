## 1. Manifest bump

- [ ] 1.1 Bump `rmcp = "0.10"` → `rmcp = "1.5"` in the three manifests (`vectorizer-core`, `vectorizer`, `vectorizer-server`).

## 2. Struct-shape migrations (the 71 compile errors)

- [ ] 2.1 `Implementation { description: Some(..) }` — add at every construction site (server info advertise path).
- [ ] 2.2 `Tool { execution: .. }` — research the v1 execution-policy enum and choose the mapping from our existing per-tool config.
- [ ] 2.3 `ListToolsResult { meta: None }`, `ListResourcesResult { meta: None }`, `CallToolRequestParams { meta: None, task: None }` — add across 10 files.
- [ ] 2.4 Every `#[non_exhaustive]` struct literal — switch to the builder-style constructor where 1.x provides one, otherwise add the missing fields explicitly.

## 3. Verification

- [ ] 3.1 `cargo check --workspace --all-features` clean.
- [ ] 3.2 `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean.
- [ ] 3.3 `cargo test --workspace --lib --all-features` green.
- [ ] 3.4 Run the MCP integration tests under `crates/vectorizer-server/tests/mcp/` and confirm cursor + claude-desktop fixtures still pass.

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 4.1 Update or create documentation covering the implementation (`docs/specs/MCP.md` plus a CHANGELOG entry under `### Changed`).
- [ ] 4.2 Write tests covering the new behavior (golden-response tests for the new `meta` / `execution` fields).
- [ ] 4.3 Run tests and confirm they pass.
