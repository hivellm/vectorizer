# Multi-stage Dockerfile for Vectorizer
# Based on Qdrant's production-grade Docker build strategy
#
# ============================================================================
# BUILD EXAMPLES
# ============================================================================
# Local build examples:
#   docker build -t vectorizer:local .
#   docker build -t vectorizer:1.5.2 .
#   docker buildx build --platform linux/amd64,linux/arm64 -t vectorizer:latest .
#
# Multi-platform build:
#   docker buildx build --platform linux/amd64,linux/arm64 -t ghcr.io/hivellm/vectorizer:latest --push .
#
# ============================================================================
# RUN EXAMPLES
# ============================================================================
# Basic run (default port 15002):
#   docker run -d -p 15002:15002 --name vectorizer vectorizer:latest
#
# Run with persistent storage:
#   # Bash/Linux/Mac (bind mount to ./data):
#   docker run -d -p 15002:15002 \
#     -v $(pwd)/data:/vectorizer/data \
#     --name vectorizer vectorizer:latest
#
#   # PowerShell (Windows) - bind mount to ./data:
#   docker run -d -p 15002:15002 `
#     -v ${PWD}/data:/vectorizer/data `
#     --name vectorizer vectorizer:latest
#
#   # Using named volume (Docker manages the location):
#   docker run -d -p 15002:15002 \
#     -v vectorizer-data:/vectorizer/data \
#     --name vectorizer vectorizer:latest
#
# Run with workspace configuration (monorepo):
#   # Bash/Linux/Mac:
#   docker run -d -p 15002:15002 \
#     -v $(pwd)/data:/vectorizer/data \
#     -v $(pwd)/workspace.docker.yml:/vectorizer/workspace.yml:ro \
#     -v $(pwd)/../../:/workspace:ro \
#     --name vectorizer vectorizer:latest
#
#   # PowerShell (Windows):
#   docker run -d -p 15002:15002 `
#     -v ${PWD}/data:/vectorizer/data `
#     -v ${PWD}/workspace.docker.yml:/vectorizer/workspace.yml:ro `
#     -v ${PWD}/../../:/workspace:ro `
#     --name vectorizer vectorizer:latest
#
# Run with custom host/port:
#   docker run -d -p 8080:15002 \
#     -e VECTORIZER_HOST=0.0.0.0 \
#     -e VECTORIZER_PORT=15002 \
#     --name vectorizer vectorizer:latest
#
# Run with custom user (non-root):
#   docker run -d -p 15002:15002 \
#     --user 1000:1000 \
#     -v $(pwd)/data:/vectorizer/data \
#     --name vectorizer vectorizer:latest
#
# Run with environment variables:
#   docker run -d -p 15002:15002 \
#     -e VECTORIZER_HOST=0.0.0.0 \
#     -e VECTORIZER_PORT=15002 \
#     -e RUN_MODE=production \
#     -e TZ=America/Sao_Paulo \
#     --name vectorizer vectorizer:latest
#
# Run with custom authentication (RECOMMENDED FOR PRODUCTION):
#   docker run -d -p 15002:15002 \
#     -e VECTORIZER_AUTH_ENABLED=true \
#     -e VECTORIZER_ADMIN_USERNAME=admin \
#     -e VECTORIZER_ADMIN_PASSWORD=your-secure-password \
#     -e VECTORIZER_JWT_SECRET=your-jwt-secret-key \
#     -v $(pwd)/data:/vectorizer/data \
#     --name vectorizer vectorizer:latest
#
# Run with workspace (recommended for monorepo):
#   # Bash/Linux/Mac:
#   docker run -d -p 15002:15002 \
#     -v $(pwd)/data:/vectorizer/data \
#     -v $(pwd)/workspace.docker.yml:/vectorizer/workspace.yml:ro \
#     -v $(pwd)/../../:/workspace:ro \
#     -e VECTORIZER_HOST=0.0.0.0 \
#     -e VECTORIZER_PORT=15002 \
#     --name vectorizer vectorizer:latest
#
#   # PowerShell (Windows):
#   docker run -d -p 15002:15002 `
#     -v ${PWD}/data:/vectorizer/data `
#     -v ${PWD}/workspace.docker.yml:/vectorizer/workspace.yml:ro `
#     -v ${PWD}/../../:/workspace:ro `
#     -e VECTORIZER_HOST=0.0.0.0 `
#     -e VECTORIZER_PORT=15002 `
#     --name vectorizer vectorizer:latest
#
# Run with Docker Compose:
#   docker-compose up -d
#
# Access logs:
#   docker logs vectorizer
#   docker logs -f vectorizer  # follow logs
#
# Stop container:
#   docker stop vectorizer
#   docker rm vectorizer
#
# ============================================================================
# DOCKER COMPOSE EXAMPLE
# ============================================================================
# Create docker-compose.yml:
#   version: '3.8'
#   services:
#     vectorizer:
#       image: vectorizer:latest
#       ports:
#         - "15002:15002"
#       volumes:
#         - ./data:/vectorizer/data
#         - ./workspace.docker.yml:/vectorizer/workspace.yml:ro
#         - ../../:/workspace:ro
#       environment:
#         - VECTORIZER_HOST=0.0.0.0
#         - VECTORIZER_PORT=15002
#         - RUN_MODE=production
#       restart: unless-stopped

# Which runtime the build produces. Declared here because it is consumed by a
# `FROM` at the end of the file, and a global ARG must precede the first FROM.
#
#   static (default) — `scratch`. Zero OS packages, so zero OS-package CVEs.
#   glibc            — distroless cc (Debian 13). Required by the `-fastembed` variant,
#                      whose ONNX Runtime links libstdc++ dynamically and so
#                      cannot be a static binary.
ARG RUNTIME_VARIANT=static

# Cross-compiling using Docker multi-platform builds
FROM --platform=${BUILDPLATFORM:-linux/amd64} tonistiigi/xx AS xx

# Utilizing Docker layer caching with cargo-chef.
#
# Base: `rust:1.95-slim-trixie` (Debian 13, glibc 2.40).
# - sysinfo@0.39.x (the default-features dep that backs
#   GET /metrics/runtime) requires rustc 1.95.
# - Edition 2024 (every workspace crate) requires rustc 1.85+.
# - The glibc runtime stage is `gcr.io/distroless/cc-debian13` (Debian 13 too)
#   — aligning the builder and runtime libc avoids
#   `undefined symbol: __isoc23_strtol` / `__isoc23_strtoull` link
#   errors when the prebuilt ORT static library (pulled by the
#   `fastembed` Cargo feature) references glibc 2.38+ C23 symbols.
#   This was the link failure that surfaced when phase33 §5.2
#   introduced the optional fastembed Docker variant — see
#   `.rulebook/archive/2026-06-06-phase33_dense-embedding-provider-coercion/design.md` D6.
# - cargo-chef is installed manually because the official
#   `lukemathwalker/cargo-chef` image only ships bookworm tags.
FROM --platform=${BUILDPLATFORM:-linux/amd64} rust:1.95-slim-trixie AS chef
RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config libssl-dev clang lld git curl ca-certificates \
        && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef --locked --version 0.1.73
WORKDIR /vectorizer

FROM chef AS planner
WORKDIR /vectorizer
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Dashboard builder stage
FROM node:20-bookworm AS dashboard-builder
WORKDIR /dashboard

# Install pnpm.
#
# PINNED, and not to `@latest`. Every workflow already pins `pnpm@10` and
# `dashboard/package.json` declares `packageManager: pnpm@10.15.1`; this line
# was the one place still floating, and it broke the moment pnpm's latest
# started requiring Node >= 22.13 — it imports `node:sqlite`, which does not
# exist on the node:20 base, so `pnpm install` died with
# ERR_UNKNOWN_BUILTIN_MODULE. Nothing in this repo changed; the outside world
# did.
RUN npm install -g pnpm@10.15.1

# Copy dashboard files
COPY dashboard/package.json dashboard/pnpm-lock.yaml dashboard/pnpm-workspace.yaml ./
COPY dashboard/tsconfig.json dashboard/vite.config.ts dashboard/eslint.config.js ./
COPY dashboard/index.html ./
COPY dashboard/src ./src
COPY dashboard/public ./public

# Install dependencies and build dashboard
RUN pnpm install --frozen-lockfile && \
    pnpm run build:skip-check

FROM chef AS builder
WORKDIR /vectorizer

COPY --from=xx / /

# NOTE on OPENSSL_DIR leakage: Docker Desktop for Windows + buildx
# desktop-linux leaks the Windows-host `OPENSSL_DIR` env var
# (`C:/Program Files/OpenSSL-Win64`) into Linux RUN steps, where
# `openssl-sys` build script then panics with "OpenSSL include
# directory does not exist". Setting `ENV OPENSSL_DIR=` to empty
# also breaks because openssl-sys rejects an empty path. The fix
# is `unset` inside each affected RUN command — see the explicit
# `unset OPENSSL_DIR ...` prefix on the cargo-chef cook and the
# cargo build commands below.

# Install dependencies
RUN apt-get update \
    && apt-get install -y clang lld cmake protobuf-compiler jq \
    && rustup component add rustfmt

# ARG/ENV pair for docker build backward-compatibility
ARG BUILDPLATFORM
ENV BUILDPLATFORM=${BUILDPLATFORM:-linux/amd64}

ARG MOLD_VERSION=2.36.0

# Install mold linker for faster builds
RUN case "$BUILDPLATFORM" in \
        */amd64 ) PLATFORM=x86_64 ;; \
        */arm64 | */arm64/* ) PLATFORM=aarch64 ;; \
        * ) echo "Unexpected BUILDPLATFORM '$BUILDPLATFORM'" >&2; exit 1 ;; \
    esac; \
    \
    mkdir -p /opt/mold; \
    cd /opt/mold; \
    \
    TARBALL="mold-$MOLD_VERSION-$PLATFORM-linux.tar.gz"; \
    curl -sSLO "https://github.com/rui314/mold/releases/download/v$MOLD_VERSION/$TARBALL"; \
    tar -xf "$TARBALL" --strip-components 1; \
    rm "$TARBALL"

# ARG/ENV pair for docker build backward-compatibility
ARG TARGETPLATFORM
ENV TARGETPLATFORM=${TARGETPLATFORM:-linux/amd64}

# Install cross-compilation dependencies including OpenSSL
RUN xx-apt-get install -y pkg-config gcc g++ libc6-dev libssl-dev

# Select Cargo profile. Default is `release-docker` (defined in workspace
# `Cargo.toml`): inherits `release`, but disables LTO and bumps
# codegen-units to 16 so peak rustc memory + wall time stay sane inside
# BuildKit. Override at build time with `--build-arg PROFILE=release` if
# you need the workspace `release` binary (LTO=thin, codegen-units=4).
ARG PROFILE=release-docker

# Enable crate features (empty = use default features; set to disable defaults)
ARG FEATURES

# Build without default features when set (avoids hive-gpu/fastembed/transmutation in Docker)
ARG NO_DEFAULT_FEATURES=0

# Pass custom RUSTFLAGS
ARG RUSTFLAGS



# Build dependencies with cargo-chef (cached layer)
COPY --from=planner /vectorizer/recipe.json recipe.json
RUN unset OPENSSL_DIR OPENSSL_INCLUDE_DIR OPENSSL_LIB_DIR OPENSSL_STATIC; \
    PKG_CONFIG="/usr/bin/$(xx-info)-pkg-config" \
    PATH="$PATH:/opt/mold/bin" \
    RUSTFLAGS="${LINKER:+-C link-arg=-fuse-ld=}$LINKER $RUSTFLAGS" \
    xx-cargo chef cook --profile $PROFILE --package vectorizer-server --bin vectorizer ${NO_DEFAULT_FEATURES:+--no-default-features} ${FEATURES:+--features} $FEATURES --recipe-path recipe.json

# Build application
COPY . .
# Embed dashboard at compile time (rust-embed requires dashboard/dist to exist)
COPY --from=dashboard-builder /dashboard/dist /vectorizer/dashboard/dist
ARG GIT_COMMIT_ID
# Limit parallel jobs to reduce peak memory (avoids OOM in cross-build / low-memory env)
ENV CARGO_BUILD_JOBS=2
RUN unset OPENSSL_DIR OPENSSL_INCLUDE_DIR OPENSSL_LIB_DIR OPENSSL_STATIC; \
    PKG_CONFIG="/usr/bin/$(xx-info)-pkg-config" \
    PATH="$PATH:/opt/mold/bin" \
    RUSTFLAGS="${LINKER:+-C link-arg=-fuse-ld=}$LINKER $RUSTFLAGS" \
    xx-cargo build --profile $PROFILE --package vectorizer-server ${NO_DEFAULT_FEATURES:+--no-default-features} ${FEATURES:+--features} $FEATURES --bin vectorizer \
    && PROFILE_DIR=$(if [ "$PROFILE" = dev ]; then echo debug; else echo $PROFILE; fi) \
    && mv target/$(xx-cargo --print-target-triple)/$PROFILE_DIR/vectorizer /vectorizer/vectorizer

# Stage the **target-arch** libstdc++ for the runtime stage to copy.
#
# `COPY` cannot interpolate the architecture triple, so the runtime used to
# hardcode `/usr/lib/x86_64-linux-gnu/libstdc++.so.6` — which put an *amd64*
# library inside the arm64 image. Verified against the published artifact:
# `docker run --platform linux/arm64 hivehub/vectorizer:3.5.0-fastembed` dies
# with `error while loading shared libraries: libstdc++.so.6` (exit 127),
# while the same arm64 tag of the BM25-only default image boots fine because
# it never loads the lib. Cross-linking still succeeded, which is why nobody
# caught it: the linker resolves `-lstdc++` from the cross toolchain's own
# sysroot copy at `/usr/<triple>/lib/`, a path the runtime never reads.
#
# `xx-apt-get install libstdc++6` materializes the library at the canonical
# multiarch path for the target (a no-op for a native amd64 target, where the
# host package already provides it). Staging it under the destination layout
# lets a single static `COPY /staging/usr/ /usr/` land it at the right
# multiarch path on every architecture.
#
# This RUN is deliberately *after* the cargo build: the library is not needed
# to compile or link, and keeping it out of the pre-build layers preserves the
# expensive `cargo chef cook` + workspace-compile cache entries.
RUN xx-apt-get install -y libstdc++6 \
    && mkdir -p "/staging/usr/lib/$(xx-info triple)" \
    && cp -aL "/usr/lib/$(xx-info triple)/libstdc++.so.6" \
              "/staging/usr/lib/$(xx-info triple)/libstdc++.so.6"

# SBOM is provided by BuildKit's `--sbom=true` syft attestation
# (attached per-arch to the manifest list). The previous in-image
# `cargo sbom > vectorizer.spdx.json` step recompiled `cargo-sbom`
# from source on every build, once per arch (~1m × N arches), and
# produced a file no downstream consumer reads — Scout policies
# already pull from the syft attestation. See spec at
# `.rulebook/tasks/phase10_optimize-docker-build-time/specs/build/spec.md`.

# Writable data dir for distroless nonroot.
#
# `/vectorizer/data` is kept for backward compatibility (binary still
# writes a placeholder `vectorizer.vecdb` here from older code paths).
#
# `/data` is the canonical persistent state dir starting in v3.4.0 (see
# phase32_fix-container-data-persistence / issue #300). The runtime
# defaults `VECTORIZER_DATA_DIR=/data` so a single
# `--volume vec-data:/data` mount captures collections, auth keys,
# JWT secret, and snapshots — without it, the operator had to mount a
# second volume on `/.local/share/vectorizer` and a routine
# `docker compose up -d --force-recreate` would wipe every collection.
FROM debian:bookworm-slim AS writable-dirs
RUN mkdir -p /vectorizer/data /data && chown -R 65532:65532 /vectorizer /data

# Optional FastEmbed model pre-fetch (phase33 / issue #306).
#
# When the operator builds with `--build-arg ENABLE_FASTEMBED=1`, this
# stage downloads one dense model at image-build time so the first
# container boot does not need a network round-trip to Hugging Face.
# The runtime stage copies the result into `/vectorizer/models/fastembed/`
# and points `VECTORIZER_FASTEMBED_CACHE_DIR` at it
# (`vectorizer_core::paths::fastembed_cache_dir()`). It lives outside `/data`
# on purpose: a volume mounted over `/data` (every Kubernetes deployment)
# would hide a model baked there and force a download on each pod's first
# boot.
#
# fastembed resolves models through `hf-hub`, whose cache lookup only
# finds `models--<org>--<name>/refs/main` (holding a commit sha) plus
# `snapshots/<sha>/<repo-relative path>` — so that is the layout written
# here. (Before 3.8 this stage wrote a flat `<org>/<name>/` directory,
# flattened `onnx/model.onnx`, and skipped `tokenizer_config.json`; the
# resolver never found any of it and every boot downloaded the model.)
#
# `FASTEMBED_MODEL` is the Hugging Face repo fastembed loads for the id
# in `embedding.model` / `embedding.additional_models` (see the id table
# in docs/specs/EMBEDDING.md), e.g.
# `--build-arg FASTEMBED_MODEL=intfloat/multilingual-e5-small` for
# `fastembed:multilingual-e5-small`. The default is the repo behind
# `fastembed:all-MiniLM-L6-v2`. `FASTEMBED_MODEL_FILES` overrides the
# ONNX file list (space-separated, repo-relative) for repos the `case`
# below does not know; everything else defaults to `onnx/model.onnx`.
#
# Default builds set `ENABLE_FASTEMBED=0`, leaving the stage as a
# no-op so the published slim image (BM25-only, ~release-docker
# profile) stays unchanged. Operators who want dense out of the box
# build with `--build-arg ENABLE_FASTEMBED=1 --build-arg
# NO_DEFAULT_FEATURES=0 --build-arg FEATURES=fastembed`.
FROM debian:bookworm-slim AS fastembed-models
ARG ENABLE_FASTEMBED=0
ARG FASTEMBED_MODEL=Qdrant/all-MiniLM-L6-v2-onnx
ARG FASTEMBED_MODEL_FILES=
RUN set -eu; \
    if [ "$ENABLE_FASTEMBED" = "1" ]; then \
      apt-get update && apt-get install -y --no-install-recommends curl ca-certificates; \
      FILES="$FASTEMBED_MODEL_FILES"; \
      if [ -z "$FILES" ]; then \
        case "$FASTEMBED_MODEL" in \
          Qdrant/all-MiniLM-L6-v2-onnx) FILES="model.onnx" ;; \
          Qdrant/multilingual-e5-large-onnx) FILES="model.onnx model.onnx_data" ;; \
          Qdrant/*-onnx-Q) FILES="model_optimized.onnx" ;; \
          Xenova/all-MiniLM-L6-v2) FILES="onnx/model_quantized.onnx" ;; \
          Xenova/all-MiniLM-L12-v2) FILES="onnx/model.onnx onnx/model_quantized.onnx" ;; \
          *) FILES="onnx/model.onnx" ;; \
        esac; \
      fi; \
      COMMIT="$(curl --fail --silent --show-error \
        "https://huggingface.co/api/models/${FASTEMBED_MODEL}/revision/main" \
        | grep -o '"sha":"[0-9a-f]\{40\}"' | head -n 1 | cut -d '"' -f 4)"; \
      [ -n "$COMMIT" ]; \
      REPO_DIR="/models/fastembed/models--$(printf '%s' "$FASTEMBED_MODEL" | sed 's#/#--#g')"; \
      SNAPSHOT="${REPO_DIR}/snapshots/${COMMIT}"; \
      mkdir -p "${REPO_DIR}/refs"; \
      printf '%s' "$COMMIT" > "${REPO_DIR}/refs/main"; \
      for f in $FILES tokenizer.json config.json special_tokens_map.json tokenizer_config.json; do \
        mkdir -p "$(dirname "${SNAPSHOT}/$f")"; \
        curl --fail --silent --show-error --location -o "${SNAPSHOT}/$f" \
          "https://huggingface.co/${FASTEMBED_MODEL}/resolve/${COMMIT}/$f"; \
      done; \
    else \
      mkdir -p /models/fastembed; \
    fi; \
    chown -R 65532:65532 /models

# Static busybox — the runtime is distroless (no shell, no curl, no wget), so
# docker-compose / orchestrator healthchecks against /health need their own
# HTTP probe binary. busybox:stable-musl is a ~1 MB static binary that
# supplies `wget`, satisfying the HEALTHCHECK below without re-introducing a
# shell or a package manager.
FROM busybox:stable-musl AS busybox

# ============================================================================
# STATIC MUSL BUILDER — for the `scratch` runtime (default variant)
# ============================================================================
# The published 3.7.0 image carried 30 CVEs, every one of them in a base-OS
# package (openssl, glibc, tar) rather than in our code. Bumping the base did
# not help: the newest `dhi.io/debian-base:trixie` digest ships the identical
# package versions, and the fix (openssl 3.5.7) had not reached it.
#
# A `scratch` runtime has no packages at all, so that entire class of finding
# disappears rather than being managed. It is what Nexus, Synap and Fluxum
# already do, and it needs one thing: a statically linked binary.
#
# Two prerequisites that were NOT true until recently, recorded because they
# are what makes this reachable at all:
#   * OpenSSL had to leave the dependency graph. `umicp-core`'s `http2`
#     feature pulled reqwest with default features -> native-tls -> OpenSSL,
#     so the binary dynamically linked libssl.so.3 (verified against the
#     published artifact). Dropped in phase7.
#   * The binary had to link statically. Verified: `aws-lc-sys` and
#     `zstd-sys` both cross the musl boundary, and the result reports
#     `static-pie linked`.
#
# NOT pinned to $BUILDPLATFORM: this stage builds once per target platform,
# so arm64 runs under emulation. That is slower than the xx cross-compile the
# glibc builder uses, and it is the same trade Nexus makes — correctness of
# the artifact over build wall-clock. The `xx` toolchain resolves musl targets
# to `-linux-gnu` (`xx-info triple`), and pointing clang at Debian's arm64
# musl sysroot fails on missing `crtend.o`, so cross-compiling this would mean
# hand-assembling a toolchain rather than using one.
# ----------------------------------------------------------------------------
# Cross-compilation toolchain — its own stage on purpose
# ----------------------------------------------------------------------------
# Split out so a network hiccup while fetching it cannot invalidate the
# expensive compile layer below, and vice versa. That is not hypothetical: a
# publish attempt died here when crates.io served 503s from its CDN mid-fetch,
# and because the toolchain shared a stage with the build, the whole thing had
# to start over.
#
# `cargo-zigbuild` arrives as the project's own prebuilt binary rather than
# `cargo install --locked`, which compiled it from source on every build —
# minutes of work, and a crates.io dependency, both on the critical path of
# something that is just a tool.
FROM --platform=${BUILDPLATFORM:-linux/amd64} debian:trixie-slim AS zig-toolchain
ARG ZIG_VERSION=0.13.0
ARG CARGO_ZIGBUILD_VERSION=0.23.3
RUN apt-get update     && apt-get install -y --no-install-recommends curl xz-utils ca-certificates     && rm -rf /var/lib/apt/lists/*     && curl -sSL --retry 5 --retry-all-errors         "https://ziglang.org/download/${ZIG_VERSION}/zig-linux-x86_64-${ZIG_VERSION}.tar.xz"        | tar -xJ -C /opt     && mv "/opt/zig-linux-x86_64-${ZIG_VERSION}" /opt/zig     && curl -sSL --retry 5 --retry-all-errors -o /tmp/czb.tar.xz         "https://github.com/rust-cross/cargo-zigbuild/releases/download/v${CARGO_ZIGBUILD_VERSION}/cargo-zigbuild-x86_64-unknown-linux-musl.tar.xz"     && tar -xJf /tmp/czb.tar.xz -C /usr/local/bin --strip-components=1 "cargo-zigbuild-x86_64-unknown-linux-musl/cargo-zigbuild"     && rm /tmp/czb.tar.xz     && /opt/zig/zig version     && cargo-zigbuild --version

# ----------------------------------------------------------------------------
# STATIC MUSL BUILD
# ----------------------------------------------------------------------------
# Pinned to $BUILDPLATFORM and cross-compiled, NOT built per-target. That
# distinction is the whole cost of this stage:
#
#   emulated arm64 (no --platform pin) : killed at 6h42m, still compiling
#   cross-compiled with zig            : 7m05s
#
# Measured, after the first form was tried and abandoned. Nexus builds its
# arm64 under emulation and lives with it; its tree is far smaller than this
# one, so copying that choice without measuring was the mistake.
#
# Why zig rather than clang: Rust links musl targets with rustup's own
# self-contained CRT, so pure-Rust code cross-compiles unaided — but
# `aws-lc-sys` and `zstd-sys` are C and need a cross C compiler. `xx-info
# triple` resolves musl targets to `-linux-gnu`, and clang against Debian's
# arm64 musl sysroot fails on a missing `crtend.o`. `zig cc` ships a complete
# cross toolchain for every target it supports, which is exactly the gap.
FROM --platform=${BUILDPLATFORM:-linux/amd64} rust:1.95-slim-trixie AS builder-musl
WORKDIR /vectorizer

COPY --from=zig-toolchain /opt/zig /opt/zig
COPY --from=zig-toolchain /usr/local/bin/cargo-zigbuild /usr/local/bin/cargo-zigbuild
ENV PATH="/opt/zig:${PATH}"

ARG TARGETARCH
RUN apt-get update     && apt-get install -y --no-install-recommends        cmake protobuf-compiler pkg-config file perl make     && rm -rf /var/lib/apt/lists/*     && case "${TARGETARCH:-amd64}" in          amd64) TARGET_TRIPLE=x86_64-unknown-linux-musl ;;          arm64) TARGET_TRIPLE=aarch64-unknown-linux-musl ;;          *) echo "unsupported TARGETARCH '${TARGETARCH}'" >&2; exit 1 ;;        esac     && rustup target add "${TARGET_TRIPLE}"

ARG PROFILE=release-docker
ARG GIT_COMMIT_ID

COPY . .
# rust-embed reads dashboard/dist at compile time, so it has to exist.
COPY --from=dashboard-builder /dashboard/dist /vectorizer/dashboard/dist

# `--no-default-features`: the scratch variant is the BM25-only build.
# fastembed needs the ONNX Runtime, which links libstdc++ dynamically and
# therefore cannot go in a static binary — that variant keeps the glibc
# runtime below.
#
# The `file` check is a gate, not a diagnostic: a dynamically linked binary
# would build fine here and then fail to exec in `scratch`, where there is no
# loader. `ldd` is NOT usable for this — glibc's ldd prints "statically
# linked" and exits 0 for static-PIE binaries, so it would pass either way.
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked     unset OPENSSL_DIR OPENSSL_INCLUDE_DIR OPENSSL_LIB_DIR OPENSSL_STATIC;     case "${TARGETARCH:-amd64}" in       amd64) TARGET_TRIPLE=x86_64-unknown-linux-musl ;;       arm64) TARGET_TRIPLE=aarch64-unknown-linux-musl ;;     esac  && cargo zigbuild --profile "$PROFILE" --package vectorizer-server --bin vectorizer       --no-default-features --target "$TARGET_TRIPLE"  && PROFILE_DIR=$(if [ "$PROFILE" = dev ]; then echo debug; else echo "$PROFILE"; fi)  && cp "target/${TARGET_TRIPLE}/${PROFILE_DIR}/vectorizer" /vectorizer-static  && if ! file /vectorizer-static | grep -Eq 'static-pie linked|statically linked'; then echo "::error::binary is not statically linked — it would not exec in scratch"; file /vectorizer-static; exit 1; fi

# ============================================================================
# USER PREP — throwaway stage, only text files survive into the runtime
# ============================================================================
# `scratch` has no `mkdir`, no `chown` and no `/etc/passwd`, so `USER` cannot
# resolve a name and directories cannot be created in-image. Everything
# arrives via COPY from here.
#
# Pinned to $BUILDPLATFORM: the output is arch-neutral text and empty
# directories, so there is no reason to run it under emulation.
#
# A public base on purpose: only the text files and empty directories written
# here reach the scratch runtime, so none of this stage's packages ship, and
# building the default image needs no registry login (the Docker Hardened
# Images base at dhi.io requires a Docker Hub account). The user database is
# written explicitly — root plus `nonroot` 65532:65532, the account the glibc
# variant runs as — so both variants agree on file ownership for anyone
# switching between them on the same volume.
FROM --platform=${BUILDPLATFORM:-linux/amd64} debian:trixie-slim AS user-prep
RUN printf 'root:x:0:0:root:/root:/sbin/nologin\nnonroot:x:65532:65532:nonroot:/home/nonroot:/sbin/nologin\n' > /etc/passwd \
 && printf 'root:x:0:\nnonroot:x:65532:\n' > /etc/group \
 && mkdir -p /vectorizer/data /data /tmp-skel /home/nonroot \
 && chown -R 65532:65532 /vectorizer /data /home/nonroot \
 && chmod 1777 /tmp-skel

# ============================================================================
# RUNTIME IMAGE — scratch: zero OS packages, zero OS CVEs (default variant)
# ============================================================================
FROM scratch AS vectorizer-static

ARG BUILD_DATE
ARG GIT_COMMIT_ID

# User database so `USER nonroot` resolves, plus the writable skeleton with
# ownership already applied.
COPY --from=user-prep /etc/passwd /etc/passwd
COPY --from=user-prep /etc/group /etc/group
COPY --from=user-prep --chown=65532:65532 /vectorizer /vectorizer
COPY --from=user-prep --chown=65532:65532 /data /data
COPY --from=user-prep --chown=65532:65532 /home/nonroot /home/nonroot
COPY --from=user-prep --chown=65532:65532 /tmp-skel /tmp

# CA bundle for outbound TLS. rustls reads the system bundle through
# rustls-native-certs; without this every HTTPS request the server makes
# fails to verify, which looks like a network fault rather than a missing
# file.
COPY --from=builder-musl /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

# The static binary — the only executable in the image.
COPY --from=builder-musl --chown=65532:65532 --chmod=0755 /vectorizer-static /vectorizer/vectorizer

WORKDIR /vectorizer
USER nonroot

ENV TZ=Etc/UTC \
    RUN_MODE=production \
    VECTORIZER_HOST=0.0.0.0 \
    VECTORIZER_PORT=15002 \
    VECTORIZER_ADMIN_USERNAME=admin \
    VECTORIZER_DATA_DIR=/data

EXPOSE 15503
EXPOSE 15002

LABEL org.opencontainers.image.title="Vectorizer"
LABEL org.opencontainers.image.description="Official Vectorizer image - High-Performance Vector Database"
LABEL org.opencontainers.image.url="https://github.com/hivellm/vectorizer"
LABEL org.opencontainers.image.documentation="https://github.com/hivellm/vectorizer/docs"
LABEL org.opencontainers.image.source="https://github.com/hivellm/vectorizer"
LABEL org.opencontainers.image.vendor="HiveLLM"
LABEL org.opencontainers.image.version="${GIT_COMMIT_ID:-latest}"
LABEL org.opencontainers.image.revision="${GIT_COMMIT_ID:-unknown}"
LABEL org.opencontainers.image.created="${BUILD_DATE:-unknown}"
LABEL org.opencontainers.image.licenses="Apache-2.0"
LABEL org.opencontainers.image.base.name="scratch"

LABEL security.scan.enabled="true"
LABEL security.non-root-user="nonroot"
LABEL security.user-id="65532"

# Probes `/ready`, not `/health`. `/health` answers 200 while the collection
# catalog is still loading (issue #391), so a probe pointed at it reports a
# half-warm server as ready and the orchestrator starts routing to an instance
# still filling its store — tens of seconds on a large one. The previous
# busybox probe had exactly that bug; the compose file worked around it by
# probing /ready itself.
#
# The binary is its own probe because scratch has no wget and no shell.
HEALTHCHECK --interval=30s --timeout=5s --start-period=40s --retries=3 \
    CMD ["/vectorizer/vectorizer", "--healthcheck"]

ENTRYPOINT ["/vectorizer/vectorizer"]

# ============================================================================
# RUNTIME IMAGE - glibc (distroless cc, Debian 13) — the `-fastembed` variant
# ============================================================================
# fastembed's ONNX Runtime links libstdc++ dynamically and references glibc
# 2.38+ symbols, so that build cannot be static and cannot live in `scratch`.
# `gcr.io/distroless/cc-debian13:nonroot` is the smallest public base that
# carries what it needs:
#   - glibc 2.41, libssl3, ca-certificates, tzdata, libgcc and libstdc++;
#   - no shell and no package manager;
#   - runs as `nonroot` (UID 65532), matching every `--chown=65532:65532`
#     below and the static variant's user;
#   - pulls anonymously, so CI publishes it with nothing but GITHUB_TOKEN.
# It replaced `dhi.io/debian-base:trixie` in 3.8.2: that base needs a Docker
# Hub login to pull, which kept the `-fastembed` variant unpublished.
#
# The base is pinned by digest so the image contents are a function of
# the git commit, not the build date (spec: phase35 image-security).
# Bump procedure: docs/development/docker-builds.md § "Base digest bump".
# Pinned 2026-09-25.
# It still carries OS packages and their advisories — a known, accepted
# difference from the static default, not an oversight. `latest` points at
# the static default.
FROM gcr.io/distroless/cc-debian13:nonroot@sha256:54df941ed0d06a1bd95ef5e0ce391fd8d9f94b64782dc9a60062727849ee3f97 AS vectorizer-glibc

# Build metadata for supply chain attestation
ARG BUILD_DATE
ARG GIT_COMMIT_ID

# Copy binary and assets.
#
# Every `COPY` is `--chown=65532:65532` so the runtime `nonroot` user
# owns the whole `/vectorizer` tree — the server writes `config.yml` +
# `workspace.yml` into CWD on first boot, and without the chown those
# files land root-owned and the bootstrap fails with
# `Permission denied (os error 13)`. The first `COPY` from
# `writable-dirs` also seeds `/vectorizer` itself as nonroot-owned so
# later copies don't implicitly recreate the parent as root.
COPY --from=writable-dirs --chown=65532:65532 /vectorizer /vectorizer
COPY --from=writable-dirs --chown=65532:65532 /data /data
# phase33 §5.2 (#306): bring the optional FastEmbed model into the image,
# outside `/data` so a mounted volume cannot hide it (see the
# fastembed-models stage). `VECTORIZER_FASTEMBED_CACHE_DIR` below points the
# server at it. When ENABLE_FASTEMBED=0 the source dir is just an empty
# `/models/fastembed` placeholder, so the COPY is a cheap no-op.
COPY --from=fastembed-models --chown=65532:65532 /models/fastembed /vectorizer/models/fastembed
# phase33 §5.2 (#306): libstdc++.so.6 is a hard runtime dep of the
# ONNX Runtime that the `fastembed` Cargo feature dynamically links
# against. distroless `cc` ships a libstdc++ too; this copy pins the one
# from the builder's own toolchain (same Debian 13 release) so the binary
# runs against exactly the library it was linked with.
#
# Sourced from `/staging` (see the builder stage) so each architecture gets
# its own library at its own multiarch path. The earlier form hardcoded
# `x86_64-linux-gnu` on both sides and shipped an amd64 library inside the
# arm64 image, leaving `:3.5.0-fastembed` on linux/arm64 unable to start.
COPY --from=builder /staging/usr/ /usr/
COPY --from=builder --chown=65532:65532 /vectorizer/vectorizer /vectorizer/vectorizer
COPY --from=dashboard-builder --chown=65532:65532 /dashboard/dist /vectorizer/dashboard/dist
COPY --from=builder --chown=65532:65532 /vectorizer/config/config.example.yml /vectorizer/config/config.yml
# Static busybox for the HEALTHCHECK probe. Invoked as
# `/busybox wget ...` so the single binary covers every applet we'd
# ever need without seeding a PATH or shell inside the image. Stays
# root-owned (perms 755, world-executable) since it's exec-only.
COPY --from=busybox /bin/busybox /busybox

WORKDIR /vectorizer

# Distroless runs as nonroot (UID 65532) by default - no need to create user
# This is more secure than custom UID as it's a well-known unprivileged user

# Non-sensitive defaults only (do not bake secrets into image; pass at runtime)
# For auth, set at run: -e VECTORIZER_AUTH_ENABLED -e VECTORIZER_ADMIN_PASSWORD -e VECTORIZER_JWT_SECRET
#
# VECTORIZER_DATA_DIR pins persistent state under `/data` so the
# documented single `--volume vec-data:/data` mount survives a
# `docker compose up -d --force-recreate` (issue #300 / phase32). The
# resolver in `vectorizer-core::paths::data_dir` honours the env var
# before falling back to the XDG default; without this line every
# recreate wiped collections because the XDG path lived in the
# container's writable layer.
ENV TZ=Etc/UTC \
    RUN_MODE=production \
    VECTORIZER_HOST=0.0.0.0 \
    VECTORIZER_PORT=15002 \
    VECTORIZER_ADMIN_USERNAME=admin \
    VECTORIZER_DATA_DIR=/data \
    VECTORIZER_FASTEMBED_CACHE_DIR=/vectorizer/models/fastembed

# Ports: RPC (binary, recommended primary) listed first per
# phase6_make-rpc-default-transport. REST (15002) stays exposed for the
# dashboard, ops tooling, and browser clients.
EXPOSE 15503
EXPOSE 15002

# OpenContainer labels for better supply chain attestation
LABEL org.opencontainers.image.title="Vectorizer"
LABEL org.opencontainers.image.description="Official Vectorizer image - High-Performance Vector Database"
LABEL org.opencontainers.image.url="https://github.com/hivellm/vectorizer"
LABEL org.opencontainers.image.documentation="https://github.com/hivellm/vectorizer/docs"
LABEL org.opencontainers.image.source="https://github.com/hivellm/vectorizer"
LABEL org.opencontainers.image.vendor="HiveLLM"
LABEL org.opencontainers.image.version="${GIT_COMMIT_ID:-latest}"
LABEL org.opencontainers.image.revision="${GIT_COMMIT_ID:-unknown}"
LABEL org.opencontainers.image.created="${BUILD_DATE:-unknown}"
LABEL org.opencontainers.image.licenses="Apache-2.0"
LABEL org.opencontainers.image.base.name="gcr.io/distroless/cc-debian13:nonroot"

# Security labels
LABEL security.scan.enabled="true"
LABEL security.non-root-user="nonroot"
LABEL security.user-id="65532"

# Healthcheck via static busybox wget against the anonymous /health route.
# `--spider` issues a HEAD-style probe (no body download), exits 0 on 2xx.
# `start-period=40s` covers cold-start (dashboard mount + first auto-save
# snapshot); `interval=30s` keeps load low, `timeout=5s` detects hangs.
HEALTHCHECK --interval=30s --timeout=5s --start-period=40s --retries=3 \
    CMD ["/busybox", "wget", "-q", "--spider", "http://127.0.0.1:15002/health"]

# Direct binary execution (no shell in distroless)
ENTRYPOINT ["/vectorizer/vectorizer"]


# ============================================================================
# RUNTIME SELECTOR
# ============================================================================
# `static` (default) -> scratch, zero OS packages.
# `glibc`            -> distroless cc base, required by the fastembed/ONNX variant.
#
# BuildKit only builds the stages the selected target depends on, so the
# musl builder never runs for a glibc build and the xx builder never runs
# for a static one.
FROM vectorizer-${RUNTIME_VARIANT} AS vectorizer
