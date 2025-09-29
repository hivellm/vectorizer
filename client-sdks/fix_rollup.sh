#!/bin/bash

# Fix Rollup Build Issues
# This script fixes common rollup build problems in JavaScript SDK

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

# Function to fix rollup issues
fix_rollup() {
    print_status "Fixing rollup build issues..."
    
    # Navigate to JavaScript SDK directory
    if [ ! -d "javascript" ]; then
        print_error "JavaScript SDK directory not found"
        return 1
    fi
    
    cd javascript
    
    print_status "Cleaning existing build artifacts..."
    rm -rf node_modules package-lock.json dist
    
    print_status "Reinstalling dependencies..."
    npm install
    
    print_status "Testing build..."
    if npm run build; then
        print_success "Build successful!"
    else
        print_error "Build still failing"
        cd ..
        return 1
    fi
    
    print_status "Testing publish preparation..."
    if npm run prepublishOnly; then
        print_success "Publish preparation successful!"
    else
        print_error "Publish preparation failed"
        cd ..
        return 1
    fi
    
    cd ..
    print_success "Rollup issues fixed successfully!"
}

# Main execution
main() {
    echo "=============================================="
    echo "    Rollup Build Fix Script"
    echo "=============================================="
    echo ""
    
    fix_rollup
    
    echo ""
    print_status "You can now try publishing again:"
    echo "  - npm publish (in javascript directory)"
    echo "  - ./publish_sdks.sh javascript"
    echo "  - ./publish_sdks.sh all"
}

# Run main function
main "$@"
