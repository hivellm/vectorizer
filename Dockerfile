# Multi-stage Dockerfile for Vectorizer (GRPC Architecture)
# Stage 1: Build
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    curl \
    git \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy manifest files
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./

# Copy source code
COPY src/ src/
COPY proto/ proto/
COPY examples/ examples/
COPY docs/ docs/
COPY benches/ benches/
COPY tests/ tests/
COPY config.example.yml ./config/config.yml
COPY vectorize-workspace.yml ./config/vectorize-workspace.yml
COPY audit.toml ./

# Build all binaries with GRPC features
RUN cargo build --release --features full

# Run tests to ensure build quality
RUN cargo test --lib --quiet

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false vectorizer

# Set working directory
WORKDIR /app

# Copy binaries from builder stage
COPY --from=builder /app/target/release/vzr /app/vzr
COPY --from=builder /app/target/release/vectorizer-server /app/vectorizer-server
COPY --from=builder /app/target/release/vectorizer-mcp-server /app/vectorizer-mcp-server

# Copy configuration files
COPY --from=builder /app/config/config.yml /app/config.yml
COPY --from=builder /app/config/vectorize-workspace.yml /app/vectorize-workspace.yml

# Copy scripts for easier management
COPY scripts/start.sh /app/scripts/start.sh
COPY scripts/stop.sh /app/scripts/stop.sh
COPY scripts/status.sh /app/scripts/status.sh
RUN chmod +x /app/scripts/*.sh

# Create data directory
RUN mkdir -p /app/data && chown -R vectorizer:vectorizer /app

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
CMD ["./vzr", "start", "--workspace", "vectorize-workspace.yml"]