#!/bin/bash

# Vectorizer Test Runner Script
# This script runs all tests with appropriate configurations for different environments

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default configuration
TEST_TYPE="all"
VERBOSE=false
PARALLEL=true
COVERAGE=false
PERFORMANCE=false
INTEGRATION=false
MCP=false
CI=false

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

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -t, --test-type TYPE     Test type: unit, integration, performance, mcp, all (default: all)"
    echo "  -v, --verbose           Enable verbose output"
    echo "  -p, --parallel          Run tests in parallel (default: true)"
    echo "  -c, --coverage          Generate test coverage report"
    echo "  -f, --performance       Run performance tests"
    echo "  -i, --integration       Run integration tests"
    echo "  -m, --mcp               Run MCP tests"
    echo "  --ci                    Run in CI mode"
    echo "  -h, --help              Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                      # Run all tests"
    echo "  $0 -t unit -v           # Run unit tests with verbose output"
    echo "  $0 -f -c                # Run performance tests with coverage"
    echo "  $0 --ci                 # Run tests in CI mode"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--test-type)
            TEST_TYPE="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -p|--parallel)
            PARALLEL=true
            shift
            ;;
        -c|--coverage)
            COVERAGE=true
            shift
            ;;
        -f|--performance)
            PERFORMANCE=true
            shift
            ;;
        -i|--integration)
            INTEGRATION=true
            shift
            ;;
        -m|--mcp)
            MCP=true
            shift
            ;;
        --ci)
            CI=true
            shift
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Set environment variables based on configuration
if [ "$CI" = true ]; then
    export CI=true
    export TEST_TIMEOUT_SECS=60
    export TEST_CONCURRENT_OPERATIONS=5
    export TEST_BATCH_SIZE=50
    export TEST_ENABLE_PERFORMANCE_CHECKS=false
    export TEST_ENABLE_DETAILED_LOGGING=true
    export RUST_LOG=warn
    print_status "Running in CI mode"
elif [ "$PERFORMANCE" = true ]; then
    export TEST_TIMEOUT_SECS=120
    export TEST_CONCURRENT_OPERATIONS=50
    export TEST_BATCH_SIZE=1000
    export TEST_ENABLE_PERFORMANCE_CHECKS=true
    export TEST_MAX_RESPONSE_TIME_MS=500
    export TEST_ENABLE_DETAILED_LOGGING=true
    export RUST_LOG=info
    print_status "Running performance tests"
elif [ "$INTEGRATION" = true ]; then
    export TEST_TIMEOUT_SECS=180
    export TEST_CONCURRENT_OPERATIONS=20
    export TEST_BATCH_SIZE=200
    export TEST_ENABLE_PERFORMANCE_CHECKS=true
    export TEST_MAX_RESPONSE_TIME_MS=2000
    export TEST_ENABLE_DETAILED_LOGGING=true
    export RUST_LOG=info
    print_status "Running integration tests"
else
    export TEST_TIMEOUT_SECS=30
    export TEST_CONCURRENT_OPERATIONS=10
    export TEST_BATCH_SIZE=100
    export TEST_ENABLE_PERFORMANCE_CHECKS=true
    export TEST_MAX_RESPONSE_TIME_MS=1000
    export TEST_ENABLE_DETAILED_LOGGING=false
    export RUST_LOG=warn
fi

# Set MCP test configuration
if [ "$MCP" = true ] || [ "$TEST_TYPE" = "mcp" ] || [ "$TEST_TYPE" = "all" ]; then
    export TEST_MCP_HOST=127.0.0.1
    export TEST_MCP_PORT=15003
    export TEST_MCP_TIMEOUT_SECS=10
    export TEST_MCP_MAX_CONNECTIONS=5
    export TEST_MCP_ENABLE_AUTH=false
    export TEST_MCP_API_KEY=test_api_key_123
fi

# Build test arguments
CARGO_TEST_ARGS=""

if [ "$VERBOSE" = true ]; then
    CARGO_TEST_ARGS="$CARGO_TEST_ARGS --verbose"
fi

if [ "$PARALLEL" = true ]; then
    CARGO_TEST_ARGS="$CARGO_TEST_ARGS -- --test-threads=4"
else
    CARGO_TEST_ARGS="$CARGO_TEST_ARGS -- --test-threads=1"
fi

# Function to run unit tests
run_unit_tests() {
    print_status "Running unit tests..."
    
    if [ "$COVERAGE" = true ]; then
        print_status "Generating coverage report..."
        cargo test --lib $CARGO_TEST_ARGS
    else
        cargo test --lib $CARGO_TEST_ARGS
    fi
    
    if [ $? -eq 0 ]; then
        print_success "Unit tests passed"
    else
        print_error "Unit tests failed"
        exit 1
    fi
}

# Function to run integration tests
run_integration_tests() {
    print_status "Running integration tests..."
    
    cargo test --test integration_tests $CARGO_TEST_ARGS
    
    if [ $? -eq 0 ]; then
        print_success "Integration tests passed"
    else
        print_error "Integration tests failed"
        exit 1
    fi
}

# Function to run API tests
run_api_tests() {
    print_status "Running API tests..."
    
    cargo test --test api_comprehensive_tests $CARGO_TEST_ARGS
    
    if [ $? -eq 0 ]; then
        print_success "API tests passed"
    else
        print_error "API tests failed"
        exit 1
    fi
}

# Function to run performance tests
run_performance_tests() {
    print_status "Running performance tests..."
    
    cargo test --test api_performance_tests $CARGO_TEST_ARGS
    
    if [ $? -eq 0 ]; then
        print_success "Performance tests passed"
    else
        print_error "Performance tests failed"
        exit 1
    fi
}

# Function to run MCP tests
run_mcp_tests() {
    print_status "Running MCP tests..."
    
    cargo test --test mcp_tests $CARGO_TEST_ARGS
    
    if [ $? -eq 0 ]; then
        print_success "MCP tests passed"
    else
        print_error "MCP tests failed"
        exit 1
    fi
}

# Function to run CI tests
run_ci_tests() {
    print_status "Running CI tests..."
    
    cargo test --test ci_tests $CARGO_TEST_ARGS
    
    if [ $? -eq 0 ]; then
        print_success "CI tests passed"
    else
        print_error "CI tests failed"
        exit 1
    fi
}

# Function to run benchmarks
run_benchmarks() {
    print_status "Running benchmarks..."
    
    cargo bench --features full
    
    if [ $? -eq 0 ]; then
        print_success "Benchmarks completed"
    else
        print_error "Benchmarks failed"
        exit 1
    fi
}

# Function to run all tests
run_all_tests() {
    print_status "Running all tests..."
    
    # Run tests in order of dependency
    run_unit_tests
    run_api_tests
    run_mcp_tests
    run_integration_tests
    
    if [ "$PERFORMANCE" = true ]; then
        run_performance_tests
    fi
    
    if [ "$CI" = true ]; then
        run_ci_tests
    fi
    
    print_success "All tests completed successfully"
}

# Function to check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    # Check if cargo is available
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo is not installed or not in PATH"
        exit 1
    fi
    
    # Check if rustc is available
    if ! command -v rustc &> /dev/null; then
        print_error "Rust compiler is not installed or not in PATH"
        exit 1
    fi
    
    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ]; then
        print_error "Cargo.toml not found. Please run this script from the project root."
        exit 1
    fi
    
    # Check if test dependencies are available
    if [ "$COVERAGE" = true ]; then
        if ! cargo install --list | grep -q "cargo-tarpaulin"; then
            print_warning "cargo-tarpaulin not found. Installing..."
            cargo install cargo-tarpaulin
        fi
    fi
    
    print_success "Prerequisites check passed"
}

# Function to clean up
cleanup() {
    print_status "Cleaning up..."
    
    # Remove temporary files
    rm -rf target/test-results/
    rm -rf test_data/
    
    print_success "Cleanup completed"
}

# Main execution
main() {
    print_status "Starting Vectorizer test suite..."
    print_status "Test type: $TEST_TYPE"
    print_status "Verbose: $VERBOSE"
    print_status "Parallel: $PARALLEL"
    print_status "Coverage: $COVERAGE"
    print_status "Performance: $PERFORMANCE"
    print_status "Integration: $INTEGRATION"
    print_status "MCP: $MCP"
    print_status "CI: $CI"
    
    # Check prerequisites
    check_prerequisites
    
    # Run tests based on type
    case $TEST_TYPE in
        "unit")
            run_unit_tests
            ;;
        "integration")
            run_integration_tests
            ;;
        "api")
            run_api_tests
            ;;
        "performance")
            run_performance_tests
            ;;
        "mcp")
            run_mcp_tests
            ;;
        "ci")
            run_ci_tests
            ;;
        "benchmarks")
            run_benchmarks
            ;;
        "all")
            run_all_tests
            ;;
        *)
            print_error "Unknown test type: $TEST_TYPE"
            show_usage
            exit 1
            ;;
    esac
    
    # Cleanup
    cleanup
    
    print_success "Test suite completed successfully!"
}

# Trap to ensure cleanup on exit
trap cleanup EXIT

# Run main function
main "$@"
