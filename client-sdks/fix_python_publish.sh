#!/bin/bash

# Fix Python Publishing Issues
# This script fixes common Python publishing problems

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

# Function to fix Python publishing issues
fix_python_publish() {
    print_status "Fixing Python publishing issues..."
    
    # Navigate to Python SDK directory
    if [ ! -d "python" ]; then
        print_error "Python SDK directory not found"
        return 1
    fi
    
    cd python
    
    print_status "Installing twine with system packages override..."
    if pip install twine --break-system-packages; then
        print_success "Twine installed successfully"
    else
        print_warning "Failed to install twine with --break-system-packages, trying pipx..."
        if command -v pipx >/dev/null 2>&1; then
            pipx install twine
            print_success "Twine installed with pipx"
        else
            print_error "Neither pip nor pipx could install twine"
            print_status "Please install twine manually:"
            echo "  sudo apt install python3-twine"
            echo "  or"
            echo "  pip3 install twine --user"
            cd ..
            return 1
        fi
    fi
    
    print_status "Building Python package..."
    if python3 setup.py sdist bdist_wheel; then
        print_success "Python package built successfully"
    else
        print_error "Failed to build Python package"
        cd ..
        return 1
    fi
    
    print_status "Checking built packages..."
    if [ -d "dist" ] && [ "$(ls -A dist)" ]; then
        print_success "Package files created successfully:"
        ls -la dist/
    else
        print_error "No package files found in dist/"
        cd ..
        return 1
    fi
    
    cd ..
    print_success "Python publishing issues fixed successfully!"
}

# Main execution
main() {
    echo "=============================================="
    echo "    Python Publishing Fix Script"
    echo "=============================================="
    echo ""
    
    fix_python_publish
    
    echo ""
    print_status "You can now try publishing again:"
    echo "  - ./publish_sdks.sh python"
    echo "  - ./publish_sdks.sh all"
    echo ""
    print_warning "Note: Make sure you have PyPI credentials configured:"
    echo "  - Set up ~/.pypirc file"
    echo "  - Or configure twine: twine register"
}

# Run main function
main "$@"
