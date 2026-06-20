---
title: Rust `target/` hygiene
module: development
id: rust-target-hygiene
order: 5
description: Keep the Cargo build directory bounded with line-tables-only debuginfo, `cargo-sweep`, and `CARGO_INCREMENTAL=0` on CI.
tags: [rust, cargo, target, debuginfo, cargo-sweep, disk, ci]
---

# Rust `target/` hygiene

Cargo never garbage-collects `target/`. Stale object files,
incremental caches, and compiled rlibs from **old dependency
versions** accumulate indefinitely until something explicitly
deletes them. Combined with the dev profile's default full
debuginfo for every workspace crate AND every transitive
dependency, `target/` grows without bound — the sibling
[`hivellm/cortex`] repo hit **500+ GB** before anyone noticed.

This page is the maintainer-facing runbook for keeping the Cargo
build directory under control on a developer machine. CI runners
are immune (cold cache, fresh container) but local boxes pay the
cost.

[`hivellm/cortex`]: https://github.com/hivellm/cortex

> Background: see [issue #320] and the Rust perf-team blog post
> [Disable debuginfo to improve Rust compile times][kobzol-2025].

[issue #320]: https://github.com/hivellm/vectorizer/issues/320
[kobzol-2025]: https://kobzol.github.io/rust/rustc/2025/05/20/disable-debuginfo-to-improve-rust-compile-times.html

## What `target/` actually holds

| Subdir | Contents | Typical size |
|---|---|---|
| `target/debug/` | dev-profile rlibs + .o + executables for every workspace member + transitive dep | 10–30 GB |
| `target/release/` | release-profile rlibs + binaries (LTO=thin, codegen-units=4) | 5–15 GB |
| `target/release-docker/` | container-build profile (LTO=off, codegen-units=16) when the Docker build is run host-side | 4–10 GB |
| `target/.rustc_info.json` | incremental compilation marker — DO NOT delete | <1 KB |
| `target/<triple>/...` | per-target outputs when cross-compiling (xx-cargo) | varies |
| `target/{ci,release-fast,perf}/` | per-profile outputs from one-off runs | grows quietly |

Every change to `Cargo.lock` or a feature flag invalidates a whole
column. Rust does not free the old artifacts — it just builds the
new ones alongside.

## The three knobs this repo already turns on

All three live in [`Cargo.toml`](../../Cargo.toml) at the workspace
root with comments referencing issue #320.

### 1. `[profile.dev] debug = "line-tables-only"`

Keeps `file:line` in panics and backtraces so debugging stays
productive, but drops the full DWARF tree (`.debug_info`,
`.debug_pubnames`, etc.). Per the Rust perf-team blog post linked
above, this shrinks `target/debug/` by ~30–40% and yields the
same percentage improvement on incremental rebuild wall time.

### 2. `[profile.release] strip = true`

Drops the residual symbol table from the shipped release binary.
Shrinks the final executable on disk and limits the information
leaked through stack-trace reverse engineering. The build itself
still produces full debuginfo intermediates; `strip` only applies
when linking the final binary.

### 3. `CARGO_INCREMENTAL=0` on every CI workflow

Wired in [`.github/workflows/*.yml`](../../.github/workflows/) at
the workflow `env:` block. Each GitHub Actions run starts from a
cold cache, so incremental compilation only adds artifacts (the
`.rustc_info.json` ledger plus per-codegen-unit fingerprints)
without ever benefiting from them. Turning it off shaves a few
percent off cold compile time and reduces the artifact upload
weight.

## When to use `cargo sweep` vs `cargo clean`

| Tool | Removes | Keeps | Use when |
|---|---|---|---|
| `cargo sweep --time N` | rlibs / object files / docs **not accessed** in the last N days | Incremental hot set, `.rustc_info.json`, recently-touched profiles | Routine hygiene — daily, weekly, or whenever `target/` crosses your size budget. Next build stays fast. |
| `cargo clean` | Everything under `target/` | Nothing | Debugging a build-script regression, switching toolchains, or reproducing CI's cold-start exactly. Next build is FULL cold rebuild. |

Reach for `sweep` first. Fall back to `clean` only when you need
a fully cold build.

## Running the sweep

Two parity scripts live under [`scripts/`](../../scripts/):

### Linux / macOS

```bash
bash scripts/sweep-target.sh           # default: 14-day cutoff
bash scripts/sweep-target.sh 7         # tighter cutoff (kept = last 7 days)
```

### Windows

```powershell
pwsh scripts/sweep-target.ps1                  # default: 14-day cutoff
pwsh scripts/sweep-target.ps1 -Days 7          # tighter cutoff
```

Both scripts:

1. Install `cargo-sweep` if missing (`cargo install --locked cargo-sweep`).
2. Run `cargo sweep --time N` (default `N=14`).
3. Print bytes reclaimed and remaining `target/` size.
4. Preserve the incremental hot set so the next build stays fast.

## Scheduling

Drop the sweep into the OS scheduler so it runs without manual
intervention.

### cron (Linux / macOS)

```cron
# ~/.config/cron.d/vectorizer-sweep — daily at 03:00
0 3 * * * cd /path/to/vectorizer && bash scripts/sweep-target.sh 14 \
            >> /tmp/vectorizer-sweep.log 2>&1
```

### Task Scheduler (Windows)

1. Open **Task Scheduler** → **Create Basic Task**.
2. Trigger: Daily, 03:00.
3. Action: **Start a program**.
4. Program/script: `pwsh.exe`
5. Add arguments: `-NoLogo -File C:\path\to\vectorizer\scripts\sweep-target.ps1 -Days 14`

### launchd (macOS GUI)

Save under `~/Library/LaunchAgents/com.hivellm.vectorizer-sweep.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>Label</key><string>com.hivellm.vectorizer-sweep</string>
  <key>ProgramArguments</key>
  <array>
    <string>/bin/bash</string>
    <string>/path/to/vectorizer/scripts/sweep-target.sh</string>
    <string>14</string>
  </array>
  <key>StartCalendarInterval</key>
  <dict><key>Hour</key><integer>3</integer><key>Minute</key><integer>0</integer></dict>
  <key>StandardOutPath</key><string>/tmp/vectorizer-sweep.log</string>
  <key>StandardErrorPath</key><string>/tmp/vectorizer-sweep.log</string>
</dict></plist>
```

Load with `launchctl load ~/Library/LaunchAgents/com.hivellm.vectorizer-sweep.plist`.

## Inspecting `target/` size

```bash
# Linux / macOS
du -sh target            # whole tree
du -sh target/*          # per-subdir
du -h target | sort -hr | head -20    # heaviest dirs first
```

```powershell
# Windows
(Get-ChildItem -Path target -Recurse -Force -ErrorAction SilentlyContinue |
   Measure-Object -Property Length -Sum).Sum / 1GB
```

## When the sweep isn't enough

If `target/` is still uncomfortably large after a sweep, run
`cargo clean` and rebuild. Common culprits:

- A `Cargo.lock` overhaul (e.g., a deps-batch PR) generated a full
  parallel artifact set that the dev hot set keeps alive.
- Multiple toolchains (`stable`, `nightly`, `1.95.0`) each maintain
  their own slice — `rustup toolchain list` to confirm.
- A foreign profile (`cargo test --profile ci`, `--profile release-fast`,
  `--profile perf`) was used once and is now stale.

After `cargo clean`, the first build will be cold — expect 5–15 min
on a typical dev box.
