# NPM Authentication Script - OTP Only
# This script simplifies npm authentication to only request OTP

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
function Test-NpmAuth {
    $npmUser = npm whoami 2>$null
    if ($npmUser) {
        Write-Success "Already logged in to npm as $npmUser"
        return $true
    } else {
        return $false
    }
}

# Function to setup authentication with OTP
function Set-NpmAuth {
    Write-Status "Setting up npm authentication..."
    
    # Check if already logged in
    if (Test-NpmAuth) {
        return $true
    }
    
    Write-Warning "Not logged in to npm. Setting up authentication..."
    
    # Check if NPM_TOKEN is available
    if ($env:NPM_TOKEN) {
        Write-Status "Using NPM_TOKEN for authentication..."
        $npmrcContent = "//registry.npmjs.org/:_authToken=$($env:NPM_TOKEN)"
        $npmrcPath = Join-Path $env:USERPROFILE ".npmrc"
        $npmrcContent | Out-File -FilePath $npmrcPath -Encoding utf8
        
        if (Test-NpmAuth) {
            return $true
        } else {
            Write-Error "NPM_TOKEN authentication failed"
            return $false
        }
    }
    
    # Interactive login with OTP only
    Write-Status "Starting npm login process..."
    Write-Host ""
    Write-Warning "You will be prompted for:"
    Write-Host "  1. Username"
    Write-Host "  2. Password" 
    Write-Host "  3. Email"
    Write-Host "  4. OTP (One-Time Password) - This is the main step"
    Write-Host ""
    Write-Status "Setting browser to 'wslview' for WSL environment..."
    
    # Set browser for WSL environment
    $env:BROWSER = "wslview"
    
    # Attempt npm login
    Write-Status "Running 'npm login'..."
    npm login
    
    if ($LASTEXITCODE -eq 0) {
        $npmUser = npm whoami 2>$null
        Write-Success "Successfully logged in to npm as $npmUser"
        return $true
    } else {
        Write-Error "npm login failed"
        return $false
    }
}

# Main execution
function Main {
    Write-Host "=============================================="
    Write-Host "    NPM Authentication - OTP Only"
    Write-Host "=============================================="
    Write-Host ""
    
    if (Set-NpmAuth) {
        Write-Host ""
        Write-Success "Authentication completed successfully!"
        Write-Status "You can now publish packages using:"
        Write-Host "  - npm publish"
        Write-Host "  - .\publish_sdks.ps1 -TypeScript"
        Write-Host "  - .\publish_sdks.ps1 -JavaScript"
        Write-Host "  - .\publish_sdks.ps1 -All"
    } else {
        Write-Host ""
        Write-Error "Authentication failed!"
        Write-Status "You can try:"
        Write-Host "  1. Run this script again"
        Write-Host "  2. Set NPM_TOKEN environment variable"
        Write-Host "  3. Run 'npm login' manually"
        exit 1
    }
}

# Run main function
Main
