# Proposal: phase1_secret-newtype-and-log-gate

## Why

Sibling task `phase1_remove-password-stdout-logging` (already completed) closed the single worst leakage point — the first-run root-credential banner. But two structural pieces remain from that task's original scope:

1. Introduce a `Secret<T>` newtype with redacting `Debug`/`Display` impls, so any future developer who puts a `jwt_secret`, `api_key`, or `password_hash` into a `{:?}` struct log gets `"<redacted>"` instead of the raw value. Today the codebase relies on every author remembering to not log those fields — fragile.

2. Add a CI grep gate to `.github/workflows/rust-lint.yml` that fails PRs if a newly introduced `println!`/`info!`/`debug!`/`warn!`/`error!`/`trace!` in `src/auth/` or `src/server/auth_handlers.rs` references the words `password`, `secret`, or `api_key` in its format string.

These are preventative: no known leakage today, but the code review alone is too thin for a security-critical surface.

## What Changes

1. Create `src/auth/secret.rs` defining a wrapper:

   ```rust
   pub struct Secret<T: zeroize::Zeroize>(T);
   impl<T: zeroize::Zeroize> Debug for Secret<T> { /* write "<redacted>" */ }
   impl<T: zeroize::Zeroize> Display for Secret<T> { /* write "<redacted>" */ }
   impl<T: Serialize + zeroize::Zeroize> Serialize for Secret<T> { /* round-trip */ }
   impl<T: Deserialize + zeroize::Zeroize> Deserialize for Secret<T> { /* round-trip */ }
   ```

2. Migrate the following fields to `Secret<String>`:
   - `AuthConfig::jwt_secret`
   - `JwtManager::secret`
   - `ApiKey::key_hash`
   - `UserRecord::password_hash`
   - `PersistedUser::password_hash`

   Everywhere the plaintext is needed, call `.expose_secret()` explicitly, which grep can audit.

3. Add to `.github/workflows/rust-lint.yml`:

   ```yaml
   - name: Log-leakage gate
     run: |
       if grep -rnE '(println|info|debug|warn|error|trace)!\([^)]*(password|secret|api_key)' src/auth/ src/server/auth_handlers.rs; then
         echo "Forbidden credential reference in log macro"; exit 1;
       fi
   ```

4. Add a unit test that formats a `Secret<String>` with `{:?}` and asserts the output is `<redacted>`.

## Impact

- Affected specs: security spec, auth module spec
- Affected code: new `src/auth/secret.rs`; `src/auth/mod.rs`, `src/auth/jwt.rs`, `src/auth/persistence.rs`, `src/server/auth_handlers.rs`; `.github/workflows/rust-lint.yml`
- Breaking change: NO externally; serialized `jwt_secret` wire format unchanged (Serde passes through).
- User benefit: mechanical protection against accidental credential logging; CI catches the class of mistake before it ships.
