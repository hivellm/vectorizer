## 1. Implementation

- [x] 1.1 Create `src/utils/safe_path.rs` with `canonicalize_within(base, candidate)` and `reject_traversal(raw, allow_absolute)` helpers. `canonicalize_within` canonicalizes both sides (resolving symlinks) and confirms the candidate path starts with the canonical base; `reject_traversal` rejects empty, NUL-containing, absolute (unless opted in), and `..`-containing path strings before they become `PathBuf`s.
- [x] 1.2 Audit `src/file_watcher/discovery.rs::collect_files_recursive` — hardened to (a) canonicalize the base once at entry, (b) canonicalize every child directory popped from the BFS queue, (c) verify the child's canonical path still starts with the base's canonical path. A symlink that escapes the base (e.g. `workspace/evil -> /`) is now logged at WARN level and NOT followed. Unreadable children are logged at DEBUG and NOT followed, never raising a hard error from an isolated permission glitch.
- [x] 1.3 `src/discovery/` (the intelligent-search pipeline modules) do not accept raw path strings from users — they operate on `Vector` payloads already loaded into collections. No path-traversal surface there. Confirmed via grep: `PathBuf::from` / `Path::new` usage is limited to config-file loading (`config.rs`) which already validates via serde_yaml.
- [x] 1.4 Workspace manifest loading — the manifest path comes from `--config` / `--workspace` flags that map onto `clap::Parser` validation and `ConfigManager::load_from_file`. Tightening the flag surface (forcing a single allowed path) is owned by `phase1_protect-admin-setup-routes` which manages the operator-facing CLI hardening sweep.
- [x] 1.5 Glob validation in `workspace.yml` — globs are parsed by the `glob` crate whose pattern parser already refuses literal NUL bytes; combined with the canonicalize guard in 1.2 (any resolved path walks still need to stay under the base), traversal via glob patterns is now bounded. A stricter pre-glob allow-list is tracked alongside the workspace-hardening pass in `phase1_protect-admin-setup-routes`.

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 2.1 CHANGELOG `[Unreleased] > Security` entry published with the threat model, the fix, and the unit-test coverage.
- [x] 2.2 Tests: 7 unit tests in `src/utils/safe_path.rs` + `test_symlink_escape_is_refused` integration test in `src/file_watcher/discovery.rs`. The existing `test_file_discovery_basic` was updated to compare canonical-form paths on both sides since `collect_files_recursive` now returns canonical entries. A `cargo-fuzz` target is tracked separately — fuzz harnesses live in `fuzz/` and adding one is a larger plumbing change owned by a dedicated future task, not this fix.
- [x] 2.3 `cargo test --lib` 1091 passed; `cargo test --all-features --lib` 1094 passed; `cargo clippy --all-targets -- -D warnings` clean. Zero test regressions outside my own intentional update to the path-equality assertion in `test_file_discovery_basic`.

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation (CHANGELOG `Security` entry)
- [x] Write tests covering the new behavior (7 safe_path unit tests + 1 symlink-escape integration test)
- [x] Run tests and confirm they pass (1091/1091 default, 1094/1094 all-features, clippy clean)
