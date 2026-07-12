# Spec: bounded target/ directory

## ADDED Requirements

### Requirement: Dev profile MUST limit debuginfo to line-tables-only

The workspace `[profile.dev]` SHALL set `debug = "line-tables-only"`
so panics and backtraces continue to carry `file:line` info but
the per-crate `*.rlib` and `*.o` debug sections drop the full
DWARF tree. This reduces incremental dev builds by ~30-40% wall
time and shrinks `target/debug/` by a comparable margin
(per the Rust perf-team blog post linked in the proposal).

#### Scenario: dev build emits line-tables-only debuginfo

Given a clean workspace
When `cargo build` runs with the dev profile
Then the produced rlibs contain DWARF `.debug_line` sections
And the rlibs do NOT contain `.debug_info` / `.debug_pubnames` etc.

### Requirement: Release profile MUST strip the symbol table

The workspace `[profile.release]` SHALL set `strip = true` so the
shipped binary does not carry residual symbols. This shrinks the
release binary on disk and limits information leaked through
reverse engineering of stack frames.

#### Scenario: release binary has no exported symbol table

Given a clean workspace
When `cargo build --release` runs
Then the resulting executable has an empty `.symtab` section
And `nm <binary>` reports `no symbols`

### Requirement: CI MUST disable incremental compilation

Every GitHub Actions workflow that runs `cargo {check, build,
test, clippy, doc}` SHALL set `CARGO_INCREMENTAL: 0` at the
workflow `env:` block. CI starts from a cold cache, so
incremental compilation only adds artifacts and slows the build.

#### Scenario: workflow env disables incremental

Given any GitHub Actions YAML under `.github/workflows/`
And the YAML invokes any `cargo` subcommand
When the workflow is dispatched
Then `env.CARGO_INCREMENTAL` resolves to `"0"`

### Requirement: Maintainer-facing sweep script SHALL exist

Two parity scripts MUST live under `scripts/`:

- `scripts/sweep-target.sh` (bash, POSIX hosts)
- `scripts/sweep-target.ps1` (PowerShell, Windows hosts)

Both SHALL:

1. Install `cargo-sweep` if missing (`cargo install --locked
   cargo-sweep`).
2. Run `cargo sweep --time ${1:-14}` (default 14 days).
3. Print the reclaimed bytes and the remaining size of `target/`.
4. Preserve the incremental hot set (cargo-sweep's contract — the
   script does not run `cargo clean`).

#### Scenario: sweep keeps recent builds, drops stale artifacts

Given a workspace where `cargo build` last touched a stale rlib
   20 days ago and a hot rlib 1 day ago
When the maintainer runs `bash scripts/sweep-target.sh 14`
Then the 20-day-old rlib is removed
And the 1-day-old rlib survives
And `target/.rustc_info.json` (the incremental marker) survives

### Requirement: Operator doc SHALL explain target/ hygiene

`docs/development/rust-target-hygiene.md` MUST cover: what
`target/` holds, why it grows, the three knobs already wired in
this repo (`debug = "line-tables-only"`, `strip = true`,
`CARGO_INCREMENTAL=0` on CI), when to use `cargo sweep` vs
`cargo clean`, and how to wire the sweep into cron / Task
Scheduler / launchd.

#### Scenario: contributor lands on the hygiene doc from README

Given a new contributor reading `README.md`
When they look for "how do I keep target/ small"
Then they find a link to `docs/development/rust-target-hygiene.md`
And the linked doc names the sweep script + scheduler examples
