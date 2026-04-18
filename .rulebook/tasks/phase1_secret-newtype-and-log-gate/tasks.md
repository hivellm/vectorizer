## 1. Secret<T> newtype

- [ ] 1.1 Add `zeroize = "1"` to `Cargo.toml` if not already present
- [ ] 1.2 Create `src/auth/secret.rs` with `Secret<T>` + redacting Debug/Display impls and a typed `expose_secret(&self) -> &T` accessor
- [ ] 1.3 Unit test proving `{:?}` on a `Secret<String>` yields `<redacted>` (not the inner value)

## 2. Migration

- [ ] 2.1 Migrate `AuthConfig::jwt_secret` and call-sites (preserve Serde wire shape via custom Serialize/Deserialize if needed)
- [ ] 2.2 Migrate `JwtManager::secret`
- [ ] 2.3 Migrate `ApiKey::key_hash`, `UserRecord::password_hash`, `PersistedUser::password_hash`
- [ ] 2.4 Replace raw field accesses with `.expose_secret()` at the three or four sites where plaintext is genuinely needed

## 3. CI gate

- [ ] 3.1 Add the grep-based log-leakage check to `.github/workflows/rust-lint.yml`
- [ ] 3.2 Run locally on the current tree and confirm zero hits

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 4.1 Document the Secret<T> pattern + the CI gate in `docs/security.md`
- [ ] 4.2 Write unit tests for each migrated field confirming round-trip serde works and Debug is redacted
- [ ] 4.3 Run `cargo test --all-features` and confirm all tests pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
