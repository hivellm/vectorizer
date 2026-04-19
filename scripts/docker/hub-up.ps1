#!/usr/bin/env pwsh
# Start Vectorizer with HiveHub integration

param(
    [switch]$Build,
    [switch]$Logs
)

$ErrorActionPreference = "Stop"

Write-Host "🚀 Starting Vectorizer with HiveHub Integration" -ForegroundColor Cyan

# Check if .env.hub exists
if (-not (Test-Path ".env.hub")) {
    Write-Host "⚠️  .env.hub not found. Creating from template..." -ForegroundColor Yellow
    Copy-Item ".env.hub.example" ".env.hub"
    Write-Host "❗ Please edit .env.hub and set your HIVEHUB_SERVICE_API_KEY" -ForegroundColor Red
    Write-Host "   Then run this script again." -ForegroundColor Red
    exit 1
}

# Check if API key is set
$envContent = Get-Content ".env.hub" -Raw
if ($envContent -match "HIVEHUB_SERVICE_API_KEY=your-service-api-key-here") {
    Write-Host "❗ Please set your HIVEHUB_SERVICE_API_KEY in .env.hub" -ForegroundColor Red
    Write-Host "   Get your API key from: https://hivehub.cloud/dashboard/api-keys" -ForegroundColor Yellow
    exit 1
}

# Build if requested
if ($Build) {
    Write-Host "🔨 Building Docker image..." -ForegroundColor Yellow
    docker-compose -f docker-compose.hub.yml build
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Build failed" -ForegroundColor Red
        exit 1
    }
}

# Start container
Write-Host "▶️  Starting container..." -ForegroundColor Green
docker-compose --env-file .env.hub -f docker-compose.hub.yml up -d

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Failed to start container" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Container started successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "📊 Service Information:" -ForegroundColor Cyan
Write-Host "  REST API:   http://localhost:15002" -ForegroundColor White
Write-Host "  Dashboard:  http://localhost:15002" -ForegroundColor White
Write-Host "  Health:     http://localhost:15002/health" -ForegroundColor White
Write-Host "  Metrics:    http://localhost:15002/prometheus/metrics" -ForegroundColor White
Write-Host "  gRPC:       localhost:15003" -ForegroundColor White
Write-Host ""

# Show logs if requested
if ($Logs) {
    Write-Host "📝 Following logs (Ctrl+C to stop):" -ForegroundColor Cyan
    docker logs -f vectorizer-hub
} else {
    Write-Host "💡 Tip: Use -Logs flag to follow logs" -ForegroundColor Yellow
    Write-Host "   Example: .\scripts\docker-hub-up.ps1 -Logs" -ForegroundColor Gray
    Write-Host ""
    Write-Host "   Or manually: docker logs -f vectorizer-hub" -ForegroundColor Gray
}
