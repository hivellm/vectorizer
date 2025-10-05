#!/bin/bash

# Hive Vectorizer SDK Readiness Checker
# This script checks if all SDKs are ready for publishing

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

# Function to check if file exists and is not empty
check_file() {
    local file="$1"
    local description="$2"
    
    if [ -f "$file" ] && [ -s "$file" ]; then
        print_success "$description exists and is not empty"
        return 0
    else
        print_error "$description missing or empty: $file"
        return 1
    fi
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check package.json version
check_package_version() {
    local dir="$1"
    local name="$2"
    
    if [ -f "$dir/package.json" ]; then
        local version=$(node -p "require('$dir/package.json').version" 2>/dev/null || echo "unknown")
        if [ "$version" != "unknown" ] && [ "$version" != "" ]; then
            print_success "$name version: $version"
            return 0
        else
            print_error "$name version not found or invalid"
            return 1
        fi
    else
        print_error "$name package.json not found"
        return 1
    fi
}

# Function to check Python setup.py version
check_python_version() {
    local dir="$1"
    local name="$2"
    
    if [ -f "$dir/setup.py" ]; then
        local version=$(python -c "import sys; sys.path.insert(0, '$dir'); import setup; print(setup.version)" 2>/dev/null || echo "unknown")
        if [ "$version" != "unknown" ] && [ "$version" != "" ]; then
            print_success "$name version: $version"
            return 0
        else
            print_error "$name version not found or invalid"
            return 1
        fi
    else
        print_error "$name setup.py not found"
        return 1
    fi
}

# Function to check Cargo.toml version
check_cargo_version() {
    local dir="$1"
    local name="$2"
    
    if [ -f "$dir/Cargo.toml" ]; then
        local version=$(grep '^version = ' "$dir/Cargo.toml" | sed 's/version = "\(.*\)"/\1/' 2>/dev/null || echo "unknown")
        if [ "$version" != "unknown" ] && [ "$version" != "" ]; then
            print_success "$name version: $version"
            return 0
        else
            print_error "$name version not found or invalid"
            return 1
        fi
    else
        print_error "$name Cargo.toml not found"
        return 1
    fi
}

# Function to check if tests pass
check_tests() {
    local dir="$1"
    local name="$2"
    local test_command="$3"
    
    print_status "Running tests for $name..."
    if eval "$test_command"; then
        print_success "$name tests passed"
        return 0
    else
        print_error "$name tests failed"
        return 1
    fi
}

# Function to check build process
check_build() {
    local dir="$1"
    local name="$2"
    local build_command="$3"
    
    print_status "Checking build for $name..."
    if eval "$build_command"; then
        print_success "$name builds successfully"
        return 0
    else
        print_error "$name build failed"
        return 1
    fi
}

# Main function
main() {
    local all_good=true
    
    echo "=================================================="
    echo "    Hive Vectorizer SDK Readiness Checker"
    echo "=================================================="
    echo ""
    
    # Check prerequisites
    print_status "Checking prerequisites..."
    local missing_tools=()
    
    if ! command_exists npm; then
        missing_tools+=("npm")
    fi
    
    if ! command_exists python3; then
        missing_tools+=("python3")
    fi
    
    if ! command_exists cargo; then
        missing_tools+=("cargo")
    fi
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        print_error "Missing required tools: ${missing_tools[*]}"
        all_good=false
    else
        print_success "All prerequisites are installed"
    fi
    
    echo ""
    
    # Check TypeScript SDK
    print_status "Checking TypeScript SDK..."
    if check_file "typescript/package.json" "TypeScript package.json"; then
        check_package_version "typescript" "TypeScript"
        check_file "typescript/dist/index.js" "TypeScript build output"
        check_file "typescript/README.md" "TypeScript README"
        check_file "typescript/CHANGELOG.md" "TypeScript CHANGELOG"
        
        if command_exists npm; then
            check_tests "typescript" "TypeScript" "cd typescript && npm test"
            check_build "typescript" "TypeScript" "cd typescript && npm run build"
        fi
    else
        all_good=false
    fi
    
    echo ""
    
    # Check JavaScript SDK
    print_status "Checking JavaScript SDK..."
    if check_file "javascript/package.json" "JavaScript package.json"; then
        check_package_version "javascript" "JavaScript"
        check_file "javascript/dist/index.js" "JavaScript build output"
        check_file "javascript/README.md" "JavaScript README"
        check_file "javascript/CHANGELOG.md" "JavaScript CHANGELOG"
        
        if command_exists npm; then
            check_tests "javascript" "JavaScript" "cd javascript && npm test"
            check_build "javascript" "JavaScript" "cd javascript && npm run build"
        fi
    else
        all_good=false
    fi
    
    echo ""
    
    # Check Python SDK
    print_status "Checking Python SDK..."
    if check_file "python/setup.py" "Python setup.py"; then
        check_python_version "python" "Python"
        check_file "python/README.md" "Python README"
        check_file "python/CHANGELOG.md" "Python CHANGELOG"
        check_file "python/client.py" "Python main module"
        check_file "python/models.py" "Python models"
        check_file "python/exceptions.py" "Python exceptions"
        
        if command_exists python3; then
            check_tests "python" "Python" "cd python && python run_tests.py"
        fi
    else
        all_good=false
    fi
    
    echo ""
    
    # Check Rust SDK
    print_status "Checking Rust SDK..."
    if check_file "rust/Cargo.toml" "Rust Cargo.toml"; then
        check_cargo_version "rust" "Rust"
        check_file "rust/README.md" "Rust README"
        check_file "rust/CHANGELOG.md" "Rust CHANGELOG"
        check_file "rust/src/lib.rs" "Rust main library"
        
        if command_exists cargo; then
            check_tests "rust" "Rust" "cd rust && cargo test"
            check_build "rust" "Rust" "cd rust && cargo build --release"
        fi
    else
        all_good=false
    fi
    
    echo ""
    
    # Check common files
    print_status "Checking common files..."
    check_file "README.md" "Main README"
    check_file "CHANGELOG.md" "Main CHANGELOG"
    check_file "COVERAGE_REPORT.md" "Coverage report"
    check_file "TESTING.md" "Testing documentation"
    check_file "PUBLISHING.md" "Publishing documentation"
    
    echo ""
    
    # Final result
    if [ "$all_good" = true ]; then
        print_success "All SDKs are ready for publishing!"
        echo ""
        print_status "Next steps:"
        echo "  1. Review version numbers in all package files"
        echo "  2. Update CHANGELOG files with release notes"
        echo "  3. Commit all changes to git"
        echo "  4. Run the publishing script: ./publish_sdks.sh"
        exit 0
    else
        print_error "Some issues found. Please fix them before publishing."
        echo ""
        print_status "Common fixes:"
        echo "  1. Run 'npm install' in TypeScript/JavaScript directories"
        echo "  2. Run 'pip install -r requirements.txt' in Python directory"
        echo "  3. Run 'cargo build' in Rust directory"
        echo "  4. Update version numbers if needed"
        echo "  5. Ensure all tests pass"
        exit 1
    fi
}

# Run main function
main "$@"








