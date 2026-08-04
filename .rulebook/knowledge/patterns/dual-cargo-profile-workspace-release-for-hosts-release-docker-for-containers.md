# Dual Cargo profile: workspace `release` for hosts, `release-docker` for containers

**Category**: build
**Tags**: rust, cargo, docker, lto, phase10

## Description

Define a sibling Cargo profile that inherits `release` but disables LTO and bumps codegen-units. Container builds select it via Dockerfile ARG; host `cargo build --release` is unaffected. Trades ~10-15% runtime throughput for ~30% faster compile and ~50% lower peak rustc memory inside BuildKit, where the binary is throwaway anyway.

## Example

[profile.release-docker]
inherits = "release"
lto = false
codegen-units = 16
incremental = false

# Dockerfile:
ARG PROFILE=release-docker
RUN xx-cargo build --profile $PROFILE ...

## When to Use

When workspace `release` profile uses LTO (thin or fat) and Docker builds OOM or take too long inside BuildKit

## When NOT to Use

When the shipped binary needs peak runtime perf for hot loops the user calls (e.g. SIMD vector kernels). Document the trade in dockerhub README so consumers know.
