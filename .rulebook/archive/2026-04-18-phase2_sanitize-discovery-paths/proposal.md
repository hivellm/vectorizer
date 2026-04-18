# Proposal: phase2_sanitize-discovery-paths

## Why

`src/file_watcher/discovery.rs` and the broader `src/discovery/` pipeline build `PathBuf` values from configuration and (in some code paths) from API-provided workspace patterns, then feed them to `tokio::fs::read_dir` and `serde_yaml::from_str`. The audit identified risk areas:

- Workspace manifests (`workspace.yml`) can contain relative patterns that, when combined with user-configured base dirs, resolve to `../../etc/passwd` or similar.
- The `PathBuf::from(user_input)` pattern appears without canonicalization or allow-listing.
- No check that a discovered file path remains within the configured root.

In multi-tenant cluster mode (HiveHub), the blast radius is higher: a tenant-crafted manifest could enumerate files outside its allotted tenant directory.

## What Changes

1. Introduce `src/utils/safe_path.rs` with:
   - `canonicalize_within(base: &Path, candidate: &Path) -> Result<PathBuf>` — canonicalizes `candidate`, verifies it remains a descendant of `base` (after resolving symlinks), returns error otherwise.
   - `reject_traversal(raw: &str) -> Result<&str>` — rejects strings containing `..`, null bytes, or absolute paths when the caller expects a relative path.
2. Wrap every user/config-derived path construction in `src/discovery/` and `src/file_watcher/` with these helpers.
3. Restrict `workspace.yml` load path to a single well-known location; reject arbitrary `--workspace <path>` flags unless `--allow-external-workspace` is set.
4. Add tests proving `../`, absolute paths, and symlink-escape attacks are rejected.

## Impact

- Affected specs: security spec, discovery module spec
- Affected code: `src/file_watcher/discovery.rs`, `src/discovery/*`, `src/utils/mod.rs` (new safe_path module)
- Breaking change: possibly — previously-accepted traversal paths now error. Document in CHANGELOG.
- User benefit: closes a file-enumeration vector; makes tenant isolation sound in cluster mode.
