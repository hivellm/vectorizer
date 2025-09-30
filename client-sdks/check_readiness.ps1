# Hive Vectorizer SDK Readiness Checker (PowerShell)
# This script checks if all SDKs are ready for publishing

param(
    [switch]$Help
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
    Write-Host "[✓] $Message" -ForegroundColor $Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[!] $Message" -ForegroundColor $Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[✗] $Message" -ForegroundColor $Red
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

# Function to check if file exists and is not empty
function Test-FileReady {
    param(
        [string]$Path,
        [string]$Description
    )
    
    if ((Test-Path $Path) -and ((Get-Item $Path).Length -gt 0)) {
        Write-Success "$Description exists and is not empty"
        return $true
    } else {
        Write-Error "$Description missing or empty: $Path"
        return $false
    }
}

# Function to check package.json version
function Test-PackageVersion {
    param(
        [string]$Directory,
        [string]$Name
    )
    
    $packageJsonPath = Join-Path $Directory "package.json"
    if (Test-Path $packageJsonPath) {
        try {
            $packageJson = Get-Content $packageJsonPath | ConvertFrom-Json
            $version = $packageJson.version
            if ($version -and $version -ne "") {
                Write-Success "$Name version: $version"
                return $true
            } else {
                Write-Error "$Name version not found or invalid"
                return $false
            }
        }
        catch {
            Write-Error "$Name package.json is invalid JSON"
            return $false
        }
    } else {
        Write-Error "$Name package.json not found"
        return $false
    }
}

# Function to check Python setup.py version
function Test-PythonVersion {
    param(
        [string]$Directory,
        [string]$Name
    )
    
    $setupPyPath = Join-Path $Directory "setup.py"
    if (Test-Path $setupPyPath) {
        try {
            Push-Location $Directory
            $version = python -c "import setup; print(setup.version)" 2>$null
            Pop-Location
            if ($version -and $version -ne "") {
                Write-Success "$Name version: $version"
                return $true
            } else {
                Write-Error "$Name version not found or invalid"
                return $false
            }
        }
        catch {
            Pop-Location
            Write-Error "$Name setup.py version check failed"
            return $false
        }
    } else {
        Write-Error "$Name setup.py not found"
        return $false
    }
}

# Function to check Cargo.toml version
function Test-CargoVersion {
    param(
        [string]$Directory,
        [string]$Name
    )
    
    $cargoTomlPath = Join-Path $Directory "Cargo.toml"
    if (Test-Path $cargoTomlPath) {
        try {
            $content = Get-Content $cargoTomlPath
            $versionLine = $content | Where-Object { $_ -match '^version\s*=\s*"([^"]+)"' }
            if ($versionLine) {
                $version = $matches[1]
                Write-Success "$Name version: $version"
                return $true
            } else {
                Write-Error "$Name version not found in Cargo.toml"
                return $false
            }
        }
        catch {
            Write-Error "$Name Cargo.toml version check failed"
            return $false
        }
    } else {
        Write-Error "$Name Cargo.toml not found"
        return $false
    }
}

# Function to check if tests pass
function Test-SDKTests {
    param(
        [string]$Directory,
        [string]$Name,
        [string]$TestCommand
    )
    
    Write-Status "Running tests for $Name..."
    try {
        Push-Location $Directory
        Invoke-Expression $TestCommand
        if ($LASTEXITCODE -eq 0) {
            Write-Success "$Name tests passed"
            Pop-Location
            return $true
        } else {
            Write-Error "$Name tests failed"
            Pop-Location
            return $false
        }
    }
    catch {
        Write-Error "$Name test execution failed"
        Pop-Location
        return $false
    }
}

# Function to check build process
function Test-SDKBuild {
    param(
        [string]$Directory,
        [string]$Name,
        [string]$BuildCommand
    )
    
    Write-Status "Checking build for $Name..."
    try {
        Push-Location $Directory
        Invoke-Expression $BuildCommand
        if ($LASTEXITCODE -eq 0) {
            Write-Success "$Name builds successfully"
            Pop-Location
            return $true
        } else {
            Write-Error "$Name build failed"
            Pop-Location
            return $false
        }
    }
    catch {
        Write-Error "$Name build execution failed"
        Pop-Location
        return $false
    }
}

# Function to display help
function Show-Help {
    Write-Host "Hive Vectorizer SDK Readiness Checker" -ForegroundColor $Green
    Write-Host ""
    Write-Host "Usage: .\check_readiness.ps1 [OPTIONS]" -ForegroundColor $White
    Write-Host ""
    Write-Host "Options:" -ForegroundColor $Yellow
    Write-Host "  -Help       Show this help message" -ForegroundColor $White
    Write-Host ""
    Write-Host "This script checks if all SDKs are ready for publishing by verifying:" -ForegroundColor $White
    Write-Host "  - Prerequisites are installed" -ForegroundColor $White
    Write-Host "  - Package files exist and are valid" -ForegroundColor $White
    Write-Host "  - Version numbers are set" -ForegroundColor $White
    Write-Host "  - Tests pass" -ForegroundColor $White
    Write-Host "  - Builds succeed" -ForegroundColor $White
    Write-Host "  - Documentation exists" -ForegroundColor $White
}

# Main function
function Main {
    if ($Help) {
        Show-Help
        return
    }
    
    $allGood = $true
    
    Write-Host "==================================================" -ForegroundColor $Green
    Write-Host "    Hive Vectorizer SDK Readiness Checker" -ForegroundColor $Green
    Write-Host "==================================================" -ForegroundColor $Green
    Write-Host ""
    
    # Check prerequisites
    Write-Status "Checking prerequisites..."
    $missingTools = @()
    
    if (-not (Test-Command "npm")) {
        $missingTools += "npm"
    }
    
    if (-not (Test-Command "python")) {
        $missingTools += "python"
    }
    
    if (-not (Test-Command "cargo")) {
        $missingTools += "cargo"
    }
    
    if ($missingTools.Count -gt 0) {
        Write-Error "Missing required tools: $($missingTools -join ', ')"
        $allGood = $false
    } else {
        Write-Success "All prerequisites are installed"
    }
    
    Write-Host ""
    
    # Check TypeScript SDK
    Write-Status "Checking TypeScript SDK..."
    if (Test-FileReady "typescript/package.json" "TypeScript package.json") {
        Test-PackageVersion "typescript" "TypeScript"
        Test-FileReady "typescript/dist/index.js" "TypeScript build output"
        Test-FileReady "typescript/README.md" "TypeScript README"
        Test-FileReady "typescript/CHANGELOG.md" "TypeScript CHANGELOG"
        
        if (Test-Command "npm") {
            Test-SDKTests "typescript" "TypeScript" "npm test"
            Test-SDKBuild "typescript" "TypeScript" "npm run build"
        }
    } else {
        $allGood = $false
    }
    
    Write-Host ""
    
    # Check JavaScript SDK
    Write-Status "Checking JavaScript SDK..."
    if (Test-FileReady "javascript/package.json" "JavaScript package.json") {
        Test-PackageVersion "javascript" "JavaScript"
        Test-FileReady "javascript/dist/index.js" "JavaScript build output"
        Test-FileReady "javascript/README.md" "JavaScript README"
        Test-FileReady "javascript/CHANGELOG.md" "JavaScript CHANGELOG"
        
        if (Test-Command "npm") {
            Test-SDKTests "javascript" "JavaScript" "npm test"
            Test-SDKBuild "javascript" "JavaScript" "npm run build"
        }
    } else {
        $allGood = $false
    }
    
    Write-Host ""
    
    # Check Python SDK
    Write-Status "Checking Python SDK..."
    if (Test-FileReady "python/setup.py" "Python setup.py") {
        Test-PythonVersion "python" "Python"
        Test-FileReady "python/README.md" "Python README"
        Test-FileReady "python/CHANGELOG.md" "Python CHANGELOG"
        Test-FileReady "python/client.py" "Python main module"
        Test-FileReady "python/models.py" "Python models"
        Test-FileReady "python/exceptions.py" "Python exceptions"
        
        if (Test-Command "python") {
            Test-SDKTests "python" "Python" "python run_tests.py"
        }
    } else {
        $allGood = $false
    }
    
    Write-Host ""
    
    # Check Rust SDK
    Write-Status "Checking Rust SDK..."
    if (Test-FileReady "rust/Cargo.toml" "Rust Cargo.toml") {
        Test-CargoVersion "rust" "Rust"
        Test-FileReady "rust/README.md" "Rust README"
        Test-FileReady "rust/CHANGELOG.md" "Rust CHANGELOG"
        Test-FileReady "rust/src/lib.rs" "Rust main library"
        
        if (Test-Command "cargo") {
            Test-SDKTests "rust" "Rust" "cargo test"
            Test-SDKBuild "rust" "Rust" "cargo build --release"
        }
    } else {
        $allGood = $false
    }
    
    Write-Host ""
    
    # Check common files
    Write-Status "Checking common files..."
    Test-FileReady "README.md" "Main README"
    Test-FileReady "CHANGELOG.md" "Main CHANGELOG"
    Test-FileReady "COVERAGE_REPORT.md" "Coverage report"
    Test-FileReady "TESTING.md" "Testing documentation"
    Test-FileReady "PUBLISHING.md" "Publishing documentation"
    
    Write-Host ""
    
    # Final result
    if ($allGood) {
        Write-Success "All SDKs are ready for publishing!"
        Write-Host ""
        Write-Status "Next steps:"
        Write-Host "  1. Review version numbers in all package files" -ForegroundColor $White
        Write-Host "  2. Update CHANGELOG files with release notes" -ForegroundColor $White
        Write-Host "  3. Commit all changes to git" -ForegroundColor $White
        Write-Host "  4. Run the publishing script: .\publish_sdks.ps1" -ForegroundColor $White
    } else {
        Write-Error "Some issues found. Please fix them before publishing."
        Write-Host ""
        Write-Status "Common fixes:"
        Write-Host "  1. Run 'npm install' in TypeScript/JavaScript directories" -ForegroundColor $White
        Write-Host "  2. Run 'pip install -r requirements.txt' in Python directory" -ForegroundColor $White
        Write-Host "  3. Run 'cargo build' in Rust directory" -ForegroundColor $White
        Write-Host "  4. Update version numbers if needed" -ForegroundColor $White
        Write-Host "  5. Ensure all tests pass" -ForegroundColor $White
        exit 1
    }
}

# Run main function
Main
