# Setup script for sccache compilation cache (Windows PowerShell)
# This significantly speeds up Rust builds, especially in large projects

Write-Host "🔧 Setting up sccache for Rust builds..." -ForegroundColor Cyan

# Check if sccache is installed
$sccachePath = Get-Command sccache -ErrorAction SilentlyContinue

if (-not $sccachePath) {
    Write-Host "📦 Installing sccache..." -ForegroundColor Yellow
    cargo install sccache
    $sccachePath = Get-Command sccache
}

$sccacheExe = $sccachePath.Source
Write-Host "📍 sccache found at: $sccacheExe" -ForegroundColor Green

# Set RUSTC_WRAPPER environment variable for current session
$env:RUSTC_WRAPPER = $sccacheExe
Write-Host "✅ RUSTC_WRAPPER set to: $env:RUSTC_WRAPPER" -ForegroundColor Green

# Check sccache stats
Write-Host ""
Write-Host "📊 sccache statistics:" -ForegroundColor Cyan
sccache --show-stats 2>&1 | Out-String

Write-Host ""
Write-Host "✅ sccache setup complete!" -ForegroundColor Green
Write-Host ""
Write-Host "💡 To make this permanent, add to your PowerShell profile:" -ForegroundColor Yellow
Write-Host "   `$env:RUSTC_WRAPPER = `"$sccacheExe`""
Write-Host ""
Write-Host "💡 Or run this script before building:" -ForegroundColor Yellow
Write-Host "   .\scripts\setup-sccache.ps1"
Write-Host ""
Write-Host "🚀 Your Rust builds will now use sccache for caching!" -ForegroundColor Green

