#!/usr/bin/env pwsh
# Safe build script for Windows - Prevents BSODs during compilation
# 
# This script configures environment variables to minimize resource usage
# and prevent GPU driver crashes during build/test operations.
#
# Usage:
#   .\scripts\build-windows-safe.ps1
#   .\scripts\build-windows-safe.ps1 test

param(
    [string]$Command = "build"
)

Write-Host "=" -ForegroundColor Cyan -NoNewline
Write-Host "=".PadRight(79, "=") -ForegroundColor Cyan
Write-Host "  Vectorizer Safe Build for Windows" -ForegroundColor Yellow
Write-Host "  Prevents BSODs by limiting parallelism and disabling GPU" -ForegroundColor Gray
Write-Host "=" -ForegroundColor Cyan -NoNewline
Write-Host "=".PadRight(79, "=") -ForegroundColor Cyan
Write-Host ""

# Set safe environment variables
Write-Host "[1/5] Configuring environment..." -ForegroundColor Cyan
$env:RAYON_NUM_THREADS = "1"          # Single-threaded Rayon
$env:TOKIO_WORKER_THREADS = "2"       # Minimal Tokio workers
$env:CARGO_BUILD_JOBS = "2"           # Limit parallel compilation
$env:RUST_BACKTRACE = "1"             # Enable backtraces for debugging

Write-Host "      RAYON_NUM_THREADS = 1" -ForegroundColor Gray
Write-Host "      TOKIO_WORKER_THREADS = 2" -ForegroundColor Gray
Write-Host "      CARGO_BUILD_JOBS = 2" -ForegroundColor Gray

# Check Rust toolchain
Write-Host ""
Write-Host "[2/5] Checking Rust toolchain..." -ForegroundColor Cyan
$rustVersion = & rustc --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "  ERROR: Rust not found. Install from https://rustup.rs" -ForegroundColor Red
    exit 1
}
Write-Host "      $rustVersion" -ForegroundColor Gray

# Check if nightly is available
$nightlyVersion = & rustc +nightly --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "  WARNING: Nightly toolchain not found. Installing..." -ForegroundColor Yellow
    & rustup toolchain install nightly
}

# Clean previous build artifacts (optional)
Write-Host ""
Write-Host "[3/5] Checking build artifacts..." -ForegroundColor Cyan
$targetSize = 0
if (Test-Path "target") {
    $targetSize = (Get-ChildItem -Path "target" -Recurse -File | Measure-Object -Property Length -Sum).Sum / 1GB
    Write-Host "      target/ size: $([math]::Round($targetSize, 2)) GB" -ForegroundColor Gray
    
    if ($targetSize -gt 10) {
        Write-Host "      WARNING: Large target directory. Consider running 'cargo clean'" -ForegroundColor Yellow
    }
}

# Run build/test with safe profile
Write-Host ""
Write-Host "[4/5] Running: $Command" -ForegroundColor Cyan

$features = ""  # No features by default (safest)

switch ($Command) {
    "build" {
        Write-Host "      Building with NO GPU features (safest)..." -ForegroundColor Green
        Write-Host ""
        & cargo +nightly build --profile=safe --no-default-features
    }
    "build-fast" {
        Write-Host "      Building with fastembed only (no GPU drivers)..." -ForegroundColor Yellow
        Write-Host ""
        & cargo +nightly build --profile=safe --no-default-features --features "fastembed"
    }
    "build-full" {
        Write-Host "      Building with ALL features (risky - may cause BSOD!)..." -ForegroundColor Red
        Write-Host ""
        Read-Host "      Press Enter to continue or Ctrl+C to cancel"
        & cargo +nightly build --profile=safe --all-features
    }
    "test" {
        Write-Host "      Running tests (single-threaded, no GPU)..." -ForegroundColor Green
        Write-Host ""
        & cargo +nightly test --profile=test-safe --no-default-features -- --test-threads=1
    }
    "test-fast" {
        Write-Host "      Running tests with fastembed (no GPU drivers)..." -ForegroundColor Yellow
        Write-Host ""
        & cargo +nightly test --profile=test-safe --no-default-features --features "fastembed" -- --test-threads=1
    }
    "clean" {
        Write-Host "      Cleaning build artifacts..." -ForegroundColor Yellow
        Write-Host ""
        & cargo clean
        Write-Host ""
        Write-Host "      Cleaned successfully" -ForegroundColor Green
        exit 0
    }
    default {
        Write-Host "      ERROR: Unknown command: $Command" -ForegroundColor Red
        Write-Host ""
        Write-Host "      Available commands:" -ForegroundColor Cyan
        Write-Host "        build       - Safe build (no GPU)" -ForegroundColor Gray
        Write-Host "        build-fast  - Build with fastembed (no GPU drivers)" -ForegroundColor Gray
        Write-Host "        build-full  - Build with all features (RISKY!)" -ForegroundColor Gray
        Write-Host "        test        - Run tests (single-threaded)" -ForegroundColor Gray
        Write-Host "        test-fast   - Run tests with fastembed" -ForegroundColor Gray
        Write-Host "        clean       - Clean build artifacts" -ForegroundColor Gray
        exit 1
    }
}

# Check result
Write-Host ""
Write-Host "[5/5] Checking result..." -ForegroundColor Cyan

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "  SUCCESS: Build completed without errors!" -ForegroundColor Green
    Write-Host ""
    
    if ($Command -eq "build" -or $Command -like "build-*") {
        $exePath = "target\safe\vectorizer.exe"
        if (Test-Path $exePath) {
            $exeSize = (Get-Item $exePath).Length / 1MB
            Write-Host "  Binary size: $([math]::Round($exeSize, 2)) MB" -ForegroundColor Gray
            Write-Host "  Location: $exePath" -ForegroundColor Gray
        }
    }
    
    Write-Host ""
    exit 0
} else {
    Write-Host ""
    Write-Host "  FAILED: Build encountered errors" -ForegroundColor Red
    Write-Host ""
    Write-Host "  Troubleshooting:" -ForegroundColor Yellow
    Write-Host "    1. Check error messages above" -ForegroundColor Gray
    Write-Host "    2. Run 'cargo clean' and try again" -ForegroundColor Gray
    Write-Host "    3. Update Rust: 'rustup update nightly'" -ForegroundColor Gray
    Write-Host "    4. Check docs/BSOD_ANALYSIS.md for more info" -ForegroundColor Gray
    Write-Host ""
    exit 1
}


