#!/bin/bash
set -e

echo "🚀 Quick build test - checking if musl configuration works..."
echo ""

# Just check if the build would work (cargo check is much faster than build)
echo "🔍 Running cargo check with musl configuration..."
echo "   Command: cargo check --no-default-features"
echo ""

if cargo check --no-default-features; then
    echo ""
    echo "✅ Configuration is valid! The build should work in CI."
    echo ""
    echo "💡 To do a full build test (takes longer):"
    echo "   ./scripts/test-musl-build.sh"
    echo ""
else
    echo ""
    echo "❌ Configuration check failed!"
    echo ""
    echo "Fix the errors before pushing to GitHub."
    echo ""
    exit 1
fi

