# Proposal: phase10b_docker-build-owner-actions

Follow-up to: phase10_optimize-docker-build-time
Source: `.rulebook/archive/.../phase10_optimize-docker-build-time/tasks.md`
        section "Outstanding owner actions"

## Why

Phase10 landed all in-repo code changes for the Docker build pipeline
overhaul (cfafcda8): buildx registry cache wiring in the local PowerShell
scripts, the `release-docker` Cargo profile, the `cargo sbom` removal,
the per-arch matrix in `release-artifacts.yml::publish-docker`, the
`publish-docker-manifest` stitching job, the `docker-image-smoke.yml`
workflow, and the `docs/development/docker-builds.md` runbook.

Six items in phase10 cannot be completed from inside an editor session —
they require Docker Hub admin access, real release-candidate tags, or a
maintainer's local workstation for cold builds. They are tracked here so
phase10 can archive cleanly without orphan deferred items.

## What Changes

No code changes. This task tracks operational steps the repo owner must
perform to validate the phase10 implementation end-to-end.

### Items

1. Create the `hivehub/vectorizer-cache` repo on Docker Hub.
2. Provision `DOCKERHUB_USERNAME` + `DOCKERHUB_TOKEN` repo secrets so
   `release-artifacts.yml::publish-docker-manifest` can push to
   `hivehub/vectorizer` and gate on `docker scout policy`.
3. Cold local build with `-NoCache` to record fresh baseline numbers
   (amd64 + arm64 wall times, peak Docker Desktop memory) into
   `docs/development/docker-builds.md`.
4. Seed the registry cache (one full cold push) and re-run on a
   one-line dashboard change; assert warm wall time < 10m.
5. Cut a `v3.2.x-rc` tag to validate the new matrix workflow:
   - `grep -i qemu` on the arm64 job log returns nothing.
   - `docker buildx imagetools inspect` reports 2 attestation manifests.
   - `docker scout policy` shows ≥5/7 policies met.
   - `docker-image-smoke.yml` passes container-start probe.
6. Audit the four sibling Hive repos that consume the
   `hivehub/vectorizer` image for any reader of
   `/vectorizer/vectorizer.spdx.json` (the file was removed in phase10
   §3.2/3.3; consumers must read the syft attestation instead).

## Impact

- Affected specs: none — phase10's spec already encodes the parity
  targets (≥5/7 Scout policies, 2 attestation manifests, warm <10m).
- Affected code: none.
- Breaking change: NO.
- User benefit: closes phase10's owner-action queue so the build
  pipeline is end-to-end verified, not just code-complete.

## Acceptance

Each owner action above checked off with the verification artifact
(Docker Hub URL, GitHub Actions run id, `imagetools inspect` output, or
maintainer note in the runbook).
