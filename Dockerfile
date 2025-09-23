# ---- Builder Stage ----
# Use the official Rust image with the slim-bullseye variant for a smaller build environment.
FROM rust:1-slim-bullseye AS builder

# Create a new empty workspace and set it as the working directory.
WORKDIR /usr/src/vectorizer

# Copy manifests to cache dependencies.
COPY ./Cargo.toml ./Cargo.lock ./

# Create a dummy main.rs to allow caching dependencies without the full source code.
# This is a common optimization technique.
RUN mkdir -p src && echo "fn main() {println!(\"dependency caching...\");}" > src/main.rs
# Build dependencies and cache them.
RUN cargo build --release
# Clean up the dummy source file.
RUN rm -f src/main.rs

# Copy the actual source code.
COPY ./src ./src

# Build the application. This will be much faster as dependencies are already cached.
RUN rm -f target/release/deps/vectorizer*
RUN cargo build --release

# ---- Runtime Stage ----
# Use a minimal, secure base image for the final container.
FROM debian:slim-bullseye

# Set the working directory.
WORKDIR /app

# Copy the compiled binary from the builder stage.
COPY --from=builder /usr/src/vectorizer/target/release/vectorizer .

# Copy the example configuration file.
COPY ./config.example.yml ./config.example.yml

# Expose the server port and the dashboard port.
EXPOSE 15001
EXPOSE 15002

# Define the default command to run the application.
# The user is expected to provide a `config.yml` when running the container.
CMD ["./vectorizer", "--config", "config.yml"]
