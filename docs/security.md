# Security

## JWT secret

The JWT signing secret is the single most sensitive piece of auth state —
anyone who has it can mint tokens that impersonate any user. Production
deployments MUST set it explicitly.

### Configuring the secret

In order of precedence:

1. **`VECTORIZER_JWT_SECRET` env var.** Highest priority; overrides config.yml.
2. **`auth.jwt_secret` in `config.yml`.**
3. **Auto-generated first-boot key** — opt-in, see below.

The loaded value must be at least 32 characters, not empty, and not the
historical legacy default. `AuthManager::new` refuses to boot otherwise.

### Auto-generated first-boot key (opt-in)

For local development and Docker first-runs where manually setting a
secret is friction, the server can generate and persist a 512-bit random
key on first boot. Enable with either:

- `--auto-generate-jwt-secret` CLI flag, or
- `VECTORIZER_AUTO_GEN_JWT_SECRET=1` environment variable.

Behavior when enabled:

- On first boot with `auth.jwt_secret` empty, the server writes
  `<data_dir>/jwt_secret.key` with a 128-char hex-encoded key (64 random
  bytes from `OsRng`).
- On every subsequent boot the same file is loaded verbatim.
- Only the file **path** is logged — the secret value never reaches stdout.
- An explicit `VECTORIZER_JWT_SECRET` or config.yml value always wins
  over the auto-generated one; the file is only consulted when both are
  empty.

### File permissions

- **POSIX:** the key file is opened with mode `0o600` (owner read/write
  only). The write is atomic: a `.tmp` sibling is created, permissions
  set, contents synced to disk, then renamed onto the final path.
- **Windows:** `std::fs::OpenOptions` does not expose an NTFS ACL
  equivalent of `0o600`. The file inherits the ACL of its parent
  directory (`data/` under the server's working directory). Operators
  deploying on Windows should ensure the data directory itself is
  ACL-restricted to the service account that runs the server. For
  containerised deployments, mount `data/` from a dedicated volume with
  appropriate host-side permissions.

### Gitignore / dockerignore

`data/` (which includes `jwt_secret.key`) is in both `.gitignore` and
`.dockerignore`. Never commit the file, never bake it into an image;
mount a persistent volume if the key needs to survive container restarts.

### Rotating the secret

1. Stop the server.
2. Delete `data/jwt_secret.key` (or update `auth.jwt_secret` in
   config.yml / `VECTORIZER_JWT_SECRET`).
3. Start the server — a fresh key is generated.

Rotation invalidates all outstanding tokens; clients must re-authenticate.

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
