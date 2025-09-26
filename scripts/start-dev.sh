#!/bin/bash

# Vectorizer Development Start Script
# Always uses cargo run for development (never uses compiled binaries)

echo "🚀 Starting Vectorizer Servers (Development Mode)..."
echo "===================================================="

# Function to cleanup processes on exit
cleanup() {
    echo ""
    echo "Stopping development servers..."
    if [ ! -z "$VZR_PID" ]; then
        echo "Stopping vzr orchestrator (PID: $VZR_PID)"
        kill $VZR_PID 2>/dev/null || true
    fi
    if [ ! -z "$MCP_PID" ]; then
        echo "Stopping MCP server (PID: $MCP_PID)"
        kill $MCP_PID 2>/dev/null || true
    fi
    if [ ! -z "$REST_PID" ]; then
        echo "Stopping REST server (PID: $REST_PID)"
        kill $REST_PID 2>/dev/null || true
    fi
    echo "Development servers stopped."
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
    echo ""
    echo "Note: This script always uses cargo run (development mode)"
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

# Set trap to cleanup on exit
trap cleanup EXIT INT TERM

echo "📁 Workspace File: $WORKSPACE_FILE"
echo "🖥️  Operating System: $(uname -s)"
echo "🔧 Mode: Development (cargo run)"
echo "⚡ Hot reloading enabled"

# Start vzr orchestrator first (GRPC server)
echo "Starting vzr orchestrator (GRPC server)..."
cargo run --bin vzr -- start --workspace "$WORKSPACE_FILE" &
VZR_PID=$!
echo "✅ vzr orchestrator started (PID: $VZR_PID) - Port 15003 (GRPC)"

# Wait for vzr to initialize
sleep 5

# Start MCP server (GRPC client)
echo "Starting MCP server (GRPC client)..."
cargo run --bin vectorizer-mcp-server -- --workspace "$WORKSPACE_FILE" &
MCP_PID=$!
echo "✅ MCP server started (PID: $MCP_PID) - Port 15002"

# Wait a moment for MCP server to initialize
sleep 3

# Start REST server (GRPC client)
echo "Starting REST API server (GRPC client)..."
cargo run --bin vectorizer-server -- --host 127.0.0.1 --port 15001 --workspace "$WORKSPACE_FILE" &
REST_PID=$!
echo "✅ REST API server started (PID: $REST_PID) - Port 15001"

echo ""
echo "🎉 All development servers are running!"
echo "===================================================="
echo "📡 REST API: http://127.0.0.1:15001"
echo "🔧 MCP Server: ws://127.0.0.1:15002/mcp"
echo "⚡ GRPC Orchestrator: http://127.0.0.1:15003"
echo ""
echo "📋 Server PIDs:"
echo "   vzr (GRPC): $VZR_PID"
echo "   MCP: $MCP_PID"
echo "   REST: $REST_PID"
echo ""
echo "🏗️  Architecture:"
echo "   Client → REST/MCP → GRPC → vzr → Vector Store"
echo ""
echo "💡 Development Features:"
echo "   - Hot reloading enabled"
echo "   - Debug logging active"
echo "   - Source code changes trigger rebuilds"
echo ""
echo "⚡ Press Ctrl+C to stop all servers"

# Wait for all processes
wait $VZR_PID $MCP_PID $REST_PID
