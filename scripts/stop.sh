#!/bin/bash

echo "🛑 Stopping Vectorizer Servers (GRPC Architecture)..."
echo "====================================================="

# Find and kill vzr orchestrator processes
VZR_PIDS=$(pgrep -f "vzr" || true)
if [ ! -z "$VZR_PIDS" ]; then
    echo "Stopping vzr orchestrator (PIDs: $VZR_PIDS)"
    echo "$VZR_PIDS" | xargs kill 2>/dev/null || true
    echo "✅ vzr orchestrator stopped"
else
    echo "ℹ️  No vzr orchestrator running"
fi

# Find and kill vectorizer-mcp-server processes
MCP_PIDS=$(pgrep -f "vectorizer-mcp-server" || true)
if [ ! -z "$MCP_PIDS" ]; then
    echo "Stopping MCP servers (PIDs: $MCP_PIDS)"
    echo "$MCP_PIDS" | xargs kill 2>/dev/null || true
    echo "✅ MCP servers stopped"
else
    echo "ℹ️  No MCP servers running"
fi

# Find and kill vectorizer-server processes
REST_PIDS=$(pgrep -f "vectorizer-server" || true)
if [ ! -z "$REST_PIDS" ]; then
    echo "Stopping REST servers (PIDs: $REST_PIDS)"
    echo "$REST_PIDS" | xargs kill 2>/dev/null || true
    echo "✅ REST servers stopped"
else
    echo "ℹ️  No REST servers running"
fi

# Also kill any cargo processes that might be running the servers
CARGO_PIDS=$(pgrep -f "cargo.*run.*--bin.*vectorizer\|cargo.*run.*--bin.*vzr" || true)
if [ ! -z "$CARGO_PIDS" ]; then
    echo "Stopping cargo processes (PIDs: $CARGO_PIDS)"
    echo "$CARGO_PIDS" | xargs kill 2>/dev/null || true
    echo "✅ Cargo processes stopped"
fi

# Kill any processes using the ports
echo "Checking for processes using Vectorizer ports..."
for port in 15001 15002 15003; do
    PORT_PIDS=$(lsof -ti:$port 2>/dev/null || true)
    if [ ! -z "$PORT_PIDS" ]; then
        echo "Killing processes using port $port (PIDs: $PORT_PIDS)"
        echo "$PORT_PIDS" | xargs kill -9 2>/dev/null || true
    fi
done

echo ""
echo "🎉 All Vectorizer servers stopped successfully!"
echo "🏗️  Architecture: vzr (GRPC) + MCP + REST servers"
