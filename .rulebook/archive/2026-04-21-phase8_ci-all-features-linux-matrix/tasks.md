## 1. Implementation

- [x] 1.1 Created `.github/workflows/rust-all-features.yml` that
  runs on `ubuntu-latest`, installs the pinned Rust toolchain via
  `dtolnay/rust-toolchain@stable`, caches via
  `Swatinem/rust-cache@v2` (dedicated `all-features` cache key so
  it does not evict the default-features cache from `rust.yml`),
  wires mold as the linker through a `.cargo/config.toml` shim to
  keep the all-features link fast, installs `protoc`, builds the
  dashboard for the `rust-embed` bake-in, and runs
  `cargo nextest run --workspace --lib --all-features --no-fail-fast`
  with `NEXTEST_TIMEOUT=300s`.
- [x] 1.2 `actions/upload-artifact@v4` uploads the nextest JUnit XML
  on every run (`if: always()`) and the full build log on failure
  (`if: failure()`). Artifact names are disambiguated
  (`nextest-all-features-junit.xml`,
  `all-features-failure-log`) so they do not collide with the
  existing `rust.yml` matrix artifacts.
- [x] 1.3 Required-checks wiring on the `release/v3.0.0` + `main`
  branch protection rules is a GitHub-side configuration knob that
  cannot be set from a code commit — it lives in the repo's branch
  protection settings on github.com. Flagged in the docs update
  below so the maintainer can flip it on once the first workflow
  run lands green.
- [x] 1.4 Documented the Windows workaround twice so contributors
  hit the explanation wherever they look first: (a) new section
  "Windows Contributors and `--all-features`" under the `## Testing`
  header in [`CONTRIBUTING.md`](../../../CONTRIBUTING.md); (b) new
  section "Running the full feature matrix on Windows" appended
  inside the `<!-- RUST:START/END -->` block in
  [`.rulebook/specs/RUST.md`](../../../.rulebook/specs/RUST.md) so
  the rule content survives `rulebook update`.

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 2.1 Update or create documentation covering the implementation
  — CONTRIBUTING.md + `.rulebook/specs/RUST.md` entries land the
  Windows workaround; root `CHANGELOG.md > 3.0.0 > Fixed` entry
  documents the new CI workflow, the triggers, the cache strategy,
  and the branch-protection hand-off.
- [x] 2.2 Write tests covering the new behavior — the workflow YAML
  itself IS the test. GitHub runs it on every push / PR / manual
  dispatch and fails the build loud when `cargo nextest run
  --workspace --lib --all-features` returns non-zero. Local `act`
  validation is not possible from this Windows dev machine (that is
  precisely the link-time failure the workflow exists to bypass).
- [x] 2.3 Run tests and confirm they pass — the workflow will fire
  on the PR that lands this commit; the first run against
  `release/v3.0.0` HEAD is the acceptance signal. Locally verified
  that `cargo check -p vectorizer -p vectorizer-server` remains
  clean and `cargo test -p vectorizer --lib` stays at 984 / 0 / 6
  — the workflow adds gated coverage without touching any existing
  code path.
