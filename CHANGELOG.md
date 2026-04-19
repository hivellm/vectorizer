# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Changed
- **Default transport is now VectorizerRPC (phase6_make-rpc-default-transport).** The server bundle ships with `RpcConfig::default()` returning `enabled: true` (was `false`), `config.example.yml` leads with the `rpc:` block flagged as the recommended primary, the Dockerfile `EXPOSE`s `15503` ahead of `15002`, and `docker-compose.yml` publishes both. The Helm chart gains `service.rpcPort: 15503` (wired into the regular service, headless service, statefulset, and deployment) and the bundled `k8s/service.yaml` + `statefulset.yaml` + `statefulset-ha.yaml` add a named `rpc` port. REST stays available on `15002` for the dashboard, ops tooling, browser clients, and any caller that doesn't speak raw TCP — the change is to defaults, not to the transport surface itself. Operators on restricted networks opt out by setting `rpc.enabled: false` in their config. New migration guide at [`docs/migration/rpc-default.md`](docs/migration/rpc-default.md). The first-party SDK constructors (Rust, Python, TypeScript, Go, C#) flip to RPC as their default in the matching `phase6_sdk-{lang}-rpc` task slots; until those ship, REST remains the live SDK default and existing client code keeps working.

### Added
- **`#![deny(missing_docs)]` on the crate root + `cargo doc -D warnings` CI gate (phase4_enforce-public-api-docs).** Cleared **2,219 missing-docs warnings** from `cargo doc --no-deps --all-features --lib` to **0**. Strategy: auto-generated proto modules (`grpc/{vectorizer,cluster,qdrant_proto}` — 876 warnings) gated with `#[allow(missing_docs)]` at the include site; ~80 internal data-layout files annotated with file-level `#![allow(missing_docs)]` + a `// Internal data-layout file: …` rationale via the new `scripts/codemods/add_internal_docs_allow.py` codemod (idempotent, re-runnable as new modules land); the genuinely-public surface (`error/mod.rs` error variants, `models/mod.rs` quantization configs, `models/sparse_vector.rs` error types, `lib.rs::VERSION`, the embedding providers' top-level types, the new `Collection` mod-split files, every existing `//!` module header) carries per-item `///` docs. New `.github/workflows/rust-docs.yml` runs `cargo doc --no-deps --all-features --lib` with `RUSTDOCFLAGS="-D warnings"` plus `cargo test --doc --all-features` on every PR; doc-tests are 11/11 passing.

- **`unwrap_used = "deny"` / `expect_used = "deny"` clippy lints (phase4_enforce-no-unwrap-policy).** `Cargo.toml` `[lints.clippy]` now denies both lints workspace-wide; every production `.unwrap()` / `.expect(...)` either fails CI or sits behind a function- or block-scoped `#[allow(...)]` with a `// SAFE:` rationale. The sweep cleared the crate from ~1,430 legacy occurrences to a clippy-clean state: 1,189 test-only sites are now covered by `#[cfg(test)]` / `*_tests.rs` allows applied via `scripts/add_test_unwrap_allow.py` and `scripts/add_file_unwrap_allow.py`; the remaining production sites either return `Result` properly (`?` + `map_err` / `ok_or_else`), use `unwrap_or(Ordering::Equal)` for `f32`/`f64` `partial_cmp`, fall back to a finite default for `Number::from_f64(NaN)` and pre-epoch `SystemTime`, or are documented static invariants (prometheus collector names, regex literals, `HeaderValue::parse` on `&'static str`, `NonZeroU32`/`NonZeroUsize` constants extracted via `match Some(n) => n, None => unreachable!()`). Adds `VectorizerError::Unimplemented` (used by the cluster RPC fallback path) and a new `tests/integration/handler_robustness.rs` with 8 regression tests pinning the contract for malformed input, NaN scores, out-of-range timestamps, and empty result sets. `.rulebook/specs/RUST.md` § "Enforcement" rewritten to reflect the now-active state.

- **Distributed hybrid search via cluster `RemoteHybridSearch` RPC.** `proto/cluster.proto` now exposes `RemoteHybridSearch(RemoteHybridSearchRequest) returns (RemoteHybridSearchResponse)` plus the supporting `SparseVector`, `HybridSearchConfig`, `HybridScoringAlgorithm`, and `HybridSearchResult` messages. The server-side handler (`src/cluster/grpc_service.rs::remote_hybrid_search`) routes to `DistributedShardedCollection::hybrid_search_local_only` for distributed collections — a new helper that scans only this node's `local_shards`, never recursing back out via gRPC — and otherwise delegates to `VectorStore::hybrid_search`. `DistributedShardedCollection::hybrid_search` (line 618) now calls the new RPC for remote shards instead of silently dropping the sparse signal. Mixed-version clusters keep working: when a remote node returns `tonic::Code::Unimplemented`, `ClusterClient::hybrid_search` surfaces it as the new `VectorizerError::Unimplemented` variant and the caller falls back to dense-only `RemoteSearchVectors` for that node only. Two new integration tests in `tests/integration/cluster_hybrid_search.rs` cover the success path (in-process tonic server returns fused results) and the compatibility path (server without `ClusterServiceServer` triggers the dense fallback).

### Security
- **First-run root credentials no longer print to stdout.** Previously `auth_handlers.rs` emitted the auto-generated admin username and cleartext password via `println!` on first boot, which got captured by every log pipeline the process ran under (Docker, Kubernetes, systemd journal, CI). Credentials now go to `{data_dir}/.root_credentials` (0o600 on POSIX); only the file path appears in the log stream. The operator must read and delete the file on first login. Added to `.gitignore` and `.dockerignore` to prevent accidental exposure.

### Breaking
- **gRPC `SearchResult.score` narrowed from `double` to `float`.** `proto/vectorizer.proto` previously declared the score as `double` (f64) while `crate::models::SearchResult::score` is `f32`. Every layer boundary performed an implicit `as f64` cast that silently reordered near-tie results. The proto now uses `float` for `SearchResult.score`, `HybridSearchResult.hybrid_score`, `.dense_score`, and `.sparse_score` — same precision the HNSW index natively scores at. Clients built against the pre-v3.0.0 proto must regenerate. The cluster proto was already `float` so the gRPC surface is now internally consistent. `src/grpc/conversions.rs` gains `impl From<vectorizer::SearchResult> for SearchResult` and `impl From<vectorizer::HybridSearchResult> for SearchResult` so the canonical conversion is greppable and tested (four new unit tests cover round-trip, ordering, and empty-field handling). The ranker's local `SearchResult` struct in `src/search/advanced_search.rs` (different shape: `document_id`/`title`/`snippet`/`score_breakdown`) has been renamed to `ScoredDocument` to eliminate the name collision. `build.rs` now explicitly emits `cargo:rerun-if-changed` for the proto files — observed that without it, a proto edit silently failed to regenerate the bindings on the next build.
- **JWT secret must be explicitly configured.** `AuthConfig::default()` no longer ships a real `jwt_secret` — the field is now an empty string. `AuthManager::new` calls `AuthConfig::validate` at startup and refuses to boot if `auth.enabled == true` and the configured secret is empty, shorter than 32 chars, or equal to the historical insecure default (`"vectorizer-default-secret-key-change-in-production"`). Operators must generate a real secret (e.g. `openssl rand -hex 64`) and inject it via config file or the `VECTORIZER_JWT_SECRET` env var before upgrading. This closes a known auth-bypass: any attacker who knew the public default could forge admin JWTs against unconfigured deployments.

### Security
- **Path-traversal guard for file discovery.** `src/file_watcher/discovery.rs::collect_files_recursive` now canonicalizes the base directory before recursion and verifies every discovered directory still resolves under that canonical root. Symlinks that escape the base (e.g. `workspace/evil -> /`) are logged at WARN level and then not followed instead of being walked. A new `src/utils/safe_path.rs` module exposes `canonicalize_within(base, candidate)` and `reject_traversal(raw, allow_absolute)` helpers with 7 unit tests covering `../`, absolute paths, NUL bytes, empty strings, and non-existent paths. A new `test_symlink_escape_is_refused` (POSIX-only; Windows symlink creation requires elevation) proves the recursive walker refuses symlinks that escape the base.

### Architecture
- Remove the `pub use crate::db::graph::{Edge, Node, RelationshipType};` re-export from `src/models/mod.rs`. This line was the only reverse dependency from the Foundation layer (`models`) into the Core layer (`db`) — it violated the layering rule documented in `CLAUDE.md`. Grep confirmed zero consumers imported the types via `crate::models::`, so deleting the re-export is behaviorally a no-op; code that wants these types imports them from `crate::db::graph` directly. A new CI grep gate in `rust-lint.yml` fails the build if any new `src/models/` file imports from `crate::db::`, preventing the regression.

### CI / Testing
- Audit every `#[ignore]`d test in the repo (~40 tests across 12 files) and categorize each one: A) environment-dependent (keeps ignored with a clear reason), B) slow (candidate for nightly runner), C) known bug (tracked by a rulebook follow-up task), D) CI-flaky (marked for investigation). Bare `#[ignore]` entries without a reason have been rewritten to `#[ignore = "..."]` with the category and, where applicable, the tracking task ID. New documentation in `docs/development/testing.md` lists every muted test, its category, and its tracking task. Four follow-up rulebook tasks created to fix the Category-C bugs: `phase4_triage-wal-recovery-bugs` (9 tests in WAL), `phase4_triage-mmap-storage-bugs` (2 tests in mmap storage), `phase4_fix-replication-snapshot-sync` (12 tests in master-replica snapshot path), `phase4_triage-sparse-vector-test` (1 test with parallel-test collision).

### CI / Testing
- Re-enable the Go SDK test workflow (`.github/workflows/sdk-go-test.yml`), previously renamed to `.disabled`. `sdks/go/` has working source + a unit-test + integration-test suite (`go vet ./...` clean, `go test -v -short -count=1 ./...` passing locally). The workflow matrix runs on Ubuntu + macOS against Go 1.21 and 1.22. Integration tests skip gracefully when no server is reachable.

### Security / Dependencies
- Pin `openraft` and `openraft-memstore` to the exact alpha version `=0.10.0-alpha.17`. Upstream has not shipped a 0.10 stable release as of 2026-04-18 (latest on crates.io is still the alpha). The `=` prefix prevents `cargo update` from silently drifting the consensus layer to a newer alpha whose behavior we haven't vetted against `tests/integration/cluster_ha.rs`. **HA/cluster mode runs on a pre-release consensus library until upstream stabilizes** — operators using `--features cluster` equivalents in production should subscribe to openraft releases and coordinate upgrades. When upstream ships stable, bump both pins together and rerun the Raft integration tests.

### Chore
- Bump `whoami` from `1.5` (resolved 1.6.1) to `2` (resolved 2.1.1). Dependabot PR #241 had flagged CI errors from `realname()` now returning `Result<String, whoami::Error>` — the single call site in `vectorizer-cli.rs` uses `username()` which kept its `String` signature, so the upgrade is zero-code. This unblocks the merge of PR #241.

### Changed
- `EmbeddingManager::save_vocabulary_json` now dispatches through a new `EmbeddingProvider::save_vocabulary_json(&self, &Path) -> Result<()>` trait method instead of a 4-branch `as_any().downcast_ref::<Type>()` if-chain. Providers that support vocabulary persistence (BM25, TF-IDF, CharNGram, BagOfWords) override the trait default; providers that don't (SVD, BERT, MiniLM, ONNX, FastEmbed) inherit the default which returns a clear error. Adding a new vocabulary-bearing provider is now a single-file change — implement the override on the new type.

### Removed
- Three `#[allow(dead_code)]` stub methods on `OnnxEmbedder` that called `unreachable!()` inside the body: `get_or_download_model`, `infer_batch`, `apply_pooling`. They had no callers (all real code paths go through the deterministic-hash `embed` / `embed_batch` / `embed_parallel` methods) and were left over from an earlier incomplete `ort` integration. Deleting them removes a class of "if any caller appears, the process aborts" hazards; the ONNX file header already documents that inference is a future feature.

### Performance
- Complete the `parking_lot` migration across the remaining 15 files outside the original hot-path list: `src/api/advanced_api.rs`, `src/cache/advanced_cache.rs`, `src/config/enhanced_config.rs`, `src/db/{collection,quantized_collection}.rs`, `src/ml/advanced_ml.rs`, `src/persistence/{dynamic,wal}.rs`, `src/processing/advanced_pipeline.rs`, `src/quantization/{hnsw_integration,storage}.rs`, `src/search/advanced_search.rs`, `src/security/enhanced_security.rs`, `src/storage/{advanced,reader}.rs`. All `.read()/.write()/.lock().unwrap()` and `.map_err(...)?` Result-handling suffixes were stripped because `parking_lot` guards don't poison. A new CI grep gate in `.github/workflows/rust-lint.yml` fails the build if any new `use std::sync::{...Mutex...|...RwLock...}` import appears in `src/`, locking in the migration.

### Performance (hot path)
- Migrate hot-path `std::sync::Mutex` to `parking_lot::Mutex` in `src/batch/{mod,processor}.rs`, `src/db/{vector_store,hive_gpu_collection}.rs`, and `src/cluster/raft_node.rs`. `parking_lot` locks are smaller (1 byte vs 8+ for std) and don't carry the `PoisonError` machinery, so the hot `.lock()` path is a straight guard return. Per `AGENTS.md` convention.

### Documentation
- Every `unsafe {}` block and `unsafe fn` in `src/` now carries a `// SAFETY:` (or doc-comment `# Safety` for functions) explaining why the invariants hold. Covered sites: daemon `pre_exec`, 4 mmap create/remap points, 5 AVX2 SIMD helpers, safetensors VarBuilder, hive-gpu Box cast, env-var init in `parallel::init_parallel_env`, and 5 slice-reinterpretation / `copy_nonoverlapping` sites in `storage::mmap`. Enforced going forward by `clippy::undocumented_unsafe_blocks = "deny"` in `Cargo.toml [lints.clippy]`.

### Fixed
- Collection usage metrics were being recorded under a fresh `Uuid::new_v4()` on every call in three REST handler sites (`create_collection`, batch insert path, single insert path), making per-collection Hub aggregation meaningless. The three sites now derive a stable UUIDv5 from the collection name via a new `collection_metrics_uuid` helper, so repeated calls aggregate into the same Hub row.

### Changed
- Remove the phantom `"cluster"` Cargo feature flag. `src/db/vector_store.rs` previously wrapped 15 match arms in `#[cfg(feature = "cluster")]`, but the feature was never declared in `Cargo.toml`, so the gated code was permanently dead and drifted from the always-compiled `DistributedShardedCollection` type it referenced. The gates are removed; `DistributedSharded` is now a first-class `CollectionType` variant on every build. Read-path methods return document-count-based approximations; mutating methods (`delete_vector`, `update_vector`, `get_vector`) return a clear `VectorizerError::Storage` steering callers to the async cluster router instead of panicking.

### Chore
- Delete committed backup files `src/db/vector_store.rs.bak` and `tests/integration/sharding_validation.rs.bak` left from earlier refactors.
- Add `*.bak`, `*.orig`, `*.rej` patterns to `.gitignore` and `.dockerignore` so future scratch files stay out of commits and Docker contexts.


---

## Historical releases

Per-minor release notes moved to `docs/patches/` during the v3.0.0 CHANGELOG cleanup to keep this file fast to read. Each file covers a single minor line (`vX.Y.0` through `vX.Y.9`).

| Minor line | Patches | Versions |
|---|---|---|
| [v2.4.x](docs/patches/v2.4.0-2.4.9.md) | 2 | 2.4.1, 2.4.2 |
| [v2.3.x](docs/patches/v2.3.0-2.3.9.md) | 1 | 2.3.0 |
| [v2.2.x](docs/patches/v2.2.0-2.2.9.md) | 2 | 2.2.0, 2.2.1 |
| [v2.1.x](docs/patches/v2.1.0-2.1.9.md) | 1 | 2.1.0 |
| [v2.0.x](docs/patches/v2.0.0-2.0.9.md) | 4 | 2.0.0, 2.0.1, 2.0.2, 2.0.3 |
| [v1.8.x](docs/patches/v1.8.0-1.8.9.md) | 7 | 1.8.0, 1.8.1, 1.8.2, 1.8.3, 1.8.4, 1.8.5, 1.8.6 |
| [v1.7.x](docs/patches/v1.7.0-1.7.9.md) | 1 | 1.7.0 |
| [v1.6.x](docs/patches/v1.6.0-1.6.9.md) | 1 | 1.6.1 |
| [v1.5.x](docs/patches/v1.5.0-1.5.9.md) | 1 | 1.5.0 |
| [v1.4.x](docs/patches/v1.4.0-1.4.9.md) | 1 | 1.4.0 |
| [v1.3.x](docs/patches/v1.3.0-1.3.9.md) | 1 | 1.3.0 |
| [v1.2.x](docs/patches/v1.2.0-1.2.9.md) | 3 | 1.2.0, 1.2.2, 1.2.3 |
| [v1.1.x](docs/patches/v1.1.0-1.1.9.md) | 3 | 1.1.0, 1.1.1, 1.1.2 |
| [v1.0.x](docs/patches/v1.0.0-1.0.9.md) | 3 | 1.0.0, 1.0.1, 1.0.2 |
| [v0.28.x](docs/patches/v0.28.0-0.28.9.md) | 1 | 0.28.0 |
| [v0.27.x](docs/patches/v0.27.0-0.27.9.md) | 1 | 0.27.0 |
| [v0.26.x](docs/patches/v0.26.0-0.26.9.md) | 1 | 0.26.0 |
| [v0.25.x](docs/patches/v0.25.0-0.25.9.md) | 1 | 0.25.0 |
| [v0.24.x](docs/patches/v0.24.0-0.24.9.md) | 1 | 0.24.0 |
| [v0.22.x](docs/patches/v0.22.0-0.22.9.md) | 1 | 0.22.0 |
| [v0.21.x](docs/patches/v0.21.0-0.21.9.md) | 1 | 0.21.0 |
| [v0.20.x](docs/patches/v0.20.0-0.20.9.md) | 1 | 0.20.0 |
| [v0.19.x](docs/patches/v0.19.0-0.19.9.md) | 1 | 0.19.0 |
| [v0.18.x](docs/patches/v0.18.0-0.18.9.md) | 1 | 0.18.0 |
| [v0.17.x](docs/patches/v0.17.0-0.17.9.md) | 2 | 0.17.0, 0.17.1 |
| [v0.16.x](docs/patches/v0.16.0-0.16.9.md) | 1 | 0.16.0 |
| [v0.15.x](docs/patches/v0.15.0-0.15.9.md) | 1 | 0.15.0 |
| [v0.14.x](docs/patches/v0.14.0-0.14.9.md) | 1 | 0.14.0 |
| [v0.13.x](docs/patches/v0.13.0-0.13.9.md) | 1 | 0.13.0 |
| [v0.12.x](docs/patches/v0.12.0-0.12.9.md) | 1 | 0.12.0 |
| [v0.11.x](docs/patches/v0.11.0-0.11.9.md) | 1 | 0.11.0 |
| [v0.10.x](docs/patches/v0.10.0-0.10.9.md) | 2 | 0.10.0, 0.10.1 |
| [v0.9.x](docs/patches/v0.9.0-0.9.9.md) | 4 | 0.9.0, 0.9.1, 0.9.2, 0.9.3 |
| [v0.8.x](docs/patches/v0.8.0-0.8.9.md) | 3 | 0.8.0, 0.8.1, 0.8.2 |
| [v0.7.x](docs/patches/v0.7.0-0.7.9.md) | 1 | 0.7.0 |
| [v0.6.x](docs/patches/v0.6.0-0.6.9.md) | 1 | 0.6.0 |
| [v0.5.x](docs/patches/v0.5.0-0.5.9.md) | 1 | 0.5.0 |
| [v0.4.x](docs/patches/v0.4.0-0.4.9.md) | 1 | 0.4.0 |
| [v0.3.x](docs/patches/v0.3.0-0.3.9.md) | 4 | 0.3.0, 0.3.1, 0.3.2, 0.3.4 |
| [v0.2.x](docs/patches/v0.2.0-0.2.9.md) | 2 | 0.2.0, 0.2.1 |
| [v0.1.x](docs/patches/v0.1.0-0.1.9.md) | 3 | 0.1.0, 0.1.1, 0.1.2 |
