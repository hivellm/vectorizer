#!/bin/bash
set -e

# Vectorizer Docker entrypoint script

# Print version information
echo "Starting Vectorizer..."
echo "Working directory: $(pwd)"
echo "User: $(whoami)"

# Set default host and port if not specified
HOST="${VECTORIZER_HOST:-0.0.0.0}"
PORT="${VECTORIZER_PORT:-15002}"

# Create necessary directories
mkdir -p data storage snapshots .logs

# Start Vectorizer with host and port
exec ./vectorizer --host "$HOST" --port "$PORT" "$@"

