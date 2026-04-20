#!/bin/bash

# Vectorizer Benchmark Runner
# This script provides convenient commands for running different types of benchmarks

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
PROFILE="quick"
FEATURES=""
BENCHMARK=""
VERBOSE=false

# Function to print usage
usage() {
    echo "Usage: $0 [OPTIONS] [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  all              Run all benchmarks"
    echo "  core             Run core operation benchmarks"
    echo "  search           Run search performance benchmarks"
    echo "  gpu              Run GPU acceleration benchmarks"
    echo "  replication      Run replication benchmarks"
    echo "  storage          Run storage benchmarks"
    echo "  quantization     Run quantization benchmarks"
    echo "  embeddings       Run embedding benchmarks"
    echo "  performance      Run performance/scale benchmarks"
    echo "  integration      Run integration benchmarks"
    echo ""
    echo "Options:"
    echo "  -p, --profile    Benchmark profile (quick|comprehensive|regression)"
    echo "  -f, --features   Cargo features to enable"
    echo "  -b, --benchmark  Specific benchmark to run"
    echo "  -v, --verbose    Verbose output"
    echo "  -h, --help       Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 core                    # Run core benchmarks with quick profile"
    echo "  $0 gpu -f hive-gpu-metal   # Run GPU benchmarks with Metal support"
    echo "  $0 all -p comprehensive    # Run all benchmarks with comprehensive profile"
    echo "  $0 -b query_cache_bench    # Run specific benchmark"
}

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

# Function to run benchmarks
run_benchmarks() {
    local category=$1
    local cmd="cargo bench"
    
    # Add features if specified
    if [ -n "$FEATURES" ]; then
        cmd="$cmd --features $FEATURES"
    fi
    
    # Add specific benchmark if specified
    if [ -n "$BENCHMARK" ]; then
        cmd="$cmd --bench $BENCHMARK"
    elif [ -n "$category" ] && [ "$category" != "all" ]; then
        # Map category to benchmark pattern
        case $category in
            "core")
                cmd="$cmd --bench query_cache_bench --bench update_bench --bench core_operations_bench --bench cache_bench"
                ;;
            "gpu")
                cmd="$cmd --bench metal_hnsw_search_bench --bench metal_native_search_bench --bench cuda_bench"
                ;;
            "replication")
                cmd="$cmd --bench replication_bench"
                ;;
            "storage")
                cmd="$cmd --bench storage_bench"
                ;;
            "quantization")
                cmd="$cmd --bench quantization_bench"
                ;;
            "embeddings")
                cmd="$cmd --bench embeddings_bench"
                ;;
            "performance")
                cmd="$cmd --bench scale_bench --bench large_scale_bench"
                ;;
            *)
                print_error "Unknown category: $category"
                exit 1
                ;;
        esac
    fi
    
    # Add verbose flag if requested
    if [ "$VERBOSE" = true ]; then
        cmd="$cmd --verbose"
    fi
    
    print_status "Running command: $cmd"
    eval $cmd
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--profile)
            PROFILE="$2"
            shift 2
            ;;
        -f|--features)
            FEATURES="$2"
            shift 2
            ;;
        -b|--benchmark)
            BENCHMARK="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        all|core|search|gpu|replication|storage|quantization|embeddings|performance|integration)
            COMMAND="$1"
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Set default command if none specified
if [ -z "$COMMAND" ]; then
    COMMAND="all"
fi

# Validate profile
case $PROFILE in
    quick|comprehensive|regression)
        ;;
    *)
        print_error "Invalid profile: $PROFILE. Must be one of: quick, comprehensive, regression"
        exit 1
        ;;
esac

# Print configuration
print_status "Benchmark Configuration:"
print_status "  Profile: $PROFILE"
print_status "  Features: ${FEATURES:-none}"
print_status "  Benchmark: ${BENCHMARK:-all}"
print_status "  Verbose: $VERBOSE"
print_status "  Command: $COMMAND"
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Not in a Rust project directory. Please run from the project root."
    exit 1
fi

# Run the benchmarks
print_status "Starting benchmarks..."
run_benchmarks "$COMMAND"

if [ $? -eq 0 ]; then
    print_success "Benchmarks completed successfully!"
    print_status "Results are available in target/criterion/"
else
    print_error "Benchmarks failed!"
    exit 1
fi
