# Multi-stage Dockerfile for Vectorizer
# Stage 1: Build
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src/ src/
COPY examples/ examples/
COPY docs/ docs/
COPY config.example.yml ./

# Build the application
RUN cargo build --release --features full

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false vectorizer

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/vectorizer-server /app/vectorizer-server
COPY --from=builder /app/target/release/vectorizer-cli /app/vectorizer-cli
COPY --from=builder /app/config.example.yml /app/config.yml

# Create data directory
RUN mkdir -p /app/data && chown -R vectorizer:vectorizer /app

# Switch to non-root user
USER vectorizer

# Expose ports
EXPOSE 15001 15002 15003

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:15001/health || exit 1

# Default command
CMD ["./vectorizer-server", "--config", "config.yml"]