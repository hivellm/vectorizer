## 1. Implementation

- [ ] 1.1 Create `.github/workflows/rust-all-features.yml` that
  runs on `ubuntu-latest`, installs the pinned Rust toolchain via
  `dtolnay/rust-toolchain@stable`, caches via
  `Swatinem/rust-cache@v2`, and executes
  `cargo test --workspace --lib --all-features --no-fail-fast`.
- [ ] 1.2 Upload the full test log as a workflow artifact on
  failure (`actions/upload-artifact@v4` with `if: failure()`).
- [ ] 1.3 Add the new workflow to the required-checks list on the
  `release/v3.0.0` and `main` branch protection rules so the
  all-features signal gates merges.
- [ ] 1.4 Document the Windows workaround in `docs/specs/RUST.md`:
  "Windows contributors run `cargo test --workspace --lib`
  (no `--all-features`); all-features coverage lives on CI".

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 2.1 Update or create documentation covering the implementation
  (`docs/specs/RUST.md` + `CONTRIBUTING.md` Windows note;
  `CHANGELOG.md > 3.0.0 > Chore` entry).
- [ ] 2.2 Write tests covering the new behavior (the workflow itself
  IS the test; validate locally via `act` or a `workflow_dispatch`
  trigger run before landing branch protection).
- [ ] 2.3 Run tests and confirm they pass (trigger the workflow on
  the PR that lands it + confirm it goes green on `ubuntu-latest`
  against `release/v3.0.0` HEAD).
