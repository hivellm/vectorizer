## 1. Skeleton (no refactor — every `use crate::X` keeps working)

- [x] 1.1 `mkdir crates/vectorizer`
- [x] 1.2 `git mv src crates/vectorizer/src` and `git mv tests crates/vectorizer/tests`
- [x] 1.3 `git mv Cargo.toml crates/vectorizer/Cargo.toml` and `git mv build.rs crates/vectorizer/build.rs`. New root `Cargo.toml` is `[workspace] resolver = "2"` + `members = ["crates/*"]` + `[workspace.lints.{clippy,rust,rustdoc}]` + the full `[profile.*]` set (release, ci, release-fast, perf, dev). `[workspace.dependencies]` centralisation is its own follow-up — phase 1 keeps `[dependencies]` inline on the moved crate to minimise diff.
- [x] 1.4 `[lints.clippy]`, `[lints.rust]`, `[lints.rustdoc]` lifted into `[workspace.lints.*]` on the root manifest. `crates/vectorizer/Cargo.toml` opts in via `[lints] workspace = true`.
- [x] 1.5 `[[bench]]`, `[[bin]]`, `[features]`, `[package.metadata.deb]` stayed in the moved `crates/vectorizer/Cargo.toml` (no movement needed — they're crate-scoped).
- [x] 1.6 Path-relative entries in `crates/vectorizer/Cargo.toml` repointed to walk two levels up: `[[bench]] path = "benches/..."` → `"../../benches/..."`, `[[bin]] path = "scripts/dev/test_routes.rs"` → `"../../scripts/dev/test_routes.rs"`, `[package.metadata.deb] assets[]`: `target/release/...` → `../../target/release/...`, `README.md` → `../../README.md`, `config/...` → `../../config/...`, `dashboard/...` → `../../dashboard/...`, `workspace.example.yml` → `../../workspace.example.yml`. `license-file` → `../../LICENSE`, `maintainer-scripts` → `../../debian/`. `crates/vectorizer/build.rs` repointed: `proto/...` → `../../proto/...`, `assets/icon.ico` → `../../assets/icon.ico`. `crates/vectorizer/src/server/embedded_assets.rs` `#[folder = "dashboard/dist"]` → `"../../dashboard/dist"`. `crates/vectorizer/tests/config/layered_real_files.rs::repo_root()` walks up two parents now (also fixes a latent regression from `phase4_consolidate-repo-layout` phase 2 where the test still pointed at `config.example.yml` instead of `config/config.example.yml`).
- [x] 1.7 `cargo check --workspace` clean
- [x] 1.8 `cargo build --workspace --bin vectorizer` clean (server binary still builds end-to-end)
- [x] 1.9 `cargo test --workspace --lib` → **1210/1210 passing** (7 ignored, no behaviour change vs pre-split baseline)
- [x] 1.10 `cargo clippy --workspace -- -D warnings` clean

## 2. Extract `vectorizer-protocol`

- [x] 2.1 Created `crates/vectorizer-protocol/{Cargo.toml, src/lib.rs, build.rs}`. Lib is `#![deny(missing_docs)]` and exposes `pub mod rpc_wire` + `pub mod grpc_gen { vectorizer; cluster; qdrant_proto }`.
- [x] 2.2 `git mv` of the wire-only RPC pieces: `crates/vectorizer/src/protocol/rpc/{types.rs,codec.rs}` → `crates/vectorizer-protocol/src/rpc_wire/`. The `dispatch.rs` + `server.rs` halves stay in `vectorizer` because they pull in `crate::db::VectorStore`, `crate::embedding::EmbeddingManager`, `crate::server::AuthHandlerState`, and `crate::auth::roles::Role` — none of which belong on the wire-protocol crate.
- [x] 2.3 `crates/vectorizer/src/codec.rs` (generic bincode wrapper used by cluster/embedding-cache/normalization/persistence) **stays** in `vectorizer`. Not moved — it's a serialization helper, not a wire type.
- [x] 2.4 Wire-shaped types in `models/` **stay in vectorizer for sub-phase 2** to keep this commit narrow. They're tangled with server-internal types (`Vector`, `Payload`, `SparseVector` are used by both REST/gRPC handlers and the storage engine). The orphan-rule fallout this would create is large; a dedicated sub-phase covers it once the server crate is extracted.
- [x] 2.5 `git mv proto crates/vectorizer-protocol/proto/`. Generated proto modules removed via `git rm crates/vectorizer/src/grpc/{vectorizer.rs,vectorizer.cluster.rs,qdrant/qdrant.rs}`; `vectorizer-protocol/build.rs` regenerates them into `crates/vectorizer-protocol/src/grpc_gen/` on the next build.
- [x] 2.6 `crates/vectorizer-protocol/build.rs` owns proto compilation now (with `protoc-bin-vendored`, same vendoring trick as before). `crates/vectorizer/build.rs` slimmed to the Windows icon-resource embed only; `tonic-build` / `tonic-prost-build` / `protoc-bin-vendored` build-deps moved to the new crate.
- [x] 2.7 `vectorizer-protocol = { path = "../vectorizer-protocol" }` added to `crates/vectorizer/Cargo.toml`.
- [x] 2.8 Re-exports added: `vectorizer::grpc::{vectorizer,cluster,qdrant_proto}` re-export from `vectorizer_protocol::grpc_gen::*`; `vectorizer::protocol::rpc::{codec,types,Request,Response,VectorizerValue}` re-export from `vectorizer_protocol::rpc_wire::*`. Two `include!("../grpc/vectorizer.cluster.rs")` call sites in `cluster/{server_client,state_sync}.rs` migrated to `use crate::grpc::cluster as cluster_proto`. One orphan-rule fallout fixed: `impl TryFrom<&vectorizer::HybridSearchRequest> for (Vec<f32>, Option<SparseVector>, HybridSearchConfig)` rewritten as a free function `hybrid_search_request_to_engine_args` (every component foreign to `vectorizer` once `HybridSearchRequest` moved out).
- [x] 2.9 `cargo build --workspace --bin vectorizer` + `cargo test --workspace --lib` clean.
- [x] 2.10 `cargo clippy --workspace -- -D warnings` clean.

## 3. Extract `vectorizer-core`

- [ ] 3.1 Create `crates/vectorizer-core/{Cargo.toml, src/lib.rs}`
- [ ] 3.2 Move storage/indexing engine: `git mv crates/vectorizer/src/db crates/vectorizer-core/src/db`
- [ ] 3.3 Move embedding pipeline: `git mv crates/vectorizer/src/embedding crates/vectorizer-core/src/embedding`
- [ ] 3.4 Move quantization, cache, parallel, normalization, hybrid_search, intelligent_search, search/, models/ (server-internal subset only — wire types already moved to protocol)
- [ ] 3.5 Move `src/error/` into core (it's the shared error type)
- [ ] 3.6 Move `src/persistence/`, `src/file_loader/`, `src/file_operations/`, `src/file_watcher/`, `src/discovery/` into core
- [ ] 3.7 Move `src/quantization/`, `src/compression/` into core
- [ ] 3.8 Add `vectorizer-core = { path = "../vectorizer-core" }` to vectorizer + vectorizer-protocol's reverse-dep deps
- [ ] 3.9 Update umbrella crate to re-export from `vectorizer-core::*`
- [ ] 3.10 `cargo build --workspace` + `cargo test --workspace --lib` + `cargo clippy --workspace -- -D warnings` clean

## 4. Extract `vectorizer-server`

- [ ] 4.1 Create `crates/vectorizer-server/{Cargo.toml, src/lib.rs, src/bin/server.rs}`
- [ ] 4.2 Move HTTP/gRPC/MCP layers: `git mv crates/vectorizer/src/server crates/vectorizer-server/src/server`
- [ ] 4.3 Move REST API: `git mv crates/vectorizer/src/api crates/vectorizer-server/src/api`
- [ ] 4.4 Move auth: `git mv crates/vectorizer/src/auth crates/vectorizer-server/src/auth`
- [ ] 4.5 Move replication, cluster, hub: `git mv crates/vectorizer/src/{replication,cluster,hub} crates/vectorizer-server/src/`
- [ ] 4.6 Move gRPC server-side handlers (server impls, not generated proto code): `git mv crates/vectorizer/src/grpc crates/vectorizer-server/src/grpc`
- [ ] 4.7 Move `src/bin/vectorizer.rs` into `crates/vectorizer-server/src/bin/server.rs` and rewire as the server binary entry point
- [ ] 4.8 `crates/vectorizer-server/Cargo.toml` depends on `vectorizer-core` + `vectorizer-protocol`
- [ ] 4.9 Umbrella `crates/vectorizer/` is now empty or just a re-export shim — keep as a thin facade that re-exports from `vectorizer-server` so `use vectorizer::server::X` keeps working
- [ ] 4.10 `cargo build --workspace --bin vectorizer` runs the server binary unchanged
- [ ] 4.11 `cargo build --workspace` + `cargo test --workspace --lib` + `cargo clippy --workspace -- -D warnings` clean

## 5. Extract `vectorizer-cli`

- [ ] 5.1 Create `crates/vectorizer-cli/{Cargo.toml, src/lib.rs, src/bin/cli.rs}`
- [ ] 5.2 `git mv crates/vectorizer/src/cli crates/vectorizer-cli/src/cli`
- [ ] 5.3 Move CLI binaries: `src/bin/vectorizer-cli.rs` → `crates/vectorizer-cli/src/bin/cli.rs`; `src/bin/create_mcp_key.rs` → `crates/vectorizer-cli/src/bin/create_mcp_key.rs`
- [ ] 5.4 `crates/vectorizer-cli/Cargo.toml` depends on `vectorizer-core` + `vectorizer-protocol` (NOT `vectorizer-server` — CLI is offline)
- [ ] 5.5 `cargo build --workspace --bin vectorizer-cli` + `cargo build --workspace --bin create_mcp_key` clean
- [ ] 5.6 `cargo build --workspace` + `cargo test --workspace --lib` + `cargo clippy --workspace -- -D warnings` clean

## 6. Wire `sdks/rust` as a workspace member

- [ ] 6.1 Add `"sdks/rust"` to root `[workspace] members`
- [ ] 6.2 Update `sdks/rust/Cargo.toml`: add `[package].workspace = true` only for fields it inherits from workspace; keep its own `name`, `version` (publishable to crates.io)
- [ ] 6.3 Replace SDK's wire-type stubs (`serde_json::Value`-shaped requests/responses) with `vectorizer-protocol::types::*` imports
- [ ] 6.4 Update SDK examples and tests to import from the new types
- [ ] 6.5 `cargo build --workspace -p hivellm-vectorizer-sdk` (or equivalent) clean
- [ ] 6.6 `cargo test --workspace -p hivellm-vectorizer-sdk` passes — including the existing `tests/mock_transport_regression.rs` (proves the per-surface clients still route through the mocked Transport after the wire-type swap)
- [ ] 6.7 `cargo clippy --workspace -- -D warnings` clean

## 7. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 7.1 Update `docs/specs/RUST.md` with the workspace layout + crate-dependency diagram
- [ ] 7.2 Update `AGENTS.md` § "Dependency Architecture (DAG)" to reflect the new crate boundaries
- [ ] 7.3 Add a CHANGELOG entry under `### Changed` documenting the new crate names + the umbrella re-export shim that preserves `use vectorizer::*` for one release
- [ ] 7.4 Update `Cargo.toml` description / metadata for each new crate so `cargo doc --workspace` produces a navigable structure
- [ ] 7.5 Run `cargo doc --workspace --no-deps -D warnings` clean
- [ ] 7.6 Update `.github/workflows/` if any reference per-crate paths (the existing release pipeline targets the umbrella, should keep working unchanged)
- [ ] 7.7 Run the full verification once: `cargo check --workspace --all-features`, `cargo clippy --workspace --all-features -- -D warnings`, `cargo test --workspace --all-features`
- [ ] 7.8 Capture rulebook knowledge entry: "Cargo workspace extraction playbook" — sub-phase pattern, when to use re-export shims, how to detect cross-crate `pub use` cycles

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
