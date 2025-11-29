#!/bin/bash
# Install LLD (LLVM Linker) for faster Rust builds
# LLD is significantly faster than the default linker for large binaries

set -e

echo "🔧 Installing LLD (LLVM Linker) for faster Rust builds..."

# Detect OS
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "📦 Detected Linux - installing via apt..."
    
    # Check if already installed
    if command -v lld &> /dev/null && command -v clang &> /dev/null; then
        echo "✅ LLD and clang are already installed"
        lld --version
    else
        echo "📥 Installing lld and clang..."
        sudo apt update
        sudo apt install -y lld clang
        echo "✅ LLD and clang installed successfully"
        lld --version
    fi
    
elif [[ "$OSTYPE" == "darwin"* ]]; then
    echo "📦 Detected macOS - installing via Homebrew..."
    
    if command -v brew &> /dev/null; then
        if command -v lld &> /dev/null; then
            echo "✅ LLD is already installed"
        else
            echo "📥 Installing llvm (includes lld)..."
            brew install llvm
            echo "✅ LLVM (with LLD) installed successfully"
        fi
    else
        echo "❌ Homebrew not found. Please install Homebrew first: https://brew.sh"
        exit 1
    fi
    
else
    echo "⚠️  Unsupported OS: $OSTYPE"
    echo "💡 For Windows, use rust-lld.exe (bundled with Rust) - no installation needed"
    exit 1
fi

echo ""
echo "✅ LLD installation complete!"
echo ""
echo "💡 LLD is now configured in .cargo/config.toml"
echo "🚀 Your Rust builds will use the faster LLD linker!"
echo ""
echo "📊 Expected improvement: 2-5x faster linking for large binaries"

