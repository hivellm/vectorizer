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
mkdir -p data .logs

# Debug: check if binary exists and is executable
echo "Binary info:"
ls -lh ./vectorizer || echo "Binary not found!"
which ./vectorizer || echo "Binary not in PATH"

# Start Vectorizer with host and port
# Capture stderr to see any errors
./vectorizer --host "$HOST" --port "$PORT" "$@" 2>&1 || {
    EXIT_CODE=$?
    echo "Vectorizer exited with code: $EXIT_CODE"
    exit $EXIT_CODE
}

