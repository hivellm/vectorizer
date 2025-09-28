# CUDA Build Script for Vectorizer with CUHNSW Integration (PowerShell)
# This script builds CUDA libraries and integrates CUHNSW dependency

param(
    [switch]$Force,
    [switch]$SkipBenchmark
)

# Function to print colored output
function Write-Status {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

# Function to check CUDA installation
function Test-CudaInstallation {
    Write-Status "Checking CUDA installation..."
    
    try {
        $nvccVersion = & nvcc --version 2>$null
        if ($LASTEXITCODE -ne 0) {
            throw "CUDA Toolkit not found"
        }
        
        $cudaVersion = ($nvccVersion | Select-String "release").Line | ForEach-Object { 
            if ($_ -match "release (\d+\.\d+)") { $matches[1] }
        }
        Write-Success "CUDA Toolkit found: version $cudaVersion"
        
        $nvidiaSmi = & nvidia-smi 2>$null
        if ($LASTEXITCODE -ne 0) {
            throw "NVIDIA driver not found"
        }
        
        Write-Success "NVIDIA driver found"
    }
    catch {
        Write-Error "CUDA installation check failed: $_"
        Write-Error "Please install CUDA 12.6 or compatible version and NVIDIA drivers."
        exit 1
    }
}

# Function to clone and build CUHNSW
function Build-Cuhnsw {
    Write-Status "Cloning and building CUHNSW..."
    
    $tempDir = "C:\temp\cuhnsw-build-$(Get-Random)"
    New-Item -ItemType Directory -Path $tempDir -Force | Out-Null
    
    try {
        # Clone CUHNSW repository
        if (-not (Test-Path "$tempDir\cuhnsw")) {
            Write-Status "Cloning CUHNSW repository..."
            git clone https://github.com/js1010/cuhnsw.git "$tempDir\cuhnsw"
        }
        
        Set-Location "$tempDir\cuhnsw"
        
        # Initialize submodules
        Write-Status "Initializing CUHNSW submodules..."
        git submodule update --init --recursive
        
        # Install Python dependencies
        Write-Status "Installing CUHNSW Python dependencies..."
        pip install -r requirements.txt
        
        # Generate protobuf files
        Write-Status "Generating CUHNSW protobuf files..."
        python -m grpc_tools.protoc --python_out cuhnsw/ --proto_path cuhnsw/proto/ config.proto
        
        # Build and install CUHNSW
        Write-Status "Building and installing CUHNSW..."
        python setup.py install
        
        # Verify installation
        Write-Status "Verifying CUHNSW installation..."
        python -c "import cuhnsw; print('CUHNSW installed successfully')"
        
        Write-Success "CUHNSW built and installed successfully!"
    }
    catch {
        Write-Error "CUHNSW build failed: $_"
        exit 1
    }
    finally {
        # Cleanup
        Set-Location $PSScriptRoot
        if (Test-Path $tempDir) {
            Remove-Item -Path $tempDir -Recurse -Force
        }
    }
}

# Function to build CUDA libraries
function Build-CudaLibraries {
    Write-Status "Building CUDA libraries..."
    
    # Create lib directory if it doesn't exist
    if (-not (Test-Path "lib")) {
        New-Item -ItemType Directory -Path "lib" | Out-Null
    }
    
    # Build CUDA library (placeholder - replace with actual build commands)
    Write-Status "Building CUDA HNSW implementation..."
    
    # This would be replaced with actual CUDA compilation commands
    # For now, we'll create a placeholder
    if ((Test-Path "lib\cuhnsw.lib") -or (Test-Path "lib\libcuhnsw.so")) {
        Write-Success "CUDA library already exists"
    }
    else {
        Write-Warning "CUDA library build not implemented yet - using CUHNSW Python bindings"
    }
}

# Function to run CUDA benchmark
function Invoke-CudaBenchmark {
    if ($SkipBenchmark) {
        Write-Warning "Skipping CUDA benchmark"
        return
    }
    
    Write-Status "Running CUDA benchmark..."
    
    try {
        if (Get-Command cargo -ErrorAction SilentlyContinue) {
            cargo run --bin cuda_benchmark --features cuda
        }
        else {
            Write-Warning "Cargo not found - skipping benchmark"
        }
    }
    catch {
        Write-Warning "CUDA benchmark failed: $_"
    }
}

# Main script logic
function Main {
    Write-Status "Starting CUDA build process for Vectorizer..."
    
    # Check prerequisites
    Test-CudaInstallation
    
    # Build CUHNSW
    Build-Cuhnsw
    
    # Build CUDA libraries
    Build-CudaLibraries
    
    # Run benchmark
    Invoke-CudaBenchmark
    
    Write-Success "CUDA build process completed successfully!"
    Write-Status "CUHNSW integration ready for use"
}

# Run main function
Main