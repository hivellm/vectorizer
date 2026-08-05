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
    [switch]$NoCache,

    # Build the optional dense variant (phase33 §5.2 / issue #306) instead of
    # the slim default: default Cargo features off plus `fastembed`, with the
    # MiniLM model pre-fetched into the image. Published as
    # `<Tag>-fastembed`, mirroring the 3.4.0 / 3.5.0 releases.
    [Parameter(Mandatory=$false)]
    [switch]$Fastembed
)

$ImageName = "vectorizer"
if ($Fastembed) {
    $Tag = "${Tag}-fastembed"
}
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
if ($Fastembed) {
    Write-Host "   Variant: fastembed (dense, MiniLM baked in, no default features)" -ForegroundColor Yellow
} else {
    Write-Host "   Variant: default (slim, BM25-only)" -ForegroundColor Yellow
}
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

# Dense-variant build args (runbook § "Optional FastEmbed model pre-fetch").
# `NO_DEFAULT_FEATURES=0` reads like "false", but the Dockerfile expands it as
# `${NO_DEFAULT_FEATURES:+--no-default-features}`, which fires on any non-empty
# value — so 0 *does* disable default features, and the variant compiles as
# `--no-default-features --features fastembed`. Leave the 0 alone: emptying it
# would silently pull hive-gpu and transmutation back into the image.
if ($Fastembed) {
    $buildArgs += @(
        "--build-arg", "ENABLE_FASTEMBED=1",
        "--build-arg", "NO_DEFAULT_FEATURES=0",
        "--build-arg", "FEATURES=fastembed"
    )
}

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

# If tag is not "latest", also tag as latest. The `-fastembed` variant is
# excluded on purpose: it is a side-channel image, and moving `latest` onto it
# would hand every plain `docker pull hivehub/vectorizer` the dense build.
if ($Tag -ne "latest" -and -not $Fastembed) {
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
# Mirror the tagging condition above exactly. These were separate copies of
# `$Tag -ne "latest"` until the -Fastembed switch landed, and the summary kept
# announcing a `latest` move the build had (correctly) skipped — a log that
# invites someone to "fix" a problem that does not exist.
if ($Tag -ne "latest" -and -not $Fastembed) {
    Write-Host "   Also tagged as: docker.io/${Organization}/${Repository}:latest" -ForegroundColor Cyan
} elseif ($Fastembed) {
    Write-Host "   `latest` left pointing at the default variant (by design)." -ForegroundColor Cyan
}
Write-Host ""
Write-Host "📊 Check Docker Scout score:" -ForegroundColor Yellow
Write-Host "   docker scout cves ${FullTag}" -ForegroundColor White

