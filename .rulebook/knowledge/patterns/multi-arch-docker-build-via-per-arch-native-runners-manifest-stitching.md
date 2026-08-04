# Multi-arch Docker build via per-arch native runners + manifest stitching

**Category**: build
**Tags**: docker, ci, multi-arch, qemu, phase10

## Description

Build each arch on its native GitHub Actions runner (ubuntu-latest for amd64, ubuntu-24.04-arm for arm64), push by digest only, then stitch the digests into a multi-arch manifest list with `docker buildx imagetools create`. Eliminates QEMU emulation for arm64 (2-3x speedup) without losing per-arch attestations.

## Example

strategy:
  matrix:
    include:
      - {arch: amd64, runner: ubuntu-latest, platform: linux/amd64}
      - {arch: arm64, runner: ubuntu-24.04-arm, platform: linux/arm64}
runs-on: ${{ matrix.runner }}
steps:
  - uses: docker/build-push-action@v6
    with:
      platforms: ${{ matrix.platform }}
      outputs: type=image,name=ghcr.io/...,push-by-digest=true,name-canonical=true,push=true
  - run: echo "${{ steps.build.outputs.digest }}" > /tmp/digests/...
  - uses: actions/upload-artifact@v4

# Manifest job downloads digest artifacts and runs:
docker buildx imagetools create -t ghcr.io/x:tag $(printf 'ghcr.io/x@sha256:%s ' *)

## When to Use

Multi-arch Docker images with significant Rust/C++ compile cost where QEMU arm64 emulation dominates wall time

## When NOT to Use

Cheap dockerfiles (only COPY/ENV) — single-runner buildx with QEMU is simpler and fast enough
