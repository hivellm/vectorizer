# Cargo Authentication Setup Script
# This script helps set up authentication for publishing to crates.io

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

# Function to check if already logged in
function Test-CargoAuth {
    $credentialsPath = Join-Path $env:USERPROFILE ".cargo\credentials"
    if (Test-Path $credentialsPath) {
        Write-Success "Cargo credentials found"
        return $true
    } else {
        return $false
    }
}

# Function to setup cargo authentication
function Set-CargoAuth {
    Write-Status "Setting up Cargo authentication for crates.io..."
    
    # Check if already logged in
    if (Test-CargoAuth) {
        Write-Success "Already authenticated with Cargo"
        return $true
    }
    
    Write-Warning "Cargo authentication not configured"
    Write-Host ""
    Write-Status "To publish to crates.io, you need to:"
    Write-Host "  1. Create an account at https://crates.io"
    Write-Host "  2. Verify your email address at https://crates.io/settings/profile"
    Write-Host "  3. Get your API token from https://crates.io/settings/tokens"
    Write-Host "  4. Run 'cargo login' with your API token"
    Write-Host ""
    
    if (-not $Force) {
        $response = Read-Host "Do you want to run 'cargo login' now? (y/N)"
        if ($response -notmatch "^[Yy]$") {
            Write-Warning "Skipping cargo login"
            Write-Status "You can run 'cargo login' later when you're ready"
            return $false
        }
    }
    
    Write-Status "Running 'cargo login'..."
    Write-Host "You will be prompted for your API token from crates.io"
    Write-Host ""
    
    cargo login
    
    if ($LASTEXITCODE -eq 0) {
        Write-Success "Successfully authenticated with Cargo!"
        return $true
    } else {
        Write-Error "Cargo login failed"
        return $false
    }
}

# Function to test cargo publishing
function Test-CargoPublish {
    Write-Status "Testing cargo publishing setup..."
    
    if (-not (Test-Path "rust")) {
        Write-Error "Rust SDK directory not found"
        return $false
    }
    
    Push-Location rust
    try {
        Write-Status "Running cargo package --dry-run..."
        cargo package --dry-run
        
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Cargo package validation successful!"
            Write-Status "Your setup is ready for publishing"
            return $true
        } else {
            Write-Error "Cargo package validation failed"
            return $false
        }
    }
    finally {
        Pop-Location
    }
}

# Main execution
function Main {
    Write-Host "=============================================="
    Write-Host "    Cargo Authentication Setup"
    Write-Host "=============================================="
    Write-Host ""
    
    if (Set-CargoAuth) {
        Write-Host ""
        if (Test-CargoPublish) {
            Write-Host ""
            Write-Success "Cargo authentication setup completed successfully!"
            Write-Status "You can now publish the Rust SDK:"
            Write-Host "  - cargo publish (in rust directory)"
            Write-Host "  - .\publish_sdks.ps1 -Rust"
            Write-Host "  - .\publish_sdks.ps1 -All"
        } else {
            Write-Host ""
            Write-Warning "Authentication setup completed, but package validation failed"
            Write-Status "Please check your Cargo.toml configuration"
        }
    } else {
        Write-Host ""
        Write-Warning "Cargo authentication setup incomplete"
        Write-Status "Please complete the setup manually:"
        Write-Host "  1. Visit https://crates.io/settings/profile"
        Write-Host "  2. Verify your email address"
        Write-Host "  3. Get API token from https://crates.io/settings/tokens"
        Write-Host "  4. Run 'cargo login'"
    }
}

# Run main function
Main

