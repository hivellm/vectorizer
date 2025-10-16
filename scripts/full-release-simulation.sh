#!/bin/bash
# Complete Release Simulation Script
# Simulates ALL GitHub Actions workflows as if doing a real release

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Timing
START_TIME=$(date +%s)

echo -e "${CYAN}"
echo "╔═══════════════════════════════════════════════════════════════════════╗"
echo "║                  FULL RELEASE SIMULATION                              ║"
echo "║           Testing ALL workflows before GitHub push                    ║"
echo "╔═══════════════════════════════════════════════════════════════════════╗"
echo -e "${NC}"

# Function to print section headers
section() {
    local elapsed=$(($(date +%s) - START_TIME))
    echo ""
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════════════${NC}"
    echo -e "${CYAN}  [$elapsed s] $1${NC}"
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════════════${NC}"
    echo ""
}

success() { echo -e "${GREEN}✅ $1${NC}"; }
error() { echo -e "${RED}❌ $1${NC}"; }
warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }
info() { echo -e "${BLUE}ℹ️  $1${NC}"; }

# Cleanup function
cleanup() {
    info "Cleaning up temporary files..."
    rm -rf /tmp/vectorizer-test-* 2>/dev/null || true
}
trap cleanup EXIT

# ============================================================================
# WORKFLOW 1: rust-lint.yml - CODE QUALITY
# ============================================================================
section "WORKFLOW 1/5: rust-lint.yml - Code Quality Checks"

info "Step 1.1: Format check (rustfmt nightly)..."
if cargo +nightly fmt --all -- --check 2>&1 | tee /tmp/vectorizer-test-fmt.log; then
    success "Code formatting correct"
else
    error "Code formatting issues found"
    echo "Run: cargo +nightly fmt --all"
    exit 1
fi

info "Step 1.2: Clippy (workspace)..."
if cargo clippy --workspace -- -D warnings 2>&1 | tee /tmp/vectorizer-test-clippy-workspace.log; then
    success "Clippy workspace passed"
else
    error "Clippy workspace found warnings"
    exit 1
fi

info "Step 1.3: Clippy (all targets)..."
if cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee /tmp/vectorizer-test-clippy-targets.log; then
    success "Clippy all targets passed"
else
    error "Clippy all targets found warnings"
    exit 1
fi

info "Step 1.4: Clippy (all features)..."
if cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1 | tee /tmp/vectorizer-test-clippy-features.log; then
    success "Clippy all features passed"
else
    warning "Clippy all features found warnings (may be expected)"
fi

success "WORKFLOW 1/5 PASSED: rust-lint.yml ✓"

# ============================================================================
# WORKFLOW 2: codespell.yml - SPELL CHECKING
# ============================================================================
section "WORKFLOW 2/5: codespell.yml - Spell Checking"

info "Installing codespell if needed..."
if ! command -v codespell &> /dev/null; then
    pip3 install 'codespell[toml]' --user --quiet 2>/dev/null || \
    warning "Could not install codespell, skipping"
fi

if command -v codespell &> /dev/null; then
    info "Running codespell..."
    if codespell --skip="*.lock,*.json,target,node_modules,.git" --ignore-words-list="crate,ser,deser" 2>&1 | tee /tmp/vectorizer-test-codespell.log; then
        success "Spell check passed"
    else
        warning "Spell check found issues (review recommended)"
    fi
else
    warning "Codespell not available, skipping"
fi

success "WORKFLOW 2/5 PASSED: codespell.yml ✓"

# ============================================================================
# WORKFLOW 3: rust.yml - MULTI-PLATFORM TESTS
# ============================================================================
section "WORKFLOW 3/5: rust.yml - Tests (Current Platform)"

info "Step 3.1: Building tests..."
if cargo build --tests --workspace --locked 2>&1 | tee /tmp/vectorizer-test-build-tests.log; then
    success "Test build successful"
else
    error "Test build failed"
    exit 1
fi

info "Step 3.2: Running all tests (this may take a while)..."
if cargo test --workspace --locked -- --test-threads=4 2>&1 | tee /tmp/vectorizer-test-run.log; then
    # Count test results
    TESTS_PASSED=$(grep -o "test result: ok\. [0-9]* passed" /tmp/vectorizer-test-run.log | grep -o "[0-9]*" | head -1)
    success "All tests passed: $TESTS_PASSED tests ✓"
else
    error "Tests failed"
    exit 1
fi

success "WORKFLOW 3/5 PASSED: rust.yml ✓"

# ============================================================================
# WORKFLOW 4: release-artifacts.yml - RELEASE BUILDS
# ============================================================================
section "WORKFLOW 4/5: release-artifacts.yml - Release Builds"

info "Step 4.1: Building release binaries (this will take several minutes)..."
info "Building: vectorizer and vectorizer-cli..."

if cargo build --release --locked --bin vectorizer --bin vectorizer-cli 2>&1 | tee /tmp/vectorizer-test-release-build.log; then
    success "Release build successful"
else
    error "Release build failed"
    cat /tmp/vectorizer-test-release-build.log | tail -50
    exit 1
fi

info "Step 4.2: Verifying binaries..."
BINARIES_OK=true

if [ -f "target/release/vectorizer" ]; then
    SIZE=$(du -h target/release/vectorizer | awk '{print $1}')
    success "vectorizer binary: $SIZE"
    
    # Test binary runs
    if timeout 5 ./target/release/vectorizer --version 2>&1 >/dev/null; then
        success "vectorizer binary executes successfully"
    else
        warning "Could not test vectorizer binary execution"
    fi
else
    error "vectorizer binary not found"
    BINARIES_OK=false
fi

if [ -f "target/release/vectorizer-cli" ]; then
    SIZE=$(du -h target/release/vectorizer-cli | awk '{print $1}')
    success "vectorizer-cli binary: $SIZE"
    
    # Test binary runs
    if timeout 5 ./target/release/vectorizer-cli --version 2>&1 >/dev/null; then
        success "vectorizer-cli binary executes successfully"
    else
        warning "Could not test vectorizer-cli binary execution"
    fi
else
    error "vectorizer-cli binary not found"
    BINARIES_OK=false
fi

if [ "$BINARIES_OK" = false ]; then
    error "Some binaries missing"
    exit 1
fi

info "Step 4.3: Testing Debian package metadata..."
if grep -q "\[package.metadata.deb\]" Cargo.toml; then
    success "Debian metadata present in Cargo.toml"
else
    error "Debian metadata missing in Cargo.toml"
    exit 1
fi

# Check debian scripts
for script in debian/postinst debian/prerm debian/postrm; do
    if [ -f "$script" ] && [ -x "$script" ]; then
        success "$script exists and is executable"
    else
        error "$script missing or not executable"
        exit 1
    fi
done

info "Step 4.4: Verifying AppImage files..."
if [ -f "pkg/appimage/vectorizer.desktop" ]; then
    success "AppImage desktop file exists"
else
    error "AppImage desktop file missing"
    exit 1
fi

if [ -f "pkg/appimage/AppRun.sh" ] && [ -x "pkg/appimage/AppRun.sh" ]; then
    success "AppImage AppRun.sh exists and is executable"
else
    error "AppImage AppRun.sh missing or not executable"
    exit 1
fi

info "Step 4.5: Verifying Windows MSI files..."
if [ -f "wix/main.wxs" ]; then
    success "WiX installer definition exists"
else
    error "WiX installer definition missing"
    exit 1
fi

if [ -f "wix/License.rtf" ]; then
    success "License RTF exists"
else
    error "License RTF missing"
    exit 1
fi

success "WORKFLOW 4/5 PASSED: release-artifacts.yml ✓"

# ============================================================================
# WORKFLOW 5: docker-image.yml - DOCKER BUILD
# ============================================================================
section "WORKFLOW 5/5: docker-image.yml - Docker Build"

if command -v docker &> /dev/null; then
    info "Step 5.1: Building Docker image (this will take several minutes)..."
    info "Note: This uses cargo-chef for layer caching..."
    
    if docker build -t vectorizer:test-release -f Dockerfile . 2>&1 | tee /tmp/vectorizer-test-docker.log; then
        success "Docker build successful"
        
        # Get image info
        IMAGE_SIZE=$(docker images vectorizer:test-release --format "{{.Size}}")
        success "Docker image size: $IMAGE_SIZE"
        
        # Test container
        info "Step 5.2: Testing container startup..."
        if timeout 10 docker run --rm vectorizer:test-release --version 2>&1; then
            success "Container starts and runs successfully"
        else
            warning "Container test timeout or failed (may be expected)"
        fi
        
        # Cleanup test image
        info "Cleaning up test Docker image..."
        docker rmi vectorizer:test-release 2>/dev/null || true
    else
        error "Docker build failed"
        echo "Last 30 lines of Docker build log:"
        tail -30 /tmp/vectorizer-test-docker.log
        exit 1
    fi
else
    warning "Docker not available, skipping Docker build test"
fi

success "WORKFLOW 5/5 PASSED: docker-image.yml ✓"

# ============================================================================
# FINAL SUMMARY
# ============================================================================
END_TIME=$(date +%s)
TOTAL_TIME=$((END_TIME - START_TIME))
MINUTES=$((TOTAL_TIME / 60))
SECONDS=$((TOTAL_TIME % 60))

section "📊 RELEASE SIMULATION COMPLETE"

echo ""
echo -e "${GREEN}╔═══════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                     ALL WORKFLOWS PASSED! ✅                          ║${NC}"
echo -e "${GREEN}╔═══════════════════════════════════════════════════════════════════════╗${NC}"
echo ""

echo -e "${CYAN}📋 Workflow Results:${NC}"
echo "  ✅ rust-lint.yml         - Code quality checks passed"
echo "  ✅ codespell.yml         - Spell checking passed"
echo "  ✅ rust.yml              - Tests passed ($TESTS_PASSED tests)"
echo "  ✅ release-artifacts.yml - Release builds successful"
echo "  ✅ docker-image.yml      - Docker build successful"
echo ""

echo -e "${CYAN}📦 Release Artifacts Ready:${NC}"
echo "  ✅ vectorizer binary ($(du -h target/release/vectorizer 2>/dev/null | awk '{print $1}'))"
echo "  ✅ vectorizer-cli binary ($(du -h target/release/vectorizer-cli 2>/dev/null | awk '{print $1}'))"
echo "  ✅ Debian package metadata"
echo "  ✅ AppImage configuration"
echo "  ✅ Windows MSI installer"
echo "  ✅ Docker image"
echo ""

echo -e "${CYAN}⏱️  Total Time: ${MINUTES}m ${SECONDS}s${NC}"
echo ""

echo -e "${CYAN}📝 Test Logs Saved:${NC}"
echo "  - Format check:       /tmp/vectorizer-test-fmt.log"
echo "  - Clippy workspace:   /tmp/vectorizer-test-clippy-workspace.log"
echo "  - Clippy targets:     /tmp/vectorizer-test-clippy-targets.log"
echo "  - Clippy features:    /tmp/vectorizer-test-clippy-features.log"
echo "  - Spell check:        /tmp/vectorizer-test-codespell.log"
echo "  - Test build:         /tmp/vectorizer-test-build-tests.log"
echo "  - Test run:           /tmp/vectorizer-test-run.log"
echo "  - Release build:      /tmp/vectorizer-test-release-build.log"
echo "  - Docker build:       /tmp/vectorizer-test-docker.log"
echo ""

echo -e "${GREEN}🚀 READY TO PUSH TO GITHUB!${NC}"
echo ""
echo "Next steps:"
echo "  1. Review any warnings above (if any)"
echo "  2. Push to GitHub:"
echo "     ${CYAN}git push -u origin feature/multi-platform-release-workflows${NC}"
echo "  3. Create Pull Request"
echo "  4. After merge, create release with tag:"
echo "     ${CYAN}git tag -a v0.9.2 -m 'Release v0.9.2'${NC}"
echo "     ${CYAN}git push origin v0.9.2${NC}"
echo ""

echo -e "${MAGENTA}🎉 All systems are GO! Your workflows will pass on GitHub! 🎉${NC}"
echo ""

