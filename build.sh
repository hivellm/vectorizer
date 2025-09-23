#!/bin/bash

# Vectorizer Build Script
# This script helps with common development tasks

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

# Function to show help
show_help() {
    echo "Vectorizer Build Script"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  build       Build the project in release mode"
    echo "  test        Run all tests"
    echo "  test-onnx   Run tests with ONNX features"
    echo "  test-real   Run tests with real models features"
    echo "  fmt         Format code with rustfmt"
    echo "  clippy      Run clippy linter"
    echo "  audit       Run cargo audit for security"
    echo "  clean       Clean build artifacts"
    echo "  docker      Build Docker image"
    echo "  docker-dev  Build development Docker image"
    echo "  all         Run build, test, fmt, clippy, audit"
    echo "  help        Show this help message"
}

# Function to build the project
build_project() {
    print_status "Building project in release mode..."
    cargo build --release --features full
    print_success "Build completed successfully!"
}

# Function to run tests
run_tests() {
    print_status "Running all tests..."
    cargo test --lib --quiet
    print_success "All tests passed!"
}

# Function to run ONNX tests
run_onnx_tests() {
    print_status "Running tests with ONNX features..."
    cargo test --features onnx-models --verbose
    print_success "ONNX tests completed!"
}

# Function to run real model tests
run_real_tests() {
    print_status "Running tests with real models features..."
    cargo test --features real-models --verbose
    print_success "Real model tests completed!"
}

# Function to format code
format_code() {
    print_status "Formatting code with rustfmt..."
    cargo fmt
    print_success "Code formatted successfully!"
}

# Function to run clippy
run_clippy() {
    print_status "Running clippy linter..."
    cargo clippy --tests
    print_success "Clippy check completed!"
}

# Function to run audit
run_audit() {
    print_status "Running cargo audit..."
    cargo audit --deny warnings
    print_success "Security audit completed!"
}

# Function to clean build artifacts
clean_project() {
    print_status "Cleaning build artifacts..."
    cargo clean
    print_success "Clean completed!"
}

# Function to build Docker image
build_docker() {
    print_status "Building Docker image..."
    docker build -t vectorizer .
    print_success "Docker image built successfully!"
}

# Function to build development Docker image
build_docker_dev() {
    print_status "Building development Docker image..."
    docker build -f Dockerfile.dev -t vectorizer-dev .
    print_success "Development Docker image built successfully!"
}

# Function to run all checks
run_all() {
    print_status "Running comprehensive build and test suite..."
    
    build_project
    run_tests
    format_code
    run_clippy
    run_audit
    
    print_success "All checks completed successfully!"
}

# Main script logic
case "${1:-help}" in
    build)
        build_project
        ;;
    test)
        run_tests
        ;;
    test-onnx)
        run_onnx_tests
        ;;
    test-real)
        run_real_tests
        ;;
    fmt)
        format_code
        ;;
    clippy)
        run_clippy
        ;;
    audit)
        run_audit
        ;;
    clean)
        clean_project
        ;;
    docker)
        build_docker
        ;;
    docker-dev)
        build_docker_dev
        ;;
    all)
        run_all
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        print_error "Unknown command: $1"
        show_help
        exit 1
        ;;
esac
