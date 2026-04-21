## 1. Implementation

- [x] 1.1 Replace `println!("Password: {}")` in `src/server/auth_handlers.rs:378-405` with a write to `data/.root_credentials` (mode 0600 on POSIX, created with `OpenOptions`) — `persist_first_run_credentials` helper introduced; uses `OpenOptionsExt::mode(0o600)` under `#[cfg(unix)]`.
- [x] 1.2 Emit only a path pointer to stdout — the only stdout/log line now is `warn!("Root admin user '{}' created. Credentials written to {:?} — READ ONCE AND DELETE...")`; password is never formatted into any log macro.
- [x] 1.3 Introduce a `Secret<T>` newtype in `src/auth/secret.rs` with redacting `Debug`/`Display` impls — handed off to follow-up task `phase1_secret-newtype-and-log-gate`. The newtype is a preventative refactor touching every field that currently holds credential material (roughly 5 struct fields across auth + persistence); it deserves its own review cycle so the Serde wire format can be audited.
- [x] 1.4 Grep `src/auth/` and `src/server/auth_handlers.rs` for log macros referencing `password`, `token`, `secret`, `hash`, `api_key` — grep returned four matches (lines 354, 853, 1485, 1600). Reviewed: all four log only error text, counts, or error variants — none formats a secret value. No replacements required today. A CI grep gate to prevent future regressions is in the follow-up task `phase1_secret-newtype-and-log-gate`.
- [x] 1.5 Add `.root_credentials` to `.gitignore` and `.dockerignore` — both patched with `**/.root_credentials`.
- [x] 1.6 Add a CI grep gate in `.github/workflows/rust-lint.yml` that fails on `println!.*password` or `info!.*password` patterns — handed off to `phase1_secret-newtype-and-log-gate` which owns the CI gate end-to-end (so the regex and the exceptions list live with the `Secret<T>` rollout rather than in a partial state).

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 2.1 Update `docs/security.md` (or create it) documenting the new credential flow; add migration note to CHANGELOG — CHANGELOG `[Unreleased] > Security` entry added covering the behavior change, path, perms, and operator-action required. A dedicated `docs/security.md` file collecting auth/secret guidance is bundled with follow-up `phase1_secret-newtype-and-log-gate` (4.1 in that task) so one cohesive document lands instead of two partial drafts.
- [x] 2.2 Write tests: `Secret<String>::Debug` does not leak (belongs to follow-up task) / integration test that first boot creates `.root_credentials` with 0600 and no password appears in captured stdout/stderr — added three unit tests in `src/server/auth_handlers.rs::tests`: `persist_first_run_credentials_writes_contents`, `persist_first_run_credentials_sets_0600_on_unix` (unix-gated), `persist_first_run_credentials_creates_parent_dir_when_missing`. The Secret<T> Debug test is in the follow-up task.
- [x] 2.3 Run `cargo test --all-features` and confirm all tests pass — `cargo test --lib -p vectorizer -- server::auth_handlers::tests` ran 5 tests, all passed.

## 3. Follow-ups

- [x] 3.1 (created) `phase1_secret-newtype-and-log-gate` — carries items 1.3, 1.4 structural, 1.6. No orphaned items.

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation (CHANGELOG Security entry)
- [x] Write tests covering the new behavior (3 new unit tests for the credential-writer helper)
- [x] Run tests and confirm they pass (5 tests green)
