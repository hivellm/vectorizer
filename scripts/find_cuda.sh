#!/bin/bash
# Script to help find CUDA installation

echo "Searching for CUDA installation..."
echo

echo "1. Checking common CUDA paths:"
paths=(
    "/usr/local/cuda"
    "/usr/local/cuda-13.0"
    "/usr/local/cuda-12.0"
    "/usr/local/cuda-11.8"
    "/usr/lib/cuda"
    "/opt/cuda"
)

for path in "${paths[@]}"; do
    if [ -d "$path" ]; then
        echo "  ✓ Found: $path"
        if [ -f "$path/bin/nvcc" ]; then
            echo "    - nvcc found at: $path/bin/nvcc"
        fi
    else
        echo "  ✗ Not found: $path"
    fi
done

echo
echo "2. Searching for nvcc in system:"
which nvcc 2>/dev/null && echo "  ✓ nvcc in PATH: $(which nvcc)"

echo
echo "3. Searching for cuda directories:"
find /usr -name "cuda*" -type d 2>/dev/null | grep -E "(cuda|cuda-[0-9]+\.[0-9]+)$" | head -10

echo
echo "4. Checking dpkg for CUDA packages:"
dpkg -l 2>/dev/null | grep -i cuda | grep -E "^ii" | awk '{print $2}' | head -10

echo
echo "5. Checking environment variables:"
echo "  CUDA_PATH: ${CUDA_PATH:-not set}"
echo "  CUDA_HOME: ${CUDA_HOME:-not set}"
echo "  PATH includes cuda: $(echo $PATH | grep -o "[^:]*cuda[^:]*" | tr '\n' ' ')"

echo
echo "If CUDA is installed via apt, you might need to install cuda-toolkit-XX-X"
echo "The nvcc compiler is typically in the cuda-toolkit package."
