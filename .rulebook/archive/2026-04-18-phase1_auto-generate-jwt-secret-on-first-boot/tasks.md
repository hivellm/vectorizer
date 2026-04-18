## 1. Implementation

- [x] 1.1 Create `src/auth/jwt_secret.rs` with `load_or_generate(path: &Path) -> Result<String>` (OsRng + hex encode + atomic write + 0o600 on POSIX)
- [x] 1.2 Unit test: generating twice returns the same value; deleting the file and re-running generates a new value; short/corrupt files fail cleanly
- [x] 1.3 Add `--auto-generate-jwt-secret` CLI flag and `VECTORIZER_AUTO_GEN_JWT_SECRET` env var (both default `false`)
- [x] 1.4 In `src/server/mod.rs` boot path, when the flag is set AND `auth.jwt_secret` is empty, call `load_or_generate(data_dir.join("jwt_secret.key"))`
- [x] 1.5 Log only the path, never the secret value

## 2. Hygiene

- [x] 2.1 Add `data/jwt_secret.key` (and the full `data/` pattern if not already) to `.gitignore` and `.dockerignore`
- [x] 2.2 Document Windows perms behavior (Windows ACL vs POSIX 0o600) in `docs/security.md#jwt-secret`

## 3. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 3.1 Update `docs/security.md#jwt-secret` and `docs/deployment/docker.md` explaining the opt-in flag
- [x] 3.2 Integration test: start server with no secret + flag set, observe boot succeeds and `jwt_secret.key` is created with expected perms; restart uses the same key
- [x] 3.3 Run `cargo test --all-features -- jwt_secret` and confirm all pass

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation
- [x] Write tests covering the new behavior
- [x] Run tests and confirm they pass
