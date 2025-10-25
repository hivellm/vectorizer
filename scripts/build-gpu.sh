#!/bin/bash
# Build script for vectorizer with hive-gpu backend selection

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default backend
BACKEND="metal"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --backend)
            BACKEND="$2"
            shift 2
            ;;
        --help|-h)
            echo "Usage: $0 [--backend metal|cuda|wgpu]"
            echo ""
            echo "Backends:"
            echo "  metal  - Metal Native (macOS only, default)"
            echo "  cuda   - CUDA (NVIDIA GPUs)"
            echo "  wgpu   - wgpu (Vulkan, DirectX12, Metal via wgpu)"
            echo ""
            echo "Examples:"
            echo "  $0                    # Build with Metal (default)"
            echo "  $0 --backend cuda    # Build with CUDA"
            echo "  $0 --backend wgpu    # Build with wgpu"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}🔧 Building vectorizer with GPU backend: ${BACKEND}${NC}"
echo ""

# Validate backend
case $BACKEND in
    metal)
        FEATURE="hive-gpu-metal"
        echo -e "${YELLOW}📱 Using Metal Native backend (macOS only)${NC}"
        ;;
    cuda)
        FEATURE="hive-gpu-cuda"
        echo -e "${YELLOW}🎮 Using CUDA backend (NVIDIA GPUs)${NC}"
        ;;
    wgpu)
        FEATURE="hive-gpu-wgpu"
        echo -e "${YELLOW}🌐 Using wgpu backend (Cross-platform)${NC}"
        ;;
    *)
        echo -e "${RED}❌ Invalid backend: $BACKEND${NC}"
        echo "Valid backends: metal, cuda, wgpu"
        exit 1
        ;;
esac

echo ""
echo -e "${BLUE}🚀 Building with feature: $FEATURE${NC}"
echo ""

# Build command
BUILD_CMD="cargo build --features $FEATURE"

echo -e "${GREEN}Executing: $BUILD_CMD${NC}"
echo ""

# Execute build
if $BUILD_CMD; then
    echo ""
    echo -e "${GREEN}✅ Build successful!${NC}"
    echo -e "${GREEN}🎉 vectorizer built with GPU acceleration ($BACKEND backend)${NC}"
    echo ""
    echo -e "${BLUE}To run tests:${NC}"
    echo "  cargo test --features $FEATURE"
    echo ""
    echo -e "${BLUE}To run examples:${NC}"
    echo "  cargo run --example hive_gpu_integration --features $FEATURE"
    echo ""
    echo -e "${BLUE}Note: Default build is CPU-only. Use --features to enable GPU.${NC}"
    echo ""
else
    echo ""
    echo -e "${RED}❌ Build failed!${NC}"
    exit 1
fi
