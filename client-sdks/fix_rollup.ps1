# Fix Rollup Build Issues
# This script fixes common rollup build problems in JavaScript SDK

param(
    [switch]$Force
)

# Function to write colored output
function Write-Status {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

# Function to fix rollup issues
function Fix-Rollup {
    Write-Status "Fixing rollup build issues..."
    
    # Navigate to JavaScript SDK directory
    if (-not (Test-Path "javascript")) {
        Write-Error "JavaScript SDK directory not found"
        return $false
    }
    
    Push-Location javascript
    try {
        Write-Status "Cleaning existing build artifacts..."
        if (Test-Path "node_modules") { Remove-Item -Recurse -Force "node_modules" }
        if (Test-Path "package-lock.json") { Remove-Item -Force "package-lock.json" }
        if (Test-Path "dist") { Remove-Item -Recurse -Force "dist" }
        
        Write-Status "Reinstalling dependencies..."
        npm install
        
        Write-Status "Testing build..."
        npm run build
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Build successful!"
        } else {
            Write-Error "Build still failing"
            Pop-Location
            return $false
        }
        
        Write-Status "Testing publish preparation..."
        npm run prepublishOnly
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Publish preparation successful!"
        } else {
            Write-Error "Publish preparation failed"
            Pop-Location
            return $false
        }
        
        Write-Success "Rollup issues fixed successfully!"
        return $true
    }
    finally {
        Pop-Location
    }
}

# Main execution
function Main {
    Write-Host "=============================================="
    Write-Host "    Rollup Build Fix Script"
    Write-Host "=============================================="
    Write-Host ""
    
    if (Fix-Rollup) {
        Write-Host ""
        Write-Status "You can now try publishing again:"
        Write-Host "  - npm publish (in javascript directory)"
        Write-Host "  - .\publish_sdks.ps1 -JavaScript"
        Write-Host "  - .\publish_sdks.ps1 -All"
    } else {
        Write-Host ""
        Write-Error "Failed to fix rollup issues"
        exit 1
    }
}

# Run main function
Main
