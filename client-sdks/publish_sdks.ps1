# Hive Vectorizer SDK Publisher Script (PowerShell)
# This script publishes all client SDKs to their respective package registries

param(
    [switch]$Help,
    [switch]$Test,
    [switch]$Force,
    [switch]$NoTest,
    [ValidateSet("typescript", "javascript", "python", "rust", "all")]
    [string]$SDK = "all"
)

# Colors for output
$Red = "Red"
$Green = "Green"
$Yellow = "Yellow"
$Blue = "Blue"
$White = "White"

# Function to print colored output
function Write-Status {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor $Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor $Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor $Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor $Red
}

# Function to check if command exists
function Test-Command {
    param([string]$Command)
    try {
        Get-Command $Command -ErrorAction Stop | Out-Null
        return $true
    }
    catch {
        return $false
    }
}

# Function to check prerequisites
function Test-Prerequisites {
    Write-Status "Checking prerequisites..."
    
    $missingTools = @()
    
    if (-not (Test-Command "npm")) {
        $missingTools += "npm"
    }
    
    if (-not (Test-Command "python3")) {
        $missingTools += "python3"
    }
    
    if (-not (Test-Command "pip")) {
        $missingTools += "pip"
    }
    
    if (-not (Test-Command "cargo")) {
        $missingTools += "cargo"
    }
    
    if ($missingTools.Count -gt 0) {
        Write-Error "Missing required tools: $($missingTools -join ', ')"
        Write-Error "Please install the missing tools and try again."
        exit 1
    }
    
    Write-Success "All prerequisites are installed"
}

# Function to run tests before publishing
function Test-AllSDKs {
    Write-Status "Running tests before publishing..."
    
    # TypeScript SDK Tests
    Write-Status "Running TypeScript SDK tests..."
    Push-Location typescript
    try {
        npm test
        if ($LASTEXITCODE -eq 0) {
            Write-Success "TypeScript SDK tests passed"
        } else {
            Write-Error "TypeScript SDK tests failed"
            Pop-Location
            exit 1
        }
    }
    finally {
        Pop-Location
    }
    
    # JavaScript SDK Tests
    Write-Status "Running JavaScript SDK tests..."
    Push-Location javascript
    try {
        npm test
        if ($LASTEXITCODE -eq 0) {
            Write-Success "JavaScript SDK tests passed"
        } else {
            Write-Error "JavaScript SDK tests failed"
            Pop-Location
            exit 1
        }
    }
    finally {
        Pop-Location
    }
    
    # Python SDK Tests
    Write-Status "Running Python SDK tests..."
    Push-Location python
    try {
        python3 run_tests.py
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Python SDK tests passed"
        } else {
            Write-Error "Python SDK tests failed"
            Pop-Location
            exit 1
        }
    }
    finally {
        Pop-Location
    }
    
    # Rust SDK Tests
    Write-Status "Running Rust SDK tests..."
    Push-Location rust
    try {
        cargo test
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Rust SDK tests passed"
        } else {
            Write-Error "Rust SDK tests failed"
            Pop-Location
            exit 1
        }
    }
    finally {
        Pop-Location
    }
    
    Write-Success "All tests passed! Proceeding with publishing..."
}

# Function to handle npm authentication with OTP
function Setup-NpmAuth {
    Write-Status "Setting up npm authentication..."
    
    # Check if already logged in to npm
    $npmUser = npm whoami 2>$null
    if ($npmUser) {
        Write-Success "Already logged in to npm as $npmUser"
        return $true
    }
    
    Write-Warning "Not logged in to npm. Setting up authentication..."
    
    # Check if NPM_TOKEN is available
    if ($env:NPM_TOKEN) {
        Write-Status "Using NPM_TOKEN for authentication..."
        $npmrcContent = "//registry.npmjs.org/:_authToken=$($env:NPM_TOKEN)"
        $npmrcContent | Out-File -FilePath "$env:USERPROFILE\.npmrc" -Encoding utf8
        
        $npmUser = npm whoami 2>$null
        if ($npmUser) {
            Write-Success "Authenticated with NPM_TOKEN as $npmUser"
            return $true
        } else {
            Write-Error "NPM_TOKEN authentication failed"
            return $false
        }
    }
    
    # Interactive login with OTP only
    Write-Status "Please enter your npm credentials..."
    Write-Host "Note: You will only need to provide your OTP (One-Time Password)" -ForegroundColor Yellow
    
    # Set browser for WSL environment
    $env:BROWSER = "wslview"
    
    # Attempt npm login
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

# Function to publish TypeScript SDK
function Publish-TypeScript {
    Write-Status "Publishing TypeScript SDK..."
    
    Push-Location typescript
    try {
        # Setup npm authentication
        if (-not (Setup-NpmAuth)) {
            Write-Error "Authentication failed. Publishing cancelled."
            Pop-Location
            return $false
        }
        
        # Build the package
        Write-Status "Building TypeScript package..."
        npm run build
        
        # Check if package exists
        if (Test-Path "package.json") {
            # Get version from package.json
            $version = (Get-Content package.json | ConvertFrom-Json).version
            Write-Status "Publishing version $version to npm..."
            
            # Publish to npm
            npm publish
            if ($LASTEXITCODE -eq 0) {
                Write-Success "TypeScript SDK v$version published to npm!"
                return $true
            } else {
                Write-Error "Failed to publish TypeScript SDK"
                Pop-Location
                return $false
            }
        } else {
            Write-Error "package.json not found in TypeScript SDK directory"
            Pop-Location
            return $false
        }
    }
    finally {
        Pop-Location
    }
}

# Function to publish JavaScript SDK
function Publish-JavaScript {
    Write-Status "Publishing JavaScript SDK..."
    
    Push-Location javascript
    try {
        # Setup npm authentication (reuse existing auth from TypeScript)
        $npmUser = npm whoami 2>$null
        if (-not $npmUser) {
            Write-Status "Reusing npm authentication..."
            if (-not (Setup-NpmAuth)) {
                Write-Error "Authentication failed. Publishing cancelled."
                Pop-Location
                return $false
            }
        }
        
        # Build the package
        Write-Status "Building JavaScript package..."
        npm run build
        
        # Check if package exists
        if (Test-Path "package.json") {
            # Get version from package.json
            $version = (Get-Content package.json | ConvertFrom-Json).version
            Write-Status "Publishing version $version to npm..."
            
            # Publish to npm
            npm publish
            if ($LASTEXITCODE -eq 0) {
                Write-Success "JavaScript SDK v$version published to npm!"
                return $true
            } else {
                Write-Error "Failed to publish JavaScript SDK"
                Pop-Location
                return $false
            }
        } else {
            Write-Error "package.json not found in JavaScript SDK directory"
            Pop-Location
            return $false
        }
    }
    finally {
        Pop-Location
    }
}

# Function to publish Python SDK
function Publish-Python {
    Write-Status "Publishing Python SDK..."
    
    Push-Location python
    try {
        # Check if twine is installed, install with system packages override if needed
        if (-not (Test-Command "twine")) {
            Write-Status "Installing twine for Python package publishing..."
            pip install twine --break-system-packages
            if ($LASTEXITCODE -eq 0) {
                Write-Success "Twine installed successfully"
            } else {
                Write-Warning "Failed to install twine with --break-system-packages, trying pipx..."
                if (Test-Command "pipx") {
                    pipx install twine
                } else {
                    Write-Error "Neither pip nor pipx could install twine. Please install manually."
                    Pop-Location
                    return $false
                }
            }
        }
        
        # Build the package
        Write-Status "Building Python package..."
        python3 setup.py sdist bdist_wheel
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Python package built successfully"
        } else {
            Write-Error "Failed to build Python package"
            Pop-Location
            return $false
        }
        
        # Check if package was built
        if ((Test-Path "dist") -and ((Get-ChildItem dist).Count -gt 0)) {
            # Get version from setup.py
            $version = python3 -c "import setup; print(setup.version)"
            Write-Status "Publishing version $version to PyPI..."
            
            # Check if credentials are configured
            if (-not (Test-Path "$env:USERPROFILE\.pypirc")) {
                Write-Warning "PyPI credentials not configured. Please run 'twine configure' or set up ~/.pypirc"
                if (-not $Force) {
                    $response = Read-Host "Continue anyway? (y/N)"
                    if ($response -notmatch "^[Yy]$") {
                        Write-Error "Publishing cancelled"
                        Pop-Location
                        return $false
                    }
                }
            }
            
            # Upload to PyPI
            twine upload dist/*
            if ($LASTEXITCODE -eq 0) {
                Write-Success "Python SDK v$version published to PyPI!"
                return $true
            } else {
                Write-Error "Failed to publish Python SDK"
                Pop-Location
                return $false
            }
        } else {
            Write-Error "Failed to build Python package"
            Pop-Location
            return $false
        }
    }
    finally {
        Pop-Location
    }
}

# Function to publish Rust SDK
function Publish-Rust {
    Write-Status "Publishing Rust SDK..."
    
    Push-Location rust
    try {
        # Check if cargo login has been run
        if (-not (Test-Path "$env:USERPROFILE\.cargo\credentials")) {
            Write-Warning "Cargo credentials not configured. Please run 'cargo login' first."
            Write-Status "You need to:"
            Write-Host "  1. Visit https://crates.io/settings/profile"
            Write-Host "  2. Set and verify your email address"
            Write-Host "  3. Run 'cargo login' with your API token"
            if (-not $Force) {
                $response = Read-Host "Continue anyway? (y/N)"
                if ($response -notmatch "^[Yy]$") {
                    Write-Error "Publishing cancelled"
                    Pop-Location
                    return $false
                }
            }
        }
        
        # Check if package is ready for publishing
        cargo package --dry-run
        if ($LASTEXITCODE -eq 0) {
            Write-Status "Package validation successful"
        } else {
            Write-Error "Package validation failed"
            Pop-Location
            return $false
        }
        
        # Get version from Cargo.toml
        $version = (Select-String -Path "Cargo.toml" -Pattern '^version = "(.+)"' | ForEach-Object { $_.Matches[0].Groups[1].Value })
        Write-Status "Publishing version $version to crates.io..."
        
        # Publish to crates.io
        cargo publish
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Rust SDK v$version published to crates.io!"
            return $true
        } else {
            Write-Error "Failed to publish Rust SDK"
            Write-Warning "Common issues:"
            Write-Host "  - Verified email address required: https://crates.io/settings/profile"
            Write-Host "  - API token not configured: run 'cargo login'"
            Write-Host "  - Package name already exists or version conflict"
            Pop-Location
            return $false
        }
    }
    finally {
        Pop-Location
    }
}

# Function to display help
function Show-Help {
    Write-Host "Hive Vectorizer SDK Publisher" -ForegroundColor $Green
    Write-Host ""
    Write-Host "Usage: .\publish_sdks.ps1 [OPTIONS] [SDK]" -ForegroundColor $White
    Write-Host ""
    Write-Host "Options:" -ForegroundColor $Yellow
    Write-Host "  -Help       Show this help message" -ForegroundColor $White
    Write-Host "  -Test       Run tests only (don't publish)" -ForegroundColor $White
    Write-Host "  -Force      Skip confirmation prompts" -ForegroundColor $White
    Write-Host "  -NoTest     Skip running tests before publishing" -ForegroundColor $White
    Write-Host ""
    Write-Host "SDKs:" -ForegroundColor $Yellow
    Write-Host "  typescript   Publish only TypeScript SDK" -ForegroundColor $White
    Write-Host "  javascript   Publish only JavaScript SDK" -ForegroundColor $White
    Write-Host "  python       Publish only Python SDK" -ForegroundColor $White
    Write-Host "  rust         Publish only Rust SDK" -ForegroundColor $White
    Write-Host "  all          Publish all SDKs (default)" -ForegroundColor $White
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor $Yellow
    Write-Host "  .\publish_sdks.ps1                    # Publish all SDKs" -ForegroundColor $White
    Write-Host "  .\publish_sdks.ps1 -Test              # Run tests for all SDKs" -ForegroundColor $White
    Write-Host "  .\publish_sdks.ps1 typescript         # Publish only TypeScript SDK" -ForegroundColor $White
    Write-Host "  .\publish_sdks.ps1 -Force python      # Publish Python SDK without prompts" -ForegroundColor $White
}

# Main function
function Main {
    if ($Help) {
        Show-Help
        return
    }
    
    # Print banner
    Write-Host "==================================================" -ForegroundColor $Green
    Write-Host "    Hive Vectorizer SDK Publisher" -ForegroundColor $Green
    Write-Host "==================================================" -ForegroundColor $Green
    Write-Host ""
    
    # Check prerequisites
    Test-Prerequisites
    
    # Run tests if not skipping
    if (-not $NoTest) {
        Test-AllSDKs
        
        # If only running tests, exit here
        if ($Test) {
            Write-Success "All tests completed successfully!"
            return
        }
    } else {
        Write-Warning "Skipping tests as requested"
    }
    
    # Confirmation prompt unless forced
    if (-not $Force) {
        Write-Host ""
        Write-Warning "This will publish the SDKs to their respective registries."
        Write-Warning "Make sure you have the necessary credentials configured."
        Write-Host ""
        $response = Read-Host "Do you want to continue? (y/N)"
        if ($response -notmatch "^[Yy]$") {
            Write-Error "Publishing cancelled"
            return
        }
    }
    
    # Publish based on selection
    $success = $true
    switch ($SDK) {
        "typescript" {
            $success = Publish-TypeScript
        }
        "javascript" {
            $success = Publish-JavaScript
        }
        "python" {
            $success = Publish-Python
        }
        "rust" {
            $success = Publish-Rust
        }
        "all" {
            Write-Status "Publishing all SDKs..."
            $success = (Publish-TypeScript -and Publish-JavaScript -and Publish-Python -and Publish-Rust)
        }
    }
    
    # Final success message
    if ($success) {
        Write-Host ""
        Write-Success "SDK publishing completed successfully!"
        Write-Host ""
        Write-Status "Published SDKs:"
        if ($SDK -eq "all") {
            Write-Host "  ✅ TypeScript SDK to npm" -ForegroundColor $Green
            Write-Host "  ✅ JavaScript SDK to npm" -ForegroundColor $Green
            Write-Host "  ✅ Python SDK to PyPI" -ForegroundColor $Green
            Write-Host "  ✅ Rust SDK to crates.io" -ForegroundColor $Green
        } else {
            Write-Host "  ✅ $SDK SDK" -ForegroundColor $Green
        }
        Write-Host ""
        Write-Status "Next steps:"
        Write-Host "  1. Verify the packages are available in their registries" -ForegroundColor $White
        Write-Host "  2. Update documentation with new version numbers" -ForegroundColor $White
        Write-Host "  3. Announce the release to users" -ForegroundColor $White
    } else {
        Write-Error "SDK publishing failed!"
        exit 1
    }
}

# Run main function
Main
