#!/bin/bash

echo "📊 Vectorizer Servers Status"
echo "============================"

# Check MCP server
MCP_PIDS=$(pgrep -f "vectorizer-mcp-server" || true)
if [ ! -z "$MCP_PIDS" ]; then
    echo "✅ MCP Server: RUNNING"
    echo "   PIDs: $MCP_PIDS"
    echo "   Port: 15002 (SSE endpoint: /sse)"

    # Test MCP server health
    if curl -s --max-time 2 http://127.0.0.1:15002/sse > /dev/null 2>&1; then
        echo "   Health: 🟢 OK"
    else
        echo "   Health: 🟡 UNREACHABLE"
    fi
else
    echo "❌ MCP Server: NOT RUNNING"
fi

echo ""

# Check REST server
REST_PIDS=$(pgrep -f "vectorizer-server" || true)
if [ ! -z "$REST_PIDS" ]; then
    echo "✅ REST API Server: RUNNING"
    echo "   PIDs: $REST_PIDS"
    echo "   Port: 15001"

    # Test REST server health
    if curl -s --max-time 2 http://127.0.0.1:15001/health > /dev/null 2>&1; then
        echo "   Health: 🟢 OK"

        # Get collection stats
        COLLECTIONS=$(curl -s --max-time 2 http://127.0.0.1:15001/collections | jq -r '.collections | length' 2>/dev/null || echo "?")
        echo "   Collections: $COLLECTIONS"
    else
        echo "   Health: 🟡 UNREACHABLE"
    fi
else
    echo "❌ REST API Server: NOT RUNNING"
fi

echo ""
echo "💡 Commands:"
echo "   Start both servers: ./start.sh"
echo "   Stop all servers: ./stop.sh"
echo "   Check status: ./status.sh"
