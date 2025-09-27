#!/bin/bash

# Vectorizer Build Script
# Builds optimized binaries for production deployment

echo "🔨 Building Vectorizer Binaries..."
echo "=================================="

# Detect OS
case "$(uname -s)" in
    Linux*)     OS="linux";;
    Darwin*)    OS="macos";;
    CYGWIN*|MINGW*|MSYS*) OS="windows";;
    *)          OS="unknown";;
esac

echo "🖥️  Operating System: $OS"

# Build release binaries
echo "Building release binaries..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo ""
    echo "📦 Built binaries:"
    echo "   vzr (GRPC orchestrator)"
    echo "   vectorizer-server (REST API)"
    echo "   vectorizer-mcp-server (MCP server)"
    echo ""
    echo "📁 Location: target/release/"
    echo ""
    echo "🚀 Ready for production deployment!"
    echo "   Use scripts/start.sh to run with compiled binaries"
else
    echo "❌ Build failed!"
    exit 1
fi
