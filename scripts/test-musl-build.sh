#!/bin/bash
set -e

echo "🔧 Testing musl builds locally..."
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Targets to test
TARGETS=(
    "x86_64-unknown-linux-musl"
    "aarch64-unknown-linux-musl"
)

echo "📦 Installing Rust targets..."
for target in "${TARGETS[@]}"; do
    echo "  Installing $target..."
    rustup target add "$target" || true
done
echo ""

echo "🔨 Installing cross-compilation tools..."
# For Ubuntu/Debian
if command -v apt-get &> /dev/null; then
    echo "  Installing musl-tools..."
    sudo apt-get update -qq
    sudo apt-get install -y musl-tools gcc-aarch64-linux-gnu 2>/dev/null || true
fi
echo ""

# Test each target
for target in "${TARGETS[@]}"; do
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🏗️  Building for $target"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    echo "  Command: cargo build --release --target $target --no-default-features"
    echo ""
    
    if cargo build --release --target "$target" --no-default-features; then
        echo ""
        echo -e "${GREEN}✅ Build succeeded for $target${NC}"
        echo ""
        
        # Check binary size
        if [ -f "target/$target/release/vectorizer" ]; then
            size=$(du -h "target/$target/release/vectorizer" | cut -f1)
            echo "  📊 Binary size: $size"
            
            # Test if binary is actually musl
            if command -v file &> /dev/null; then
                file_info=$(file "target/$target/release/vectorizer")
                echo "  🔍 File info: $file_info"
            fi
        fi
        echo ""
    else
        echo ""
        echo -e "${RED}❌ Build failed for $target${NC}"
        echo ""
        exit 1
    fi
done

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}🎉 All musl builds succeeded!${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📂 Binaries location:"
for target in "${TARGETS[@]}"; do
    if [ -f "target/$target/release/vectorizer" ]; then
        echo "  - target/$target/release/vectorizer"
    fi
done
echo ""
echo "💡 You can now push to GitHub with confidence!"
echo ""

