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

# Cross-compiling using Docker multi-platform builds
FROM --platform=${BUILDPLATFORM:-linux/amd64} tonistiigi/xx AS xx

# Utilizing Docker layer caching with cargo-chef.
#
# Base: `rust:1.95-slim-trixie` (Debian 13, glibc 2.40).
# - sysinfo@0.39.x (the default-features dep that backs
#   GET /metrics/runtime) requires rustc 1.95.
# - Edition 2024 (every workspace crate) requires rustc 1.85+.
# - The runtime stage is `dhi.io/debian-base:trixie` (glibc 2.40 too)
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

# Install pnpm
RUN npm install -g pnpm@latest

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
# stage downloads the bundled dense model (design.md D5:
# `all-MiniLM-L6-v2`, 384-dim) at image-build time so the first
# container boot does not need a network round-trip to Hugging Face.
# The files land at `/models/fastembed/<model-id>/` and the runtime
# stage copies them into `/data/fastembed/` so the existing resolver
# (`vectorizer_core::paths::data_dir().join("fastembed")`) picks them
# up without code changes.
#
# Default builds set `ENABLE_FASTEMBED=0`, leaving the stage as a
# no-op so the published slim image (BM25-only, ~release-docker
# profile) stays unchanged. Operators who want dense out of the box
# build with `--build-arg ENABLE_FASTEMBED=1 --build-arg
# NO_DEFAULT_FEATURES=0 --build-arg FEATURES=fastembed`.
FROM debian:bookworm-slim AS fastembed-models
ARG ENABLE_FASTEMBED=0
ARG FASTEMBED_MODEL=Xenova/all-MiniLM-L6-v2
RUN if [ "$ENABLE_FASTEMBED" = "1" ]; then \
      apt-get update && apt-get install -y --no-install-recommends curl ca-certificates && \
      DEST="/models/fastembed/${FASTEMBED_MODEL}" && \
      mkdir -p "$DEST" && \
      for f in onnx/model.onnx tokenizer.json config.json special_tokens_map.json; do \
        curl --fail --silent --show-error --location \
          -o "${DEST}/$(basename $f)" \
          "https://huggingface.co/${FASTEMBED_MODEL}/resolve/main/$f"; \
      done && \
      chown -R 65532:65532 /models; \
    else \
      mkdir -p /models/fastembed && chown -R 65532:65532 /models; \
    fi

# Static busybox — the runtime is distroless (no shell, no curl, no wget), so
# docker-compose / orchestrator healthchecks against /health need their own
# HTTP probe binary. busybox:stable-musl is a ~1 MB static binary that
# supplies `wget`, satisfying the HEALTHCHECK below without re-introducing a
# shell or a package manager.
FROM busybox:stable-musl AS busybox

# ============================================================================
# RUNTIME IMAGE - Docker Hardened Image (DHI) — Debian 13 base
# ============================================================================
# `dhi.io/debian-base:trixie` is Docker's hardened minimal Debian 13 runtime:
#   - full glibc + libssl/libcrypto + ca-certificates (our Rust binary
#     dynamically links `libssl.so.3` via indirect reqwest deps even with
#     rustls on — the Docker `static` variant is too thin and crashes with
#     `error while loading shared libraries: libssl.so.3`);
#   - no package manager baked into the runtime image (`package-manager=""`),
#     bash is included purely so docker exec / Kubernetes liveness probes
#     can shell in when debugging;
#   - runs as `nonroot` (UID 65532, same as Google's distroless so the
#     existing `--chown=65532:65532` lines and compose `user: root` override
#     stay untouched);
#   - Docker-signed + Scout-approved by default (flips Scout "Approved Base
#     Images" and "Up-to-Date Base Images" from Unknown → Compliant);
#   - rebuilt weekly, Debian 13 (trixie) base carries fewer transitive CVEs
#     than the Debian 12 bookworm base gcr.io/distroless is on;
#   - CIS-compliant, end-of-life 2028-08-09.
# Pull requires `docker login dhi.io` with Docker Hub credentials; CI does
# the same before `docker buildx build --push`.
#
# The base is pinned by digest so the image contents are a function of
# the git commit, not the build date (spec: phase35 image-security).
# Bump procedure: docs/development/docker-builds.md § "Base digest bump".
# Pinned 2026-07-11 — carries openssl 3.5.6-1~deb13u2+dhi0 (fixes
# CVE-2026-45447, CVE-2026-7383, CVE-2026-9076, CVE-2026-34180).
FROM dhi.io/debian-base:trixie@sha256:17dc256ec746f1168765cab1fc552418b60d09de8337d03ffa92cc529ed2ea7a AS vectorizer

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
# phase33 §5.2 (#306): bring the optional FastEmbed model into
# /data/fastembed so the resolver picks it up on first boot. When
# ENABLE_FASTEMBED=0 (default) the source dir is just an empty
# `/models/fastembed` placeholder, so the COPY is a cheap no-op.
COPY --from=fastembed-models --chown=65532:65532 /models/fastembed /data/fastembed
# phase33 §5.2 (#306): libstdc++.so.6 is a hard runtime dep of the
# ONNX Runtime that the `fastembed` Cargo feature dynamically links
# against. The DHI base ships full glibc + libssl but not libstdc++,
# so a fastembed-enabled binary boots with
# `error while loading shared libraries: libstdc++.so.6` without
# this copy. For the BM25-only default build the lib is unused but
# costs ~2 MB on the image — cheaper than gating the COPY on an
# ARG and risking the conditional drifting out of sync with the
# Cargo features.
COPY --from=builder /usr/lib/x86_64-linux-gnu/libstdc++.so.6 /usr/lib/x86_64-linux-gnu/libstdc++.so.6
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
    VECTORIZER_DATA_DIR=/data

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
LABEL org.opencontainers.image.base.name="dhi.io/debian-base:trixie"

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

