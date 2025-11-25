# Script to build Docker image with supply chain attestations
# Usage: .\scripts\docker-build.ps1 -Tag 1.5.0 -Push

param(
    [Parameter(Mandatory=$false)]
    [string]$Tag = "latest",
    
    [Parameter(Mandatory=$false)]
    [string]$Repository = "vectorizer",
    
    [Parameter(Mandatory=$false)]
    [string]$Organization = "hivehub",
    
    [Parameter(Mandatory=$false)]
    [switch]$Push = $false,
    
    [Parameter(Mandatory=$false)]
    [string]$Platform = "linux/amd64,linux/arm64"
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

# Build command with attestations
$buildArgs = @(
    "buildx", "build",
    "--platform", $Platform,
    "--tag", $SourceTag,
    "--tag", $FullTag,
    "--build-arg", "GIT_COMMIT_ID=$GitCommitId",
    "--build-arg", "BUILD_DATE=$BuildDate",
    "--provenance", "mode=max",
    "--sbom", "true"
)

if ($Push) {
    $buildArgs = $buildArgs | Where-Object { $_ -ne "--load" }
    $buildArgs += "--push"
    
    Write-Host "📤 Will push to Docker Hub after build (multi-platform)" -ForegroundColor Yellow
    Write-Host ""
} else {
    # For multi-platform builds without push, we need to load only the native platform
    # Extract first platform for local load
    $firstPlatform = $Platform.Split(',')[0]
    $buildArgs = $buildArgs | Where-Object { $_ -ne "--platform" }
    $buildArgs = $buildArgs | Where-Object { $_ -ne $Platform }
    $buildArgs += "--platform"
    $buildArgs += $firstPlatform
    $buildArgs += "--load"
    
    Write-Host "⚠️  Multi-platform build without push: loading only $firstPlatform for local use" -ForegroundColor Yellow
    Write-Host "   Use -Push to build and push all platforms" -ForegroundColor Yellow
    Write-Host ""
}

Write-Host "🚀 Starting build..." -ForegroundColor Cyan
docker @buildArgs .

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "✅ Build completed successfully!" -ForegroundColor Green
Write-Host "   Local tag: ${SourceTag}" -ForegroundColor Cyan
Write-Host "   Docker Hub tag: ${FullTag}" -ForegroundColor Cyan

if (-not $Push) {
    Write-Host ""
    Write-Host "To push the image:" -ForegroundColor Yellow
    Write-Host "   .\scripts\docker-push.ps1 -Tag $Tag" -ForegroundColor White
}

