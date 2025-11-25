# Script to build Docker image with supply chain attestations (local build only)
# Usage: .\scripts\docker-build.ps1 -Tag 1.5.2
# Note: Push must be done manually using: docker buildx build --push ...

param(
    [Parameter(Mandatory=$false)]
    [string]$Tag = "latest",
    
    [Parameter(Mandatory=$false)]
    [string]$Repository = "vectorizer",
    
    [Parameter(Mandatory=$false)]
    [string]$Organization = "hivellm",
    
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

# Check if attestation-builder exists, create if not
$builderExists = docker buildx ls --format "{{.Name}}" | Select-String -Pattern "attestation-builder"
if (-not $builderExists) {
    Write-Host "🔧 Creating attestation-builder with multi-platform support..." -ForegroundColor Cyan
    docker buildx create --name attestation-builder --driver docker-container --use --platform linux/amd64,linux/arm64 | Out-Null
    docker buildx inspect --bootstrap | Out-Null
} else {
    Write-Host "🔧 Using attestation-builder..." -ForegroundColor Cyan
    docker buildx use attestation-builder | Out-Null
    docker buildx inspect --bootstrap | Out-Null
}

# Build command with attestations (local build only)
$buildArgs = @(
    "buildx", "build",
    "--platform", $Platform,
    "--tag", $SourceTag,
    "--tag", $FullTag,
    "--build-arg", "GIT_COMMIT_ID=$GitCommitId",
    "--build-arg", "BUILD_DATE=$BuildDate",
    "--provenance", "mode=max",
    "--sbom", "true",
    "--load"
)

Write-Host "📦 Building locally (no push)" -ForegroundColor Yellow
Write-Host "   To push manually, use:" -ForegroundColor Yellow
Write-Host "   docker buildx build --platform linux/amd64,linux/arm64 --provenance=true --sbom=true --push -t hivellm/vectorizer:latest -t hivellm/vectorizer:$Tag ." -ForegroundColor White
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
Write-Host "   docker buildx build --platform linux/amd64,linux/arm64 --provenance=true --sbom=true --push -t hivellm/vectorizer:latest -t hivellm/vectorizer:$Tag ." -ForegroundColor White

