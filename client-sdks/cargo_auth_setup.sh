#!/bin/bash

# Cargo Authentication Setup Script
# This script helps set up authentication for publishing to crates.io

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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
check_cargo_auth() {
    if [ -f "$HOME/.cargo/credentials" ]; then
        print_success "Cargo credentials found"
        return 0
    else
        return 1
    fi
}

# Function to setup cargo authentication
setup_cargo_auth() {
    print_status "Setting up Cargo authentication for crates.io..."
    
    # Check if already logged in
    if check_cargo_auth; then
        print_success "Already authenticated with Cargo"
        return 0
    fi
    
    print_warning "Cargo authentication not configured"
    echo ""
    print_status "To publish to crates.io, you need to:"
    echo "  1. Create an account at https://crates.io"
    echo "  2. Verify your email address at https://crates.io/settings/profile"
    echo "  3. Get your API token from https://crates.io/settings/tokens"
    echo "  4. Run 'cargo login' with your API token"
    echo ""
    
    read -p "Do you want to run 'cargo login' now? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_status "Running 'cargo login'..."
        echo "You will be prompted for your API token from crates.io"
        echo ""
        
        if cargo login; then
            print_success "Successfully authenticated with Cargo!"
            return 0
        else
            print_error "Cargo login failed"
            return 1
        fi
    else
        print_warning "Skipping cargo login"
        print_status "You can run 'cargo login' later when you're ready"
        return 1
    fi
}

# Function to test cargo publishing
test_cargo_publish() {
    print_status "Testing cargo publishing setup..."
    
    if [ ! -d "rust" ]; then
        print_error "Rust SDK directory not found"
        return 1
    fi
    
    cd rust
    
    print_status "Running cargo package --dry-run..."
    if cargo package --dry-run; then
        print_success "Cargo package validation successful!"
        print_status "Your setup is ready for publishing"
    else
        print_error "Cargo package validation failed"
        cd ..
        return 1
    fi
    
    cd ..
    return 0
}

# Main execution
main() {
    echo "=============================================="
    echo "    Cargo Authentication Setup"
    echo "=============================================="
    echo ""
    
    if setup_cargo_auth; then
        echo ""
        if test_cargo_publish; then
            echo ""
            print_success "Cargo authentication setup completed successfully!"
            print_status "You can now publish the Rust SDK:"
            echo "  - cargo publish (in rust directory)"
            echo "  - ./publish_sdks.sh rust"
            echo "  - ./publish_sdks.sh all"
        else
            echo ""
            print_warning "Authentication setup completed, but package validation failed"
            print_status "Please check your Cargo.toml configuration"
        fi
    else
        echo ""
        print_warning "Cargo authentication setup incomplete"
        print_status "Please complete the setup manually:"
        echo "  1. Visit https://crates.io/settings/profile"
        echo "  2. Verify your email address"
        echo "  3. Get API token from https://crates.io/settings/tokens"
        echo "  4. Run 'cargo login'"
    fi
}

# Run main function
main "$@"








