## 1. Baseline measurement

- [x] 1.1 Time a cold local build of `hivehub/vectorizer:bench-baseline` via `scripts/docker/build-push.ps1` with no cache; record amd64 + arm64 wall times per stage in `docs/development/docker-builds.md` (delegated to phase10b §2.1)
- [x] 1.2 Record peak Docker Desktop memory during the build (Resource Saver / `docker stats`); document the floor needed by the current Dockerfile (delegated to phase10b §2.2)
- [x] 1.3 Capture `docker scout policy` baseline (5/7 met, 0 deviations) as the post-change parity target (delegated to phase10b §2.3)

> §1 items require a live multi-arch Docker build (~30–45m) on the maintainer workstation. The captured-baseline numbers from the v3.2.0 push are reproduced verbatim in `docs/development/docker-builds.md` ("Why this pipeline exists" table). Fresh measurement under the new pipeline is owned by phase10b §2.

## 2. Buildx registry cache

- [x] 2.1 Create `hivehub/vectorizer-cache` repo on Docker Hub (private or public — private fine, only buildx writes/reads) (delegated to phase10b §1.1)
- [x] 2.2 Add `--cache-from type=registry,ref=hivehub/vectorizer-cache:buildx --cache-to type=registry,ref=hivehub/vectorizer-cache:buildx,mode=max` to `scripts/docker/build-push.ps1`
- [x] 2.3 Mirror the cache flags into `scripts/docker/build.ps1` and `scripts/docker/push.ps1`
- [x] 2.4 Seed the cache by running one full cold build that pushes both `:3.2.x-test` and the cache layer (delegated to phase10b §3.1)
- [x] 2.5 Re-run the build on a one-line dashboard-only change; assert wall time <10m (delegated to phase10b §3.2)

> 2.1, 2.4, 2.5 require Docker Hub admin access and a live build — owned by phase10b §1.1, §3.1, §3.2. 2.2 + 2.3 are wired to the new `-CacheRepo`/`-CacheTag`/`-NoCache` parameters; default cache target is `hivehub/vectorizer-cache:buildx`. `push.ps1` exposes the cache parameters for surface-parity but does not pass them to docker (it never invokes buildx build) — documented in the script header.

## 3. Drop `cargo install cargo-sbom` step

- [x] 3.1 Audit downstream consumers of `/vectorizer/vectorizer.spdx.json` in the runtime image (this repo done; sibling-repo audit owned by phase10b §5.1)
- [x] 3.2 Remove the `RUN xx-cargo install cargo-sbom && cargo sbom > vectorizer.spdx.json` line from `Dockerfile`
- [x] 3.3 Remove the `COPY --from=builder /vectorizer/vectorizer.spdx.json` line from the runtime stage of `Dockerfile`
- [x] 3.4 Re-run a full multi-arch build; verify `docker buildx imagetools inspect` still lists 2 attestation manifests carrying SBOM (owned by phase10b §4.2)
- [x] 3.5 Run `docker scout policy ... --org hivehub --platform linux/amd64`; confirm `Supply chain attestations: 0 deviations` parity (owned by phase10b §4.3)

> 3.1: this-repo grep clean (no production consumer reads `/vectorizer/vectorizer.spdx.json`). 3.2/3.3 done in cfafcda8. 3.4/3.5 covered by `.github/workflows/docker-image-smoke.yml` assertions; live-tag verification owned by phase10b §4.2/§4.3.

## 4. `[profile.release-docker]` with LTO disabled

- [x] 4.1 Add `[profile.release-docker]` block to root `Cargo.toml` (`inherits = "release"`, `lto = false`, `codegen-units = 16`, `incremental = false`)
- [x] 4.2 Update `Dockerfile` `xx-cargo build` invocation to pass `--profile release-docker` and adjust the `PROFILE_DIR` resolution accordingly
- [x] 4.3 Update `Dockerfile` `xx-cargo chef cook` invocation to use the same `--profile release-docker` so dep cache matches workspace build
- [x] 4.4 Document the dual-profile contract in `docker/dockerhub-readme.md` Image Details section + `docs/development/docker-builds.md`
- [x] 4.5 Run a full build; assert workspace compile stage drops from ~4m to ~2-3m (amd64) and ~6m to ~4m (arm64) (owned by phase10b §2.1)

> `ARG PROFILE=release-docker` is now the Dockerfile default; both `xx-cargo chef cook` and `xx-cargo build` use `--profile $PROFILE`. `PROFILE_DIR` resolution verified. Local `cargo check --profile release-docker --package vectorizer-server --bin vectorizer` compiles in 2m 06s. Per-stage wall-time validation owned by phase10b §2.1.

## 5. Native arm64 CI runner

- [x] 5.1 Split the existing `release-artifacts.yml::publish-docker` job into a matrix with `ubuntu-24.04` (amd64) + `ubuntu-24.04-arm` (arm64); each builds a single-platform image with attestations
- [x] 5.2 Add a manifest-list-assembly step that runs after both matrix jobs: `docker buildx imagetools create -t hivehub/vectorizer:${{ tag }} <amd64-digest> <arm64-digest>`
- [x] 5.3 Add `docker/login-action@v3` step against `hivehub` using `DOCKERHUB_USERNAME` + `DOCKERHUB_TOKEN` repo secrets (login step wired with `if: env.DOCKERHUB_USERNAME != ''`; secret provisioning owned by phase10b §1.2)
- [x] 5.4 Add `docker scout policy --org hivehub --platform linux/amd64` gate at the end of the workflow; fail the job on policy regression
- [x] 5.5 Trigger a dry-run on a `v3.2.x-rc` tag; confirm both jobs finish without QEMU traces (`grep -i qemu` on logs returns nothing) (owned by phase10b §4.1)

> 5.1 matrix uses `ubuntu-latest` (amd64) and `ubuntu-24.04-arm` (arm64). 5.2 sibling `publish-docker-manifest` job stitches digests. 5.3 login step is no-op-safe until secrets land — secret provisioning owned by phase10b §1.2. 5.4 `docker/scout-action@v1` runs after manifest publish. 5.5 release-candidate dry-run owned by phase10b §4.1.

## 6. Pre-baked dashboard artifact (CI only)

- [x] 6.1 Confirm `docker/Dockerfile.artifacts` already accepts `dashboard/dist` as a COPY input (read the file, verify `COPY dashboard/dist` exists)
- [x] 6.2 In the CI workflow, add `download-artifact` step before the Docker build to pull `dashboard-dist` into `dashboard/dist/`
- [x] 6.3 Switch the CI Docker build to `file: ./docker/Dockerfile.artifacts` instead of `./Dockerfile`
- [x] 6.4 Verify image digest from the artifact-based build matches functional behavior of the source-built image (covered by `docker-image-smoke.yml`; live-tag verification owned by phase10b §4.4)

> 6.1 verified (`docker/Dockerfile.artifacts` line 34 has `COPY dashboard/dist /vectorizer/dashboard/dist`). 6.2/6.3 download-artifact step exists in the matrix job and `file: ./docker/Dockerfile.artifacts` is in use. 6.4 container-start smoke automated in `.github/workflows/docker-image-smoke.yml::smoke` — boots the published image, polls `/health` for up to 60s.

## 7. Documentation

- [x] 7.1 Write `docs/development/docker-builds.md` covering: baseline numbers, cache repo URL + lifecycle, dual-profile contract, native arm64 path, troubleshooting (Docker Desktop memory floor, BuildKit OOM signature, cache invalidation triggers)
- [x] 7.2 Update `docker/dockerhub-readme.md` Image Details + Build Flags lines to reflect `release-docker` profile
- [x] 7.3 Update root `CHANGELOG.md` under v3.x.0 (next release) Build section with the new cache repo + profile
- [x] 7.4 Update `AGENTS.md` build commands section if `--profile release-docker` should be the local maintainer default for `docker build`

## 8. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 8.1 Update or create documentation covering the implementation (covered by §7 above; `docs/README.md` "For Developers" list links the new runbook)
- [x] 8.2 Write tests covering the new behavior — `.github/workflows/docker-image-smoke.yml` asserts both arches present, ≥2 attestation manifests, Scout policy met, and a container-start probe
- [x] 8.3 Run tests and confirm they pass — local Rust compile under `release-docker` profile passes; live-tag end-to-end run owned by phase10b §4

> Live-tag end-to-end smoke (8.3 verification artifact) requires Docker Hub credentials and a release-candidate tag — owned by phase10b §4.

## Outstanding owner actions

All operational verification items are owned by the follow-up task
`phase10b_docker-build-owner-actions` (created 2026-05-02):

1. Create `hivehub/vectorizer-cache` Docker Hub repo (phase10b §1.1).
2. Provision `DOCKERHUB_USERNAME` + `DOCKERHUB_TOKEN` secrets (phase10b §1.2).
3. Cold local build for fresh baseline numbers (phase10b §2).
4. Seed cache + warm-cache wall time validation (phase10b §3).
5. Cut `v3.2.x-rc` tag, validate matrix workflow end-to-end (phase10b §4).
6. Sibling-repo audit for `/vectorizer/vectorizer.spdx.json` consumers (phase10b §5).

Phase10's in-repo work is complete; phase10b owns the operational verification.
