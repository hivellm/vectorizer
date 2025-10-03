#!/bin/bash
# Build scripts for Vectorizer Docker images
# Usage: ./build.sh [cpu|cuda|dev-cpu|dev-cuda|all]

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

# Function to build CPU-only production image
build_cpu() {
    print_status "Building CPU-only production image..."
    docker build -f devops/docker/Dockerfile -t vectorizer:cpu-latest .
    print_success "CPU-only production image built successfully!"
}

# Function to build CUDA production image
build_cuda() {
    print_status "Building CUDA production image..."
    docker build -f devops/docker/Dockerfile.cuda -t vectorizer:cuda-latest .
    print_success "CUDA production image built successfully!"
}

# Function to build CPU-only development image
build_dev_cpu() {
    print_status "Building CPU-only development image..."
    docker build -f devops/docker/Dockerfile.dev -t vectorizer:cpu-dev .
    print_success "CPU-only development image built successfully!"
}

# Function to build CUDA development image
build_dev_cuda() {
    print_status "Building CUDA development image..."
    docker build -f devops/docker/Dockerfile.dev.cuda -t vectorizer:cuda-dev .
    print_success "CUDA development image built successfully!"
}

# Function to build all images
build_all() {
    print_status "Building all images..."
    build_cpu
    build_cuda
    build_dev_cpu
    build_dev_cuda
    print_success "All images built successfully!"
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [cpu|cuda|dev-cpu|dev-cuda|all]"
    echo ""
    echo "Options:"
    echo "  cpu      - Build CPU-only production image"
    echo "  cuda     - Build CUDA production image"
    echo "  dev-cpu  - Build CPU-only development image"
    echo "  dev-cuda - Build CUDA development image"
    echo "  all      - Build all images"
    echo ""
    echo "Examples:"
    echo "  $0 cpu        # Build CPU-only production"
    echo "  $0 cuda       # Build CUDA production"
    echo "  $0 all        # Build all images"
}

# Main script logic
case "${1:-}" in
    cpu)
        build_cpu
        ;;
    cuda)
        build_cuda
        ;;
    dev-cpu)
        build_dev_cpu
        ;;
    dev-cuda)
        build_dev_cuda
        ;;
    all)
        build_all
        ;;
    *)
        show_usage
        exit 1
        ;;
esac
