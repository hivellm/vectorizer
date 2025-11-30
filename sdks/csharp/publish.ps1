# Publish script for Vectorizer C# SDK to NuGet (PowerShell)
# Usage: .\publish.ps1 [API_KEY]

param(
    [string]$ApiKey
)

$ErrorActionPreference = "Stop"

# Get script directory
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $ScriptDir

Write-Host "📤 Publishing Vectorizer C# SDK to NuGet..." -ForegroundColor Blue

# Check if dotnet is installed
if (-not (Get-Command dotnet -ErrorAction SilentlyContinue)) {
    Write-Host "❌ .NET SDK not found." -ForegroundColor Red
    exit 1
}

# Check if artifacts directory exists
if (-not (Test-Path "artifacts") -or (Get-ChildItem "artifacts\*.nupkg" -ErrorAction SilentlyContinue).Count -eq 0) {
    Write-Host "❌ No NuGet packages found in artifacts\. Run .\build.ps1 first." -ForegroundColor Red
    exit 1
}

# Get package files
$PackageFile = Get-ChildItem "artifacts\*.nupkg" | Where-Object { $_.Name -notlike "*.symbols.nupkg" } | Select-Object -First 1
$SymbolsFile = Get-ChildItem "artifacts\*.snupkg" -ErrorAction SilentlyContinue | Select-Object -First 1

if (-not $PackageFile) {
    Write-Host "❌ No .nupkg file found." -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Files to upload:"
$PackageFile | Format-Table Name, Length
if ($SymbolsFile) {
    $SymbolsFile | Format-Table Name, Length
}

# Get API key
if (-not $ApiKey) {
    Write-Host "⚠️  No API key provided as argument." -ForegroundColor Yellow
    Write-Host "Looking for NUGET_API_KEY environment variable..." -ForegroundColor Yellow
    $ApiKey = $env:NUGET_API_KEY
}

if (-not $ApiKey) {
    Write-Host "❌ No API key found!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Usage:"
    Write-Host "  .\publish.ps1 YOUR_API_KEY"
    Write-Host "  or set NUGET_API_KEY environment variable"
    Write-Host ""
    Write-Host "Get your API key from: https://www.nuget.org/account/apikeys"
    exit 1
}

# Confirm upload (unless in CI)
if (-not $env:CI) {
    Write-Host "⚠️  You are about to upload to PRODUCTION NuGet.org!" -ForegroundColor Yellow
    $confirm = Read-Host "Are you sure you want to continue? (yes/no)"
    if ($confirm -ne "yes") {
        Write-Host "Upload cancelled."
        exit 0
    }
}

# Push to NuGet
Write-Host "📤 Uploading to NuGet.org..." -ForegroundColor Blue
dotnet nuget push $PackageFile.FullName `
    --api-key $ApiKey `
    --source https://api.nuget.org/v3/index.json `
    --skip-duplicate

# Push symbols if exists
if ($SymbolsFile) {
    Write-Host "📤 Uploading symbols package..." -ForegroundColor Blue
    dotnet nuget push $SymbolsFile.FullName `
        --api-key $ApiKey `
        --source https://api.nuget.org/v3/index.json `
        --skip-duplicate
}

# Success message
$Version = $PackageFile.Name -replace 'Vectorizer\.Sdk\.(.*)\.nupkg', '$1'

Write-Host ""
Write-Host "✅ Package uploaded successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "View on NuGet: https://www.nuget.org/packages/Vectorizer.Sdk/$Version"
Write-Host ""
Write-Host "Install with:"
Write-Host "  dotnet add package Vectorizer.Sdk --version $Version"
Write-Host ""
Write-Host "Note: It may take a few minutes for the package to be indexed and available."
Write-Host ""

