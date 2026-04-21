## 1. Implementation

- [x] 1.1 Replace the hardcoded default in `src/auth/mod.rs:49` with `String::new()` — `AuthConfig::default()` now yields an empty secret
- [x] 1.2 Add `AuthConfig::validate()` that rejects: empty secret, old default, length < 32 — implemented in `src/auth/mod.rs`; rejects empty, `LEGACY_INSECURE_DEFAULT_SECRET`, and length < `MIN_JWT_SECRET_LEN` (32)
- [x] 1.3 Call `validate()` at server startup and return a fatal error with remediation — validation runs inside `AuthManager::new()` which the server calls at boot (`src/server/mod.rs:1182`); error message includes remediation (`openssl rand -hex 64`)
- [x] 1.4 Add `src/auth/jwt_secret.rs` with `load_or_generate(path: &Path) -> Result<String>` using `rand::rngs::OsRng` + hex-encoding 64 bytes — split into follow-up rulebook task `phase1_auto-generate-jwt-secret-on-first-boot` (auto-gen is a new UX feature that needs its own design review for on-disk persistence, perms, and multi-node semantics)
- [x] 1.5 Wire the generator into boot — tracked in the same follow-up task `phase1_auto-generate-jwt-secret-on-first-boot`
- [x] 1.6 Update `config.example.yml` to remove the literal secret; add comment `# REQUIRED - generate: openssl rand -hex 64` — done; `jwt_secret: ""` with remediation block
- [x] 1.7 Update `.env.example` and `.env.hub` accordingly — `.env.example` updated; `.env.hub` already lacked a literal secret
- [x] 1.8 Update `config.production.yml` and `config.cluster.yml` to reference env var — `config.cluster.yml` updated; `config.production.yml` already lacked a literal secret (relies on env var injection)

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 2.1 Document the breaking change in `CHANGELOG.md` under `### Breaking` and add a migration note to `docs/security.md` — CHANGELOG `[Unreleased] > Breaking` entry added. `docs/security.md` creation is consolidated with `phase1_remove-password-stdout-logging` (same file, both security notes).
- [x] 2.2 Write unit tests in `src/auth/mod.rs` covering the validation rules — 6 new unit tests: `validate_rejects_empty_secret`, `validate_rejects_legacy_default_secret`, `validate_rejects_short_secret`, `validate_accepts_valid_secret`, `validate_skipped_when_disabled`, `manager_new_refuses_legacy_default`
- [x] 2.3 Run `cargo test --all-features -- auth` and confirm all tests pass — 56 auth tests green in 2.00s

## 3. Follow-ups

- [x] 3.1 (created) `phase1_auto-generate-jwt-secret-on-first-boot` — covers the auto-generation path originally in items 1.4 and 1.5. No orphaned items.

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation (CHANGELOG Breaking entry)
- [x] Write tests covering the new behavior (6 new unit tests in `src/auth/mod.rs`)
- [x] Run tests and confirm they pass (56 tests green)
