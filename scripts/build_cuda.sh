#!/bin/bash
# CUDA Build Script for Vectorizer with CUHNSW Integration
# This script builds CUDA libraries and integrates CUHNSW dependency

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

# Function to check CUDA installation
check_cuda() {
    print_status "Checking CUDA installation..."
    
    if ! command -v nvcc &> /dev/null; then
        print_error "CUDA Toolkit not found. Please install CUDA 12.6 or compatible version."
        exit 1
    fi
    
    CUDA_VERSION=$(nvcc --version | grep "release" | sed 's/.*release \([0-9]\+\.[0-9]\+\).*/\1/')
    print_success "CUDA Toolkit found: version $CUDA_VERSION"
    
    if ! command -v nvidia-smi &> /dev/null; then
        print_error "NVIDIA driver not found. Please install NVIDIA drivers."
        exit 1
    fi
    
    print_success "NVIDIA driver found"
}

# Function to clone and build CUHNSW
build_cuhnsw() {
    print_status "Cloning and building CUHNSW..."
    
    # Create temporary directory
    TEMP_DIR="/tmp/cuhnsw-build-$$"
    mkdir -p "$TEMP_DIR"
    
    # Clone CUHNSW repository
    if [ ! -d "$TEMP_DIR/cuhnsw" ]; then
        print_status "Cloning CUHNSW repository..."
        git clone https://github.com/js1010/cuhnsw.git "$TEMP_DIR/cuhnsw"
    fi
    
    cd "$TEMP_DIR/cuhnsw"
    
    # Initialize submodules
    print_status "Initializing CUHNSW submodules..."
    git submodule update --init --recursive
    
    # Install Python dependencies
    print_status "Installing CUHNSW Python dependencies..."
    pip3 install -r requirements.txt
    
    # Generate protobuf files
    print_status "Generating CUHNSW protobuf files..."
    python3 -m grpc_tools.protoc --python_out cuhnsw/ --proto_path cuhnsw/proto/ config.proto
    
    # Build and install CUHNSW
    print_status "Building and installing CUHNSW..."
    python3 setup.py install
    
    # Verify installation
    print_status "Verifying CUHNSW installation..."
    python3 -c "import cuhnsw; print('CUHNSW installed successfully')"
    
    # Cleanup
    cd /
    rm -rf "$TEMP_DIR"
    
    print_success "CUHNSW built and installed successfully!"
}

# Function to build CUDA libraries
build_cuda_libs() {
    print_status "Building CUDA libraries..."
    
    # Create lib directory if it doesn't exist
    mkdir -p lib
    
    # Build CUDA library (placeholder - replace with actual build commands)
    print_status "Building CUDA HNSW implementation..."
    
    # This would be replaced with actual CUDA compilation commands
    # For now, we'll create a placeholder
    if [ -f "lib/cuhnsw.lib" ] || [ -f "lib/libcuhnsw.so" ]; then
        print_success "CUDA library already exists"
    else
        print_warning "CUDA library build not implemented yet - using CUHNSW Python bindings"
    fi
}

# Function to run CUDA benchmark
run_benchmark() {
    print_status "Running CUDA benchmark..."
    
    if command -v cargo &> /dev/null; then
        cargo run --bin cuda_benchmark --features cuda
    else
        print_warning "Cargo not found - skipping benchmark"
    fi
}

# Main script logic
main() {
    print_status "Starting CUDA build process for Vectorizer..."
    
    # Check prerequisites
    check_cuda
    
    # Build CUHNSW
    build_cuhnsw
    
    # Build CUDA libraries
    build_cuda_libs
    
    # Run benchmark
    run_benchmark
    
    print_success "CUDA build process completed successfully!"
    print_status "CUHNSW integration ready for use"
}

# Run main function
main "$@"