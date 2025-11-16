#!/bin/bash
# Vectorizer Installation Script for Linux/macOS
# Installs Vectorizer directly from GitHub repository
# Based on Bun's installation script pattern

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO_URL="https://github.com/hivellm/vectorizer.git"
INSTALL_DIR="${VECTORIZER_INSTALL_DIR:-$HOME/.vectorizer}"
BIN_DIR="${VECTORIZER_BIN_DIR:-/usr/local/bin}"
VERSION="${VECTORIZER_VERSION:-latest}"
NO_PATH_UPDATE="${VECTORIZER_NO_PATH_UPDATE:-false}"

# Detect OS and Architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

# Check architecture compatibility
if [[ "$ARCH" != "x86_64" && "$ARCH" != "arm64" && "$ARCH" != "aarch64" ]]; then
    echo -e "${RED}❌ Unsupported architecture: $ARCH${NC}"
    echo "Vectorizer currently supports x86_64 and arm64/aarch64 architectures."
    exit 1
fi

echo -e "${GREEN}🚀 Vectorizer Installation Script${NC}"
echo -e "${BLUE}OS: $OS | Architecture: $ARCH${NC}"
echo ""

# Check for Rust
if ! command -v rustc &> /dev/null; then
    echo -e "${YELLOW}⚠️  Rust not found. Installing Rust...${NC}"
    if curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; then
        source "$HOME/.cargo/env" 2>/dev/null || true
        export PATH="$HOME/.cargo/bin:$PATH"
    else
        echo -e "${RED}❌ Failed to install Rust.${NC}"
        echo "Please install Rust manually from https://rustup.rs/"
        exit 1
    fi
else
    RUST_VERSION=$(rustc --version 2>/dev/null || echo "unknown")
    echo -e "${GREEN}✓ Rust found: $RUST_VERSION${NC}"
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
    git fetch --all --tags --quiet
    
    # Check if version is specified
    if [ "$VERSION" = "latest" ]; then
        git checkout main --quiet
        git pull origin main --quiet
        DISPLAY_VERSION="latest (main branch)"
    else
        # Normalize version format
        if [[ "$VERSION" =~ ^v?[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
            VERSION_TAG="v${VERSION#v}"
        else
            VERSION_TAG="$VERSION"
        fi
        
        if git checkout "$VERSION_TAG" --quiet 2>/dev/null; then
            DISPLAY_VERSION="$VERSION_TAG"
        elif git checkout "$VERSION" --quiet 2>/dev/null; then
            DISPLAY_VERSION="$VERSION"
        else
            echo -e "${RED}❌ Version/tag '$VERSION' not found.${NC}"
            echo "Available tags:"
            git tag --list | tail -10
            exit 1
        fi
    fi
else
    echo -e "${GREEN}📦 Cloning repository...${NC}"
    git clone "$REPO_URL" vectorizer --quiet
    cd vectorizer
    
    if [ "$VERSION" != "latest" ]; then
        # Normalize version format
        if [[ "$VERSION" =~ ^v?[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
            VERSION_TAG="v${VERSION#v}"
        else
            VERSION_TAG="$VERSION"
        fi
        
        if git checkout "$VERSION_TAG" --quiet 2>/dev/null; then
            DISPLAY_VERSION="$VERSION_TAG"
        elif git checkout "$VERSION" --quiet 2>/dev/null; then
            DISPLAY_VERSION="$VERSION"
        else
            echo -e "${RED}❌ Version/tag '$VERSION' not found.${NC}"
            exit 1
        fi
    else
        DISPLAY_VERSION="latest (main branch)"
    fi
fi

echo -e "${GREEN}✓ Repository ready (version: $DISPLAY_VERSION)${NC}"

# Build the project
echo -e "${GREEN}🔨 Building Vectorizer (this may take a few minutes)...${NC}"
if cargo build --release --quiet 2>&1 | grep -v "Compiling\|Finished"; then
    # If quiet mode shows errors, rebuild without quiet
    echo -e "${YELLOW}⚠️  Build had warnings, rebuilding with output...${NC}"
    cargo build --release
else
    echo -e "${GREEN}✓ Build completed successfully${NC}"
fi

# Create bin directory if it doesn't exist
if [ ! -d "$BIN_DIR" ]; then
    echo -e "${YELLOW}⚠️  Creating $BIN_DIR...${NC}"
    sudo mkdir -p "$BIN_DIR"
fi

# Install binary
BINARY_PATH="$INSTALL_DIR/vectorizer/target/release/vectorizer"
if [ -f "$BINARY_PATH" ]; then
    echo -e "${GREEN}📦 Installing binary to $BIN_DIR...${NC}"
    
    # Check if binary is already running
    if pgrep -f "$BIN_DIR/vectorizer" > /dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  Vectorizer is currently running. Please stop it before installing.${NC}"
        exit 1
    fi
    
    # Backup existing binary if it exists
    if [ -f "$BIN_DIR/vectorizer" ]; then
        OLD_VERSION=$("$BIN_DIR/vectorizer" --version 2>/dev/null || echo "unknown")
        echo -e "${BLUE}Backing up existing installation (version: $OLD_VERSION)${NC}"
        sudo mv "$BIN_DIR/vectorizer" "$BIN_DIR/vectorizer.old" 2>/dev/null || true
    fi
    
    sudo cp "$BINARY_PATH" "$BIN_DIR/vectorizer"
    sudo chmod +x "$BIN_DIR/vectorizer"
    
    # Remove backup if installation succeeded
    sudo rm -f "$BIN_DIR/vectorizer.old" 2>/dev/null || true
    
    # Verify installation
    if command -v vectorizer &> /dev/null; then
        INSTALLED_VERSION=$(vectorizer --version 2>/dev/null || echo "unknown")
        echo ""
        echo -e "${GREEN}✅ Vectorizer installed successfully!${NC}"
        echo ""
        echo -e "${GREEN}Version: $INSTALLED_VERSION${NC}"
        echo "Binary location: $BIN_DIR/vectorizer"
        echo ""
        
        # Check if there's another vectorizer in PATH
        EXISTING_VECTORIZER=$(which -a vectorizer 2>/dev/null | grep -v "$BIN_DIR/vectorizer" | head -1 || true)
        if [[ -n "$EXISTING_VECTORIZER" ]]; then
            echo -e "${YELLOW}⚠️  Note: Another vectorizer is already in PATH at: $EXISTING_VECTORIZER${NC}"
            echo -e "${YELLOW}Typing 'vectorizer' will use the existing installation.${NC}"
            echo ""
        fi
        
        echo "Run 'vectorizer --help' to get started."
        echo ""
        if [[ "$NO_PATH_UPDATE" == "false" ]]; then
            echo -e "${BLUE}💡 Tip: Restart your terminal or run 'source ~/.bashrc' to use vectorizer immediately.${NC}"
        fi
    else
        if [[ "$NO_PATH_UPDATE" == "false" ]]; then
            echo -e "${YELLOW}⚠️  Binary installed but not in PATH.${NC}"
            echo "Please add $BIN_DIR to your PATH environment variable:"
            echo "  export PATH=\"$BIN_DIR:\$PATH\""
            echo ""
            echo "Or restart your terminal."
        else
            echo -e "${GREEN}✅ Binary installed to $BIN_DIR/vectorizer${NC}"
            echo -e "${YELLOW}Skipped adding to PATH (VECTORIZER_NO_PATH_UPDATE=true)${NC}"
        fi
    fi
else
    echo -e "${RED}❌ Build failed. Binary not found.${NC}"
    exit 1
fi

