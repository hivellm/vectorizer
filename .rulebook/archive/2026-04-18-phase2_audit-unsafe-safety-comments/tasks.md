## 1. Audit

- [x] 1.1 Enumerate every unsafe block/function in `src/` — 23 sites across 11 files. One already had `// SAFETY:` (`raft_watcher.rs:280`); the other 22 were undocumented.
- [x] 1.2 Classification: all 22 sites were category (a) "add comment" — none needed to be replaced with a safe wrapper or promoted to `unsafe fn`. The existing designs are sound; only the documentation was missing.

## 2. Implementation

- [x] 2.1 `src/embedding/cache.rs:334` — `Mmap::map` on cache data file (process-owned, append-only writer discipline).
- [x] 2.2 `src/embedding/candle_models.rs:135, 296` — `VarBuilder::from_mmaped_safetensors` (read-only model weights owned by the cache dir).
- [x] 2.3 `src/bin/vectorizer-cli.rs:240` — `pre_exec` closure invoking `setsid` (async-signal-safe call inside the post-fork / pre-exec window).
- [x] 2.4 `src/storage/mmap.rs:42, 91, 99, 133, 153` — all 5 sites: two `map_mut` calls for exclusive-access RW file mapping, two `slice::from_raw_parts` reinterpreting `&[f32]` as bytes (f32 has no padding), one `copy_nonoverlapping` with disjoint source/destination.
- [x] 2.5 All 5 AVX2 sites in `src/models/vector_utils_simd.rs` (lines 32, 52, 87, 111, 147) — two call-sites gated by `is_avx2_available()` + three `unsafe fn` declarations with `#[target_feature(enable = "avx2")]` now carry `# Safety` doc-comments.
- [x] 2.6 `src/db/hive_gpu_collection.rs:59, 472` — Box cast adding `Send` bound to the hive-gpu trait object.
- [x] 2.7 `src/normalization/cache/warm_store.rs:144` — cache-file mmap with atomic-rename writer discipline.
- [x] 2.8 `src/parallel/mod.rs:61` — batch `env::set_var` calls, documented as "main-thread only, pre-worker-pool init".
- [x] 2.9 `src/storage/advanced.rs:209` — per-collection storage-file mmap with exclusive-writer-lock invariant.
- [x] 2.10 `src/cluster/raft_watcher.rs:293` — test-only `env::remove_var` (single-threaded test harness).
- [x] 2.11 `src/config/enhanced_config.rs:723` — test-only `env::set_var` with the `tokio::test` single-thread rationale.

## 3. Enforcement

- [x] 3.1 Add `undocumented_unsafe_blocks = "deny"` to `[lints.clippy]` in `Cargo.toml`. Future unsafe additions must carry the comment or the build fails.
- [x] 3.2 `cargo clippy --all-features --all-targets -- -D warnings` — green in 1m34s (zero hits).

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 4.1 CHANGELOG `[Unreleased] > Documentation` entry added naming every audited site and the new clippy lint.
- [x] 4.2 No new runtime behavior; the compile-time lint IS the regression guard.
- [x] 4.3 `cargo clippy --all-targets -- -D warnings` green; confirms every existing unsafe is documented.

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation (CHANGELOG `Documentation` entry)
- [x] Write tests covering the new behavior (compile-time clippy lint acts as the ongoing enforcement)
- [x] Run tests and confirm they pass (clippy `-D warnings` zero hits)
