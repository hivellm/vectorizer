# Multi-stage Dockerfile for Vectorizer
# Based on Qdrant's production-grade Docker build strategy
#
# ============================================================================
# BUILD EXAMPLES
# ============================================================================
# Local build examples:
#   docker build -t vectorizer:local .
#   docker build -t vectorizer:1.4.0 .
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
#     -v $(pwd)/vectorize-workspace.docker.yml:/vectorizer/vectorize-workspace.yml:ro \
#     -v $(pwd)/../../:/workspace:ro \
#     --name vectorizer vectorizer:latest
#
#   # PowerShell (Windows):
#   docker run -d -p 15002:15002 `
#     -v ${PWD}/data:/vectorizer/data `
#     -v ${PWD}/vectorize-workspace.docker.yml:/vectorizer/vectorize-workspace.yml:ro `
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
# Run with workspace (recommended for monorepo):
#   # Bash/Linux/Mac:
#   docker run -d -p 15002:15002 \
#     -v $(pwd)/data:/vectorizer/data \
#     -v $(pwd)/vectorize-workspace.docker.yml:/vectorizer/vectorize-workspace.yml:ro \
#     -v $(pwd)/../../:/workspace:ro \
#     -e VECTORIZER_HOST=0.0.0.0 \
#     -e VECTORIZER_PORT=15002 \
#     --name vectorizer vectorizer:latest
#
#   # PowerShell (Windows):
#   docker run -d -p 15002:15002 `
#     -v ${PWD}/data:/vectorizer/data `
#     -v ${PWD}/vectorize-workspace.docker.yml:/vectorizer/vectorize-workspace.yml:ro `
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
#         - ./vectorize-workspace.docker.yml:/vectorizer/vectorize-workspace.yml:ro
#         - ../../:/workspace:ro
#       environment:
#         - VECTORIZER_HOST=0.0.0.0
#         - VECTORIZER_PORT=15002
#         - RUN_MODE=production
#       restart: unless-stopped

# Cross-compiling using Docker multi-platform builds
FROM --platform=${BUILDPLATFORM:-linux/amd64} tonistiigi/xx AS xx

# Utilizing Docker layer caching with cargo-chef
FROM --platform=${BUILDPLATFORM:-linux/amd64} lukemathwalker/cargo-chef:latest-rust-1.88-bookworm AS chef

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
    pnpm run build

FROM chef AS builder
WORKDIR /vectorizer

COPY --from=xx / /

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

RUN xx-apt-get install -y pkg-config gcc g++ libc6-dev

# Select Cargo profile (e.g., release, dev or ci)
ARG PROFILE=release

# Enable crate features
ARG FEATURES

# Pass custom RUSTFLAGS
ARG RUSTFLAGS

# Select linker (e.g., mold, lld or empty for default)
ARG LINKER=mold

# Build dependencies with cargo-chef (cached layer)
COPY --from=planner /vectorizer/recipe.json recipe.json
RUN PKG_CONFIG="/usr/bin/$(xx-info)-pkg-config" \
    PATH="$PATH:/opt/mold/bin" \
    RUSTFLAGS="${LINKER:+-C link-arg=-fuse-ld=}$LINKER $RUSTFLAGS" \
    xx-cargo chef cook --profile $PROFILE ${FEATURES:+--features} $FEATURES --recipe-path recipe.json

# Build application
COPY . .
ARG GIT_COMMIT_ID
RUN PKG_CONFIG="/usr/bin/$(xx-info)-pkg-config" \
    PATH="$PATH:/opt/mold/bin" \
    RUSTFLAGS="${LINKER:+-C link-arg=-fuse-ld=}$LINKER $RUSTFLAGS" \
    xx-cargo build --profile $PROFILE ${FEATURES:+--features} $FEATURES --bin vectorizer \
    && PROFILE_DIR=$(if [ "$PROFILE" = dev ]; then echo debug; else echo $PROFILE; fi) \
    && mv target/$(xx-cargo --print-target-triple)/$PROFILE_DIR/vectorizer /vectorizer/vectorizer

# Generate SBOM
RUN xx-cargo install cargo-sbom && \
    cargo sbom > vectorizer.spdx.json

# Runtime image
FROM debian:13-slim AS vectorizer

# Install additional packages
ARG PACKAGES

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tzdata $PACKAGES \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* /var/cache/debconf/* /var/lib/dpkg/status-old

ARG APP=/vectorizer
ARG USER_ID=0

# Create user if non-root
RUN if [ "$USER_ID" != 0 ]; then \
        groupadd --gid "$USER_ID" vectorizer; \
        useradd --uid "$USER_ID" --gid "$USER_ID" -m vectorizer; \
        mkdir -p "$APP"/storage "$APP"/snapshots; \
        chown -R "$USER_ID:$USER_ID" "$APP"; \
    fi

COPY --from=builder --chown=$USER_ID:$USER_ID /vectorizer/vectorizer "$APP"/vectorizer
COPY --from=builder --chown=$USER_ID:$USER_ID /vectorizer/vectorizer.spdx.json "$APP"/vectorizer.spdx.json
COPY --from=builder --chown=$USER_ID:$USER_ID /vectorizer/tools/entrypoint.sh "$APP"/entrypoint.sh

WORKDIR "$APP"

# Create data directories with proper permissions (including dashboard directory)
RUN mkdir -p data storage snapshots dashboard .logs && \
    chown -R $USER_ID:$USER_ID data storage snapshots dashboard .logs

# Copy built dashboard from dashboard-builder stage (after creating directory)
COPY --from=dashboard-builder --chown=$USER_ID:$USER_ID /dashboard/dist "$APP"/dashboard/dist

# Volumes for persistent data (dashboard is now part of image, not a volume)
VOLUME ["$APP/data", "$APP/storage", "$APP/snapshots"]

USER "$USER_ID:$USER_ID"

ENV TZ=Etc/UTC \
    RUN_MODE=production \
    VECTORIZER_HOST=0.0.0.0 \
    VECTORIZER_PORT=15002

EXPOSE 15002

LABEL org.opencontainers.image.title="Vectorizer"
LABEL org.opencontainers.image.description="Official Vectorizer image - High-Performance Vector Database"
LABEL org.opencontainers.image.url="https://github.com/hivellm/vectorizer"
LABEL org.opencontainers.image.documentation="https://github.com/hivellm/vectorizer/docs"
LABEL org.opencontainers.image.source="https://github.com/hivellm/vectorizer"
LABEL org.opencontainers.image.vendor="HiveLLM"

CMD ["./entrypoint.sh"]

