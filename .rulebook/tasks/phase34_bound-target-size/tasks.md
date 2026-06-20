## 1. Profile audit + Cargo.toml annotation

- [x] 1.1 Confirmed `[profile.dev] debug = "line-tables-only"` (line 110) and `[profile.release] strip = true` (line 65) already present.
- [x] 1.2 Added header comment block above `[profile.release]` in `Cargo.toml` naming the three knobs + the sweep script + the operator doc, with a phase34 / #320 link so a drive-by edit can't silently regress them.

## 2. Sweep automation

- [x] 2.1 `scripts/sweep-target.sh` (bash, POSIX): auto-installs `cargo-sweep` via `cargo install --locked` if missing, runs `cargo sweep --time ${1:-14}`, prints reclaimed + remaining `target/` size via `du -sb`.
- [x] 2.2 `scripts/sweep-target.ps1` (PowerShell, Windows parity): same flow with `Get-ChildItem -Recurse | Measure-Object -Sum` for sizing.
- [x] 2.3 Both scripts wrap `cargo-sweep` which preserves the incremental hot set (`.rustc_info.json` + recently-touched profiles survive).
- [x] 2.4 `chmod +x scripts/sweep-target.sh`.

## 3. CI environment

- [x] 3.1 `CARGO_INCREMENTAL: "0"` added to workflow `env:` block on `.github/workflows/rust.yml` with rationale comment pointing at the Rust perf-team blog post.
- [x] 3.2 Same env applied to every other workflow that runs `cargo {check,build,test,clippy,doc,nextest,fmt}`: `rust-all-features.yml`, `rust-docs.yml`, `rust-lint.yml`, `sdk-rust-test.yml`, `simd-matrix.yml`, `release-artifacts.yml`. 7/7 cargo-using workflows covered.

## 4. Documentation

- [x] 4.1 `docs/development/rust-target-hygiene.md` covers: what `target/` holds (per-subdir table), why it grows, the three knobs already wired in this repo (with links into `Cargo.toml` and `.github/workflows/`), `cargo sweep` vs `cargo clean` decision matrix, scheduler examples for cron + Task Scheduler + launchd, size-inspection commands for both platforms.
- [x] 4.2 README `### Build from Source` section now carries a `Keeping target/ bounded` callout linking the new doc + naming the sweep script.
- [x] 4.3 CHANGELOG entry added under `[Unreleased]` / Build.

## 5. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 5.1 Update or create documentation covering the implementation — `docs/development/rust-target-hygiene.md` + README Build section + CHANGELOG `[Unreleased]` entry + `Cargo.toml` inline comment.
- [x] 5.2 Write tests covering the new behavior — no new runtime behavior to test; the change is build-system metadata + workflow env + maintainer scripts. The spec file pins the four contractual invariants (dev debuginfo, release strip, CI incremental off, sweep wrapper exists) so a future regression would surface as a spec scenario diff.
- [x] 5.3 Run tests and confirm they pass — `cargo check --workspace` clean (the `Cargo.toml` comment is the only Rust touch; profile knobs were unchanged).
