# Script to build and push Docker image with attestations (for Docker Scout Grade A)
# Usage: .\scripts\docker-build-push.ps1 -Tag 2.0.0

param(
    [Parameter(Mandatory=$false)]
    [string]$Tag = "latest",

    [Parameter(Mandatory=$false)]
    [string]$Repository = "vectorizer",

    [Parameter(Mandatory=$false)]
    [string]$Organization = "hivehub",

    # Buildx registry cache repo. Defaults to the dedicated
    # `hivehub/vectorizer-cache:buildx` tag (see
    # docs/development/docker-builds.md). Pass `-NoCache` to skip the
    # cache layer entirely (cold build).
    [Parameter(Mandatory=$false)]
    [string]$CacheRepo = "hivehub/vectorizer-cache",

    [Parameter(Mandatory=$false)]
    [string]$CacheTag = "buildx",

    [Parameter(Mandatory=$false)]
    [switch]$NoCache
)

$ImageName = "vectorizer"
$FullTag = "${Organization}/${Repository}:${Tag}"
$CacheRef = "${CacheRepo}:${CacheTag}"

# Get git commit ID for build metadata
$GitCommitId = git rev-parse --short HEAD 2>$null
if (-not $GitCommitId) {
    $GitCommitId = "unknown"
}

$BuildDate = Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ"

Write-Host "🔨 Building Docker image with attestations for push..." -ForegroundColor Cyan
Write-Host "   Organization: $Organization" -ForegroundColor Yellow
Write-Host "   Repository: $Repository" -ForegroundColor Yellow
Write-Host "   Tag: $Tag" -ForegroundColor Yellow
Write-Host "   Git Commit: $GitCommitId" -ForegroundColor Yellow
Write-Host "   Build Date: $BuildDate" -ForegroundColor Yellow
Write-Host ""

# Enable Docker BuildKit
$env:DOCKER_BUILDKIT = "1"

# Check/create buildx builder
$builderExists = docker buildx ls --format "{{.Name}}" | Select-String -Pattern "vectorizer-builder"
if (-not $builderExists) {
    Write-Host "🔧 Creating buildx builder..." -ForegroundColor Cyan
    docker buildx create --name vectorizer-builder --driver docker-container --use --platform linux/amd64,linux/arm64 | Out-Null
    docker buildx inspect --bootstrap | Out-Null
} else {
    Write-Host "🔧 Using buildx builder..." -ForegroundColor Cyan
    docker buildx use vectorizer-builder | Out-Null
    docker buildx inspect --bootstrap | Out-Null
}

# Build and push with attestations
Write-Host "🚀 Building and pushing (multi-platform with attestations)..." -ForegroundColor Cyan
$buildArgs = @(
    "buildx", "build",
    "--platform", "linux/amd64,linux/arm64",
    "--tag", "${FullTag}",
    "--build-arg", "GIT_COMMIT_ID=$GitCommitId",
    "--build-arg", "BUILD_DATE=$BuildDate",
    "--provenance", "mode=max",
    "--sbom", "true",
    "--push"
)

# Buildx registry cache: read previous layers, write new ones with
# `mode=max` so every intermediate layer is cached (not just the final
# image). Skipped when -NoCache is passed.
if (-not $NoCache) {
    Write-Host "   Cache: ${CacheRef} (registry, mode=max)" -ForegroundColor Yellow
    $buildArgs += "--cache-from"
    $buildArgs += "type=registry,ref=${CacheRef}"
    $buildArgs += "--cache-to"
    $buildArgs += "type=registry,ref=${CacheRef},mode=max"
} else {
    Write-Host "   Cache: disabled (-NoCache)" -ForegroundColor Yellow
}

# If tag is not "latest", also tag as latest
if ($Tag -ne "latest") {
    $latestTag = "${Organization}/${Repository}:latest"
    $buildArgs += "--tag"
    $buildArgs += $latestTag
}

$buildArgs += "."

docker @buildArgs

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build/push failed!" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "✅ Build and push completed successfully!" -ForegroundColor Green
Write-Host "   Image available at: docker.io/${FullTag}" -ForegroundColor Cyan
if ($Tag -ne "latest") {
    Write-Host "   Also tagged as: docker.io/${Organization}/${Repository}:latest" -ForegroundColor Cyan
}
Write-Host ""
Write-Host "📊 Check Docker Scout score:" -ForegroundColor Yellow
Write-Host "   docker scout cves ${FullTag}" -ForegroundColor White

