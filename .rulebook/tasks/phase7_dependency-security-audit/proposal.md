# Proposal: phase7_dependency-security-audit

Close the dependency advisories, and fix the reason nobody knew about them.

## Why

The GitHub Dependabot page shows **2** open security alerts. Running the local
tools against the same repository finds **6 npm** advisories across three
projects and **19 Rust** findings — 9 vulnerabilities, 6 unmaintained crates,
2 unsound, 2 yanked.

That gap is not a misconfiguration. `.github/dependabot.yml` watches every
directory that has a manifest. The causes are structural:

- Dependabot reads the GitHub Advisory Database; `cargo audit` reads RustSec.
  Every `RUSTSEC-2026-*` finding here is absent from the Dependabot list.
- Transitive advisories surface unevenly: `dashboard`'s `nanoid` and
  `sdks/typescript`'s `brace-expansion` are found by `pnpm audit` and are not
  open Dependabot alerts, despite both directories being watched.

**And nothing in CI runs either tool.** There are 17 workflows covering build,
lint, docs, SIMD matrices and every SDK's tests and publication; none audits
dependencies. An `audit.toml` policy file exists at the repository root and
nothing reads it.

That is the same shape as the benchmark harness this repository just retracted:
a file carrying a real decision, unregistered with anything that runs, drifting
unnoticed until someone went looking. A rule nothing enforces is not a rule.

## What this task does

Fixes the pipeline before the backlog it produced, because draining advisories
first would leave nothing watching for the next batch.

Then it works the findings — but only after separating them by what can
actually be done, which the raw tool output does not do. `cargo audit` reads
`Cargo.lock`, not the compiled graph: a crate can be listed through an optional
feature nobody enables, and can be pinned by a *parent's* semver requirement so
no `cargo update` moves it. Checking each finding twice (`cargo tree -i` for
reachability, `cargo update --dry-run` for fixability) turns "9 vulnerabilities
to fix" into:

- **2** fixable today — `h2` (HTTP/2 DoS against the server we ship) and `lru`
- **5** blocked upstream — `quick-xml` ×2 and `lopdf` ×2 on the file-upload
  path, `lru` 0.16.4 via `tantivy`
- **1** with no patch anywhere — `rsa`, the Marvin timing attack, on the auth
  path via `jsonwebtoken`
- **1** probably not compiled — `rkyv`, via an unenabled `rust_decimal` feature

Without that pass the task would promise eight fixes it cannot deliver.

## Scope decisions worth stating

**The npm advisories are all dev- or build-scoped.** None reaches a published
SDK consumer or a served page. They are still worth clearing — three open PRs
already carry the fixes — but they are not the urgent half, and the task says
so rather than treating six high-severity labels as six emergencies.

**The document parsers are the urgent half.** `lopdf` and `quick-xml` are
denial of service *on parsed input*, they sit on the path that accepts
documents from callers, and `transmutation` is in the default feature set — so
a stock build ships all of it. These are also the ones we control least, which
is why the task asks for a written mitigation when the version cannot move.

**26 open PRs are drained as a batch, with four read individually.** One is
red (`openraft`, the consensus dependency). One takes a *major* version bump to
fix a vulnerability patched in 4.3.1 (`js-yaml` #421) — merging it because it
is green and labelled security is how an unrelated breaking change lands under
a security banner. Two others cross a major boundary.

## Out of scope

Application-level security review (authn/authz logic, input validation, the
`/ready` and hub middleware allow-lists). This task is about dependencies and
the pipeline that watches them. Three unrelated findings recorded during recent
work — `create_collection` ignoring `hnsw_config`, MCP `create_collection` not
validating providers, and `MAX_SEARCH_LIMIT` sitting exactly at glove's `top` —
belong to their own tasks and are noted here only so they are not lost.

## Reference

Full findings, with the dependency path and fixability verdict for every
advisory: `docs/analysis/dependency-security-2026-08/`.
