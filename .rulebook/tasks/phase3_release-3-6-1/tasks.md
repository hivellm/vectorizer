## 1. Implementation

Runs only after phase1 and phase2 are both archived — this task exists to
publish them.

- [x] 1.1 Bump the five workspace crates to 3.6.1
      (`crates/vectorizer{,-server,-core,-grpc,-cli}/Cargo.toml`) and
      regenerate `Cargo.lock` with `cargo check` rather than editing it.
      **Done when:** `cargo check --workspace` is clean and the lockfile diff
      touches only the workspace crates' own entries.
      Clean; lockfile diff is 6 lines, all workspace entries. The crates carry
      no cross-version references (path deps only), so line 3 of each manifest
      was the whole job.
- [x] 1.2 Bump the SDK manifests: `sdks/rust/Cargo.toml`,
      `sdks/typescript/package.json`, `sdks/python/pyproject.toml` +
      `sdks/python/__init__.py` (`__version__`), `sdks/csharp/Vectorizer.csproj`
      and `sdks/csharp/src/Vectorizer.Rpc/Vectorizer.Rpc.csproj`. Leave the
      three non-carriers alone — `tailwind-merge: ^3.6.0` in
      `dashboard/package.json` and the prose in `sdk-python-test.yml` /
      `sdk-publish-typescript.yml`.
      **Done when:** every SDK manifest reads 3.6.1 and
      `grep -rn "3\.6\.0"` returns only the three known non-carriers plus
      `gui/package.json` (1.3) and history files.
      **Correction — the Go SDK does carry a version.** This task and 1.6 both
      claimed Go publishes by tag with no manifest. It has
      `sdks/go/version.go:4` (`const Version = "3.6.0"`), which the client
      reports at runtime; following the plan as written would have shipped a
      Go SDK announcing the wrong version. Bumped. Twelve carriers total, not
      the six listed here.
      Left alone as intended: the three known non-carriers, plus doc-comment
      examples (`sdks/rust/src/rpc/mod.rs:15`, `client.rs:187`,
      `sdks/python/README.md:212`) which are illustrative prose, and the
      `## [3.6.0]` headings in the per-SDK CHANGELOGs.
- [ ] 1.3 Point the GUI at the new SDK (`gui/package.json`:
      `@hivehub/vectorizer-sdk` → `^3.6.1`). The caret already admits 3.6.1,
      so this is for legibility; do it after npm has the package or the
      lockfile update will fail.
      **Done when:** `pnpm install` resolves and `pnpm type-check` still
      passes.
      **Blocked on 1.6** — npm has to serve 3.6.1 before `pnpm install` can
      resolve it. Sequenced after the publish, not skipped.
- [x] 1.4 Write the CHANGELOG entry for 3.6.1: both fixes, each naming the
      user-visible symptom rather than the patch — a partial collection list
      during warm-up that looked like data loss, and a `vectorizer://` URL
      handed to the REST client failing with an opaque reqwest error.
      **Done when:** the entry lists both issues with their numbers.
      Three fixes, not two: the arm64 `-fastembed` image that exited 127 had
      no entry either.
      **Found en route — 3.6.0 had no CHANGELOG section at all.** The file
      went straight from a 366-line `[Unreleased]` to `## [3.5.0]`, so
      everything 3.6.0 shipped to four registries was still labelled
      unreleased. Renaming `[Unreleased]` to `[3.6.1]` — the obvious move —
      would have stamped 3.6.0's content as 3.6.1. Found the real boundary
      instead: the `v3.6.0` tag points at `4ca7724e`, and
      `git diff 4ca7724e..HEAD -- CHANGELOG.md` shows only two commits touched
      the file since, adding two blocks at the top. Everything below them is
      3.6.0, now under a backfilled `## [3.6.0] - 2026-08-04` heading that
      says so and names the boundary commit.
      The arm64 entry records a subtlety worth keeping: the `3.6.0` images
      published 2026-08-05 already carry that fix (built from HEAD), while the
      source-tagged `v3.6.0` does not.
- [x] 1.5 Full quality gate before tagging: `cargo nextest run --workspace
      --lib --bins --tests`, clippy, fmt, and the SDK suites for the languages
      whose manifests moved.
      **Done when:** all green, with the numbers recorded in the task.
      **2037 passed, 0 failed, 9 skipped**; clippy exit 0; `fmt --check` clean.
      The Rust SDK is a workspace member so it is inside that number; the
      TypeScript / Python / C# / Go manifests changed a version string only,
      with no code touched.
- [ ] 1.6 Tag `v3.6.1` and confirm the SDK publish workflows land on
      crates.io, npm, PyPI and NuGet — all five SDKs in lockstep, which the
      3.6.0 cut did not achieve. Tag the `sdks/go` submodule separately (Go
      resolves a module version from its tag; the `version.go` constant bumped
      in 1.2 is what the client *reports*, and the two must agree).
      **Done when:** each registry serves 3.6.1.
      **Blocked on the user.** This shell has no SSH key, so it cannot push
      the commits or the tag. Tagging before the commits reach GitHub would
      publish a 3.6.1 that does not contain the fixes. The remote is still at
      `ead8c55b`; everything from `33fd22be` onward is local only.
- [ ] 1.7 Publish both Docker variants **manually** —
      `.\scripts\docker\build-push.ps1 -Tag 3.6.1` then the same with
      `-Fastembed`. CI does not do this: `release-artifacts.yml` has no Docker
      job. Boot-test each published tag on **both** architectures before
      calling it done, and scan with `docker scout` + the VEX. The arm64
      fastembed image was dead for two releases precisely because a green
      build was mistaken for a working image.
      **Done when:** all four image/arch combinations boot and report 3.6.1,
      `latest` points at the default variant, and both scans are clean.
      Not blocked by the push — the docker.io login is live and
      `build-push.ps1` reads the local tree. Sequenced after 1.5 so the image
      reports 3.6.1, and worth confirming with the user first: it publishes
      publicly from commits that are not yet on GitHub, the same situation the
      3.6.0 images were built under.

## 2. Tail (docs + tests — check or waive with tailWaiver)
- [x] 2.1 Update or create documentation covering the implementation: README
      / install snippets carrying a pinned version, and the Docker Hub image
      documentation if it names a version.
      The sweep found worse drift than expected — the install snippets were
      stale by whole major lines, not by one patch: C# and TypeScript both
      told users to install **2.2.0**, the Rust README carried three different
      pins (`3.5`, `3.0`, `2.2.0`) and Python said `==3.5.0`. All now point at
      3.6.x; the Rust ones use a caret (`"3.6"`) so a patch release no longer
      makes them wrong.
      `deploy/docker/dockerhub-readme.md`: 6 run/compose examples moved to
      3.6.1, a 3.6.1 highlights block added above the 3.5.0 one, and the tag
      table now carries 3.6.1 / 3.6.1-fastembed / 3.6.0 with explicit ⚠️ on
      the arm64-broken fastembed tags, so someone browsing Docker Hub can see
      which variant to avoid. `latest` documented as always the slim variant.
      **Note:** nothing publishes this file to Docker Hub any more — the docs
      claim it ships "on every release", but that was the CI job removed along
      with the rest. Updating it here is necessary and not sufficient.
- [ ] 2.2 Write tests covering the new behavior — waive if the diff is
      manifests only; the behavior under test belongs to phase1 and phase2,
      and a test asserting a version string pins nothing worth pinning.
      Candidate for `tailWaiver` on archive: the whole diff is manifests,
      CHANGELOG and docs. Deliberate exception if one is added — a test
      asserting every carrier agrees would have caught the Go constant this
      task's plan missed, and would keep catching it. Decide at archive time.
- [ ] 2.3 Run tests and confirm they pass (covered by 1.5, re-run after the
      manifest bump so the gate sees the final tree).
