#!/bin/bash

# Function to cleanup processes on exit
cleanup() {
    echo ""
    echo "Stopping servers..."
    if [ ! -z "$MCP_PID" ]; then
        echo "Stopping MCP server (PID: $MCP_PID)"
        kill $MCP_PID 2>/dev/null || true
    fi
    if [ ! -z "$REST_PID" ]; then
        echo "Stopping REST server (PID: $REST_PID)"
        kill $REST_PID 2>/dev/null || true
    fi
    echo "Servers stopped."
    exit 0
}

# Function to display usage
usage() {
    echo "Usage: $0 [--project PROJECT_DIR]"
    echo "       $0 PROJECT_DIR"
    echo ""
    echo "Options:"
    echo "  --project PROJECT_DIR    Directory containing the project to index"
    echo "  PROJECT_DIR              Directory containing the project to index (positional)"
    echo ""
    echo "Examples:"
    echo "  $0 --project ../gov"
    echo "  $0 ../my-project"
    echo "  $0                         # Uses default: ../gov"
    exit 1
}

# Parse arguments
PROJECT_DIR="../gov"

while [[ $# -gt 0 ]]; do
    case $1 in
        --project)
            if [ -z "$2" ] || [[ "$2" == --* ]]; then
                echo "Error: --project requires a directory argument"
                usage
            fi
            PROJECT_DIR="$2"
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
            # Positional argument (project directory)
            PROJECT_DIR="$1"
            shift
            ;;
    esac
done

# Validate project directory exists
if [ ! -d "$PROJECT_DIR" ]; then
    echo "Error: Project directory '$PROJECT_DIR' does not exist"
    exit 1
fi

# Set trap to cleanup on exit
trap cleanup EXIT INT TERM

echo "🚀 Starting Vectorizer Servers..."
echo "=================================="
echo "📁 Project Directory: $PROJECT_DIR"

# Start MCP server first (background)
echo "Starting MCP server..."
cargo run --bin vectorizer-mcp-server -- "$PROJECT_DIR" &
MCP_PID=$!
echo "✅ MCP server started (PID: $MCP_PID) - Port 15002"

# Wait a moment for MCP server to initialize
sleep 3

# Start REST server (background)
echo "Starting REST API server..."
cargo run --bin vectorizer-server -- --host 127.0.0.1 --port 15001 --project "$PROJECT_DIR" &
REST_PID=$!
echo "✅ REST API server started (PID: $REST_PID) - Port 15001"

echo ""
echo "🎉 Both servers are running!"
echo "=================================="
echo "📡 REST API: http://127.0.0.1:15001"
echo "🔧 MCP Server: http://127.0.0.1:15002/sse"
echo ""
echo "📋 Server PIDs:"
echo "   MCP: $MCP_PID"
echo "   REST: $REST_PID"
echo ""
echo "⚡ Press Ctrl+C to stop both servers"

# Wait for both processes
wait $MCP_PID $REST_PID
