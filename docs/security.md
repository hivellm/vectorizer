# Security

## Credential handling

### `Secret<T>` newtype

Credential material in `vectorizer` lives inside `crate::auth::Secret<T>`. The
wrapper gives three guarantees:

1. **`Debug` / `Display` redact.** Any `{:?}` or `{}` format of a `Secret<T>`
   (or of a struct that contains one) prints `<redacted>` instead of the raw
   value. This eliminates a common class of leak where a developer adds a
   credential field to a `#[derive(Debug)]` struct and later logs that struct
   with `"{:?}"`.
2. **Serde is transparent.** Serializing a `Secret<String>` writes the inner
   string verbatim; deserializing reads it back. Wire formats (YAML config,
   encrypted auth store) are unchanged.
3. **On drop, the backing memory is zeroed** via the `zeroize` crate — a
   short-lived defense against post-free memory disclosure.

The only way to read the inner value is `.expose_secret()`. Callers that
need plaintext (hashing, signing, HMAC, `bcrypt::verify`) must call this
accessor explicitly, which makes every audit path trivially greppable:

```bash
grep -rn expose_secret src/
```

### Migrated fields

The following credential-bearing fields use `Secret<String>`:

| Field | File |
|-------|------|
| `AuthConfig::jwt_secret` | `src/auth/mod.rs` |
| `AuthConfigFile::jwt_secret` | `src/cli/config.rs` |
| `JwtManager::secret` | `src/auth/jwt.rs` |
| `ApiKey::key_hash` | `src/auth/mod.rs` |
| `PersistedUser::password_hash` | `src/auth/persistence.rs` |
| `PersistedApiKey::key_hash` | `src/auth/persistence.rs` |
| `UserRecord::password_hash` | `src/server/auth_handlers.rs` |

New credential-bearing fields added to these modules **must** use
`Secret<String>`. Code review will reject raw `String` for anything named
`*_secret`, `*_hash`, `jwt_*`, `api_key`, or `password`.

### Example

```rust
use crate::auth::Secret;

// Construction
let secret = Secret::new(random_key);

// Debug is safe
println!("{:?}", secret);  // prints: Secret(<redacted>)

// Explicit plaintext access
let encoding_key = EncodingKey::from_secret(secret.expose_secret().as_bytes());
```

## CI gates

Two workflow steps in [`.github/workflows/rust-lint.yml`](../.github/workflows/rust-lint.yml)
enforce the secret-handling rules:

### 1. Tier-1 marker gate

Script: [`scripts/check-no-tier1-markers.sh`](../scripts/check-no-tier1-markers.sh)

Fails the build if any `TODO`/`FIXME`/`HACK`/`XXX` literal appears in `src/`
outside the `TASK(phaseN_<slug>)` allow-list or the `grep-ignore(tier1-markers)`
sentinel. Protects against drive-by "I'll finish it later" markers in
security-sensitive code.

### 2. Credential log-leakage gate

Script: [`scripts/check-no-credential-logs.sh`](../scripts/check-no-credential-logs.sh)

Scans `src/auth/` and `src/server/auth_handlers.rs` for log macros
(`info!`, `debug!`, `warn!`, `error!`, `trace!`, `println!`) that reference
`password`, `secret`, or `api_key` in the format string. Fails the build
unless the line carries a `// logging-allow(<reason>): <justification>`
sentinel.

The sentinel exists for unavoidable labels — e.g.
`error!("Failed to hash password: {}", bcrypt_error)` legitimately mentions
"password" in the label but substitutes an unrelated error into `{}`. The
sentinel documents that the line is safe and makes every such exception
reviewable.

#### Why both gates

`Secret<T>` protects every `{:?}` path automatically. The log-leakage gate
catches the other side: hand-rolled `info!("jwt_secret = {}", s)` where
`s` is a bare `String`. Together they cover both the struct-field path
(Debug/Display) and the explicit-variable path (format arg).

## Threat model notes

- The `Secret<T>` `PartialEq` is **not constant-time**. Callers comparing
  secrets against untrusted input must use `subtle::ConstantTimeEq` on the
  exposed value, not `==` on the wrapper.
- `Secret<T>` defends against accidental logging and post-drop memory
  disclosure. It does **not** defend against: compromised stack traces from
  panics before drop, debug symbols in release builds, or core dumps. For
  those threats, separate controls (`-C strip=symbols`, disabled core dumps)
  apply.
- Tracing spans (`tracing::Span`) can serialize field values via the
  `%` / `?` formatters — both route through `Display` / `Debug`, so `Secret<T>`
  is safe inside a span.
