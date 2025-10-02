# CUDA Library Build Instructions

## Overview

The CUDA library build has been separated from the main Vectorizer project. You can now build the CUDA libraries independently and place them in the `lib/` directory for the Vectorizer to use.

## Build Process

### Step 1: Build CUDA Library Separately

#### Windows
```powershell
# Navigate to scripts directory
cd f:\Node\hivellm\vectorizer\scripts

# Run the standalone build script
.\build_cuda_standalone.ps1
```

#### Linux/WSL
```bash
# Navigate to project directory
cd /mnt/f/Node/hivellm/vectorizer

# Run the standalone build script
./scripts/build_cuda_standalone.sh
```

### Step 2: Build Vectorizer (will use CUDA library if available)

```bash
cargo build
```

If the CUDA library exists in `lib/`, the Vectorizer will automatically link to it and enable GPU acceleration.

## Prerequisites

### Windows
1. **Visual Studio with C++ tools**
   - Download Visual Studio Community (FREE): https://visualstudio.microsoft.com/downloads/
   - During installation, select "Desktop development with C++" workload

2. **CUDA Toolkit 12.x (Required)**
   - Only CUDA 12.x versions are supported
   - Install CUDA 12.6 from: https://developer.nvidia.com/cuda-toolkit
   - CUDA 13.0 is not supported due to compatibility issues

### Linux/WSL
- CUDA Toolkit 12.x or 13.x
- GCC with C++14 support
- Development headers

### Compatibility Check (Windows)
Before building, run the compatibility check:
```
powershell.exe -ExecutionPolicy Bypass -File check_cuda_compatibility.ps1
```
This will verify your CUDA and Visual Studio installation and recommend the best configuration.

## Standalone Build Scripts

### Windows: `build_cuda_standalone.ps1`
- **Location**: `scripts/build_cuda_standalone.ps1`
- **Output**: `lib/cuhnsw.lib`
- **Features**:
  - Auto-detects CUDA and Visual Studio installations
  - Compiles CUDA kernels and C++ wrapper
  - Creates static library for linking
  - Supports GPU architectures: sm_86, sm_89
  - Includes cleanup option (`-Clean`)

### Linux: `build_cuda_standalone.sh`
- **Location**: `scripts/build_cuda_standalone.sh`
- **Output**: `lib/libcuhnsw.a`
- **Features**:
  - Auto-detects CUDA installation
  - Compiles CUDA kernels and C++ wrapper
  - Creates static library for linking
  - Supports GPU architectures: sm_86, sm_89
  - Includes cleanup option (`--clean`)

## Legacy Build Methods (Deprecated)

The following methods are deprecated but still available:

### Windows Legacy Scripts
- `build_cuda_simple.ps1` - Simple build script
- `build_cuda_direct.ps1` - Direct build with hardcoded paths
- `build_cuda_manual.bat` - Batch script with manual setup

## What the Scripts Do

- **Windows**: Compiles CUDA kernels (.cu files) and C++ wrapper code into a static library (`lib/cuhnsw.lib`)
- **Linux**: Compiles CUDA kernels and C++ code into a static library (`lib/libcuhnsw.a`)

The scripts automatically:
- Detect the project root directory
- Set up the required environment variables
- Compile all necessary files
- Clean up intermediate object files

The resulting library can be linked to other projects that need CUDA functionality without building the entire Vectorizer project.

## Why Visual Studio Developer Command Prompt?

The **"x64 Native Tools Command Prompt for VS 2022"** is essential because:

1. **MSVC Compiler**: It provides `cl.exe` (Microsoft C++ compiler) which is required by nvcc
2. **Environment Setup**: It automatically sets up all necessary environment variables
3. **PATH Configuration**: It adds MSVC tools to PATH so nvcc can find them
4. **64-bit Tools**: It ensures you're using 64-bit compilation tools

**Without it, you'll get the error: "nvcc fatal: Cannot find compiler 'cl.exe' in PATH"**

## Troubleshooting

### Windows Issues
- **"Cannot find compiler 'cl.exe'"**: Use "x64 Native Tools Command Prompt for VS 2022"
- **CUDA compilation fails**: Ensure CUDA Toolkit is installed and nvcc is in PATH
- **Script won't run**: Use `powershell.exe -ExecutionPolicy Bypass -File script.ps1`
- **CUDA version not supported**: Only CUDA 12.x is supported. Install CUDA 12.6
- **Many compilation errors**: This indicates CUDA version incompatibility. Only CUDA 12.x works

### Linux Issues
- **"nvcc: command not found"**: CUDA PATH may not be set correctly
- **Missing headers**: Check that CUDA Toolkit includes are properly installed

## Output
- Windows: `lib/cuhnsw.lib`
- Linux: `lib/libcuhnsw.a`

These libraries contain the compiled CUDA HNSW (Hierarchical Navigable Small World) implementation.
