# Script to push Docker image to Docker Hub
# Usage: .\scripts\docker-push.ps1 -Username YOUR_USERNAME -Tag 1.5.0

param(
    [Parameter(Mandatory=$true)]
    [string]$Username,
    
    [Parameter(Mandatory=$false)]
    [string]$Tag = "latest",
    
    [Parameter(Mandatory=$false)]
    [string]$Repository = "vectorizer"
)

$ImageName = "vectorizer"
$FullTag = "${Username}/${Repository}:${Tag}"
$SourceTag = "${ImageName}:${Tag}"

Write-Host "🚀 Preparing push to Docker Hub..." -ForegroundColor Cyan
Write-Host "   Username: $Username" -ForegroundColor Yellow
Write-Host "   Repository: $Repository" -ForegroundColor Yellow
Write-Host "   Tag: $Tag" -ForegroundColor Yellow
Write-Host "   Full tag: $FullTag" -ForegroundColor Yellow
Write-Host ""

# Check if image exists
Write-Host "📦 Checking if image exists..." -ForegroundColor Cyan
$imageExists = docker images -q "${ImageName}:${Tag}" 2>$null
if (-not $imageExists) {
    Write-Host "❌ Image ${SourceTag} not found!" -ForegroundColor Red
    Write-Host "   Build the image first with:" -ForegroundColor Yellow
    Write-Host "   docker build -t ${SourceTag} ." -ForegroundColor Yellow
    exit 1
}

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

# Push
Write-Host "📤 Pushing to Docker Hub..." -ForegroundColor Cyan
docker push "${FullTag}"
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Error pushing image!" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "✅ Push completed successfully!" -ForegroundColor Green
Write-Host "   Image available at: docker.io/${FullTag}" -ForegroundColor Cyan
Write-Host ""
Write-Host "To use the image:" -ForegroundColor Yellow
Write-Host "   docker pull ${FullTag}" -ForegroundColor White
Write-Host "   docker run -d -p 15002:15002 ${FullTag}" -ForegroundColor White
