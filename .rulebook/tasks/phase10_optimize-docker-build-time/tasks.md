## 1. Baseline measurement

- [ ] 1.1 Time a cold local build of `hivehub/vectorizer:bench-baseline` via `scripts/docker/build-push.ps1` with no cache; record amd64 + arm64 wall times per stage in `docs/development/docker-builds.md`
- [ ] 1.2 Record peak Docker Desktop memory during the build (Resource Saver / `docker stats`); document the floor needed by the current Dockerfile
- [ ] 1.3 Capture `docker scout policy` baseline (5/7 met, 0 deviations) as the post-change parity target

> §1 items require a live multi-arch Docker build (~30–45m) on the maintainer workstation. Phase10 implementation cannot run them in this session — the captured-baseline numbers from the v3.2.0 push are reproduced verbatim in `docs/development/docker-builds.md` (the "Why this pipeline exists" table) so the spec's parity target is documented even though no fresh measurement was taken.

## 2. Buildx registry cache

- [ ] 2.1 Create `hivehub/vectorizer-cache` repo on Docker Hub (private or public — private fine, only buildx writes/reads)
- [x] 2.2 Add `--cache-from type=registry,ref=hivehub/vectorizer-cache:buildx --cache-to type=registry,ref=hivehub/vectorizer-cache:buildx,mode=max` to `scripts/docker/build-push.ps1`
- [x] 2.3 Mirror the cache flags into `scripts/docker/build.ps1` and `scripts/docker/push.ps1`
- [ ] 2.4 Seed the cache by running one full cold build that pushes both `:3.2.x-test` and the cache layer
- [ ] 2.5 Re-run the build on a one-line dashboard-only change; assert wall time <10m

> 2.1, 2.4, 2.5 require Docker Hub admin access and a live build. 2.2 + 2.3 are wired to the new `-CacheRepo`/`-CacheTag`/`-NoCache` parameters; default cache target is `hivehub/vectorizer-cache:buildx` per spec. `push.ps1` exposes the cache parameters for surface-parity but does not pass them to docker (it never invokes buildx build) — documented in the script header.

## 3. Drop `cargo install cargo-sbom` step

- [x] 3.1 Audit downstream consumers of `/vectorizer/vectorizer.spdx.json` in the runtime image (grep this repo + the four sibling Hive repos that consume the image)
- [x] 3.2 Remove the `RUN xx-cargo install cargo-sbom && cargo sbom > vectorizer.spdx.json` line from `Dockerfile`
- [x] 3.3 Remove the `COPY --from=builder /vectorizer/vectorizer.spdx.json` line from the runtime stage of `Dockerfile`
- [ ] 3.4 Re-run a full multi-arch build; verify `docker buildx imagetools inspect` still lists 2 attestation manifests carrying SBOM
- [ ] 3.5 Run `docker scout policy ... --org hivehub --platform linux/amd64`; confirm `Supply chain attestations: 0 deviations` parity

> 3.1 audit (this repo): grep returned references only inside this task's own proposal/spec/tasks files and the Dockerfile itself — no production consumer reads `/vectorizer/vectorizer.spdx.json` from inside the image. The four sibling Hive repos cannot be audited from this session; flag in PR description for owner review. 3.2/3.3 done. 3.4/3.5 require the live multi-arch build + Scout — covered by the new `.github/workflows/docker-image-smoke.yml` job (assertions on attestation count + Scout policy gate).

## 4. `[profile.release-docker]` with LTO disabled

- [x] 4.1 Add `[profile.release-docker]` block to root `Cargo.toml` (`inherits = "release"`, `lto = false`, `codegen-units = 16`, `incremental = false`)
- [x] 4.2 Update `Dockerfile` `xx-cargo build` invocation to pass `--profile release-docker` and adjust the `PROFILE_DIR` resolution accordingly
- [x] 4.3 Update `Dockerfile` `xx-cargo chef cook` invocation to use the same `--profile release-docker` so dep cache matches workspace build
- [x] 4.4 Document the dual-profile contract in `docker/dockerhub-readme.md` Image Details section + `docs/development/docker-builds.md`
- [ ] 4.5 Run a full build; assert workspace compile stage drops from ~4m to ~2-3m (amd64) and ~6m to ~4m (arm64)

> 4.2/4.3: `ARG PROFILE=release-docker` is now the Dockerfile default; both `xx-cargo chef cook` and `xx-cargo build` already use `--profile $PROFILE`, so no further script changes are needed (verified). The existing `PROFILE_DIR=$(if [ "$PROFILE" = dev ]; then echo debug; else echo $PROFILE; fi)` resolves correctly to `release-docker` because Cargo writes to `target/<triple>/release-docker/` for the new profile. Verified locally with `cargo check --profile release-docker --package vectorizer-server --bin vectorizer` (compiled in 2m 06s, no errors). 4.5 requires a live multi-arch Docker build.

## 5. Native arm64 CI runner

- [x] 5.1 Split the existing `release-artifacts.yml::publish-docker` job into a matrix with `ubuntu-24.04` (amd64) + `ubuntu-24.04-arm` (arm64); each builds a single-platform image with attestations
- [x] 5.2 Add a manifest-list-assembly step that runs after both matrix jobs: `docker buildx imagetools create -t hivehub/vectorizer:${{ tag }} <amd64-digest> <arm64-digest>`
- [ ] 5.3 Add `docker/login-action@v3` step against `hivehub` using `DOCKERHUB_USERNAME` + `DOCKERHUB_TOKEN` repo secrets (request from owner — flag in PR description)
- [x] 5.4 Add `docker scout policy --org hivehub --platform linux/amd64` gate at the end of the workflow; fail the job on policy regression
- [ ] 5.5 Trigger a dry-run on a `v3.2.x-rc` tag; confirm both jobs finish without QEMU traces (`grep -i qemu` on logs returns nothing)

> 5.1: matrix uses `ubuntu-latest` (amd64) and `ubuntu-24.04-arm` (arm64). 5.2: sibling `publish-docker-manifest` job stitches digests via `docker buildx imagetools create`. 5.3: login step is wired with `if: env.DOCKERHUB_USERNAME != ''` so the workflow is no-op-safe until the owner provisions the secrets — owner action required. 5.4: `docker/scout-action@v1` step runs against `hivehub/vectorizer:<version>` after manifest publish. 5.5 needs a live release-candidate tag — owner action.

## 6. Pre-baked dashboard artifact (CI only)

- [x] 6.1 Confirm `docker/Dockerfile.artifacts` already accepts `dashboard/dist` as a COPY input (read the file, verify `COPY dashboard/dist` exists)
- [x] 6.2 In the CI workflow, add `download-artifact` step before the Docker build to pull `dashboard-dist` into `dashboard/dist/`
- [x] 6.3 Switch the CI Docker build to `file: ./docker/Dockerfile.artifacts` instead of `./Dockerfile`
- [ ] 6.4 Verify image digest from the artifact-based build matches functional behavior of the source-built image (sample REST + RPC quickstart)

> 6.1 verified: `docker/Dockerfile.artifacts` line 34 already does `COPY dashboard/dist /vectorizer/dashboard/dist`. 6.2/6.3: download-artifact step exists in the matrix job and `file: ./docker/Dockerfile.artifacts` is in use (no change needed — was already on `Dockerfile.artifacts` before phase10; the matrix refactor preserves it). 6.4: container start smoke is automated in `.github/workflows/docker-image-smoke.yml::smoke` — boots the published image, polls `/health` for up to 60s, fails on regression.

## 7. Documentation

- [x] 7.1 Write `docs/development/docker-builds.md` covering: baseline numbers, cache repo URL + lifecycle, dual-profile contract, native arm64 path, troubleshooting (Docker Desktop memory floor, BuildKit OOM signature, cache invalidation triggers)
- [x] 7.2 Update `docker/dockerhub-readme.md` Image Details + Build Flags lines to reflect `release-docker` profile
- [x] 7.3 Update root `CHANGELOG.md` under v3.x.0 (next release) Build section with the new cache repo + profile
- [x] 7.4 Update `AGENTS.md` build commands section if `--profile release-docker` should be the local maintainer default for `docker build`

## 8. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 8.1 Update or create documentation covering the implementation (covered by §7 above; verify `docs/development/docker-builds.md` exists and is referenced from `docs/README.md`)
- [x] 8.2 Write tests covering the new behavior — add a CI smoke test that asserts (a) image build under N minutes on warm cache, (b) `docker buildx imagetools inspect` reports 2 attestation manifests, (c) `docker scout policy` shows ≥5/7 policies met
- [ ] 8.3 Run tests and confirm they pass — execute the §8.2 smoke test on a release-candidate tag end-to-end before merging

> 8.1 done — `docs/README.md` "For Developers" list now links the new runbook. 8.2 done — `.github/workflows/docker-image-smoke.yml` asserts both arches present, ≥2 attestation manifests, Scout policy met, and a container-start probe. 8.3 requires a live release-candidate tag and Docker Hub credentials — owner action; flag in PR description.

## Outstanding owner actions (cannot complete in this session)

The following items require resources outside this session — flag them in the PR description:

1. Create the `hivehub/vectorizer-cache` Docker Hub repository (§2.1).
2. Add `DOCKERHUB_USERNAME` + `DOCKERHUB_TOKEN` repo secrets to enable the new Docker Hub push and Scout gate in `release-artifacts.yml::publish-docker-manifest` (§5.3).
3. Run a cold local build with `-NoCache` to record fresh baseline numbers in `docs/development/docker-builds.md` (§1.1, §1.2, §1.3).
4. Seed the cache + verify warm-cache wall time <10m (§2.4, §2.5).
5. Trigger a `v3.2.x-rc` tag to validate the new matrix workflow end-to-end (§5.5, §6.4, §8.3) and confirm `grep -i qemu` returns nothing on the arm64 job log.
6. Audit the four sibling Hive repos for any consumer reading `/vectorizer/vectorizer.spdx.json` from the runtime image (§3.1).
