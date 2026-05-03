# Proposal: phase19_dev-mode-auth-skip

Source: phase8 audit follow-up (section 7.5).

## Why

Local development against the dashboard or SDK requires manually:

1. Setting a JWT secret in `config.yml` (or via env var).
2. Logging in with the auto-generated root credentials (or generating a
   key via `POST /auth/keys`) to get a usable token.
3. Echoing the token in every test request.

That workflow makes sense for production. For localhost dev — single
operator, machine they own, plain-HTTP loopback bind — it's pure
friction. Today the only way to skip auth is `auth.enabled: false`,
which also disables auth in production builds (no per-bind toggle).

The `--host 127.0.0.1` startup path already restricts exposure to the
loopback. A safe dev-mode toggle that ONLY engages on loopback binds
would let developers skip the credential dance without a footgun on
0.0.0.0.

## What Changes

### 1. Config flag

Add `auth.dev_mode_skip_loopback: bool` (default `false`) to the auth
config. When `true`:

- The auth middleware short-circuits with an implicit
  `local-dev-admin` principal for every request.
- Every response carries a `X-Vectorizer-Dev-Mode: true` header so
  tooling can spot it.
- Boot logs at WARN with a multi-line banner ("AUTH IS DISABLED FOR
  LOOPBACK — DO NOT EXPOSE THIS BUILD").

### 2. Loopback enforcement

If `dev_mode_skip_loopback=true` AND host is anything other than
`127.0.0.1` / `::1` / `localhost`, boot fails with a clear error
explaining the constraint.

### 3. Production-bind hard-block

If host is `0.0.0.0` AND `dev_mode_skip_loopback=true`, boot returns
`anyhow::anyhow!(...)` from the existing security-check block (mirrors
the existing 0.0.0.0-without-auth check at routing.rs:41-63).

## Impact

- Affected specs: `.rulebook/tasks/phase19_dev-mode-auth-skip/specs/dev-mode-auth-skip/spec.md`
- Affected code:
  - `crates/vectorizer/src/auth/mod.rs` (config field)
  - `crates/vectorizer-server/src/server/auth_handlers/middleware.rs`
    (short-circuit path)
  - `crates/vectorizer-server/src/server/core/routing.rs` (boot
    enforcement)
- Breaking change: NO — defaults to off; opt-in via config
- User benefit: dev workflow drops from "set secret + login + echo
  token" to "set the flag, hit any endpoint", without any production
  exposure risk

## Acceptance

- `dev_mode_skip_loopback=true` + `--host 127.0.0.1` lets every request
  through with the implicit dev-admin principal
- `dev_mode_skip_loopback=true` + `--host 0.0.0.0` refuses to boot
- Default config (flag absent or `false`) keeps the existing auth
  enforcement unchanged
- Boot log shows the WARN banner when the flag is engaged
