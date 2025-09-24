#!/bin/bash

echo "🛑 Stopping Vectorizer Servers..."
echo "================================="

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
CARGO_PIDS=$(pgrep -f "cargo.*run.*--bin.*vectorizer" || true)
if [ ! -z "$CARGO_PIDS" ]; then
    echo "Stopping cargo processes (PIDs: $CARGO_PIDS)"
    echo "$CARGO_PIDS" | xargs kill 2>/dev/null || true
    echo "✅ Cargo processes stopped"
fi

echo ""
echo "🎉 All servers stopped successfully!"
