# PowerShell script to test musl builds via WSL
param(
    [switch]$SkipInstall
)

Write-Host "🔧 Testing musl builds via WSL..." -ForegroundColor Cyan
Write-Host ""

# Check if WSL is available
if (-not (Get-Command wsl -ErrorAction SilentlyContinue)) {
    Write-Host "❌ WSL not found. Please install WSL first." -ForegroundColor Red
    exit 1
}

# Navigate to vectorizer directory
$scriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
$vectorizerPath = Split-Path -Parent $scriptPath
Set-Location $vectorizerPath

Write-Host "📍 Working directory: $vectorizerPath" -ForegroundColor Yellow
Write-Host ""

# Convert Windows path to WSL path
$wslPath = wsl wslpath "'$vectorizerPath'"

# Run the bash script in WSL
Write-Host "🐧 Executing build in WSL Ubuntu-24.04..." -ForegroundColor Cyan
Write-Host ""

wsl -d Ubuntu-24.04 -- bash -l -c "cd '$wslPath' && chmod +x scripts/test-musl-build.sh && ./scripts/test-musl-build.sh"

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "✅ All tests passed!" -ForegroundColor Green
    Write-Host ""
    Write-Host "💡 You can now push to GitHub:" -ForegroundColor Yellow
    Write-Host "   git push origin main" -ForegroundColor White
    Write-Host "   git push origin v0.9.14" -ForegroundColor White
    Write-Host ""
} else {
    Write-Host ""
    Write-Host "❌ Tests failed! Fix the errors before pushing." -ForegroundColor Red
    Write-Host ""
    exit 1
}

