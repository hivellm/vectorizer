# Script to build Docker image
# Usage: .\scripts\docker-build.ps1 -Tag 1.5.2

param(
    [Parameter(Mandatory=$false)]
    [string]$Tag = "latest",
    
    [Parameter(Mandatory=$false)]
    [string]$Repository = "vectorizer",
    
    [Parameter(Mandatory=$false)]
    [string]$Organization = "hivehub"
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

Write-Host "🔨 Building Docker image..." -ForegroundColor Cyan
Write-Host "   Organization: $Organization" -ForegroundColor Yellow
Write-Host "   Repository: $Repository" -ForegroundColor Yellow
Write-Host "   Tag: $Tag" -ForegroundColor Yellow
Write-Host "   Git Commit: $GitCommitId" -ForegroundColor Yellow
Write-Host "   Build Date: $BuildDate" -ForegroundColor Yellow
Write-Host ""

# Build command
$buildArgs = @(
    "build",
    "--tag", $SourceTag,
    "--tag", $FullTag,
    "--build-arg", "GIT_COMMIT_ID=$GitCommitId",
    "--build-arg", "BUILD_DATE=$BuildDate",
    "."
)

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

