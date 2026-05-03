## 1. Docker Hub provisioning

- [ ] 1.1 Create `hivehub/vectorizer-cache` repository on Docker Hub (private acceptable; only buildx writes/reads); paste URL into `docs/development/docker-builds.md` "Cache" section
- [ ] 1.2 Provision `DOCKERHUB_USERNAME` + `DOCKERHUB_TOKEN` GitHub repo secrets so `release-artifacts.yml::publish-docker-manifest` and the Scout policy gate become active

## 2. Baseline measurement (maintainer workstation)

- [ ] 2.1 Run `.\scripts\docker\build-push.ps1 -Tag bench-baseline -NoCache`; record amd64 + arm64 wall times per stage in `docs/development/docker-builds.md` table
- [ ] 2.2 Capture peak Docker Desktop memory during the build (`docker stats` or Resource Saver panel); document the floor required by the new pipeline
- [ ] 2.3 Capture `docker scout policy` baseline (5/7 met, 0 deviations) as the post-change parity target

## 3. Cache validation

- [ ] 3.1 Seed the registry cache by running one full cold build that pushes both `:3.2.x-test` and the `:buildx` cache layer
- [ ] 3.2 Re-run the build on a one-line dashboard-only change with cache enabled; assert wall time < 10m

## 4. End-to-end CI validation

- [ ] 4.1 Cut a `v3.2.x-rc` tag and trigger `release-artifacts.yml`; verify both arch-matrix jobs (amd64 + arm64) finish without QEMU traces (`grep -i qemu` on the arm64 job log returns nothing)
- [ ] 4.2 Verify `publish-docker-manifest` runs `docker buildx imagetools inspect` against the published manifest and reports 2 attestation manifests carrying SBOM
- [ ] 4.3 Verify `docker scout policy --org hivehub --platform linux/amd64` reports `Supply chain attestations: 0 deviations`
- [ ] 4.4 Verify `.github/workflows/docker-image-smoke.yml` passes the container-start probe (`/health` reachable within 60s)

## 5. Sibling-repo audit

- [ ] 5.1 Audit the four sibling Hive repos that consume `hivehub/vectorizer` for any reader of `/vectorizer/vectorizer.spdx.json` (removed in phase10 §3.2/3.3); document findings; if any consumer is found, file a follow-up to migrate them to the syft attestation

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 6.1 Update `docs/development/docker-builds.md` with measured baseline + warm-cache numbers from §2 / §3
- [ ] 6.2 Capture verification artifacts (URLs, run ids, `imagetools inspect` output) inline in this task or in `docs/development/docker-builds.md`
- [ ] 6.3 Confirm phase10 spec parity targets are met (≥5/7 Scout policies, 2 attestation manifests, warm <10m) before archiving
