# Compiling cargo-sbom inside the runtime Dockerfile to embed an SPDX file

**Category**: build
**Tags**: docker, sbom, supply-chain, phase10

## Description

Adding `RUN xx-cargo install cargo-sbom && cargo sbom > vectorizer.spdx.json` to a multi-arch Dockerfile compiles cargo-sbom from source ONCE PER ARCH (~1m × N arches) and produces a file BuildKit's `--sbom=true` syft attestation already provides as an in-toto attestation manifest. Scout reads from the attestation, not from the in-image file. Drop the step; rely on the BuildKit attestation surface.

## Example

# BAD — recompiles cargo-sbom per arch, file unread by Scout
RUN xx-cargo install cargo-sbom && cargo sbom > vectorizer.spdx.json
COPY --from=builder /vectorizer/vectorizer.spdx.json /vectorizer/

# GOOD — SBOM via BuildKit attestation only
docker buildx build --sbom=true --provenance=mode=max ...
docker buildx imagetools inspect <image>  # shows attestation manifests

## When to Use

Never — always prefer BuildKit `--sbom=true`

## When NOT to Use

If a downstream consumer specifically reads the file from inside the image — audit first; then consider a sidecar stage that pre-bakes cargo-sbom (host-arch tool, no per-arch recompile)
