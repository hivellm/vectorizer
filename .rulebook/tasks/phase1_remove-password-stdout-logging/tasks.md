## 1. Implementation

- [ ] 1.1 Replace `println!("Password: {}")` in `src/server/auth_handlers.rs:378-405` with a write to `data/.root_credentials` (mode 0600, created with `OpenOptions`)
- [ ] 1.2 Emit only a path pointer to stdout: `"Root credentials written to data/.root_credentials. Read once and delete."`
- [ ] 1.3 Introduce a `Secret<T>` newtype in `src/auth/secret.rs` with redacting `Debug`/`Display` impls; use it for `jwt_secret`, `api_key`, `password_hash`
- [ ] 1.4 Grep `src/auth/` and `src/server/` for `println!`, `info!`, `debug!`, `warn!`, `error!`, `trace!` containing `password`, `token`, `secret`, `hash`, `api_key` — replace with redacted forms
- [ ] 1.5 Add `.root_credentials` to `.gitignore` and `.dockerignore`
- [ ] 1.6 Add a CI grep gate in `.github/workflows/rust-lint.yml` that fails on `println!.*password` or `info!.*password` patterns

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 2.1 Update `docs/security.md` (or create it) documenting the new credential flow; add migration note to CHANGELOG
- [ ] 2.2 Write tests: `Secret<String>::Debug` does not leak; integration test that first boot creates `.root_credentials` with 0600 and no password appears in captured stdout/stderr
- [ ] 2.3 Run `cargo test --all-features` and confirm all tests pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
