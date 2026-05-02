# Script to build Docker image
# Usage: .\scripts\docker-build.ps1 -Tag 2.1.0

param(
    [Parameter(Mandatory=$false)]
    [string]$Tag = "latest",

    [Parameter(Mandatory=$false)]
    [string]$Repository = "vectorizer",

    [Parameter(Mandatory=$false)]
    [string]$Organization = "hivehub",

    # Read-only cache repo. Defaults to the dedicated
    # `hivehub/vectorizer-cache:buildx` tag seeded by build-push.ps1.
    # Pass `-NoCache` for a fully cold build.
    [Parameter(Mandatory=$false)]
    [string]$CacheRepo = "hivehub/vectorizer-cache",

    [Parameter(Mandatory=$false)]
    [string]$CacheTag = "buildx",

    [Parameter(Mandatory=$false)]
    [switch]$NoCache
)

$ImageName = "vectorizer"
$FullTag = "${Organization}/${Repository}:${Tag}"
$SourceTag = "${ImageName}:${Tag}"
$CacheRef = "${CacheRepo}:${CacheTag}"

# Get git commit ID for build metadata
$GitCommitId = git rev-parse --short HEAD 2>$null
if (-not $GitCommitId) {
    $GitCommitId = "unknown"
}

$BuildDate = Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ"

Write-Host "🔨 Building Docker image..." -ForegroundColor Cyan
Write-Host "   Organization: $Organization" -ForegroundColor Yellow
Write-Host "   Repository: $Repository" -ForegroundColor Yellow
Write-Host "   Tag: $Tag" -ForegroundColor Yellow
Write-Host "   Git Commit: $GitCommitId" -ForegroundColor Yellow
Write-Host "   Build Date: $BuildDate" -ForegroundColor Yellow
Write-Host ""

# Build command — buildx for registry cache support. `--load` materializes
# the host-arch image into the local docker daemon (so existing
# `docker run vectorizer:$Tag` flows keep working). Multi-arch + push is
# the build-push.ps1 path.
$buildArgs = @(
    "buildx", "build",
    "--tag", $SourceTag,
    "--tag", $FullTag,
    "--build-arg", "GIT_COMMIT_ID=$GitCommitId",
    "--build-arg", "BUILD_DATE=$BuildDate",
    "--load"
)

if (-not $NoCache) {
    Write-Host "   Cache: ${CacheRef} (read-only registry pull)" -ForegroundColor Yellow
    $buildArgs += "--cache-from"
    $buildArgs += "type=registry,ref=${CacheRef}"
} else {
    Write-Host "   Cache: disabled (-NoCache)" -ForegroundColor Yellow
}

$buildArgs += "."

Write-Host "🚀 Starting build..." -ForegroundColor Cyan
docker @buildArgs

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "✅ Build completed successfully!" -ForegroundColor Green
Write-Host "   Local tag: ${SourceTag}" -ForegroundColor Cyan
Write-Host "   Full tag: ${FullTag}" -ForegroundColor Cyan
Write-Host ""
Write-Host "📤 To push manually:" -ForegroundColor Yellow
Write-Host "   docker push ${FullTag}" -ForegroundColor White

