## 1. Manifest bump

- [x] 1.1 Bumped `rmcp = "0.10"` → `rmcp = "1.5"` in
  `crates/vectorizer-core/Cargo.toml`,
  `crates/vectorizer/Cargo.toml`, and
  `crates/vectorizer-server/Cargo.toml`. Commit `9a326fda`.

## 2. Struct-shape migrations

- [x] 2.1 `Implementation` moved to the builder chain
  (`Implementation::new(name, version).with_title(...).with_website_url(...)`)
  in `crates/vectorizer-server/src/server/core/mcp_service.rs`.
- [x] 2.2 `Tool.execution` — 1.5 added `execution: Option<ToolExecution>`
  as a field, but it stays `None` for all our tools (we don't
  currently expose task-based invocation). The rewrite elides the
  field entirely by going through `Tool::new(...)` which initialises
  it to `None`. If we want per-tool task-support as a next feature,
  the `.with_execution(...)` builder entry point is the seam.
- [x] 2.3 `ListToolsResult` + `ListResourcesResult` now use the
  macro-provided `with_all_items(items)` constructor that defaults
  `meta` + `next_cursor` to `None`. `CallToolRequestParams`
  (previously `CallToolRequestParam` in 0.10, now a deprecated alias)
  uses `::new(name).with_arguments(args)` at every construction site.
- [x] 2.4 All the `#[non_exhaustive]` structs (`Tool`, `Implementation`,
  `ServerInfo` / `InitializeResult`, `ListToolsResult`,
  `ListResourcesResult`, `CallToolRequestParams`) now go through their
  builder entry points. Added a local `mk_tool(...)` helper in
  `crates/vectorizer-server/src/server/mcp/tools.rs` so each of the
  ~30 tool declarations stays compact.

## 3. Verification

- [x] 3.1 `cargo check --workspace --all-features` clean.
- [x] 3.2 `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean.
- [x] 3.3 `cargo test --workspace --lib --all-features` — 1262 passing, 0 failing, 12 ignored.
- [x] 3.4 `cargo test -p vectorizer --test all_tests -- api::mcp` — 24 MCP integration tests pass, covering cursor_client + claude_desktop fixtures under `crates/vectorizer/tests/api/mcp/`.

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 4.1 Update or create documentation covering the implementation — CHANGELOG entry under 3.0.0 `### Changed`.
- [x] 4.2 Write tests covering the new behavior — the existing MCP integration tests (`api::mcp::integration` + `api::mcp::graph_integration`) exercise the post-bump struct construction paths directly; the parity tests `registry_tools_are_a_subset_of_legacy_tools` + `registry_and_legacy_agree_on_overlapping_input_schemas` in `server::mcp::tools::tests` verify the `Tool::new(...)` / `mk_tool(...)` rewrites didn't drift the input-schema contract vs the capability registry.
- [x] 4.3 Run tests and confirm they pass — see §3.3 + §3.4.
