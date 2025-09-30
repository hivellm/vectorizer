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
    local vzr_bin="$BIN_DIR/vzr$BIN_EXT"
    local mcp_bin="$BIN_DIR/vectorizer-mcp-server$BIN_EXT"
    local rest_bin="$BIN_DIR/vectorizer-server$BIN_EXT"
    
    if [ -f "$vzr_bin" ] && [ -f "$mcp_bin" ] && [ -f "$rest_bin" ]; then
        USE_COMPILED=true
        echo "✅ Using compiled binaries from $BIN_DIR"
    else
        USE_COMPILED=false
        echo "⚠️  Compiled binaries not found, using cargo run (development mode)"
        echo "   To build binaries: cargo build --release"
    fi
}

# Function to cleanup processes on exit
cleanup() {
    echo ""
    echo "Stopping servers..."
    if [ ! -z "$VZR_PID" ]; then
        echo "Stopping vzr orchestrator (PID: $VZR_PID)"
        kill $VZR_PID 2>/dev/null || true
    fi
    # MCP and REST servers are managed by vzr, no need to kill them separately
    echo "Servers stopped."
    exit 0
}

# Function to display usage
usage() {
    echo "Usage: $0 [--workspace WORKSPACE_FILE]"
    echo "       $0 WORKSPACE_FILE"
    echo ""
    echo "Options:"
    echo "  --workspace WORKSPACE_FILE    Path to vectorize-workspace.yml file"
    echo "  WORKSPACE_FILE                Path to vectorize-workspace.yml file (positional)"
    echo ""
    echo "Examples:"
    echo "  $0 --workspace vectorize-workspace.yml"
    echo "  $0 ../my-project/vectorize-workspace.yml"
    echo "  $0                             # Uses default: vectorize-workspace.yml"
    exit 1
}

# Parse arguments
WORKSPACE_FILE="config/vectorize-workspace.yml"

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

# Set trap to cleanup on exit
trap cleanup EXIT INT TERM

echo "🚀 Starting Vectorizer Servers (GRPC Architecture)..."
echo "====================================================="
echo "📁 Workspace File: $WORKSPACE_FILE"
echo "🖥️  Operating System: $OS"
echo "🔧 Binary Mode: $([ "$USE_COMPILED" = true ] && echo "Compiled" || echo "Development")"

# Start vzr orchestrator (handles all servers internally in workspace mode)
echo "Starting vzr orchestrator (GRPC server)..."
echo "🔍 Debug: About to start vzr..."
if [ "$USE_COMPILED" = true ]; then
    echo "🔍 Debug: Running compiled binary..."
    "$BIN_DIR/vzr$BIN_EXT" start --workspace "$WORKSPACE_FILE" &
else
    echo "🔍 Debug: Running cargo run..."
    rustup run nightly cargo run --bin vzr -- start --workspace "$WORKSPACE_FILE" &
fi
VZR_PID=$!
echo "✅ vzr orchestrator started (PID: $VZR_PID) - Port 15003 (GRPC)"
echo "🔍 Debug: vzr started in background, waiting for background indexing logs..."

# In workspace mode, vzr handles all servers internally
# No need to start MCP and REST servers separately
MCP_PID=""
REST_PID=""

echo ""
echo "🎉 All servers are running!"
echo "====================================================="
echo "📡 REST API: http://127.0.0.1:15001"
echo "🔧 MCP Server: ws://127.0.0.1:15002/mcp"
echo "⚡ GRPC Orchestrator: http://127.0.0.1:15003"
echo ""
echo "📋 Server PIDs:"
echo "   vzr (GRPC): $VZR_PID"
echo "   MCP: (managed by vzr)"
echo "   REST: (managed by vzr)"
echo ""
echo "🏗️  Architecture:"
echo "   Client → REST/MCP → GRPC → vzr → Vector Store"
echo ""
echo "⚡ Press Ctrl+C to stop all servers"

# Wait for vzr process (which manages all servers internally)
wait $VZR_PID
