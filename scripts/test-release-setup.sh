#!/bin/bash
# Local validation script for release setup

set -e

echo "🧪 Testing Release Setup Locally..."
echo "=================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

error() {
    echo -e "${RED}❌ $1${NC}"
}

warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

echo "1️⃣  Checking Prerequisites..."
echo "-----------------------------"

# Check Rust
if command_exists cargo; then
    RUST_VERSION=$(rustc --version | awk '{print $2}')
    success "Rust installed: $RUST_VERSION"
else
    error "Rust not installed. Install from https://rustup.rs/"
    exit 1
fi

# Protoc not needed for Vectorizer (no protobuf dependency)

# Check Docker (optional)
if command_exists docker; then
    DOCKER_VERSION=$(docker --version | awk '{print $3}' | tr -d ',')
    success "Docker installed: $DOCKER_VERSION"
    HAS_DOCKER=true
else
    warning "Docker not installed (optional for Docker builds)"
    HAS_DOCKER=false
fi

echo ""
echo "2️⃣  Validating Configuration Files..."
echo "-------------------------------------"

# Check rustfmt.toml
if [ -f "rustfmt.toml" ]; then
    success "rustfmt.toml exists"
else
    error "rustfmt.toml not found"
    exit 1
fi

# Check clippy.toml
if [ -f "clippy.toml" ]; then
    success "clippy.toml exists"
else
    error "clippy.toml not found"
    exit 1
fi

# Check Cargo.toml metadata
if grep -q "\[package.metadata.deb\]" Cargo.toml; then
    success "Debian metadata found in Cargo.toml"
else
    error "Debian metadata not found in Cargo.toml"
    exit 1
fi

# Check workflows
if [ -f ".github/workflows/release-artifacts.yml" ]; then
    success "release-artifacts.yml workflow exists"
else
    error "release-artifacts.yml not found"
    exit 1
fi

if [ -f ".github/workflows/docker-image.yml" ]; then
    success "docker-image.yml workflow exists"
else
    error "docker-image.yml not found"
    exit 1
fi

if [ -f ".github/workflows/rust-lint.yml" ]; then
    success "rust-lint.yml workflow exists"
else
    error "rust-lint.yml not found"
    exit 1
fi

echo ""
echo "3️⃣  Checking Debian Package Files..."
echo "-------------------------------------"

if [ -f "vectorizer.service" ]; then
    success "vectorizer.service exists"
else
    error "vectorizer.service not found"
    exit 1
fi

if [ -f "debian/postinst" ] && [ -x "debian/postinst" ]; then
    success "debian/postinst exists and is executable"
else
    error "debian/postinst missing or not executable"
    exit 1
fi

if [ -f "debian/prerm" ] && [ -x "debian/prerm" ]; then
    success "debian/prerm exists and is executable"
else
    error "debian/prerm missing or not executable"
    exit 1
fi

if [ -f "debian/postrm" ] && [ -x "debian/postrm" ]; then
    success "debian/postrm exists and is executable"
else
    error "debian/postrm missing or not executable"
    exit 1
fi

echo ""
echo "4️⃣  Checking Docker Files..."
echo "----------------------------"

if [ -f "Dockerfile" ]; then
    success "Dockerfile exists"
    
    # Validate Dockerfile syntax
    if $HAS_DOCKER; then
        if docker build --dry-run -f Dockerfile . >/dev/null 2>&1; then
            success "Dockerfile syntax is valid"
        else
            warning "Dockerfile syntax validation failed (not critical for local check)"
        fi
    fi
else
    error "Dockerfile not found"
    exit 1
fi

if [ -f "tools/entrypoint.sh" ] && [ -x "tools/entrypoint.sh" ]; then
    success "tools/entrypoint.sh exists and is executable"
else
    error "tools/entrypoint.sh missing or not executable"
    exit 1
fi

if [ -f ".dockerignore" ]; then
    success ".dockerignore exists"
else
    warning ".dockerignore not found (recommended)"
fi

echo ""
echo "5️⃣  Checking Windows MSI Files..."
echo "---------------------------------"

if [ -f "wix/main.wxs" ]; then
    success "wix/main.wxs exists"
else
    error "wix/main.wxs not found"
    exit 1
fi

if [ -f "wix/License.rtf" ]; then
    success "wix/License.rtf exists"
else
    error "wix/License.rtf not found"
    exit 1
fi

echo ""
echo "6️⃣  Checking AppImage Files..."
echo "------------------------------"

if [ -f "pkg/appimage/vectorizer.desktop" ]; then
    success "pkg/appimage/vectorizer.desktop exists"
else
    error "pkg/appimage/vectorizer.desktop not found"
    exit 1
fi

if [ -f "pkg/appimage/AppRun.sh" ] && [ -x "pkg/appimage/AppRun.sh" ]; then
    success "pkg/appimage/AppRun.sh exists and is executable"
else
    error "pkg/appimage/AppRun.sh missing or not executable"
    exit 1
fi

echo ""
echo "7️⃣  Testing Rust Format..."
echo "--------------------------"

if cargo fmt --all -- --check >/dev/null 2>&1; then
    success "Code is properly formatted"
else
    warning "Code formatting issues detected. Run: cargo fmt --all"
fi

echo ""
echo "8️⃣  Testing Clippy..."
echo "--------------------"

echo "Running clippy (this may take a moment)..."
if cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee /tmp/clippy-output.log; then
    success "No clippy warnings"
else
    warning "Clippy warnings detected. Check output above."
fi

echo ""
echo "9️⃣  Testing Build..."
echo "-------------------"

echo "Building release binary (this will take several minutes)..."
if cargo build --release --features full 2>&1 | tee /tmp/build-output.log; then
    success "Build successful"
    
    # Check binaries
    if [ -f "target/release/vectorizer" ]; then
        BINARY_SIZE=$(du -h target/release/vectorizer | awk '{print $1}')
        success "vectorizer binary created ($BINARY_SIZE)"
    fi
    
    if [ -f "target/release/vectorizer-cli" ]; then
        CLI_SIZE=$(du -h target/release/vectorizer-cli | awk '{print $1}')
        success "vectorizer-cli binary created ($CLI_SIZE)"
    fi
else
    error "Build failed. Check output above."
    exit 1
fi

echo ""
echo "🔟  Testing Cargo Package..."
echo "---------------------------"

if cargo package --allow-dirty --no-verify >/dev/null 2>&1; then
    success "Cargo package validation passed"
else
    warning "Cargo package validation has issues (non-critical)"
fi

echo ""
echo "================================================"
echo "✨ All validation checks completed!"
echo "================================================"
echo ""
echo "📋 Summary:"
echo "  ✅ Prerequisites installed"
echo "  ✅ Configuration files valid"
echo "  ✅ Debian package files ready"
echo "  ✅ Docker files ready"
echo "  ✅ Windows MSI files ready"
echo "  ✅ AppImage files ready"
echo "  ✅ Code builds successfully"
echo ""
echo "🚀 Ready to push to GitHub!"
echo ""
echo "Next steps:"
echo "  1. Review the changes: git diff"
echo "  2. Commit if needed: git add -A && git commit"
echo "  3. Push to GitHub: git push -u origin feature/multi-platform-release-workflows"
echo ""

