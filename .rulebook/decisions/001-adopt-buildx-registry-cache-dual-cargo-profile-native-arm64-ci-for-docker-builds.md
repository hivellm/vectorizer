# 1. Adopt buildx registry cache + dual Cargo profile + native arm64 CI for Docker builds

**Status**: proposed
**Date**: 2026-05-02
**Related Tasks**: phase10_optimize-docker-build-time

## Context

Cold local Docker build of `hivehub/vectorizer:3.2.0` measured 30-45m on a 33 GB Docker Desktop allocation (multi-arch linux/amd64,linux/arm64). Stages: cargo chef cook 10m+, workspace compile 3-6m per arch, cargo-sbom recompile ~1m per arch, arm64 via QEMU 2-3x slower than native. CI had no Docker Hub push (only ghcr.io) so every hivehub/vectorizer push was on the maintainer's wall clock.

## Decision

Ship four coordinated changes in phase10: (1) buildx registry cache at `hivehub/vectorizer-cache:buildx`, wired into all three local scripts and CI; (2) dedicated `release-docker` Cargo profile with `lto = false` + `codegen-units = 16`, default in Dockerfile (host `release` unchanged); (3) drop `cargo install cargo-sbom` step entirely in favor of BuildKit `--sbom=true` syft attestation; (4) split CI publish-docker into per-arch native-runner matrix (`ubuntu-latest` + `ubuntu-24.04-arm`) with sibling manifest-assembly job pushing to both ghcr.io and Docker Hub.

## Alternatives Considered

- sccache + S3/GCS shared cache (deferred — non-trivial IAM/secrets/lifecycle setup)
- cargo-zigbuild for cross-compile (rejected — TLS/glibc compat issues)
- Replace cargo-chef with native cargo --target-dir caching (rejected — chef still wins for dep-warming pattern even with registry cache)
- Pre-bake cargo-sbom in a sidecar stage (option B in proposal — rejected unless audit reveals downstream consumers of the in-image SPDX file)

## Consequences

Cold build: 30-45m → 15-20m. Warm build: <10m for non-Rust changes. arm64 CI: native, no QEMU. Docker Hub publish parity (no more local-only push). Trade: shipped binary is ~10-15% slower on hot paths than a host-built `release` binary (acceptable inside container; documented in dockerhub-readme + docs/development/docker-builds.md). Owner action items (cannot complete in this session): create `hivehub/vectorizer-cache` Docker Hub repo, provision `DOCKERHUB_USERNAME`/`DOCKERHUB_TOKEN` repo secrets, run release-candidate tag to validate end-to-end.
