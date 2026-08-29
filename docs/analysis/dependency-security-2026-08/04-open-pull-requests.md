# 04 — The 26 open pull requests

All 26 are Dependabot bumps. None is a feature or a fix from a person, so the
backlog can be drained mechanically — but not blindly.

## CI status

25 of 26 report every check green. One does not:

| PR | Bump | Status |
|---|---|---|
| **#401** | `openraft` 0.10.0-alpha.30 → alpha.33 | **FAILURE ×4**, CANCELLED ×1, SUCCESS ×4 |

`openraft` is the replication/consensus dependency and this is an
alpha-to-alpha move. The failure is the signal: read it before deciding
whether to fix forward or close the PR and pin.

## The ones that carry security fixes

Three open PRs already resolve advisories from
[03-npm-advisories.md](03-npm-advisories.md). Merging them is cheaper than
writing new commits:

| PR | Bump | Clears |
|---|---|---|
| #409 | `postcss` 8.5.25 → 8.5.26 in `/gui` | `nanoid` GHSA-2v37-7h3g-55p8 |
| #413 | `eslint` 10.8.0 → 10.8.1 in `/sdks/typescript` | possibly `brace-expansion` — **verify** |
| #394 | `hyper` 1.10.1 → 1.11.0 | possibly `h2` RUSTSEC-2026-0258 — **verify** |

`cargo update -p h2` reaches 0.4.19 on its own, so #394 is not required for
that fix; check whether it helps or merely coincides.

## The one to read before merging

**#421 — `js-yaml` 4.3.0 → 5.3.0 in `/sdks/typescript`.**

GHSA-5p4m-2wfm-xmqj is fixed in **4.3.1**. This PR jumps a major version to
fix it. Dependabot does that when the manifest range allows it, and it is not
wrong — but taking a major bump imports breaking changes that the
vulnerability did not require. `js-yaml` 5 changed its API surface.

Decide deliberately: take 4.3.1 for the fix now and schedule the major
separately, or take 5.3.0 and absorb the migration. Merging it because it is
green and labelled a security fix is how an unrelated breaking change lands
under a security banner.

## Everything else

22 routine bumps, all green — Rust (`blake3`, `xxhash-rust`, `fastrand`,
`serde_json`, `rustls`, `fastembed`, `bcrypt`, `base64`), npm dev tooling
(`vite`, `vue`, `vis-data`, `typescript`, `esbuild`, `@types/node`,
`typescript-eslint`, `@typescript-eslint/parser`, `@rolldown/binding-*`), and
NuGet (`System.Text.Json`, `Microsoft.SourceLink.GitHub`).

Two deserve a glance rather than a rubber stamp, because both cross a major
boundary:

- **#393** `base64` 0.22.1 → **0.23.1**
- **#412** `typescript` 6.0.3 → **7.0.2** in `/gui`

The rest can go in as a batch, and the full workspace gate is what confirms it.
