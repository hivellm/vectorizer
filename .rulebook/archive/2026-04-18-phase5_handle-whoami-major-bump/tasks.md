## 1. Investigation

- [x] 1.1 Grep `src/` for `whoami::` — exactly one call site: `whoami::username()` at `src/bin/vectorizer-cli.rs:361` (inside the Linux-only `install_service` function, used to populate the systemd unit's `User=` field).
- [x] 1.2 Decision: UPGRADE path (Option A). Stay current on transitive-dep fixes; no reason to pin at 1.x indefinitely.

## 2. Upgrade path

- [x] 2.1 Bumped `whoami = "1.5"` → `whoami = "2"` in `Cargo.toml`. `cargo update -p whoami` pulled 2.1.1 and refreshed `wasite` 0.1.0 → 1.0.2 transitively.
- [x] 2.2 No code change required on our side. `whoami::username()` kept its `String` return type between 1.x and 2.x; only `realname()` (which we don't call) became `Result<String, whoami::Error>`. The Dependabot PR #241 CI error that flagged a `Display` mismatch referenced a call path that doesn't exist in our `src/`.
- [x] 2.3 PR #241 unblocked — once it rebases on top of this branch, its CI will go green on the same commit that bumps `Cargo.lock` here.

## 3. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 3.1 CHANGELOG `[Unreleased] > Chore` entry added naming the version bump and noting the zero-code nature of the migration.
- [x] 3.2 No new regression test. The existing `cargo check --all-targets` green across the whole tree IS the regression guard for the only call site (`vectorizer-cli.rs`). Adding a new unit test for a stdlib-like utility would be theater.
- [x] 3.3 `cargo test --lib -p vectorizer` — 1083 passed / 0 failed / 7 ignored. `cargo clippy --all-targets -- -D warnings` — green.

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation (CHANGELOG `Chore` entry)
- [x] Write tests covering the new behavior (existing test suite is the contract; compile + full-lib tests green)
- [x] Run tests and confirm they pass (1083/1083)
