#!/bin/bash
# Setup script for sccache compilation cache
# This significantly speeds up Rust builds, especially in large projects

set -e

echo "🔧 Setting up sccache for Rust builds..."

# Check if sccache is installed
if ! command -v sccache &> /dev/null; then
    echo "📦 Installing sccache..."
    cargo install sccache
else
    echo "✅ sccache is already installed"
fi

# Get sccache path
SCCACHE_PATH=$(which sccache)
echo "📍 sccache found at: $SCCACHE_PATH"

# Set RUSTC_WRAPPER environment variable
export RUSTC_WRAPPER="$SCCACHE_PATH"
echo "✅ RUSTC_WRAPPER set to: $RUSTC_WRAPPER"

# Check sccache stats
echo ""
echo "📊 sccache statistics:"
sccache --show-stats || echo "⚠️  sccache stats not available (first run)"

echo ""
echo "✅ sccache setup complete!"
echo ""
echo "💡 To make this permanent, add to your ~/.bashrc or ~/.zshrc:"
echo "   export RUSTC_WRAPPER=\"$SCCACHE_PATH\""
echo ""
echo "💡 Or run this script before building:"
echo "   source scripts/setup-sccache.sh"
echo ""
echo "🚀 Your Rust builds will now use sccache for caching!"

