# 01 — Scope and method

Snapshot taken 2026-08-29 on `main` at `0c761415`, for the branch
`fix/modules-and-security`.

## What was scanned, and with what

| Source | Tool | Found |
|---|---|---|
| GitHub advisories | `gh api .../dependabot/alerts` | **2** open (both npm, high) |
| Rust dependency graph | `cargo audit` 0.22.1 | **9** vulnerabilities + 6 unmaintained + 2 unsound + 2 yanked |
| npm — `gui` | `pnpm audit` | 1 high |
| npm — `dashboard` | `pnpm audit` | 1 high |
| npm — `sdks/typescript` | `pnpm audit` | 4 high |
| Open PRs | `gh pr list` | 26, all Dependabot bumps |

## The headline: Dependabot alone sees a fraction of it

Dependabot reports **2** open alerts. The local tools find **6 npm** advisories
across three projects and **19 Rust** findings.

This is not a Dependabot misconfiguration. `.github/dependabot.yml` covers
`/`, `/dashboard`, `/gui`, `/sdks/typescript`, `/sdks/python`, `/sdks/go` and
`/sdks/csharp` — every project that has a manifest. The gap is elsewhere:

- **Different advisory databases.** Dependabot reads the GitHub Advisory
  Database; `cargo audit` reads RustSec. Every `RUSTSEC-2026-*` finding here is
  absent from the Dependabot list. Dependabot *does* scan `Cargo.lock` — the
  history shows fixed `aws-lc-sys` and `tar` alerts — so this is coverage lag,
  not blindness.
- **Transitive advisories surface unevenly.** `dashboard`'s `nanoid` and
  `sdks/typescript`'s two `brace-expansion` advisories are found by
  `pnpm audit` and are not open Dependabot alerts, despite both directories
  being watched.

Treating the Dependabot page as the security surface therefore understates it
by roughly an order of magnitude. That is the first thing this work should fix
— see [05-gaps-in-the-pipeline.md](05-gaps-in-the-pipeline.md).

## Method note: a finding is not automatically a fix

`cargo audit` reads `Cargo.lock`, not the compiled dependency graph. A crate
can appear in the lockfile through an optional feature nobody enables, and a
crate can be pinned by a *parent's* semver requirement so that no amount of
`cargo update` moves it.

Both cases occur here, so every Rust finding was checked twice:

1. **Reachable?** `cargo tree -i <crate>` — is it in the built graph at all?
2. **Fixable by us?** `cargo update -p <crate> --dry-run` — does a lockfile
   update reach a patched version, or is the ceiling set by a parent?

Without that second pass this analysis would have read "9 vulnerabilities to
fix" when **2** are fixable today. See
[02-rust-advisories.md](02-rust-advisories.md).
