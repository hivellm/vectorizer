#!/bin/bash
set -e

# Vectorizer Docker entrypoint script

# Print version information
echo "Starting Vectorizer..."
echo "Working directory: $(pwd)"
echo "User: $(whoami)"

# Check if config file exists
if [ ! -f "config.yml" ]; then
    echo "Warning: config.yml not found. Using default configuration."
fi

# Set default host and port if not specified
export VECTORIZER_HOST="${VECTORIZER_HOST:-0.0.0.0}"
export VECTORIZER_PORT="${VECTORIZER_PORT:-15002}"

# Create necessary directories
mkdir -p storage snapshots

# Start Vectorizer
exec ./vectorizer --config config.yml "$@"

