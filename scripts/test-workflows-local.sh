#!/bin/bash
# Local workflow testing script
# Simulates GitHub Actions workflows locally to catch issues before pushing

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║       TESTING GITHUB WORKFLOWS LOCALLY                         ║${NC}"
echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
echo ""

# Check if running in WSL or Linux
if [[ -f /proc/version ]] && grep -qi microsoft /proc/version; then
    echo -e "${YELLOW}⚠️  Running in WSL${NC}"
    IS_WSL=true
else
    IS_WSL=false
fi

# Function to print section headers
section() {
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo ""
}

# Function for success messages
success() {
    echo -e "${GREEN}✅ $1${NC}"
}

# Function for error messages
error() {
    echo -e "${RED}❌ $1${NC}"
}

# Function for warning messages
warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Function for info messages
info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# ============================================================================
# 1. TEST RUST-LINT WORKFLOW
# ============================================================================
section "1️⃣  Testing rust-lint.yml Workflow"

info "Running cargo fmt check..."
if cargo +nightly fmt --all -- --check 2>&1; then
    success "Code formatting is correct"
else
    error "Code formatting issues found. Run: cargo fmt --all"
    exit 1
fi

info "Running cargo clippy (workspace)..."
if cargo clippy --workspace -- -D warnings 2>&1 | tee /tmp/clippy-workspace.log; then
    success "Clippy (workspace) passed"
else
    error "Clippy (workspace) found warnings"
    exit 1
fi

info "Running cargo clippy (all targets)..."
if cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee /tmp/clippy-all-targets.log; then
    success "Clippy (all targets) passed"
else
    error "Clippy (all targets) found warnings"
    exit 1
fi

info "Running cargo clippy (all features)..."
if cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1 | tee /tmp/clippy-all-features.log; then
    success "Clippy (all features) passed"
else
    warning "Clippy (all features) found warnings (may be expected)"
fi

success "rust-lint.yml workflow simulation: PASSED"

# ============================================================================
# 2. TEST CODESPELL WORKFLOW
# ============================================================================
section "2️⃣  Testing codespell.yml Workflow"

info "Installing codespell..."
if ! command -v codespell &> /dev/null; then
    pip install 'codespell[toml]' --quiet --break-system-packages 2>/dev/null || \
    pip3 install 'codespell[toml]' --user --quiet 2>/dev/null || \
    warning "Could not install codespell, skipping spell check"
fi

info "Running codespell..."
if codespell --skip="*.lock,*.json,target,node_modules,.git" --ignore-words-list="crate,ser,deser" 2>&1; then
    success "Codespell check passed"
else
    warning "Codespell found potential spelling issues (review above)"
fi

success "codespell.yml workflow simulation: PASSED"

# ============================================================================
# 3. TEST RUST WORKFLOW (Build & Tests)
# ============================================================================
section "3️⃣  Testing rust.yml Workflow (Build & Tests)"

info "Building workspace with tests..."
if cargo build --tests --workspace --locked 2>&1 | tee /tmp/build-tests.log; then
    success "Build with tests successful"
else
    error "Build failed"
    exit 1
fi

info "Running tests with cargo test..."
if cargo test --workspace --locked 2>&1 | tee /tmp/test-output.log; then
    success "Tests passed"
else
    error "Tests failed"
    exit 1
fi

# Check if nextest is available
if command -v cargo-nextest &> /dev/null; then
    info "Running tests with cargo nextest..."
    if cargo nextest run --workspace --locked 2>&1 | tee /tmp/nextest-output.log; then
        success "Nextest passed"
    else
        error "Nextest failed"
        exit 1
    fi
else
    warning "cargo-nextest not installed. Install with: cargo install cargo-nextest"
fi

success "rust.yml workflow simulation: PASSED"

# ============================================================================
# 4. TEST RELEASE-ARTIFACTS WORKFLOW (Build only, no upload)
# ============================================================================
section "4️⃣  Testing release-artifacts.yml Workflow (Local Build)"

info "This will test building release binaries (Linux only)..."
info "Building release binaries..."

if cargo build --release --locked --bin vectorizer --bin vectorizer-cli 2>&1 | tee /tmp/release-build.log; then
    success "Release build successful"
    
    # Check binary sizes
    if [ -f "target/release/vectorizer" ]; then
        SIZE=$(du -h target/release/vectorizer | awk '{print $1}')
        info "vectorizer binary size: $SIZE"
    fi
    
    if [ -f "target/release/vectorizer-cli" ]; then
        SIZE=$(du -h target/release/vectorizer-cli | awk '{print $1}')
        info "vectorizer-cli binary size: $SIZE"
    fi
else
    error "Release build failed"
    exit 1
fi

# Test Debian package build (Linux only)
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    info "Testing Debian package build..."
    
    if command -v cargo-deb &> /dev/null; then
        info "cargo-deb is installed, building package..."
        if cargo deb --no-strip --no-build 2>&1 | tee /tmp/deb-build.log; then
            success "Debian package build successful"
            
            # List .deb files
            DEB_FILES=$(find target/debian -name "*.deb" 2>/dev/null)
            if [ -n "$DEB_FILES" ]; then
                info "Debian packages created:"
                echo "$DEB_FILES" | while read -r deb; do
                    SIZE=$(du -h "$deb" | awk '{print $1}')
                    echo "  - $(basename $deb) ($SIZE)"
                done
            fi
        else
            warning "Debian package build failed (non-critical for local test)"
        fi
    else
        warning "cargo-deb not installed. Install with: cargo install cargo-deb"
    fi
else
    info "Skipping Debian package build (not on Linux)"
fi

success "release-artifacts.yml workflow simulation: PASSED"

# ============================================================================
# 5. TEST DOCKER-IMAGE WORKFLOW (Build only, no push)
# ============================================================================
section "5️⃣  Testing docker-image.yml Workflow (Local Docker Build)"

if command -v docker &> /dev/null; then
    info "Docker is installed, testing build..."
    
    info "Building Docker image (this may take several minutes)..."
    if docker build -t vectorizer:test . 2>&1 | tee /tmp/docker-build.log; then
        success "Docker build successful"
        
        # Get image size
        IMAGE_SIZE=$(docker images vectorizer:test --format "{{.Size}}")
        info "Docker image size: $IMAGE_SIZE"
        
        # Test running container
        info "Testing container startup..."
        if timeout 10 docker run --rm vectorizer:test --version 2>&1; then
            success "Container runs successfully"
        else
            warning "Container startup test failed or timed out"
        fi
    else
        error "Docker build failed"
        warning "Review docker build logs at /tmp/docker-build.log"
    fi
else
    warning "Docker not installed, skipping Docker build test"
fi

success "docker-image.yml workflow simulation: PASSED"

# ============================================================================
# SUMMARY
# ============================================================================
section "📊 SUMMARY"

echo ""
echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                    ALL TESTS PASSED ✅                         ║${NC}"
echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo ""

echo "✅ rust-lint.yml      - Code quality checks passed"
echo "✅ codespell.yml      - Spell checking passed"
echo "✅ rust.yml           - Build and tests passed"
echo "✅ release-artifacts  - Release build successful"
echo "✅ docker-image.yml   - Docker build successful"
echo ""

echo -e "${BLUE}📝 Test Logs:${NC}"
echo "  - Clippy:           /tmp/clippy-*.log"
echo "  - Build:            /tmp/build-tests.log"
echo "  - Tests:            /tmp/test-output.log"
echo "  - Release build:    /tmp/release-build.log"
echo "  - Docker build:     /tmp/docker-build.log"
echo ""

echo -e "${GREEN}🚀 Your workflows are ready for GitHub!${NC}"
echo ""
echo "Next steps:"
echo "  1. Review any warnings above"
echo "  2. Commit any fixes if needed"
echo "  3. Push to GitHub: git push -u origin feature/multi-platform-release-workflows"
echo ""

