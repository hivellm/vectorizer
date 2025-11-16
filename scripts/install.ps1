# Vectorizer Installation Script for Windows PowerShell
# Installs Vectorizer directly from GitHub repository

$ErrorActionPreference = "Stop"

# Configuration
$RepoUrl = "https://github.com/hivellm/vectorizer.git"
$InstallDir = if ($env:VECTORIZER_INSTALL_DIR) { $env:VECTORIZER_INSTALL_DIR } else { "$env:USERPROFILE\.vectorizer" }
$BinDir = if ($env:VECTORIZER_BIN_DIR) { $env:VECTORIZER_BIN_DIR } else { "$env:USERPROFILE\.cargo\bin" }
$Version = if ($env:VECTORIZER_VERSION) { $env:VECTORIZER_VERSION } else { "latest" }

Write-Host "🚀 Vectorizer Installation Script" -ForegroundColor Green
Write-Host ""

# Check for Rust
if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Host "⚠️  Rust not found. Installing Rust..." -ForegroundColor Yellow
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "$env:TEMP\rustup-init.exe"
    & "$env:TEMP\rustup-init.exe" -y
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
}

# Check for Git
if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Git is required but not installed." -ForegroundColor Red
    Write-Host "Please install Git from https://git-scm.com/download/win and try again."
    exit 1
}

# Create install directory
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$RepoPath = Join-Path $InstallDir "vectorizer"

# Clone or update repository
if (Test-Path $RepoPath) {
    Write-Host "📦 Updating existing repository..." -ForegroundColor Yellow
    Push-Location $RepoPath
    git fetch --all --tags
    if ($Version -eq "latest") {
        git checkout main
        git pull origin main
    } else {
        git checkout "v$Version" 2>$null
        if ($LASTEXITCODE -ne 0) {
            git checkout "$Version"
        }
    }
    Pop-Location
} else {
    Write-Host "📦 Cloning repository..." -ForegroundColor Green
    Push-Location $InstallDir
    git clone $RepoUrl vectorizer
    Push-Location vectorizer
    if ($Version -ne "latest") {
        git checkout "v$Version" 2>$null
        if ($LASTEXITCODE -ne 0) {
            git checkout "$Version"
        }
    }
    Pop-Location
    Pop-Location
}

# Build the project
Write-Host "🔨 Building Vectorizer..." -ForegroundColor Green
Push-Location $RepoPath
cargo build --release
Pop-Location

# Create bin directory if it doesn't exist
if (-not (Test-Path $BinDir)) {
    Write-Host "⚠️  Creating $BinDir..." -ForegroundColor Yellow
    New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
}

# Install binary
$BinaryPath = Join-Path $RepoPath "target\release\vectorizer.exe"
if (Test-Path $BinaryPath) {
    Write-Host "📦 Installing binary to $BinDir..." -ForegroundColor Green
    Copy-Item $BinaryPath (Join-Path $BinDir "vectorizer.exe") -Force
    
    # Add to PATH if not already there
    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($UserPath -notlike "*$BinDir*") {
        Write-Host "⚠️  Adding $BinDir to PATH..." -ForegroundColor Yellow
        [Environment]::SetEnvironmentVariable("Path", "$UserPath;$BinDir", "User")
        $env:Path += ";$BinDir"
    }
    
    # Verify installation
    if (Get-Command vectorizer -ErrorAction SilentlyContinue) {
        $InstalledVersion = & vectorizer --version 2>$null
        if (-not $InstalledVersion) { $InstalledVersion = "unknown" }
        Write-Host ""
        Write-Host "✅ Vectorizer installed successfully!" -ForegroundColor Green
        Write-Host ""
        Write-Host "Version: $InstalledVersion"
        Write-Host "Binary location: $BinDir\vectorizer.exe"
        Write-Host ""
        Write-Host "Run 'vectorizer --help' to get started."
        Write-Host ""
        Write-Host "Note: You may need to restart your terminal for PATH changes to take effect."
    } else {
        Write-Host "⚠️  Binary installed but not in PATH." -ForegroundColor Yellow
        Write-Host "Please add $BinDir to your PATH environment variable or restart your terminal."
    }
} else {
    Write-Host "❌ Build failed. Binary not found." -ForegroundColor Red
    exit 1
}

