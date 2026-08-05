## 1. Implementation

Runs only after phase1 and phase2 are both archived — this task exists to
publish them.

- [ ] 1.1 Bump the five workspace crates to 3.6.1
      (`crates/vectorizer{,-server,-core,-grpc,-cli}/Cargo.toml`) and
      regenerate `Cargo.lock` with `cargo check` rather than editing it.
      **Done when:** `cargo check --workspace` is clean and the lockfile diff
      touches only the workspace crates' own entries.
- [ ] 1.2 Bump the SDK manifests: `sdks/rust/Cargo.toml`,
      `sdks/typescript/package.json`, `sdks/python/pyproject.toml` +
      `sdks/python/__init__.py` (`__version__`), `sdks/csharp/Vectorizer.csproj`
      and `sdks/csharp/src/Vectorizer.Rpc/Vectorizer.Rpc.csproj`. Leave the
      three non-carriers alone — `tailwind-merge: ^3.6.0` in
      `dashboard/package.json` and the prose in `sdk-python-test.yml` /
      `sdk-publish-typescript.yml`.
      **Done when:** every SDK manifest reads 3.6.1 and
      `grep -rn "3\.6\.0"` returns only the three known non-carriers plus
      `gui/package.json` (1.3) and history files.
- [ ] 1.3 Point the GUI at the new SDK (`gui/package.json`:
      `@hivehub/vectorizer-sdk` → `^3.6.1`). The caret already admits 3.6.1,
      so this is for legibility; do it after npm has the package or the
      lockfile update will fail.
      **Done when:** `pnpm install` resolves and `pnpm type-check` still
      passes.
- [ ] 1.4 Write the CHANGELOG entry for 3.6.1: both fixes, each naming the
      user-visible symptom rather than the patch — a partial collection list
      during warm-up that looked like data loss, and a `vectorizer://` URL
      handed to the REST client failing with an opaque reqwest error.
      **Done when:** the entry lists both issues with their numbers.
- [ ] 1.5 Full quality gate before tagging: `cargo nextest run --workspace
      --lib --bins --tests`, clippy, fmt, and the SDK suites for the languages
      whose manifests moved.
      **Done when:** all green, with the numbers recorded in the task.
- [ ] 1.6 Tag `v3.6.1` and confirm the SDK publish workflows land on
      crates.io, npm, PyPI and NuGet — all five SDKs in lockstep, which the
      3.6.0 cut did not achieve. Tag the `sdks/go` submodule separately, since
      Go publishes by tag with no manifest.
      **Done when:** each registry serves 3.6.1.
- [ ] 1.7 Publish both Docker variants **manually** —
      `.\scripts\docker\build-push.ps1 -Tag 3.6.1` then the same with
      `-Fastembed`. CI does not do this: `release-artifacts.yml` has no Docker
      job. Boot-test each published tag on **both** architectures before
      calling it done, and scan with `docker scout` + the VEX. The arm64
      fastembed image was dead for two releases precisely because a green
      build was mistaken for a working image.
      **Done when:** all four image/arch combinations boot and report 3.6.1,
      `latest` points at the default variant, and both scans are clean.

## 2. Tail (docs + tests — check or waive with tailWaiver)
- [ ] 2.1 Update or create documentation covering the implementation: README
      / install snippets carrying a pinned version, and the Docker Hub image
      documentation if it names a version.
- [ ] 2.2 Write tests covering the new behavior — waive if the diff is
      manifests only; the behavior under test belongs to phase1 and phase2,
      and a test asserting a version string pins nothing worth pinning.
- [ ] 2.3 Run tests and confirm they pass (covered by 1.5, re-run after the
      manifest bump so the gate sees the final tree).
