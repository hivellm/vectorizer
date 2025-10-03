#!/bin/bash

# NPM Authentication Script - OTP Only
# This script simplifies npm authentication to only request OTP

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

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

# Function to check if already logged in
check_npm_auth() {
    if npm whoami >/dev/null 2>&1; then
        print_success "Already logged in to npm as $(npm whoami)"
        return 0
    else
        return 1
    fi
}

# Function to setup authentication with OTP
setup_npm_auth() {
    print_status "Setting up npm authentication..."
    
    # Check if already logged in
    if check_npm_auth; then
        return 0
    fi
    
    print_warning "Not logged in to npm. Setting up authentication..."
    
    # Check if NPM_TOKEN is available
    if [ -n "$NPM_TOKEN" ]; then
        print_status "Using NPM_TOKEN for authentication..."
        echo "//registry.npmjs.org/:_authToken=${NPM_TOKEN}" > ~/.npmrc
        if check_npm_auth; then
            return 0
        else
            print_error "NPM_TOKEN authentication failed"
            return 1
        fi
    fi
    
    # Interactive login with OTP only
    print_status "Starting npm login process..."
    echo ""
    print_warning "You will be prompted for:"
    echo "  1. Username"
    echo "  2. Password" 
    echo "  3. Email"
    echo "  4. OTP (One-Time Password) - This is the main step"
    echo ""
    print_status "Setting browser to 'wslview' for WSL environment..."
    
    # Set browser for WSL environment
    export BROWSER=wslview
    
    # Attempt npm login
    print_status "Running 'npm login'..."
    if npm login; then
        print_success "Successfully logged in to npm as $(npm whoami)"
        return 0
    else
        print_error "npm login failed"
        return 1
    fi
}

# Main execution
main() {
    echo "=============================================="
    echo "    NPM Authentication - OTP Only"
    echo "=============================================="
    echo ""
    
    if setup_npm_auth; then
        echo ""
        print_success "Authentication completed successfully!"
        print_status "You can now publish packages using:"
        echo "  - npm publish"
        echo "  - ./publish_sdks.sh typescript"
        echo "  - ./publish_sdks.sh javascript"
        echo "  - ./publish_sdks.sh all"
    else
        echo ""
        print_error "Authentication failed!"
        print_status "You can try:"
        echo "  1. Run this script again"
        echo "  2. Set NPM_TOKEN environment variable"
        echo "  3. Run 'npm login' manually"
        exit 1
    fi
}

# Run main function
main "$@"
