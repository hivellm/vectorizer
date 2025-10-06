#!/bin/bash
# Build scripts for Vectorizer Docker images
# Usage: ./build.sh [prod|dev|all]

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
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to build production image
build_prod() {
    print_status "Building production image..."
    docker build -f devops/docker/Dockerfile -t vectorizer:v0.29.0 .
    print_success "Production image built successfully!"
}

# Function to build development image
build_dev() {
    print_status "Building development image..."
    docker build -f devops/docker/Dockerfile.dev -t vectorizer:v0.29.0-dev .
    print_success "Development image built successfully!"
}

# Function to build all images
build_all() {
    print_status "Building all images..."
    build_prod
    build_dev
    print_success "All images built successfully!"
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [prod|dev|all]"
    echo ""
    echo "Options:"
    echo "  prod     - Build production image"
    echo "  dev      - Build development image"
    echo "  all      - Build all images"
    echo ""
    echo "Examples:"
    echo "  $0 prod       # Build production image"
    echo "  $0 dev        # Build development image"
    echo "  $0 all        # Build all images"
}

# Main script logic
case "${1:-}" in
    prod)
        build_prod
        ;;
    dev)
        build_dev
        ;;
    all)
        build_all
        ;;
    *)
        show_usage
        exit 1
        ;;
esac
