## 1. Implementation

- [ ] 1.1 Create `src/utils/safe_path.rs` with `canonicalize_within` and `reject_traversal` helpers
- [ ] 1.2 Audit `src/file_watcher/discovery.rs` for every `PathBuf::from(...)` and `Path::new(...)` call; wrap in the helpers
- [ ] 1.3 Audit `src/discovery/` (entire module) for the same; wrap user-derived paths
- [ ] 1.4 Restrict workspace manifest loading to a single allowed path (or a whitelist); reject external `--workspace` flags unless an explicit opt-in flag is provided
- [ ] 1.5 Validate all globs in `workspace.yml` against the allow-list before passing to `walkdir`/`glob` crates

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 2.1 Write `docs/security.md#path-traversal` explaining the policy
- [ ] 2.2 Write tests: `../` rejected, absolute path rejected, symlink escape rejected, valid nested paths accepted; fuzzer target using `cargo-fuzz` recommended
- [ ] 2.3 Run `cargo test --all-features` and confirm all tests pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
