# Multi-stage Dockerfile for Vectorizer
# Based on Qdrant's production-grade Docker build strategy
#
# Local build examples:
#   docker build -t vectorizer:local .
#   docker build -t vectorizer:1.2.3 .
#   docker buildx build --platform linux/amd64,linux/arm64 -t vectorizer:latest .
#
# Multi-platform build:
#   docker buildx build --platform linux/amd64,linux/arm64 -t ghcr.io/hivellm/vectorizer:latest --push .

# Cross-compiling using Docker multi-platform builds
FROM --platform=${BUILDPLATFORM:-linux/amd64} tonistiigi/xx AS xx

# Utilizing Docker layer caching with cargo-chef
FROM --platform=${BUILDPLATFORM:-linux/amd64} lukemathwalker/cargo-chef:latest-rust-1.88-bookworm AS chef

FROM chef AS planner
WORKDIR /vectorizer
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

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

# Create data directories with proper permissions
RUN mkdir -p data storage snapshots dashboard .logs && \
    chown -R $USER_ID:$USER_ID data storage snapshots dashboard .logs

# Volumes for persistent data
VOLUME ["$APP/data", "$APP/storage", "$APP/snapshots", "$APP/dashboard"]

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

