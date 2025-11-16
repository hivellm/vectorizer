#!/bin/bash
# Vectorizer Installation Script for Linux/macOS
# Installs Vectorizer directly from GitHub repository

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
REPO_URL="https://github.com/hivellm/vectorizer.git"
INSTALL_DIR="${VECTORIZER_INSTALL_DIR:-$HOME/.vectorizer}"
BIN_DIR="${VECTORIZER_BIN_DIR:-/usr/local/bin}"
VERSION="${VECTORIZER_VERSION:-latest}"

# Detect OS
OS="$(uname -s)"
ARCH="$(uname -m)"

echo -e "${GREEN}🚀 Vectorizer Installation Script${NC}"
echo ""

# Check for Rust
if ! command -v rustc &> /dev/null; then
    echo -e "${YELLOW}⚠️  Rust not found. Installing Rust...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Check for Git
if ! command -v git &> /dev/null; then
    echo -e "${RED}❌ Git is required but not installed.${NC}"
    echo "Please install Git and try again."
    exit 1
fi

# Create install directory
mkdir -p "$INSTALL_DIR"
cd "$INSTALL_DIR"

# Clone or update repository
if [ -d "vectorizer" ]; then
    echo -e "${YELLOW}📦 Updating existing repository...${NC}"
    cd vectorizer
    git fetch --all --tags
    if [ "$VERSION" = "latest" ]; then
        git checkout main
        git pull origin main
    else
        git checkout "v$VERSION" 2>/dev/null || git checkout "$VERSION"
    fi
else
    echo -e "${GREEN}📦 Cloning repository...${NC}"
    git clone "$REPO_URL" vectorizer
    cd vectorizer
    if [ "$VERSION" != "latest" ]; then
        git checkout "v$VERSION" 2>/dev/null || git checkout "$VERSION"
    fi
fi

# Build the project
echo -e "${GREEN}🔨 Building Vectorizer...${NC}"
cargo build --release

# Create bin directory if it doesn't exist
if [ ! -d "$BIN_DIR" ]; then
    echo -e "${YELLOW}⚠️  Creating $BIN_DIR...${NC}"
    sudo mkdir -p "$BIN_DIR"
fi

# Install binary
BINARY_PATH="$INSTALL_DIR/vectorizer/target/release/vectorizer"
if [ -f "$BINARY_PATH" ]; then
    echo -e "${GREEN}📦 Installing binary to $BIN_DIR...${NC}"
    sudo cp "$BINARY_PATH" "$BIN_DIR/vectorizer"
    sudo chmod +x "$BIN_DIR/vectorizer"
    
    # Verify installation
    if command -v vectorizer &> /dev/null; then
        INSTALLED_VERSION=$(vectorizer --version 2>/dev/null || echo "unknown")
        echo ""
        echo -e "${GREEN}✅ Vectorizer installed successfully!${NC}"
        echo ""
        echo "Version: $INSTALLED_VERSION"
        echo "Binary location: $BIN_DIR/vectorizer"
        echo ""
        echo "Run 'vectorizer --help' to get started."
    else
        echo -e "${YELLOW}⚠️  Binary installed but not in PATH.${NC}"
        echo "Please add $BIN_DIR to your PATH environment variable."
    fi
else
    echo -e "${RED}❌ Build failed. Binary not found.${NC}"
    exit 1
fi

