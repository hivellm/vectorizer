# Script to push Docker image to Docker Hub
# Usage: .\scripts\docker-push.ps1 -Tag 2.1.0
# 
# For building with attestations (recommended for better Docker Scout score):
#   .\scripts\docker-build.ps1 -Tag 2.1.0 -Push

param(
    [Parameter(Mandatory=$false)]
    [string]$Username,
    
    [Parameter(Mandatory=$false)]
    [string]$Tag = "latest",
    
    [Parameter(Mandatory=$false)]
    [string]$Repository = "vectorizer",
    
    [Parameter(Mandatory=$false)]
    [string]$Organization = "hivehub"
)

$ImageName = "vectorizer"
if ($Username) {
    $FullTag = "${Username}/${Repository}:${Tag}"
} else {
    $FullTag = "${Organization}/${Repository}:${Tag}"
}
$SourceTag = "${ImageName}:${Tag}"

Write-Host "🚀 Preparing push to Docker Hub..." -ForegroundColor Cyan
if ($Username) {
    Write-Host "   Username: $Username" -ForegroundColor Yellow
} else {
    Write-Host "   Organization: $Organization" -ForegroundColor Yellow
}
Write-Host "   Repository: $Repository" -ForegroundColor Yellow
Write-Host "   Tag: $Tag" -ForegroundColor Yellow
Write-Host "   Full tag: $FullTag" -ForegroundColor Yellow
Write-Host ""

# Check if image exists (check both possible tags)
Write-Host "📦 Checking if image exists..." -ForegroundColor Cyan
$imageExists = docker images -q "${SourceTag}" 2>$null
$fullTagExists = docker images -q "${FullTag}" 2>$null

if ($fullTagExists) {
    Write-Host "✅ Image found: ${FullTag}" -ForegroundColor Green
    $tagToPush = $FullTag
} elseif ($imageExists) {
    Write-Host "✅ Image found: ${SourceTag}" -ForegroundColor Green
    Write-Host ""
    # Create tag with correct format for Docker Hub
    Write-Host "🏷️  Creating tag for Docker Hub..." -ForegroundColor Cyan
    docker tag "${SourceTag}" "${FullTag}"
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Error creating tag!" -ForegroundColor Red
        exit 1
    }
    Write-Host "✅ Tag created: ${FullTag}" -ForegroundColor Green
    $tagToPush = $FullTag
} else {
    Write-Host "❌ Image not found!" -ForegroundColor Red
    Write-Host "   Build the image first with:" -ForegroundColor Yellow
    Write-Host "   .\scripts\docker-build.ps1 -Tag $Tag" -ForegroundColor White
    Write-Host "   or:" -ForegroundColor Yellow
    Write-Host "   docker build -t ${SourceTag} ." -ForegroundColor White
    exit 1
}
Write-Host ""

# Check login
Write-Host "🔐 Checking Docker Hub login..." -ForegroundColor Cyan
$loginCheck = docker info 2>&1 | Select-String -Pattern "Username"
if (-not $loginCheck) {
    Write-Host "⚠️  You need to login first!" -ForegroundColor Yellow
    Write-Host "   Run: docker login" -ForegroundColor Yellow
    Write-Host ""
    $login = Read-Host "Do you want to login now? (y/n)"
    if ($login -eq "y" -or $login -eq "Y") {
        docker login
        if ($LASTEXITCODE -ne 0) {
            Write-Host "❌ Login failed!" -ForegroundColor Red
            exit 1
        }
    } else {
        Write-Host "❌ Login required to push!" -ForegroundColor Red
        exit 1
    }
}

Write-Host "✅ Login verified" -ForegroundColor Green
Write-Host ""

# Push - always push both the specific tag and latest
Write-Host "📤 Pushing to Docker Hub..." -ForegroundColor Cyan

# Push the specific tag
Write-Host "   Pushing ${FullTag}..." -ForegroundColor Yellow
docker push "${tagToPush}"
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Error pushing image!" -ForegroundColor Red
    exit 1
}

# If tag is not "latest", also push as "latest"
if ($Tag -ne "latest") {
    $latestTag = "${Organization}/${Repository}:latest"
    Write-Host "   Tagging as latest..." -ForegroundColor Yellow
    docker tag "${tagToPush}" "${latestTag}"
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   Pushing ${latestTag}..." -ForegroundColor Yellow
        docker push "${latestTag}"
        if ($LASTEXITCODE -ne 0) {
            Write-Host "⚠️  Warning: Failed to push latest tag" -ForegroundColor Yellow
        }
    }
}

Write-Host ""
Write-Host "✅ Push completed successfully!" -ForegroundColor Green
Write-Host "   Image available at: docker.io/${FullTag}" -ForegroundColor Cyan
if ($Tag -ne "latest") {
    Write-Host "   Also tagged as: docker.io/${Organization}/${Repository}:latest" -ForegroundColor Cyan
}
Write-Host ""
Write-Host "To use the image:" -ForegroundColor Yellow
Write-Host "   docker pull ${FullTag}" -ForegroundColor White
Write-Host "   docker run -d -p 15002:15002 ${FullTag}" -ForegroundColor White
