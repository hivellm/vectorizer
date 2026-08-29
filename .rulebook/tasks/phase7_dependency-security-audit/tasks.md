## 1. Implementation

Full findings in `docs/analysis/dependency-security-2026-08/`.

Ordered so the thing that let this accumulate is fixed before the backlog it
produced. Draining advisories first would leave nothing watching for the next
batch — which is exactly how this one arrived unannounced.

- [ ] 1.1 Wire dependency auditing into CI. There are 17 workflows covering
      build, lint, docs, SIMD matrices and every SDK's tests and publication.
      **None audits dependencies.** `audit.toml` exists at the repo root and
      carries a real policy decision (`ignore = ["RUSTSEC-2024-0436"]`), and
      nothing reads it — a rule nothing enforces is not a rule, the same shape
      as the benchmark harness that was never a `[[bench]]` target and drifted
      until it published a void number.
      One workflow would have surfaced all 19 Rust findings the day they
      landed, which is why this is first rather than last.
      **Done when:** a workflow runs `cargo audit` (or `cargo deny`) on push
      and PR, fails on unignored vulnerabilities, and its first run reproduces
      the counts in `02-rust-advisories.md`.
- [ ] 1.2 Confirm `audit.toml` is actually effective. It carries
      `[advisories.unmaintained] warn = false`, yet the local run reported
      `10 allowed warnings` while still listing 6 unmaintained crates.
      Whether that section is honoured under cargo-audit 0.22's schema is
      unverified — and unverifiable while nothing runs the tool.
      **Done when:** the file's behaviour is demonstrated rather than assumed;
      any inert section is corrected or removed, so it stops reading as a
      decision that was never in force.
- [ ] 1.3 Fix the two Rust advisories that are fixable today. Everything else
      in `cargo audit` is pinned by a parent or has no patch — see 1.5, 1.6.
      - `h2` 0.4.15 → ≥ 0.4.16 (RUSTSEC-2026-0258, *unbounded empty DATA
        frames*). `cargo update -p h2` reaches 0.4.19. This is the one with
        real runtime exposure: an HTTP/2 denial of service against the server
        we ship.
      - `lru` 0.18.1 → 0.18.3 (RUSTSEC-2026-0253, *unsound*: use-after-free if
        `LruCache::pop()` panics). Direct dependency of `vectorizer` and
        `vectorizer-server`. The advisory listed **no patched range** in the
        snapshot — confirm 0.18.3 clears it rather than assuming the bump is
        the fix.
      **Done when:** `cargo audit` no longer reports either, verified by
      re-running it, and the full workspace gate is green.
- [ ] 1.4 Clear the npm advisories. All six are dev- or build-scoped, so none
      reaches a published SDK consumer or a served page — but three open PRs
      already carry the fixes and merging them is cheaper than new commits.
      - `nanoid` (GHSA-2v37-7h3g-55p8) in `gui`, `dashboard`,
        `sdks/typescript` → `postcss` ≥ 8.5.26. **PR #409 already does `gui`.**
      - `brace-expansion` (GHSA-mh99-v99m-4gvg, GHSA-rgw5-rvv9-x895) via
        `eslint` → `minimatch`. **PR #413** may carry it — verify, do not
        assume.
      - `js-yaml` (GHSA-5p4m-2wfm-xmqj) → **4.3.1**. See 1.7 before touching
        PR #421.
      **One thing to check rather than read off the manifest:** `nanoid` in
      `gui` is classified a *production* dependency (via `@vueuse/core` → `vue`
      → `@vue/compiler-sfc` → `postcss`) while its actual role is compile-time
      CSS processing. Inspect the built bundle. The classification and the
      reality disagree and only one of them decides urgency.
      **Done when:** `pnpm audit` is clean in all three projects, and the
      `gui` bundle question is answered in writing either way.
- [ ] 1.5 Decide, in writing, the advisories that have no fix — then record
      the decision in `audit.toml` with its reasoning. A quiet audit run for
      known reasons is the goal; silence for unknown ones is what got us here.
      - **RUSTSEC-2023-0071 `rsa` 0.9.10** (Marvin attack, key recovery via
        timing side channels) via `jsonwebtoken` 10.4.0. `patched: []` — there
        is no version to move to, and this is on the **auth path**. Determine
        whether our JWT algorithms ever exercise the RSA code path; if we only
        sign HS256 the vulnerable code may never run. **Verify against the
        code, do not reason from the default config.**
      - **RUSTSEC-2026-0235 `rkyv` 0.7.46** — only `rust_decimal` lists it and
        `cargo tree -i rkyv` prints nothing, so it is likely a lockfile
        artifact of an unenabled optional feature. Confirm, then ignore with
        that reason.
      **Done when:** each has an `audit.toml` entry whose comment states what
      was checked, not merely that it was accepted.
- [ ] 1.6 The document-parser advisories — the ones that matter most and that
      we control least. All three are denial of service **on parsed input**,
      and they sit on the file-upload / transmutation path, which takes
      documents from callers. `transmutation` is in the crate's **default**
      feature set, so a stock build ships every one of them.
      - `lopdf` 0.34.0 (via `pdf-extract`) and 0.35.0 (via `transmutation`) —
        RUSTSEC-2026-0187, stack overflow on deeply nested PDF objects.
        Patched ≥ 0.42.0.
      - `quick-xml` 0.36.2 (via `docx-rs`) and 0.37.5 (via `transmutation`,
        `umya-spreadsheet`) — RUSTSEC-2026-0194 (quadratic time on duplicate
        attribute names) and -0195 (unbounded namespace allocation). Patched
        ≥ 0.41.0.
      `cargo update` moves none of them: the ceiling is the parent's semver
      requirement, confirmed by `Locking 0 packages` on every `--dry-run`.
      Options in order: bump `transmutation` (it is ours — `hivellm`), which
      clears two rows directly; then upstream PRs to `docx-rs`,
      `umya-spreadsheet`, `pdf-extract` or replacements; then making
      `transmutation` non-default so operators opt into the document parsers.
      That last one is a behaviour change and needs its own decision, not a
      quiet flip.
      **Done when:** either the versions move, or the exposure is written down
      with the mitigation that stands in for a patch (upload size limits,
      parser timeouts, or the feature being opt-in).
- [ ] 1.7 Drain the 26 open Dependabot PRs. 25 report every check green; they
      can go in as a batch behind the full workspace gate. Four must be read
      individually:
      - **#401 `openraft` alpha.30 → alpha.33 — FAILURE ×4.** The only red PR,
        on the replication/consensus dependency. Read the failure before
        choosing to fix forward or close and pin.
      - **#421 `js-yaml` 4.3.0 → 5.3.0.** The advisory is fixed in **4.3.1**;
        this jumps a major version to fix it. Merging it because it is green
        and labelled security is how an unrelated breaking change lands under
        a security banner. Take 4.3.1 now and schedule the major separately,
        or take 5.3.0 deliberately and absorb the migration.
      - **#393 `base64` 0.22.1 → 0.23.1** and **#412 `typescript` 6.0.3 →
        7.0.2** — both cross a major boundary.
      **Done when:** every PR is merged or closed with a stated reason, and
      the workspace gate is green afterwards.
- [ ] 1.8 The two direct dependencies that are unmaintained. Not
      vulnerabilities, but this is how a dependency becomes one with nobody
      watching, and unlike the transitive ones these are ours to decide.
      - `rustls-pemfile` 2.2.0 (RUSTSEC-2025-0134) — direct in `vectorizer`
        and `vectorizer-server`. Upstream folded it into `rustls-pki-types`,
        so this is a migration, not a bump.
      - `bincode` 2.0.1 (RUSTSEC-2025-0141) — direct in all three crates and
        on the **persistence path**, which makes replacing it a format
        question, not just an API one. `bincode` 1.3.3 also arrives via
        `hnsw_rs`.
      **Done when:** each has a decision — migrate now, or an `audit.toml`
      entry saying why not and what would change that.

## 2. Tail (docs + tests — check or waive with tailWaiver)
- [ ] 2.1 Update or create documentation covering the implementation: record
      the audit workflow and the threshold choices in
      `docs/development/security.md`, and cross-link
      `docs/analysis/dependency-security-2026-08/`. State plainly that
      Dependabot is a floor rather than a ceiling — it reads the GitHub
      Advisory Database, `cargo audit` reads RustSec, and `pnpm audit` found
      advisories in *watched* directories that Dependabot had not raised.
      Reading only the Dependabot page understated this surface by roughly an
      order of magnitude.
- [ ] 2.2 Write tests covering the new behavior. The unit under test here is
      the **gate**, not the dependencies: assert the CI audit step fails on a
      known-vulnerable lockfile and passes on a clean one, so a future change
      that silently disables it is caught. An audit gate that cannot fail is
      the `audit.toml` problem again in a new place.
- [ ] 2.3 Run tests and confirm they pass: the full workspace gate
      (`cargo nextest run --workspace --lib --bins --tests`, clippy, fmt),
      `cargo audit`, and `pnpm audit` in all three JavaScript projects.
