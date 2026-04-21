# Proposal: phase1_fix-jwt-default-secret

## Why

`AuthConfig::default()` in `src/auth/mod.rs:49` initializes `jwt_secret` to the string literal `"vectorizer-default-secret-key-change-in-production"`. If an operator starts the server without overriding this value (via config file, env var, or CLI flag), **any attacker who knows this codebase can forge arbitrary JWTs and gain admin access**. The string is visible in our public GitHub repo, so this is effectively a zero-day for unconfigured deployments.

`config.example.yml` and `.env.example` also reference the default, which trains operators to copy-paste it without thinking.

This is the single most severe finding from the recent security audit. It must be fixed before any further production releases (currently at 2.5.16).

## What Changes

1. **Remove the hardcoded default**. `AuthConfig::default()` should no longer ship a usable secret.
2. **Fail-fast on startup** if the loaded configuration still contains the default secret, an empty string, or a secret shorter than 32 chars. Return a clear error: refuse to boot.
3. **Auto-generate on first install** (optional path): if no secret is present, generate a CSPRNG secret, persist it to `data/jwt_secret.key` with 0600 perms, and log the path (never the value).
4. Update `config.example.yml`, `.env.example`, and docs to show `# REQUIRED: generate with `openssl rand -hex 64`` instead of a placeholder.
5. Add a unit test proving `AuthConfig::validate()` rejects the old default and short secrets.

## Impact

- Affected specs: `/.rulebook/specs/SECURITY.md` (create if missing), auth module spec.
- Affected code: `src/auth/mod.rs`, `src/auth/jwt.rs`, `src/cli/config.rs`, `src/server/mod.rs` (boot path), `config.example.yml`, `.env.example`.
- Breaking change: **YES** — operators relying on the default secret must configure one. Document clearly in CHANGELOG under "Breaking".
- User benefit: closes a critical auth bypass; deployments can no longer silently ship with a known-weak secret.
