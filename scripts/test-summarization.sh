#!/bin/bash

# Summarization Test Suite Runner
# This script runs all summarization-related tests

set -e

echo "🧪 Running Summarization Test Suite"
echo "=================================="

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

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Please run this script from the vectorizer root directory"
    exit 1
fi

# Build the project first
print_status "Building project..."
cargo build --release

if [ $? -ne 0 ]; then
    print_error "Build failed!"
    exit 1
fi

print_success "Build completed successfully"

# Run unit tests for summarization module
print_status "Running summarization unit tests..."
cargo test --lib summarization

if [ $? -ne 0 ]; then
    print_error "Summarization unit tests failed!"
    exit 1
fi

print_success "Summarization unit tests passed"

# Run GRPC tests
print_status "Running GRPC summarization tests..."
cargo test --lib grpc::summarization_tests

if [ $? -ne 0 ]; then
    print_error "GRPC summarization tests failed!"
    exit 1
fi

print_success "GRPC summarization tests passed"

# Run REST API tests
print_status "Running REST API summarization tests..."
cargo test --lib api::summarization_tests

if [ $? -ne 0 ]; then
    print_error "REST API summarization tests failed!"
    exit 1
fi

print_success "REST API summarization tests passed"

# Run MCP tests
print_status "Running MCP summarization tests..."
cargo test --lib mcp_service_tests

if [ $? -ne 0 ]; then
    print_error "MCP summarization tests failed!"
    exit 1
fi

print_success "MCP summarization tests passed"

# Run integration tests
print_status "Running summarization integration tests..."
cargo test --test summarization_integration_tests

if [ $? -ne 0 ]; then
    print_error "Summarization integration tests failed!"
    exit 1
fi

print_success "Summarization integration tests passed"

# Run all tests to ensure nothing is broken
print_status "Running full test suite to ensure no regressions..."
cargo test

if [ $? -ne 0 ]; then
    print_error "Full test suite failed!"
    exit 1
fi

print_success "Full test suite passed"

# Generate test coverage report if available
if command -v cargo-tarpaulin &> /dev/null; then
    print_status "Generating test coverage report..."
    cargo tarpaulin --out Html --output-dir coverage
    
    if [ $? -eq 0 ]; then
        print_success "Coverage report generated in coverage/ directory"
    else
        print_warning "Coverage report generation failed (optional)"
    fi
else
    print_warning "cargo-tarpaulin not found, skipping coverage report"
fi

echo ""
echo "🎉 All summarization tests completed successfully!"
echo ""
echo "Test Summary:"
echo "============="
echo "✅ Summarization unit tests"
echo "✅ GRPC summarization tests"
echo "✅ REST API summarization tests"
echo "✅ MCP summarization tests"
echo "✅ Integration tests"
echo "✅ Full test suite"
echo ""
echo "The summarization system is fully tested and ready for production!"
