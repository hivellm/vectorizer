#!/usr/bin/env pwsh

# Build script for Windows PowerShell
Write-Host "Building Vectorizer GUI..." -ForegroundColor Green

# Change to gui directory
Set-Location $PSScriptRoot

# Run vite build using node_modules/.bin directly
Write-Host "Building Vite..." -ForegroundColor Cyan
& node_modules\.bin\vite.cmd build
if ($LASTEXITCODE -ne 0) {
    Write-Host "Vite build failed!" -ForegroundColor Red
    exit 1
}

# Run TypeScript compilation
Write-Host "Compiling TypeScript..." -ForegroundColor Cyan
& node_modules\.bin\tsc.cmd -p tsconfig.main.json
if ($LASTEXITCODE -ne 0) {
    Write-Host "TypeScript compilation failed!" -ForegroundColor Red
    exit 1
}

# Run electron-builder
Write-Host "Building Electron package..." -ForegroundColor Cyan
& node_modules\.bin\electron-builder.cmd
if ($LASTEXITCODE -ne 0) {
    Write-Host "Electron build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "Build completed successfully!" -ForegroundColor Green









