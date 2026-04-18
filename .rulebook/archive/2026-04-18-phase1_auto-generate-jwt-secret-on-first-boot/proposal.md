# Proposal: phase1_auto-generate-jwt-secret-on-first-boot

## Why

`phase1_fix-jwt-default-secret` (already completed) makes the server refuse to boot unless `auth.jwt_secret` is explicitly configured. That closes the auth-bypass vulnerability, but it trades correctness for UX friction: first-run operators and Docker users get a hard failure instead of a running server.

This follow-up task adds a safe fallback: when `auth.jwt_secret` is empty, generate a cryptographically random secret on first boot, persist it with restrictive filesystem permissions, and boot. Subsequent restarts reuse the persisted value.

Items 1.4 and 1.5 from the parent task were intentionally out of scope because auto-generation has its own design surface (on-disk format, rotation policy, multi-node sharing, perms on Windows vs POSIX) that deserves a separate review.

## What Changes

1. Add `src/auth/jwt_secret.rs` with `load_or_generate(path: &Path) -> Result<String>`:
   - If the file at `path` exists and is readable, return its content (trimmed).
   - Otherwise, generate 64 random bytes via `rand::rngs::OsRng`, hex-encode to a 128-char string, write atomically to `path` with mode 0o600 on POSIX (Windows gets an ACL equivalent or documented absence), then return the new value.
2. Wire into boot in `src/server/mod.rs`:
   - If the loaded `auth_config.jwt_secret` is empty AND the `--auto-generate-jwt-secret` flag (or `VECTORIZER_AUTO_GEN_JWT_SECRET=1` env var) is set, call `load_or_generate(data_dir.join("jwt_secret.key"))` and inject the result.
   - Log `"Using auto-generated JWT secret persisted at {path}"` (path only, never the value).
   - If auto-gen is NOT requested, keep the existing fail-fast behavior.
3. Default `--auto-generate-jwt-secret = false` so production deployments opt in deliberately.
4. Add `data/jwt_secret.key` to `.gitignore` and `.dockerignore`.
5. Document in `docs/security.md#jwt-secret` and `docs/deployment/docker.md` — especially the Docker workflow where the data volume needs to be persistent.

## Impact

- Affected specs: auth spec, security spec
- Affected code: new `src/auth/jwt_secret.rs`; `src/server/mod.rs` boot path; `src/cli/mod.rs` flag parsing; `.gitignore`; `.dockerignore`
- Breaking change: NO (opt-in flag)
- User benefit: dev/Docker UX without reopening the auth-bypass hole fixed in phase1_fix-jwt-default-secret.
