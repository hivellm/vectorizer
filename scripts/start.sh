#!/bin/bash

# Load Rust environment
source ~/.cargo/env

# Detect OS and set appropriate binary paths
detect_os() {
    case "$(uname -s)" in
        Linux*)     OS="linux";;
        Darwin*)    OS="macos";;
        CYGWIN*|MINGW*|MSYS*) OS="windows";;
        *)          OS="unknown";;
    esac
    
    # Set binary directory based on OS
    case "$OS" in
        linux|macos)
            BIN_DIR="./target/release"
            BIN_EXT=""
            ;;
        windows)
            BIN_DIR="./target/release"
            BIN_EXT=".exe"
            ;;
        *)
            echo "❌ Unsupported operating system: $(uname -s)"
            exit 1
            ;;
    esac
}

# Check if binaries exist, fallback to cargo run
check_binaries() {
    local vectorizer_bin="$BIN_DIR/vectorizer$BIN_EXT"
    
    if [ -f "$vectorizer_bin" ]; then
        USE_COMPILED=true
        echo "✅ Using compiled binary from $BIN_DIR"
    else
        USE_COMPILED=false
        echo "⚠️  Compiled binary not found, using cargo run (development mode)"
        echo "   To build binary: cargo build --release"
    fi
}

# Function to cleanup processes on exit
cleanup() {
    echo ""
    echo "Stopping server..."
    if [ ! -z "$VECTORIZER_PID" ]; then
        echo "Stopping vectorizer server (PID: $VECTORIZER_PID)"
        kill $VECTORIZER_PID 2>/dev/null || true
    fi
    echo "Server stopped."
    exit 0
}

# Function to display usage
usage() {
    echo "Usage: $0 [OPTIONS] [WORKSPACE_FILE]"
    echo ""
    echo "Options:"
    echo "  --workspace WORKSPACE_FILE    Path to vectorize-workspace.yml file"
    echo "  --daemon                      Run as daemon/service (background)"
    echo "  --help, -h                    Show this help message"
    echo "  WORKSPACE_FILE                Path to vectorize-workspace.yml file (positional)"
    echo ""
    echo "Examples:"
    echo "  $0 --workspace vectorize-workspace.yml"
    echo "  $0 --workspace vectorize-workspace.yml --daemon"
    echo "  $0 ../my-project/vectorize-workspace.yml"
    echo "  $0 --daemon                   # Uses default: vectorize-workspace.yml"
    echo "  $0                            # Uses default: vectorize-workspace.yml"
    exit 1
}

# Parse arguments
WORKSPACE_FILE="config/vectorize-workspace.yml"
DAEMON_MODE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --workspace)
            if [ -z "$2" ] || [[ "$2" == --* ]]; then
                echo "Error: --workspace requires a file argument"
                usage
            fi
            WORKSPACE_FILE="$2"
            shift 2
            ;;
        --daemon)
            DAEMON_MODE=true
            shift
            ;;
        --help|-h)
            usage
            ;;
        -*)
            echo "Unknown option: $1"
            usage
            ;;
        *)
            # Positional argument (workspace file)
            WORKSPACE_FILE="$1"
            shift
            ;;
    esac
done

# Validate workspace file exists
if [ ! -f "$WORKSPACE_FILE" ]; then
    echo "Error: Workspace file '$WORKSPACE_FILE' does not exist"
    exit 1
fi

# Detect OS and check binaries
detect_os
check_binaries

# Set trap to cleanup on exit (only if not in daemon mode)
if [ "$DAEMON_MODE" = false ]; then
    trap cleanup EXIT INT TERM
fi

echo "🚀 Starting Vectorizer Server (Unified Architecture)..."
echo "======================================================"
echo "🖥️  Operating System: $OS"
echo "🔧 Binary Mode: $([ "$USE_COMPILED" = true ] && echo "Compiled" || echo "Development")"
echo "👻 Daemon Mode: $([ "$DAEMON_MODE" = true ] && echo "Enabled" || echo "Disabled")"

# Start vectorizer server
echo "Starting vectorizer server..."
if [ "$USE_COMPILED" = true ]; then
    echo "🔍 Debug: Running compiled binary..."
    if [ "$DAEMON_MODE" = true ]; then
        # In daemon mode, run in background and don't wait
        eval "\"$BIN_DIR/vectorizer$BIN_EXT\"" &
        VECTORIZER_PID=$!
        echo "✅ Vectorizer server started in daemon mode (PID: $VECTORIZER_PID) - Port 15002"
        echo "📄 Logs: .logs/vectorizer-*.log"
        echo "🛑 Use 'scripts/stop.sh' to stop the server"
        exit 0
    else
        eval "\"$BIN_DIR/vectorizer$BIN_EXT\"" &
        VECTORIZER_PID=$!
    fi
else
    echo "🔍 Debug: Running cargo run..."
    if [ "$DAEMON_MODE" = true ]; then
        # In daemon mode, run in background and don't wait
        eval "cargo run --bin vectorizer" &
        VECTORIZER_PID=$!
        echo "✅ Vectorizer server started in daemon mode (PID: $VECTORIZER_PID) - Port 15002"
        echo "📄 Logs: .logs/vectorizer-*.log"
        echo "🛑 Use 'scripts/stop.sh' to stop the server"
        exit 0
    else
        eval "cargo run --bin vectorizer" &
        VECTORIZER_PID=$!
    fi
fi

echo "✅ Vectorizer server started (PID: $VECTORIZER_PID) - Port 15002"

echo ""
echo "🎉 Server is running!"
echo "======================================================"
echo "📡 REST API: http://127.0.0.1:15002"
echo "🔧 MCP Server: http://127.0.0.1:15002/mcp/sse"
echo "📊 Dashboard: http://127.0.0.1:15002/"
echo ""
echo "📋 Server PID: $VECTORIZER_PID"
echo ""
echo "🏗️  Architecture:"
echo "   Client → REST/MCP → Vector Store"
echo ""
echo "⚡ Press Ctrl+C to stop the server"

# Wait for vectorizer process
wait $VECTORIZER_PID
