## 1. Implementation

Full findings in `docs/analysis/dependency-security-2026-08/`.

Ordered so the thing that let this accumulate is fixed before the backlog it
produced. Draining advisories first would leave nothing watching for the next
batch — which is exactly how this one arrived unannounced.

- [x] 1.1 Wire dependency auditing into CI. There are 17 workflows covering
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
      `.github/workflows/dependency-audit.yml`. Two jobs, neither compiles the
      workspace — `cargo audit` reads `Cargo.lock` and `pnpm audit` reads the
      pnpm lockfiles, so it finishes in under a minute and has no excuse to be
      skipped. Also runs weekly on a schedule: an advisory can land against an
      unchanged lockfile, so pushes alone would never surface it.
      The npm threshold is `--prod`, chosen by measurement rather than taste.
      Measured: `--prod` yields 1 finding in `gui` and **0** in `dashboard` and
      `sdks/typescript`, against 6 unscoped. Every unscoped one reaches only
      vite / vitest / eslint / postcss, so a gate failing on those would be red
      constantly over packages that cannot reach a user — and would be switched
      off within a week. Dev advisories are still printed, just not fatal.
      Warnings (unmaintained, unsound, yanked) likewise report without failing;
      they are tracked as 1.8 instead.
- [x] 1.2 Confirm `audit.toml` is actually effective. It carries
      `[advisories.unmaintained] warn = false`, yet the local run reported
      `10 allowed warnings` while still listing 6 unmaintained crates.
      Whether that section is honoured under cargo-audit 0.22's schema is
      unverified — and unverifiable while nothing runs the tool.
      **Done when:** the file's behaviour is demonstrated rather than assumed;
      any inert section is corrected or removed, so it stops reading as a
      decision that was never in force.
      **It was worse than inert — it was doubly broken.** Demonstrated by
      running `cargo audit` with the file and with it deleted: byte-identical
      output, 9 vulnerabilities and 10 warnings either way, with
      `RUSTSEC-2024-0436` listed both times despite being in `ignore`.
      Cause: cargo-audit reads `.cargo/audit.toml`, never a file at the
      repository root. Moved there, it turned out the schema was invalid too —
      `[advisories.unmaintained]` is not a field cargo-audit accepts, and it
      rejects the file *fatally*. Being unread had hidden that for as long as
      the file existed.
      The nasty part: a parse error makes `cargo audit` exit **1**, which is
      indistinguishable from "vulnerabilities found" to a CI step that only
      reads the exit code. A broken policy would have been misread as a
      failing audit. The workflow therefore checks that the config *parses*,
      not merely that it exists — verified by sabotage, appending a bogus
      section makes the gate fire.
      Now honoured, proven by a measurable difference: warnings dropped 10 to
      9 as the `paste` ignore finally took effect. The `[advisories.unmaintained]`
      suppression was deliberately not carried over — hiding unmaintained
      crates contradicts 1.8, where two of ours are direct dependencies.
- [x] 1.3 Fix the two Rust advisories that are fixable today. Everything else
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
      `h2` 0.4.15 → **0.4.19**, `lru` 0.18.1 → **0.18.3**, lockfile only.
      Vulnerabilities 9 → 8; `h2` is gone outright.
      **The `lru` check was worth making.** After the bump `lru` still appeared
      in the report, which reads like the fix failing. It is a different copy:
      **0.16.4, via `tantivy`**, which belongs to the blocked-upstream set in
      1.6. Our direct 0.18.x is clear, and the remaining row moved from
      *vulnerability* to *unsound warning*, so it no longer fails the gate.
      Taking "still listed" at face value would have reverted a working fix.
      Full workspace gate after both: **2057 passed, 9 skipped.**
- [x] 1.4 Clear the npm advisories. All six are dev- or build-scoped, so none
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
      Fixed through `pnpm.overrides`, the convention this repo already uses for
      exactly this (30 entries in `gui`, 22 in `dashboard`, 10 in the SDK):
      `nanoid@<3.3.18 -> >=3.3.18 <4`, `brace-expansion@>=5.0.0 <5.0.9 ->
      >=5.0.9 <6` (superseding a now-insufficient `<5.0.7` entry), and the
      SDK's direct `js-yaml` devDependency `^4.3.0 -> ^4.3.1`. All three
      projects audit **clean**, dev-scoped included — better than the `--prod`
      gate requires.
      **A mistake worth recording, because it is the one this task criticises
      PR #421 for.** The overrides were first written unbounded (`>=3.3.18`),
      and pnpm resolved `nanoid` to **6.0.1** — three majors past what
      `postcss` declares. Fixing a patch-level advisory by importing a major
      version is the same error, and the repo's own
      `js-yaml@<4.3.0 -> ">=4.3.0 <5"` was already showing the right shape.
      Bounded to `<4`, it resolves to exactly 3.3.18.
      **The `gui` bundle question, answered by measurement:** `nanoid` does
      **not** ship. Built with `pnpm build:vite` and searched — 0 occurrences
      across `dist/`. The manifest classifies it production (via `@vueuse/core`
      → `vue` → `@vue/compiler-sfc` → `postcss`); its real role is build-time
      CSS processing and it is tree-shaken out. `js-yaml` *does* ship
      (`dist/assets/js-yaml-*.js`) and resolves to the patched 4.3.1.
- [x] 1.5 Decide, in writing, the advisories that have no fix — then record
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
      Both recorded in `.cargo/audit.toml`. Vulnerabilities 8 → 6.
      **`rsa` — the vulnerable path is never entered.** `jsonwebtoken` has
      exactly ONE call site in the workspace,
      `crates/vectorizer/src/auth/jwt.rs`, and it is symmetric throughout:
      `Header::new(Algorithm::HS256)`, `EncodingKey::from_secret`,
      `DecodingKey::from_secret`. A workspace search for `Algorithm::RS*`,
      `PS*`, `ES*`, `from_rsa_*` and `RsaPrivateKey` returns nothing — no RSA
      key is ever constructed. `Validation::new(Algorithm::HS256)` also pins
      the accepted algorithm on verification, so the classic
      algorithm-confusion route into that code is closed too. The entry says
      to remove itself if asymmetric JWT support is ever added.
      **`rkyv` — not compiled.** `rust_decimal` 1.42.1 is the only package
      listing it, behind an optional feature nothing enables; `cargo tree -i
      rkyv` prints nothing with or without `--all-features`. It appears at all
      only because `cargo audit` reads the lockfile rather than the build
      graph.
- [x] 1.6 The document-parser advisories — the ones that matter most and that
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
      **The versions moved — no mitigation needed, and no upstream PR either.**
      Option 1 was the whole answer: `transmutation` is ours, we pinned
      `"0.3.1"` and were resolving 0.3.3, and 0.3.5 was already published.
      `cargo update -p transmutation` moved 60 packages and carried the parsers
      with it — `lopdf` 0.34.0/0.35.0 → **0.42.0/0.44.0** (both ≥ 0.42.0) and
      `quick-xml` 0.37.5 → **0.41.0**, along with `pdf-extract` 0.8.2 → 0.12.0
      and `umya-spreadsheet` 2.3.3 → 3.1.0. Six vulnerabilities to two.
      The last two were `quick-xml` 0.36.2 via `docx-rs` 0.4.20;
      `cargo update -p docx-rs` → 0.4.22 cleared them.
      **`cargo audit` now exits 0 — zero vulnerabilities, from nine.** Full
      workspace gate after a 60-package update: 2057 passed, 9 skipped, clippy
      clean.
      Worth noting for next time: every one of these was reported as "blocked
      upstream" by `cargo update -p <child> --dry-run`, and that was true — the
      child could not move. Updating the *parent* was never blocked. The
      dry-run answers a narrower question than it appears to.
- [x] 1.7 Drain the 26 open Dependabot PRs. 25 report every check green; they
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
      **Applied locally rather than merged, so the PRs become redundant
      instead of needing 26 merge commits and 26 CI runs.** Closing them on
      GitHub is a remote write and is the maintainer's to do; Dependabot
      auto-closes a bump whose version is already in the base branch, so most
      should close themselves once this lands.
      Routine Rust bumps taken: `blake3` 1.8.5 → 1.8.7, `xxhash-rust` 0.8.16 →
      0.8.18, `fastrand` 2.4.1 → 2.5.0, `serde_json` 1.0.150 → 1.0.151,
      `rustls` 0.23.42 → 0.23.43, `bcrypt` 0.19.2 → 0.19.3, `hyper` 1.10.1 →
      1.11.1.
      `fastembed` 5.17.3 → 5.17.4 needed `--precise`: a plain
      `cargo update -p fastembed` reported `Locking 0 packages` because the
      move also requires `ort` rc.12 → rc.13, and cargo will not advance a
      pre-release on its own. The ONNX runtime bindings changing is the real
      content of that bump, not fastembed's own patch number.
      **`base64` 0.22 → 0.23 (#393), the major, was taken and verified rather
      than waved through.** It is used in `hub/request_signing.rs` and
      `security/payload_encryption.rs` — both security paths — so "it compiles"
      would not have been evidence. 17 signing/encryption tests ran and passed
      inside the full suite.
      **#401 `openraft` is closed by policy, not fixed forward.** The pins are
      `=0.10.0-alpha.30` in both crates, and the comment above them states the
      condition for lifting: *a stable 0.10 or 0.11*, both bumped together,
      with `tests/integration/cluster_ha.rs` retested. Upstream is still on
      alphas (latest alpha.34), so the condition is unmet and this PR proposes
      exactly the silent drift the pin exists to prevent.
      Its CI failure is a second, independent reason: Dependabot bumped
      `openraft` and left `openraft-memstore` at alpha.30, which implements
      `RaftStateMachine` against the older trait shape —
      `begin_receiving_snapshot` removed, `SnapshotMeta::snapshot_id` gone.
      The two only move in lockstep.
      Both crates are now in `.github/dependabot.yml`'s cargo ignore list so
      the PR does not return weekly. Ignored wholesale rather than by
      update-type, because every 0.10 release is a pre-release and a "patch"
      bump here is still the drift the pin forbids.
      **#412 `typescript` 6.0.3 → 7.0.2** in `/gui` is left for the
      maintainer: it is a dev-tooling major with no security content, and it
      belongs with a `vue-tsc` compatibility check rather than in a security
      branch.
      Gate after all of the above: **2057 passed, 9 skipped**, clippy clean.
- [x] 1.8 The two direct dependencies that are unmaintained. Not
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
      **`rustls-pemfile`: migrated and removed.** Upstream folded PEM parsing
      into `rustls-pki-types`, which rustls already re-exports, so the
      replacement was in the tree the whole time. One call site
      (`security/tls.rs`), two functions, now `CertificateDer::pem_file_iter`
      and `PrivateKeyDer::from_pem_file` — which also removed the `File` /
      `BufReader` plumbing. Gone from `Cargo.lock` entirely; 16 TLS tests pass.
      A free find along the way: **`vectorizer-server` declared it and never
      used it.** An unused direct dependency, dropped with the migration.
      One behaviour note: `from_pem_file` folds "file missing", "unparseable"
      and "no key in the file" into one error, where `rustls_pemfile::private_key`
      returned `Ok(None)` for the last. Not worth preserving — all three mean
      the same thing to an operator, and the message names the path.
      **`bincode`: kept, deliberately, and NOT silenced.** It is a maintenance
      advisory, not a vulnerability, and `cargo audit` reports it as a warning
      that does not fail the gate. Replacing it is not a library swap: it is
      the on-disk format for `.vecdb` (`persistence/`, `storage/reader.rs`)
      and the on-wire format for the replication log
      (`replication/durable_log.rs`), across 9 files and 16 call sites. A
      different encoder means every existing archive and replication log stops
      loading, so the real work is a format version bump plus a dual-format
      reader — a data migration, which does not belong in a dependency pass.
      Recorded in `vectorizer-core/src/codec.rs`, where anyone touching
      serialization will meet it, rather than in `audit.toml`: an `ignore`
      entry would hide the reminder, turning a decision we should revisit into
      one nobody sees. That module is also the seam that makes the migration
      tractable — every call site already goes through it.
      Corrected while there: the codec's doc comment claimed "bincode v3"; the
      dependency is v2.
      **Final state: `cargo audit` exits 0.** Warnings 10 → 6, all transitive
      or deliberate (`bincode` ×2, `ttf-parser`, `lru` 0.16.4 via tantivy, two
      yanked crates). `rustls-pemfile` and `fxhash` are gone outright.

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
