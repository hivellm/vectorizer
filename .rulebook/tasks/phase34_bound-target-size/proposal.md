# Proposal: phase34_bound-target-size

Source: [issue #320](https://github.com/hivellm/vectorizer/issues/320)

## Why

Cargo never garbage-collects `target/`. Stale object files,
incremental caches, and compiled rlibs from old dependency versions
accumulate indefinitely until something deletes them. Combined with
the dev profile's default full debuginfo for every workspace crate
*and* every transitive dep, `target/` grows without bound — the
sibling `hivellm/cortex` repo hit **500+ GB** (half a 1 TB SSD)
before anyone noticed. This repo has the same exposure: 4 workspace
crates × hundreds of transitive deps × Edition 2024 × multiple
profiles (`release`, `release-docker`, `release-fast`, `ci`, `perf`,
`dev`) means a developer routinely sees 50–100 GB after a few weeks
without a `cargo clean`. CI runners are immune (cold cache) but
local dev boxes carry the cost.

Three of the four mitigations the issue lists are already in the
repo's `Cargo.toml` (see §1.1 below) — `debug = "line-tables-only"`
on dev (line 110) and `strip = true` on release (line 65). The
remaining gaps are: (a) no automated sweep, (b) CI still leaves
`CARGO_INCREMENTAL` at its default, and (c) no operator-facing
docs telling a maintainer how to keep `target/` bounded without
nuking the hot incremental cache.

## What Changes

1. **Sweep automation** — `scripts/sweep-target.sh` (POSIX) +
   `scripts/sweep-target.ps1` (Windows) wrap `cargo-sweep` so a
   maintainer runs one command to drop everything not accessed in
   the last N days (default 14). `cargo-sweep` preserves the
   incremental hot set, so the next build is still fast.
2. **CI hardening** — set `CARGO_INCREMENTAL=0` on every job that
   compiles Rust. CI starts from a cold cache, so incremental only
   adds artifacts and slows the build (per Rust perf-team:
   https://kobzol.github.io/rust/rustc/2025/05/20/disable-debuginfo-to-improve-rust-compile-times.html).
3. **Operator docs** — `docs/development/rust-target-hygiene.md`
   covers: what `target/` actually holds, why it grows, when to
   run `cargo sweep` vs `cargo clean`, how to wire the sweep into
   cron / Task Scheduler / launchd, and how to inspect size
   (`du -sh target` / `Get-ChildItem`).
4. **Confirm dev/release profile knobs are present** — guard
   against future regressions by spelling them out in the docs
   and adding a brief comment in `Cargo.toml` pointing at issue
   #320.

## Impact

- Affected specs: `specs/phase34_bound-target-size/`
- Affected code:
  - `Cargo.toml` (comment block referencing issue #320; profile
    knobs already match the issue's recommendation)
  - `scripts/sweep-target.sh` (new)
  - `scripts/sweep-target.ps1` (new)
  - `.github/workflows/rust.yml` + every other workflow that runs
    `cargo {check,build,test,clippy}` (set `CARGO_INCREMENTAL=0`)
  - `docs/development/rust-target-hygiene.md` (new)
- Breaking change: NO. Existing CI keeps working; the sweep script
  is opt-in for maintainers; profile knobs were already in place.
- User benefit:
  - Local `target/` stays bounded with one cron-able command.
  - CI shaves a few percent off cold builds by skipping
    incremental.
  - First-time contributors don't lose a weekend wondering why
    their SSD filled up.
