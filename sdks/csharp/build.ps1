# Build script for Vectorizer C# SDK (PowerShell)
# This script builds and packages the NuGet package

$ErrorActionPreference = "Stop"

Write-Host "Building Vectorizer C# SDK..." -ForegroundColor Cyan

# Get script directory
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $ScriptDir

# Check if dotnet is installed
if (-not (Get-Command dotnet -ErrorAction SilentlyContinue)) {
    Write-Host "Error: .NET SDK not found. Please install .NET 8.0 SDK." -ForegroundColor Red
    Write-Host "Download from: https://dotnet.microsoft.com/download"
    exit 1
}

# Check .NET version
Write-Host "Checking .NET version..." -ForegroundColor Blue
$DotnetVersion = dotnet --version
Write-Host ".NET SDK version: $DotnetVersion"

# Clean previous builds
Write-Host "Cleaning previous builds..." -ForegroundColor Blue
dotnet clean Vectorizer.csproj --configuration Release
Remove-Item -Path "bin\Release" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -Path "obj\Release" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -Path "artifacts\*.nupkg" -Force -ErrorAction SilentlyContinue
Remove-Item -Path "artifacts\*.snupkg" -Force -ErrorAction SilentlyContinue

# Restore dependencies
Write-Host "Restoring dependencies..." -ForegroundColor Blue
dotnet restore Vectorizer.csproj

# Build the project
Write-Host "Building project..." -ForegroundColor Blue
dotnet build Vectorizer.csproj --configuration Release --no-restore

# Pack NuGet package
Write-Host "Creating NuGet package..." -ForegroundColor Blue
dotnet pack Vectorizer.csproj `
    --configuration Release `
    --no-build `
    --output artifacts `
    --include-symbols `
    --include-source

# Display results
Write-Host ""
Write-Host "Build completed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "Generated artifacts:"
Get-ChildItem -Path "artifacts" -Filter "*.nupkg" -ErrorAction SilentlyContinue | Format-Table Name, Length, LastWriteTime
Get-ChildItem -Path "artifacts" -Filter "*.snupkg" -ErrorAction SilentlyContinue | Format-Table Name, Length, LastWriteTime

# Get package version
$CsprojContent = Get-Content "Vectorizer.csproj"
$Version = ($CsprojContent | Select-String -Pattern '<Version>(.*?)</Version>').Matches.Groups[1].Value

Write-Host ""
Write-Host "Package version: $Version"
Write-Host ""
Write-Host "Next steps:"
Write-Host "  1. Test the package locally"
Write-Host "  2. Upload to NuGet using publish.ps1"
Write-Host ""
