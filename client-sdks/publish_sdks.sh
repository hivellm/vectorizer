#!/bin/bash

# Hive Vectorizer SDK Publisher Script
# This script publishes all client SDKs to their respective package registries

set -e  # Exit on any error

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
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    local missing_tools=()
    
    if ! command_exists npm; then
        missing_tools+=("npm")
    fi
    
    if ! command_exists python3; then
        missing_tools+=("python3")
    fi
    
    if ! command_exists pip; then
        missing_tools+=("pip")
    fi
    
    if ! command_exists cargo; then
        missing_tools+=("cargo")
    fi
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        print_error "Missing required tools: ${missing_tools[*]}"
        print_error "Please install the missing tools and try again."
        exit 1
    fi
    
    print_success "All prerequisites are installed"
}

# Function to run tests before publishing
run_tests() {
    print_status "Running tests before publishing..."
    
    # TypeScript SDK Tests
    print_status "Running TypeScript SDK tests..."
    cd typescript
    if npm test; then
        print_success "TypeScript SDK tests passed"
    else
        print_error "TypeScript SDK tests failed"
        exit 1
    fi
    cd ..
    
    # JavaScript SDK Tests
    print_status "Running JavaScript SDK tests..."
    cd javascript
    if npm test; then
        print_success "JavaScript SDK tests passed"
    else
        print_error "JavaScript SDK tests failed"
        exit 1
    fi
    cd ..
    
    # Python SDK Tests
    print_status "Running Python SDK tests..."
    cd python
    if python3 run_tests.py; then
        print_success "Python SDK tests passed"
    else
        print_error "Python SDK tests failed"
        exit 1
    fi
    cd ..
    
    # Rust SDK Tests
    print_status "Running Rust SDK tests..."
    cd rust
    if cargo test; then
        print_success "Rust SDK tests passed"
    else
        print_error "Rust SDK tests failed"
        exit 1
    fi
    cd ..
    
    print_success "All tests passed! Proceeding with publishing..."
}

# Function to handle npm authentication with OTP
setup_npm_auth() {
    print_status "Setting up npm authentication..."
    
    # Check if already logged in to npm
    if npm whoami >/dev/null 2>&1; then
        print_success "Already logged in to npm as $(npm whoami)"
        return 0
    fi
    
    print_warning "Not logged in to npm. Setting up authentication..."
    
    # Check if NPM_TOKEN is available
    if [ -n "$NPM_TOKEN" ]; then
        print_status "Using NPM_TOKEN for authentication..."
        echo "//registry.npmjs.org/:_authToken=${NPM_TOKEN}" > ~/.npmrc
        if npm whoami >/dev/null 2>&1; then
            print_success "Authenticated with NPM_TOKEN as $(npm whoami)"
            return 0
        else
            print_error "NPM_TOKEN authentication failed"
            return 1
        fi
    fi
    
    # Interactive login with OTP only
    print_status "Please enter your npm credentials..."
    echo "Note: You will only need to provide your OTP (One-Time Password)"
    
    # Set browser for WSL environment
    export BROWSER=wslview
    
    # Attempt npm login
    if npm login; then
        print_success "Successfully logged in to npm as $(npm whoami)"
        return 0
    else
        print_error "npm login failed"
        return 1
    fi
}

# Function to publish TypeScript SDK
publish_typescript() {
    print_status "Publishing TypeScript SDK..."
    
    cd typescript
    
    # Setup npm authentication
    if ! setup_npm_auth; then
        print_error "Authentication failed. Publishing cancelled."
        cd ..
        return 1
    fi
    
    # Build the package
    print_status "Building TypeScript package..."

    export BROWSER=wslview
    
    npm run build
    
    # Check if package exists
    if [ -f "package.json" ]; then
        # Get version from package.json
        VERSION=$(node -p "require('./package.json').version")
        print_status "Publishing version $VERSION to npm..."
        
        # Publish to npm
        if npm publish; then
            print_success "TypeScript SDK v$VERSION published to npm!"
        else
            print_error "Failed to publish TypeScript SDK"
            cd ..
            return 1
        fi
    else
        print_error "package.json not found in TypeScript SDK directory"
        cd ..
        return 1
    fi
    
    cd ..
}

# Function to publish JavaScript SDK
publish_javascript() {
    print_status "Publishing JavaScript SDK..."
    
    cd javascript
    
    # Setup npm authentication (reuse existing auth from TypeScript)
    if ! npm whoami >/dev/null 2>&1; then
        print_status "Reusing npm authentication..."
        if ! setup_npm_auth; then
            print_error "Authentication failed. Publishing cancelled."
            cd ..
            return 1
        fi
    fi
    
    # Build the package
    print_status "Building JavaScript package..."

    export BROWSER=wslview
    
    npm run build
    
    # Check if package exists
    if [ -f "package.json" ]; then
        # Get version from package.json
        VERSION=$(node -p "require('./package.json').version")
        print_status "Publishing version $VERSION to npm..."
        
        # Publish to npm
        if npm publish; then
            print_success "JavaScript SDK v$VERSION published to npm!"
        else
            print_error "Failed to publish JavaScript SDK"
            cd ..
            return 1
        fi
    else
        print_error "package.json not found in JavaScript SDK directory"
        cd ..
        return 1
    fi
    
    cd ..
}

# Function to publish Python SDK
publish_python() {
    print_status "Publishing Python SDK..."
    
    cd python
    
    # Check if twine is installed, install with system packages override if needed
    if ! command_exists twine; then
        print_status "Installing twine for Python package publishing..."
        if pip install twine --break-system-packages; then
            print_success "Twine installed successfully"
        else
            print_warning "Failed to install twine with --break-system-packages, trying pipx..."
            if command_exists pipx; then
                pipx install twine
            else
                print_error "Neither pip nor pipx could install twine. Please install manually."
                cd ..
                return 1
            fi
        fi
    fi
    
    # Build the package
    print_status "Building Python package..."
    if python3 setup.py sdist bdist_wheel; then
        print_success "Python package built successfully"
    else
        print_error "Failed to build Python package"
        cd ..
        return 1
    fi
    
    # Check if package was built
    if [ -d "dist" ] && [ "$(ls -A dist)" ]; then
        # Get version from setup.py
        VERSION=$(python3 -c "import setup; print(setup.version)")
        print_status "Publishing version $VERSION to PyPI..."
        
        # Check if credentials are configured
        if [ ! -f "$HOME/.pypirc" ]; then
            print_warning "PyPI credentials not configured. Please run 'twine configure' or set up ~/.pypirc"
            read -p "Continue anyway? (y/N): " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                print_error "Publishing cancelled"
                cd ..
                return 1
            fi
        fi
        
        # Upload to PyPI
        if twine upload dist/*; then
            print_success "Python SDK v$VERSION published to PyPI!"
        else
            print_error "Failed to publish Python SDK"
            cd ..
            return 1
        fi
    else
        print_error "Failed to build Python package"
        cd ..
        return 1
    fi
    
    cd ..
}

# Function to publish Rust SDK
publish_rust() {
    print_status "Publishing Rust SDK..."
    
    cd rust
    
    # Check if cargo login has been run
    if [ ! -f "$HOME/.cargo/credentials" ]; then
        print_warning "Cargo credentials not configured. Please run 'cargo login' first."
        print_status "You need to:"
        echo "  1. Visit https://crates.io/settings/profile"
        echo "  2. Set and verify your email address"
        echo "  3. Run 'cargo login' with your API token"
        read -p "Continue anyway? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_error "Publishing cancelled"
            cd ..
            return 1
        fi
    fi
    
    # Check if package is ready for publishing
    print_status "Validating package..."
    if cargo package --dry-run; then
        print_success "Package validation successful"
    else
        print_error "Package validation failed"
        cd ..
        return 1
    fi
    
    # Get version from Cargo.toml
    VERSION=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    print_status "Publishing version $VERSION to crates.io..."
    
    # Publish to crates.io
    if cargo publish; then
        print_success "Rust SDK v$VERSION published to crates.io!"
    else
        print_error "Failed to publish Rust SDK"
        print_warning "Common issues:"
        echo "  - Verified email address required: https://crates.io/settings/profile"
        echo "  - API token not configured: run 'cargo login'"
        echo "  - Package name already exists or version conflict"
        cd ..
        return 1
    fi
    
    cd ..
}

# Function to display help
show_help() {
    echo "Hive Vectorizer SDK Publisher"
    echo ""
    echo "Usage: $0 [OPTIONS] [SDK]"
    echo ""
    echo "Options:"
    echo "  -h, --help     Show this help message"
    echo "  -t, --test     Run tests only (don't publish)"
    echo "  -f, --force    Skip confirmation prompts"
    echo "  --no-test      Skip running tests before publishing"
    echo ""
    echo "SDKs:"
    echo "  typescript     Publish only TypeScript SDK"
    echo "  javascript     Publish only JavaScript SDK"
    echo "  python         Publish only Python SDK"
    echo "  rust           Publish only Rust SDK"
    echo "  all            Publish all SDKs (default)"
    echo ""
    echo "Examples:"
    echo "  $0                    # Publish all SDKs"
    echo "  $0 --test             # Run tests for all SDKs"
    echo "  $0 typescript         # Publish only TypeScript SDK"
    echo "  $0 --force python     # Publish Python SDK without prompts"
}

# Main function
main() {
    local RUN_TESTS_ONLY=false
    local SKIP_TESTS=false
    local FORCE=false
    local SDK="all"
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -t|--test)
                RUN_TESTS_ONLY=true
                shift
                ;;
            -f|--force)
                FORCE=true
                shift
                ;;
            --no-test)
                SKIP_TESTS=true
                shift
                ;;
            typescript|javascript|python|rust|all)
                SDK="$1"
                shift
                ;;
            *)
                print_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    # Print banner
    echo "=================================================="
    echo "    Hive Vectorizer SDK Publisher"
    echo "=================================================="
    echo ""
    
    # Check prerequisites
    check_prerequisites
    
    # Run tests if not skipping
    if [ "$SKIP_TESTS" = false ]; then
        run_tests
        
        # If only running tests, exit here
        if [ "$RUN_TESTS_ONLY" = true ]; then
            print_success "All tests completed successfully!"
            exit 0
        fi
    else
        print_warning "Skipping tests as requested"
    fi
    
    # Confirmation prompt unless forced
    if [ "$FORCE" = false ]; then
        echo ""
        print_warning "This will publish the SDKs to their respective registries."
        print_warning "Make sure you have the necessary credentials configured."
        echo ""
        read -p "Do you want to continue? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_error "Publishing cancelled"
            exit 1
        fi
    fi
    
    # Publish based on selection
    case $SDK in
        typescript)
            publish_typescript
            ;;
        javascript)
            publish_javascript
            ;;
        python)
            publish_python
            ;;
        rust)
            publish_rust
            ;;
        all)
            print_status "Publishing all SDKs..."
            publish_typescript && \
            publish_javascript && \
            publish_python && \
            publish_rust
            ;;
    esac
    
    # Final success message
    echo ""
    print_success "SDK publishing completed successfully!"
    echo ""
    print_status "Published SDKs:"
    if [ "$SDK" = "all" ]; then
        echo "  ✅ TypeScript SDK to npm"
        echo "  ✅ JavaScript SDK to npm"
        echo "  ✅ Python SDK to PyPI"
        echo "  ✅ Rust SDK to crates.io"
    else
        echo "  ✅ $SDK SDK"
    fi
    echo ""
    print_status "Next steps:"
    echo "  1. Verify the packages are available in their registries"
    echo "  2. Update documentation with new version numbers"
    echo "  3. Announce the release to users"
}

# Run main function with all arguments
main "$@"
