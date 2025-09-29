# Multi-stage Dockerfile for Vectorizer v0.20.0 (CPU-Only Production)
# Features: GRPC Architecture, Automatic Summarization, Batch Operations
# Note: This version does NOT include CUDA support for maximum compatibility

# Stage 1: Build
FROM rust:1.82-slim as builder

# Install system dependencies (CPU-only, no CUDA)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    curl \
    git \
    build-essential \
    cmake \
    protobuf-compiler \
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
COPY proto/ proto/
COPY examples/ examples/
COPY docs/ docs/
COPY benches/ benches/
COPY tests/ tests/
COPY scripts/ scripts/
COPY config/ ./config/
COPY audit.toml ./

# Build all binaries with full features (CPU-only, no CUDA)
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
COPY --from=builder /app/target/release/vzr /app/vzr
COPY --from=builder /app/target/release/vectorizer-server /app/vectorizer-server
COPY --from=builder /app/target/release/vectorizer-mcp-server /app/vectorizer-mcp-server
COPY --from=builder /app/target/release/vectorizer-cli /app/vectorizer-cli

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

# Expose ports for GRPC architecture
EXPOSE 15001 15002 15003

# Health check for GRPC architecture
HEALTHCHECK --interval=30s --timeout=15s --start-period=30s --retries=5 \
    CMD curl -f http://localhost:15001/api/v1/health && \
        curl -f http://localhost:15002/health && \
        curl -f http://localhost:15003/health || exit 1

# Default command - start all services using vzr orchestrator
CMD ["./vzr", "start", "--workspace", "vectorize-workspace.yml", "--daemon"]
