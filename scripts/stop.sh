#!/bin/bash

echo "🛑 Stopping Vectorizer ..."
echo "========================================"

STOPPED=false

# Kill processes using Vectorizer ports (primary method)
echo "Checking for processes using Vectorizer ports..."
for port in 15002; do
    PORT_PIDS=$(lsof -ti:$port 2>/dev/null || true)
    if [ ! -z "$PORT_PIDS" ]; then
        echo "Stopping server on port $port (PIDs: $PORT_PIDS)"
        echo "$PORT_PIDS" | xargs kill -15 2>/dev/null || true
        sleep 1
        # Force kill if still running
        PORT_PIDS=$(lsof -ti:$port 2>/dev/null || true)
        if [ ! -z "$PORT_PIDS" ]; then
            echo "Force stopping processes on port $port"
            echo "$PORT_PIDS" | xargs kill -9 2>/dev/null || true
        fi
        STOPPED=true
    fi
done

# Find and kill vectorizer binary processes
VECTORIZER_PIDS=$(pgrep -f "vectorizer" | grep -v "grep" || true)
if [ ! -z "$VECTORIZER_PIDS" ]; then
    echo "Stopping vectorizer processes (PIDs: $VECTORIZER_PIDS)"
    echo "$VECTORIZER_PIDS" | xargs kill -15 2>/dev/null || true
    sleep 1
    # Force kill if still running
    VECTORIZER_PIDS=$(pgrep -f "vectorizer" | grep -v "grep" || true)
    if [ ! -z "$VECTORIZER_PIDS" ]; then
        echo "Force stopping vectorizer processes"
        echo "$VECTORIZER_PIDS" | xargs kill -9 2>/dev/null || true
    fi
    STOPPED=true
fi

# Kill any cargo processes running vectorizer
CARGO_PIDS=$(pgrep -f "cargo.*run.*vectorizer" || true)
if [ ! -z "$CARGO_PIDS" ]; then
    echo "Stopping cargo processes (PIDs: $CARGO_PIDS)"
    echo "$CARGO_PIDS" | xargs kill -15 2>/dev/null || true
    sleep 1
    # Force kill if still running
    CARGO_PIDS=$(pgrep -f "cargo.*run.*vectorizer" || true)
    if [ ! -z "$CARGO_PIDS" ]; then
        echo "Force stopping cargo processes"
        echo "$CARGO_PIDS" | xargs kill -9 2>/dev/null || true
    fi
    STOPPED=true
fi

echo ""
if [ "$STOPPED" = true ]; then
    echo "🎉 Vectorizer server stopped successfully!"
else
    echo "ℹ️  No Vectorizer server was running"
fi
echo "🏗️  Architecture: Unified Server (REST/MCP on single process)"
