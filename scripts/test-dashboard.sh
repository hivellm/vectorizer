#!/bin/bash
# Test script for dashboard integration
# Tests if the dashboard build is correct and server can serve it

set -e

echo "🧪 Testing Dashboard Integration..."
echo ""

# Check if dashboard/dist exists
if [ ! -d "dashboard/dist" ]; then
    echo "❌ Error: dashboard/dist directory not found"
    echo "💡 Building dashboard first..."
    cd dashboard
    npm run build
    cd ..
fi

# Check if index.html exists
if [ ! -f "dashboard/dist/index.html" ]; then
    echo "❌ Error: dashboard/dist/index.html not found"
    echo "💡 Building dashboard first..."
    cd dashboard
    npm run build
    cd ..
fi

# Check if assets directory exists
if [ ! -d "dashboard/dist/assets" ]; then
    echo "⚠️  Warning: dashboard/dist/assets directory not found"
    echo "💡 Dashboard build may be incomplete"
fi

# Verify index.html has correct base path
if grep -q 'href="/dashboard/' dashboard/dist/index.html; then
    echo "✅ Dashboard base path is correct (/dashboard/)"
else
    echo "⚠️  Warning: Dashboard base path may be incorrect"
    echo "   Expected: /dashboard/"
    echo "   Check dashboard/dist/index.html"
fi

# Check file sizes
echo ""
echo "📊 Dashboard build statistics:"
du -sh dashboard/dist/ 2>/dev/null || echo "   Could not get size"
echo "   Files: $(find dashboard/dist -type f | wc -l)"
echo ""

# Verify key files exist
echo "🔍 Verifying key files:"
files=(
    "dashboard/dist/index.html"
    "dashboard/dist/assets/css/index-*.css"
    "dashboard/dist/assets/js/index-*.js"
)

for file in "${files[@]}"; do
    if ls $file 1> /dev/null 2>&1; then
        echo "   ✅ $(basename $(ls $file | head -1))"
    else
        echo "   ❌ $(basename $file) - NOT FOUND"
    fi
done

echo ""
echo "✅ Dashboard build verification complete!"
echo ""
echo "💡 To test the server:"
echo "   1. Build the Rust server: cargo build --release"
echo "   2. Run the server: ./target/release/vectorizer"
echo "   3. Open browser: http://localhost:15002/dashboard/"
echo ""

