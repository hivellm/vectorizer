#!/bin/bash
# Build script for Vectorizer GUI + Vectorizer backend
# Builds both the Rust binary and the Electron GUI

set -e

echo "🔨 Building Vectorizer Complete Package"
echo "=========================================="

# Step 1: Build Vectorizer Rust binary
echo ""
echo "📦 Step 1: Building Vectorizer binary..."
cd ..
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Vectorizer build failed!"
    exit 1
fi

echo "✅ Vectorizer binary built successfully"

# Step 2: Build GUI
echo ""
echo "📦 Step 2: Building Electron GUI..."
cd gui

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "Installing dependencies..."
    pnpm install
fi

# Build frontend and main process
echo "Building frontend..."
pnpm run build

if [ $? -ne 0 ]; then
    echo "❌ GUI build failed!"
    exit 1
fi

echo "✅ GUI built successfully"

# Step 3: Package for platform
echo ""
echo "📦 Step 3: Packaging application..."

# Detect platform
case "$(uname -s)" in
    Linux*)     PLATFORM="linux";;
    Darwin*)    PLATFORM="mac";;
    CYGWIN*|MINGW*|MSYS*) PLATFORM="win";;
    *)          PLATFORM="unknown";;
esac

echo "Building for platform: $PLATFORM"

case "$PLATFORM" in
    linux)
        pnpm run electron:build:linux
        ;;
    mac)
        pnpm run electron:build:mac
        ;;
    win)
        pnpm run electron:build:win
        ;;
    *)
        echo "Unknown platform, building for current platform..."
        pnpm run electron:build
        ;;
esac

if [ $? -ne 0 ]; then
    echo "❌ Packaging failed!"
    exit 1
fi

echo ""
echo "✅ Build Complete!"
echo "📁 Output directory: gui/dist-release/"
echo ""
echo "Package includes:"
echo "  - Vectorizer GUI application"
echo "  - Vectorizer binary (embedded)"
echo "  - Default configuration"
echo ""
echo "🚀 Ready for distribution!"

