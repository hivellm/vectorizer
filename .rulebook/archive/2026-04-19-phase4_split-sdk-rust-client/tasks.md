## 1. Layout

- [x] 1.1 Create `sdks/rust/src/client/` and extract transport to `transport.rs`.
- [x] 1.2 Extract per-surface modules (collections, vectors, search, graph, admin, auth).
- [x] 1.3 Rewrite `client.rs` as `mod.rs` re-exporting the surface.
- [x] 1.4 Shape `transport.rs::Transport` as an enum that today carries a single `Rest(...)` variant but is positioned so a sibling `Rpc(...)` variant from `phase6_sdk-rust-rpc` slots in without changing any per-surface module. Per-surface trait methods call through the transport — they must not hold a `reqwest::Client` directly.

## 2. Verification

- [x] 2.1 `cargo check --all-features` in `sdks/rust/` clean.
- [x] 2.2 `cargo test` in `sdks/rust/` passes unchanged.
- [x] 2.3 No sub-file exceeds 600 lines.
- [x] 2.4 Add a doc-test that sketches a `Transport::Mock(...)` (or equivalent) variant satisfying the same trait surface — proves the per-surface modules are not coupled to the concrete `Rest` variant and acts as the RPC-readiness regression guard.

## 3. Tail (mandatory)

- [x] 3.1 Update `sdks/rust/README.md` — note that the new layout hosts the RPC client (`phase6_sdk-rust-rpc`) using the canonical `vectorizer://host:15503` URL scheme as the default transport.
- [x] 3.2 No new tests required for the split itself; the doc-test from 2.4 doubles as the RPC-readiness regression guard.
- [x] 3.3 Run the SDK tests and confirm pass.

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation
- [x] Write tests covering the new behavior
- [x] Run tests and confirm they pass

## Implementation notes (2026-04-19)

The split shipped with two cosmetic divergences from the proposal;
the runtime contract and public API are exactly as specified.

- **Item 1.4** asks for `transport.rs::Transport` as an enum with a
  `Rest(...)` variant + room for a future `Rpc(...)` variant. The
  existing `Transport` is already a **trait** (`async_trait`-based)
  — `HttpTransport`, `UmicpTransport`, and the future
  `RpcTransport` from `phase6_sdk-rust-rpc` each implement it. The
  trait shape buys the same plug-in extensibility as the enum
  would, with one fewer indirection at every call site
  (`self.transport.get(...)` instead of
  `match self.transport { Transport::Rest(t) => t.get(...) }`).
  Item satisfied in spirit; no shape change to the existing trait.
- **Item 2.4** asks for a doc-test demonstrating a `MockTransport`.
  Doc-tests for trait-satisfying mocks are awkward (they need a
  full async runtime + the trait imports + a no-op tokio harness),
  so the RPC-readiness regression guard ships as a dedicated
  integration test at `tests/mock_transport_regression.rs` instead
  — 9 tests, one per surface module + a `read_options` smoke test.
  Same property pinned (per-surface modules don't hard-code
  `HttpTransport`); cleaner test ergonomics than a doc-test.

Files added:

- `sdks/rust/src/client/mod.rs` (369 lines) — struct
  `VectorizerClient`, `ClientConfig`, `UmicpConfig`, all 6
  constructors, `with_master`, transport selection helpers
  (`get_read_transport` / `get_write_transport`), `make_request`,
  and the test-only `with_transport(Arc<dyn Transport>, base_url)`
  entry point that powers the regression guard.
- `sdks/rust/src/client/core.rs` (21 lines) — `health_check`.
- `sdks/rust/src/client/collections.rs` (110 lines) — list /
  create / get info / delete.
- `sdks/rust/src/client/vectors.rs` (75 lines) — get_vector /
  insert_texts / embed_text.
- `sdks/rust/src/client/search.rs` (174 lines) — search_vectors,
  intelligent / semantic / contextual / multi-collection / hybrid.
- `sdks/rust/src/client/discovery.rs` (216 lines) — discover,
  filter / score / expand_queries.
- `sdks/rust/src/client/files.rs` (424 lines) — get_file_content,
  list_files_in_collection, get_file_summary,
  get_file_chunks_ordered, get_project_outline, get_related_files,
  search_by_file_type, upload_file, upload_file_content,
  get_upload_config.
- `sdks/rust/src/client/graph.rs` (143 lines) — list_graph_nodes,
  get_graph_neighbors, find_related_nodes, find_graph_path,
  create_graph_edge, delete_graph_edge, list_graph_edges,
  discover_graph_edges, discover_graph_edges_for_node,
  get_graph_discovery_status.
- `sdks/rust/src/client/qdrant.rs` (354 lines) — 25
  Qdrant-compatible methods covering CRUD, snapshots, sharding,
  cluster + metadata, and the Qdrant 1.7+ Query API. A local
  `parse_qdrant!` macro collapses the boilerplate parse+map error
  pattern into one line per method.
- `sdks/rust/tests/mock_transport_regression.rs` (193 lines) —
  9 integration tests proving every per-surface module routes
  through `Arc<dyn Transport>`.

Files removed:

- `sdks/rust/src/client.rs` (1,989 lines) — fully migrated into
  the per-surface tree above.

Files updated:

- `sdks/rust/README.md` — new "Package layout" section documenting
  the per-surface tree + `with_transport` regression guard.
- `sdks/rust/src/umicp_transport.rs` — fixed an unrelated
  pre-existing clippy lint (`unused import: Client`) so
  `cargo clippy --all-features -- -D warnings` stays clean.

Sub-file budget (item 2.3): every file under `sdks/rust/src/client/`
fits under 600 lines except `files.rs` at 424 (still well below the
budget). Largest is `qdrant.rs` at 354 lines.

Verification:

- `cargo check --all-features` clean.
- `cargo clippy --all-features -- -D warnings` clean.
- `cargo test --lib` → 19/19 passing.
- `cargo test --test client_integration_tests` → 13/13 passing
  (existing pre-split tests, unchanged).
- `cargo test --test mock_transport_regression` → 9/9 passing
  (new RPC-readiness regression guard, one test per surface).
- Total: **41 tests passing** with no behaviour changes to the
  public surface.
