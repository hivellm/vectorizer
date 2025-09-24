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

# Set trap to cleanup on script exit
trap cleanup EXIT INT TERM

echo "🚀 Starting Vectorizer Servers..."
echo "=================================="

# Start MCP server first (background)
echo "Starting MCP server..."
cargo run --bin vectorizer-mcp-server -- ../gov &
MCP_PID=$!
echo "✅ MCP server started (PID: $MCP_PID) - Port 15002"

# Wait a moment for MCP server to initialize
sleep 3

# Start REST server (background)
echo "Starting REST API server..."
cargo run --bin vectorizer-server -- --host 127.0.0.1 --port 15001 --project ../gov &
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

