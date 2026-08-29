# 05 — Why this backlog accumulated

The advisories in this analysis are the symptom. The reason they were unknown
until someone went looking is structural, and fixing it is worth more than any
individual bump.

## Nothing runs `cargo audit`

`audit.toml` exists at the repository root. It carries a real policy decision:

```toml
[advisories]
# Ignore paste crate unmaintained warning - it's a transitive dependency
# and the functionality is still working fine
ignore = ["RUSTSEC-2024-0436"]
```

**No workflow in `.github/workflows/` runs `cargo audit` or `cargo deny`.**
There are 17 workflows covering build, lint, docs, SIMD matrices, and every
SDK's tests and publication. None audits dependencies.

So the policy file is a statement nobody checks — the same shape as the
benchmark harness that was never registered as a `[[bench]]` target and drifted
until it published a void number. A rule that nothing enforces is not a rule.

This is the highest-value item in the whole effort: **one workflow would have
surfaced all 19 Rust findings the day they landed.**

## `audit.toml` may not be doing what it says

```toml
[advisories.unmaintained]
# Allow unmaintained crates that are still functional
warn = false
```

`cargo audit` 0.22.1 reported `10 allowed warnings found` while still listing
6 unmaintained crates. Whether that section is honoured under the current
schema is unverified — and unverifiable so long as nothing runs the tool.
Confirm the file is effective as part of wiring the workflow, rather than
carrying a config that reads as a decision but may be inert.

## Dependabot is a floor, not a ceiling

Covered in [01-scope-and-method.md](01-scope-and-method.md): Dependabot reads
the GitHub Advisory Database, `cargo audit` reads RustSec, and `pnpm audit`
found npm advisories in watched directories that Dependabot has not raised.
Each tool is a partial view. Reading only the Dependabot page understated the
surface by roughly an order of magnitude.

## No npm audit in CI either

`pnpm audit` found 6 high-severity advisories across `gui`, `dashboard` and
`sdks/typescript`. Nothing runs it. The same one-workflow argument applies,
with one caveat: because these are all dev-scoped
([03-npm-advisories.md](03-npm-advisories.md)), a gate that fails the build on
any advisory will fail constantly on tooling that cannot reach a user. Decide
the threshold — production dependencies only, or advisory severity — before
wiring it, or the gate gets disabled the first week.

## Recommendation, in priority order

1. **Wire dependency auditing into CI.** Rust first (`cargo audit` or
   `cargo deny`), npm second with a deliberately chosen threshold.
2. **Fix what is fixable now** — `h2`, `lru`, and the three PRs that already
   carry npm fixes.
3. **Decide, in writing, what cannot be fixed** — the `rsa` Marvin advisory
   with no patch, and the `quick-xml` / `lopdf` versions pinned by parents.
   Each belongs in `audit.toml` with its reasoning, so the next audit run is
   quiet for known reasons rather than noisy for unknown ones.
4. **Drain the PR backlog** with the four exceptions in
   [04-open-pull-requests.md](04-open-pull-requests.md) read individually.
