# CUDA Compatibility Check Script

Write-Host "CUDA Compatibility Check" -ForegroundColor Green
Write-Host "========================" -ForegroundColor Green

# Check installed CUDA versions
Write-Host "Checking for installed CUDA versions..." -ForegroundColor Yellow

$cudaPaths = @(
    "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.6",
    "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.5",
    "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.4",
    "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.3",
    "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.2",
    "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.1",
    "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.0",
    "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v11.8"
)

$foundVersions = @()
foreach ($path in $cudaPaths) {
    if (Test-Path $path) {
        $version = Split-Path $path -Leaf
        $foundVersions += $version
        Write-Host "  Found: $version" -ForegroundColor Green
    }
}

if ($foundVersions.Count -eq 0) {
    Write-Host "  No CUDA installations found!" -ForegroundColor Red
    Write-Host "" -ForegroundColor Yellow
    Write-Host "Please install CUDA Toolkit from: https://developer.nvidia.com/cuda-toolkit" -ForegroundColor Cyan
    exit 1
}

# Determine best version
$bestVersion = $null
$compatibility = @{}

foreach ($version in $foundVersions) {
    if ($version -like "*v12*") {
        $compatibility[$version] = "Compatible"
        if (!$bestVersion) { $bestVersion = $version }
    } else {
        $compatibility[$version] = "Not supported (use CUDA 12.x)"
    }
}

Write-Host "" -ForegroundColor Yellow
Write-Host "Compatibility Assessment:" -ForegroundColor Cyan
foreach ($version in $foundVersions) {
    $status = $compatibility[$version]
    $color = if ($status -eq "Compatible") { "Green" } else { "Red" }
    Write-Host "  $version : $status" -ForegroundColor $color
}

Write-Host "" -ForegroundColor Yellow
if ($bestVersion) {
    Write-Host "RECOMMENDED: Use $bestVersion" -ForegroundColor Green
    Write-Host "The build scripts will automatically use the best compatible version." -ForegroundColor Gray
} else {
    Write-Host "WARNING: No compatible CUDA versions found!" -ForegroundColor Red
    Write-Host "Please install CUDA 12.x for best compatibility." -ForegroundColor Yellow
}

# Check Visual Studio
Write-Host "" -ForegroundColor Yellow
Write-Host "Checking Visual Studio..." -ForegroundColor Cyan

$vsPaths = @(
    "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC",
    "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\Professional\VC\Tools\MSVC",
    "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\Enterprise\VC\Tools\MSVC",
    "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC"
)

$vsFound = $false
foreach ($vsPath in $vsPaths) {
    if (Test-Path $vsPath) {
        Write-Host "  Visual Studio C++ tools found" -ForegroundColor Green
        $vsFound = $true
        break
    }
}

if (!$vsFound) {
    Write-Host "  Visual Studio C++ tools not found" -ForegroundColor Red
    Write-Host "  Please install Visual Studio with C++ workload" -ForegroundColor Yellow
}

# Summary
Write-Host "" -ForegroundColor Yellow
Write-Host "Summary:" -ForegroundColor Cyan
if ($bestVersion -and $vsFound) {
    Write-Host "  System is ready for CUDA library build" -ForegroundColor Green
    Write-Host "  Run: .\build_cuda_direct.ps1" -ForegroundColor White
} else {
    Write-Host "  System needs configuration before building" -ForegroundColor Red
    if (!$bestVersion) {
        Write-Host "    - Install CUDA 12.x" -ForegroundColor Yellow
    }
    if (!$vsFound) {
        Write-Host "    - Install Visual Studio with C++ tools" -ForegroundColor Yellow
    }
}
