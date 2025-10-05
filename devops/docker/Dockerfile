# Multi-stage Dockerfile for Vectorizer v0.29.0 (Unified Server)
# Features: REST + MCP Architecture, Simplified Deployment
# Note: Unified server architecture without gRPC dependencies

# Stage 1: Build
FROM rust:1.82-slim as builder

# Install system dependencies (unified server)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    curl \
    git \
    build-essential \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy manifest files first for better caching
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./

# Create dummy source to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release --features full && rm -rf src

# Copy source code
COPY src/ src/
COPY examples/ examples/
COPY docs/ docs/
COPY benches/ benches/
COPY tests/ tests/
COPY scripts/ scripts/
COPY config/ ./config/
COPY audit.toml ./

# Build all binaries with unified server features
RUN cargo build --release --features full --no-default-features

# Run tests to ensure build quality
RUN cargo test --lib --quiet --features full

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false vectorizer

# Set working directory
WORKDIR /app

# Copy binaries from builder stage
COPY --from=builder /app/target/release/vectorizer /app/vectorizer

# Copy configuration files
COPY --from=builder /app/config/config.yml /app/config.yml
COPY --from=builder /app/config/vectorize-workspace.yml /app/vectorize-workspace.yml

# Copy scripts for easier management
COPY --from=builder /app/scripts/ /app/scripts/
RUN chmod +x /app/scripts/*.sh

# Create data and cache directories
RUN mkdir -p /app/data /app/cache /app/logs && \
    chown -R vectorizer:vectorizer /app

# Switch to non-root user
USER vectorizer

# Expose port for unified server (REST + MCP)
EXPOSE 15002

# Health check for unified server
HEALTHCHECK --interval=30s --timeout=15s --start-period=30s --retries=5 \
    CMD curl -f http://localhost:15002/health || exit 1

# Default command - start unified server
CMD ["./vectorizer", "--config", "config.yml"]
