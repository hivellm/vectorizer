# Proposal: phase10_optimize-docker-build-time

## Why

Cold local Docker build of `hivehub/vectorizer:3.2.0` measured at **30–45 minutes** on a 33 GB Docker Desktop allocation (Windows host, multi-arch `linux/amd64,linux/arm64`). Breakdown observed during the v3.2.0 push session on 2026-05-01:

- `cargo chef cook` (deps warm): **10m 23s**
- `vectorizer` lib + `vectorizer-server` workspace compile: **3–4m amd64 / 5–6m arm64** (per arch)
- `cargo install cargo-sbom` + `cargo sbom > vectorizer.spdx.json`: **~1m + ~1m per arch** (recompiles `cargo-sbom` source twice — once per arch)
- `arm64` everything via QEMU emulation: 2–3× slower than native amd64
- BuildKit syft SBOM attestation generation: ~5s (already optimal)

Two earlier build attempts also OOMed at 8 GB Docker Desktop allocation (BuildKit container killed with `gRPC EOF` + `cannot allocate memory` SIGKILL) before the user bumped Docker Desktop to 33 GB. Peak rustc memory of 4–6 GB on the `vectorizer` lib monomorphization + `lto = "thin"` IR retention was the culprit.

The pain compounds on every release cut + every Scout / CVE re-bake. CI has no Docker Hub push job today (only `ghcr.io` via `release-artifacts.yml::publish-docker`), so every `hivehub/vectorizer:*` push is local — the slow cold build is on the maintainer's wall clock. We also have no buildx registry cache, so any unrelated change to `Cargo.lock` or workspace `Cargo.toml` invalidates the chef cook layer and re-pays the 10m deps cost.

## What Changes

### 1. Buildx registry cache (largest win)

Add `cache-from / cache-to type=registry,ref=hivehub/vectorizer-cache:buildx` to both [`scripts/docker/build-push.ps1`](../../../scripts/docker/build-push.ps1) and the `.github/workflows/release-artifacts.yml::publish-docker` job. First build seeds the cache layer; every subsequent build reuses unchanged layers.

- Expected: cold 30–45m → warm 5–10m for incremental crate changes; ~2–3m for non-Rust-touching changes (dashboard tweaks, Dockerfile env tweaks, etc.).
- Storage cost: ~3–5 GB on Docker Hub under a sibling repo `hivehub/vectorizer-cache`. Free tier handles this.

### 2. Native arm64 builder (eliminate QEMU)

Stop emulating arm64 on amd64 host. Two paths:

- **CI**: bump `publish-docker` job in `release-artifacts.yml` to a build matrix — `runs-on: ubuntu-24.04` for amd64 + `runs-on: ubuntu-24.04-arm` for arm64 — then assemble the manifest list with a final `docker buildx imagetools create`. GHA arm64 runners are GA + free for public repos.
- **Local**: document a Buildjet / Cirrun / self-hosted M-series Mac alternative for maintainers who must push from their workstation.

Expected: arm64 6m → 2m per build (matches amd64 native time).

### 3. Drop `cargo install cargo-sbom` from the Dockerfile

The `RUN xx-cargo install cargo-sbom && cargo sbom > vectorizer.spdx.json` step at [`Dockerfile`](../../../Dockerfile) line ~252 compiles `cargo-sbom` from source on every build, **once per arch**. The resulting SPDX file at `/vectorizer/vectorizer.spdx.json` is also redundant with the BuildKit syft attestation that `--sbom=true` already generates and attaches to the manifest list (verified on the v3.2.0 push: `docker buildx imagetools inspect` shows two attestation manifests, one per arch, containing the syft SBOM).

Two options:

- **Option A — Drop entirely.** Remove the `RUN xx-cargo install cargo-sbom ...` step + the `COPY --from=builder /vectorizer/vectorizer.spdx.json` in the runtime stage. Rely on BuildKit syft attestation for SBOM (Scout reads from there). Saves ~2m × N arches.
- **Option B — Pre-bake into a sidecar stage.** New stage `FROM rust:1.90-bookworm AS sbom-tool` that does `cargo install cargo-sbom`, then `COPY --from=sbom-tool /usr/local/cargo/bin/cargo-sbom /usr/local/bin/`. cargo-sbom is a host-arch tool (it just reads `Cargo.lock` + queries crates.io), so we never need an arm64 build of it.

Recommend Option A unless we discover a downstream consumer reads `/vectorizer/vectorizer.spdx.json` from inside the image (audit shows none in this repo).

### 4. `[profile.release-docker]` with `lto = false`

Workspace `[profile.release]` in [`Cargo.toml`](../../../Cargo.toml) uses `lto = "thin"` + `codegen-units = 4` for runtime perf. LTO retains all crate IR through linking, doubling peak rustc RAM and adding ~30% to workspace compile time.

Add a sibling profile that the Dockerfile selects:

```toml
[profile.release-docker]
inherits = "release"
lto = false
codegen-units = 16  # max parallelism, container build is throwaway
```

Pass `--profile release-docker` in the Dockerfile `xx-cargo build` line. Expected: 4m amd64 → 2–3m. Cost: shipped binary 10–15% slower on hot paths (acceptable in a container — operators who need peak perf already build from source with the workspace `release` profile).

### 5. Pre-baked dashboard artifact (CI only)

The Dockerfile currently runs `node:20-bookworm` + `pnpm install` + `tsc` + `vite build` as a `dashboard-builder` stage (~2m, already parallel with cargo). For CI builds we already have a `build-dashboard` job in `release-artifacts.yml` that uploads `dashboard-dist` as an artifact. Switch the CI Docker build to `Dockerfile.artifacts` (already exists at `docker/Dockerfile.artifacts`) which `COPY`s the prebuilt `dashboard/dist` instead of building it.

For local builds keep the current Dockerfile (no change for maintainers without CI artifact access).

### 6. sccache + S3/GCS shared cache (deferred — optional)

Hook `sccache` into the cargo invocations with `RUSTC_WRAPPER=sccache`. Backend on a shared S3 bucket gives cross-machine artifact reuse. Setup overhead is non-trivial (IAM, bucket lifecycle, secret management) — defer to a follow-up task once §1–§5 land and we measure the residual gap.

## Impact

- **Affected specs**: none new; touches the build pipeline only. Optional new spec at `.rulebook/specs/build/spec.md` documenting the cache repo + the dual-profile contract.
- **Affected code**:
  - [`Dockerfile`](../../../Dockerfile) — add `--profile release-docker` selection, drop `cargo install cargo-sbom` step, drop SPDX file COPY in runtime stage.
  - [`Cargo.toml`](../../../Cargo.toml) — add `[profile.release-docker]` block.
  - [`scripts/docker/build-push.ps1`](../../../scripts/docker/build-push.ps1) — add `cache-from` / `cache-to` flags.
  - [`scripts/docker/build.ps1`](../../../scripts/docker/build.ps1) — same.
  - [`scripts/docker/push.ps1`](../../../scripts/docker/push.ps1) — same.
  - [`.github/workflows/release-artifacts.yml`](../../../.github/workflows/release-artifacts.yml) — add Docker Hub login + push step alongside the existing `ghcr.io` push; add arm64 native runner matrix; switch to `cache-from/to type=gha,scope=docker-hub` (or registry cache).
- **Affected docs**:
  - [`docker/dockerhub-readme.md`](../../../docker/dockerhub-readme.md) — note the build-time profile shipped in the image (so consumers know the binary is `release-docker`, not `release`).
  - New `docs/development/docker-builds.md` — operator runbook for the cache repo + the native arm64 path.
- **Breaking change**: NO. Every change is additive: new cache layer (rebuilds work without it), new profile (workspace `release` profile unchanged), CI workflow gains a job (existing job stays).
- **User benefit**:
  - Maintainer wall-clock per release cut: 30–45m → 5–10m on warm cache, 15–20m on first cold build after §1–§4.
  - CI Docker Hub publish parity (no more local-only push step).
  - Native arm64 in CI removes QEMU as an intermittent failure surface.
  - Smaller Dockerfile footprint (no cargo-sbom recompile, no SPDX file write).

## Verification

- Measure cold + warm build times before §1–§5 land: capture in `docs/development/docker-builds.md` baseline section.
- After §1: rerun `scripts/docker/build-push.ps1 -Tag X` on an unrelated single-line dashboard change; expect <10m wall.
- After §2: GHA matrix run end-to-end on a release tag; confirm both arch jobs finish without QEMU traces in the log.
- After §3: `docker buildx imagetools inspect hivehub/vectorizer:X` still shows 2 attestation manifests (one per arch); `docker scout policy ... --org hivehub` still shows `Supply chain attestations: 0 deviations` (parity with the v3.2.0 baseline).
- After §4: shipped binary passes the existing `tests/api/rest/` suite at parity with the workspace-`release` build (no behavior delta).
- After §5: Dockerfile builder stage no longer pulls `node:20-bookworm`; final image digest stays identical to the §1–§4 build.

## Out of scope

- Switching base image away from `dhi.io/debian-base:trixie` — Scout-compliance gate already met.
- Migrating to `cargo-zigbuild` for cross-compilation — explored in earlier sprints, rejected because of TLS / glibc compat.
- Replacing `cargo-chef` with native cargo `--target-dir` caching — chef still wins for the dep-warming pattern even with registry cache.
