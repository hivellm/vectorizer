# Script to build Docker image with supply chain attestations (local build only)
# Usage: .\scripts\docker-build.ps1 -Tag 1.5.2
# Note: Push must be done manually using: docker buildx build --push ...

param(
    [Parameter(Mandatory=$false)]
    [string]$Tag = "latest",
    
    [Parameter(Mandatory=$false)]
    [string]$Repository = "vectorizer",
    
    [Parameter(Mandatory=$false)]
    [string]$Organization = "hivehub",
    
    [Parameter(Mandatory=$false)]
    [string]$Platform = "linux/amd64"
)

$ImageName = "vectorizer"
$FullTag = "${Organization}/${Repository}:${Tag}"
$SourceTag = "${ImageName}:${Tag}"

# Get git commit ID for build metadata
$GitCommitId = git rev-parse --short HEAD 2>$null
if (-not $GitCommitId) {
    $GitCommitId = "unknown"
}

$BuildDate = Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ"

Write-Host "🔨 Building Docker image with supply chain attestations..." -ForegroundColor Cyan
Write-Host "   Organization: $Organization" -ForegroundColor Yellow
Write-Host "   Repository: $Repository" -ForegroundColor Yellow
Write-Host "   Tag: $Tag" -ForegroundColor Yellow
Write-Host "   Platform: $Platform" -ForegroundColor Yellow
Write-Host "   Git Commit: $GitCommitId" -ForegroundColor Yellow
Write-Host "   Build Date: $BuildDate" -ForegroundColor Yellow
Write-Host ""

# Enable Docker BuildKit for attestations
$env:DOCKER_BUILDKIT = "1"

# For local build with --load, we can only use a single platform
# Extract first platform if multiple are specified
$loadPlatform = $Platform
if ($Platform.Contains(',')) {
    $loadPlatform = $Platform.Split(',')[0].Trim()
    Write-Host "⚠️  Multiple platforms specified, but --load only supports single platform" -ForegroundColor Yellow
    Write-Host "   Building for: $loadPlatform (use manual push for multi-platform)" -ForegroundColor Yellow
}

# Use default builder for --load (single platform)
# Multi-platform builder causes issues with --load
Write-Host "🔧 Using default builder for local build..." -ForegroundColor Cyan
docker buildx use default | Out-Null

# Build command with attestations (local build only)
$buildArgs = @(
    "buildx", "build",
    "--platform", $loadPlatform,
    "--tag", $SourceTag,
    "--tag", $FullTag,
    "--build-arg", "GIT_COMMIT_ID=$GitCommitId",
    "--build-arg", "BUILD_DATE=$BuildDate",
    "--provenance", "mode=max",
    "--sbom", "true",
    "--load"
)

Write-Host "📦 Building locally (no push) - Platform: $loadPlatform" -ForegroundColor Yellow
Write-Host "   To push manually (multi-platform), use:" -ForegroundColor Yellow
Write-Host "   docker buildx build --platform linux/amd64,linux/arm64 --provenance=true --sbom=true --push -t hivehub/vectorizer:latest -t hivehub/vectorizer:$Tag ." -ForegroundColor White
    Write-Host ""

Write-Host "🚀 Starting build..." -ForegroundColor Cyan
docker @buildArgs .

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "✅ Build completed successfully!" -ForegroundColor Green
Write-Host "   Local tag: ${SourceTag}" -ForegroundColor Cyan
Write-Host "   Full tag: ${FullTag}" -ForegroundColor Cyan
    Write-Host ""
Write-Host "📤 To push manually (multi-platform):" -ForegroundColor Yellow
Write-Host "   docker buildx build --platform linux/amd64,linux/arm64 --provenance=true --sbom=true --push -t hivehub/vectorizer:latest -t hivehub/vectorizer:$Tag ." -ForegroundColor White

