# Simple Dockerfile for Vectorizer
FROM rust:1.88-bookworm AS builder

WORKDIR /build

# Copy source code
COPY . .

# Build the application
RUN cargo build --release --bin vectorizer

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /vectorizer

# Copy binary from builder
COPY --from=builder /build/target/release/vectorizer /vectorizer/vectorizer

# Copy configuration
COPY config.yml /vectorizer/config.yml
COPY vectorize-workspace.yml /vectorizer/vectorize-workspace.yml

# Copy entrypoint
COPY tools/entrypoint.sh /vectorizer/entrypoint.sh
RUN chmod +x /vectorizer/entrypoint.sh /vectorizer/vectorizer

# Create data directories
RUN mkdir -p /vectorizer/data /vectorizer/dashboard /vectorizer/.logs

# Environment variables
ENV RUST_LOG=debug \
    RUST_BACKTRACE=1 \
    VECTORIZER_HOST=0.0.0.0 \
    VECTORIZER_PORT=15002

# Volumes
VOLUME ["/vectorizer/data", "/vectorizer/dashboard"]

# Expose ports
EXPOSE 15002

# Start
CMD ["./entrypoint.sh"]

