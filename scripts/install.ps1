# Vectorizer Installation Script for Windows PowerShell
# Installs Vectorizer directly from GitHub repository
# Based on Bun's installation script pattern

param(
    [String]$Version = if ($env:VECTORIZER_VERSION) { $env:VECTORIZER_VERSION } else { "latest" },
    [Switch]$NoPathUpdate = $false,
    [Switch]$Force = $false
)

$ErrorActionPreference = "Stop"

# Configuration
$RepoUrl = "https://github.com/hivellm/vectorizer.git"
$InstallDir = if ($env:VECTORIZER_INSTALL_DIR) { $env:VECTORIZER_INSTALL_DIR } else { "$env:USERPROFILE\.vectorizer" }
$BinDir = if ($env:VECTORIZER_BIN_DIR) { $env:VECTORIZER_BIN_DIR } else { "$env:USERPROFILE\.cargo\bin" }

# Check architecture compatibility
$Arch = (Get-CimInstance Win32_ComputerSystem).SystemType
if (-not ($Arch -match "x64-based")) {
    Write-Host "❌ Unsupported architecture: $Arch" -ForegroundColor Red
    Write-Host "Vectorizer for Windows currently supports x64-based systems only."
    exit 1
}

Write-Host "🚀 Vectorizer Installation Script" -ForegroundColor Green
Write-Host "OS: Windows | Architecture: $Arch" -ForegroundColor Cyan
Write-Host ""

# Check for Rust
if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Host "⚠️  Rust not found. Installing Rust..." -ForegroundColor Yellow
    try {
        Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "$env:TEMP\rustup-init.exe"
        & "$env:TEMP\rustup-init.exe" -y
        $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
        Remove-Item "$env:TEMP\rustup-init.exe" -ErrorAction SilentlyContinue
    } catch {
        Write-Host "❌ Failed to install Rust." -ForegroundColor Red
        Write-Host "Please install Rust manually from https://rustup.rs/"
        exit 1
    }
} else {
    $RustVersion = & rustc --version 2>$null
    Write-Host "✓ Rust found: $RustVersion" -ForegroundColor Green
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
    git fetch --all --tags --quiet
    
    if ($Version -eq "latest") {
        git checkout main --quiet
        git pull origin main --quiet
        $DisplayVersion = "latest (main branch)"
    } else {
        # Normalize version format
        if ($Version -match "^v?\d+\.\d+\.\d+$") {
            $VersionTag = if ($Version -match "^v") { $Version } else { "v$Version" }
        } else {
            $VersionTag = $Version
        }
        
        git checkout "$VersionTag" --quiet 2>$null
        if ($LASTEXITCODE -eq 0) {
            $DisplayVersion = $VersionTag
        } else {
            git checkout "$Version" --quiet 2>$null
            if ($LASTEXITCODE -eq 0) {
                $DisplayVersion = $Version
            } else {
                Write-Host "❌ Version/tag '$Version' not found." -ForegroundColor Red
                Write-Host "Available tags:"
                git tag --list | Select-Object -Last 10
                Pop-Location
                exit 1
            }
        }
    }
    Pop-Location
} else {
    Write-Host "📦 Cloning repository..." -ForegroundColor Green
    Push-Location $InstallDir
    git clone $RepoUrl vectorizer --quiet
    
    Push-Location vectorizer
    if ($Version -ne "latest") {
        # Normalize version format
        if ($Version -match "^v?\d+\.\d+\.\d+$") {
            $VersionTag = if ($Version -match "^v") { $Version } else { "v$Version" }
        } else {
            $VersionTag = $Version
        }
        
        git checkout "$VersionTag" --quiet 2>$null
        if ($LASTEXITCODE -eq 0) {
            $DisplayVersion = $VersionTag
        } else {
            git checkout "$Version" --quiet 2>$null
            if ($LASTEXITCODE -eq 0) {
                $DisplayVersion = $Version
            } else {
                Write-Host "❌ Version/tag '$Version' not found." -ForegroundColor Red
                Pop-Location
                Pop-Location
                exit 1
            }
        }
    } else {
        $DisplayVersion = "latest (main branch)"
    }
    Pop-Location
    Pop-Location
}

Write-Host "✓ Repository ready (version: $DisplayVersion)" -ForegroundColor Green

# Build the project
Write-Host "🔨 Building Vectorizer (this may take a few minutes)..." -ForegroundColor Green
Push-Location $RepoPath
try {
    cargo build --release 2>&1 | Out-Null
    Write-Host "✓ Build completed successfully" -ForegroundColor Green
} catch {
    Write-Host "⚠️  Build had warnings or errors" -ForegroundColor Yellow
    cargo build --release
}
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
    
    # Check if binary is already running
    $RunningProcesses = Get-Process -Name vectorizer -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq (Join-Path $BinDir "vectorizer.exe") }
    if ($RunningProcesses) {
        Write-Host "⚠️  Vectorizer is currently running. Please stop it before installing." -ForegroundColor Yellow
        exit 1
    }
    
    # Backup existing binary if it exists
    $ExistingBinary = Join-Path $BinDir "vectorizer.exe"
    if (Test-Path $ExistingBinary) {
        try {
            $OldVersion = & $ExistingBinary --version 2>$null
            if (-not $OldVersion) { $OldVersion = "unknown" }
            Write-Host "Backing up existing installation (version: $OldVersion)" -ForegroundColor Cyan
            Move-Item $ExistingBinary "$ExistingBinary.old" -Force -ErrorAction SilentlyContinue
        } catch {
            # Ignore backup errors
        }
    }
    
    Copy-Item $BinaryPath $ExistingBinary -Force
    
    # Remove backup if installation succeeded
    Remove-Item "$ExistingBinary.old" -ErrorAction SilentlyContinue
    
    # Add to PATH if not already there
    if (-not $NoPathUpdate) {
        $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if ($UserPath -notlike "*$BinDir*") {
            Write-Host "⚠️  Adding $BinDir to PATH..." -ForegroundColor Yellow
            [Environment]::SetEnvironmentVariable("Path", "$UserPath;$BinDir", "User")
            $env:Path += ";$BinDir"
        }
    }
    
    # Verify installation
    $InstalledVersion = & $ExistingBinary --version 2>$null
    if (-not $InstalledVersion) { $InstalledVersion = "unknown" }
    
    Write-Host ""
    Write-Host "✅ Vectorizer CLI installed successfully!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Version: $InstalledVersion" -ForegroundColor Green
    Write-Host "Binary location: $BinDir\vectorizer.exe"
    Write-Host ""
    
    # Check if there's another vectorizer in PATH
    $ExistingCommand = Get-Command vectorizer -ErrorAction SilentlyContinue
    if ($ExistingCommand -and $ExistingCommand.Source -ne $ExistingBinary) {
        Write-Host "⚠️  Note: Another vectorizer is already in PATH at: $($ExistingCommand.Source)" -ForegroundColor Yellow
        Write-Host "Typing 'vectorizer' will use the existing installation." -ForegroundColor Yellow
        Write-Host ""
    }
    
    # Install Windows Service
    Write-Host "🔧 Installing Vectorizer as Windows Service..." -ForegroundColor Green
    
    # Check if running as Administrator
    $IsAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    
    if (-not $IsAdmin) {
        Write-Host "⚠️  Service installation requires Administrator privileges." -ForegroundColor Yellow
        Write-Host "Please run PowerShell as Administrator and execute:" -ForegroundColor Yellow
        Write-Host "  sc.exe create Vectorizer binPath= `"$ExistingBinary --host 0.0.0.0 --port 15002`" start= auto" -ForegroundColor Cyan
        Write-Host "  sc.exe start Vectorizer" -ForegroundColor Cyan
        Write-Host ""
    } else {
        # Create data directory
        $DataDir = "$env:ProgramData\Vectorizer"
        if (-not (Test-Path $DataDir)) {
            New-Item -ItemType Directory -Path $DataDir -Force | Out-Null
        }
        
        # Check if service already exists
        $ServiceExists = Get-Service -Name "Vectorizer" -ErrorAction SilentlyContinue
        
        if ($ServiceExists) {
            Write-Host "⚠️  Vectorizer service already exists. Updating..." -ForegroundColor Yellow
            Stop-Service -Name "Vectorizer" -ErrorAction SilentlyContinue
            sc.exe config Vectorizer binPath= "`"$ExistingBinary --host 0.0.0.0 --port 15002`""
        } else {
            Write-Host "Creating Vectorizer Windows Service..." -ForegroundColor Cyan
            sc.exe create Vectorizer binPath= "`"$ExistingBinary --host 0.0.0.0 --port 15002`"" start= auto DisplayName= "Vectorizer Vector Database"
        }
        
        # Configure service to start automatically
        sc.exe config Vectorizer start= auto
        sc.exe failure Vectorizer reset= 86400 actions= restart/5000/restart/10000/restart/20000
        
        # Start service
        Write-Host "Starting Vectorizer service..." -ForegroundColor Cyan
        Start-Service -Name "Vectorizer" -ErrorAction SilentlyContinue
        
        # Wait and check status
        Start-Sleep -Seconds 2
        $ServiceStatus = Get-Service -Name "Vectorizer" -ErrorAction SilentlyContinue
        
        if ($ServiceStatus -and $ServiceStatus.Status -eq "Running") {
            Write-Host "✅ Vectorizer service is running!" -ForegroundColor Green
            Write-Host ""
            Write-Host "Service commands:"
            Write-Host "  Get-Service Vectorizer           # Check status"
            Write-Host "  Restart-Service Vectorizer        # Restart service"
            Write-Host "  Stop-Service Vectorizer           # Stop service"
            Write-Host "  Start-Service Vectorizer           # Start service"
            Write-Host ""
        } else {
            Write-Host "⚠️  Service installed but not running. Check status with:" -ForegroundColor Yellow
            Write-Host "  Get-Service Vectorizer" -ForegroundColor Cyan
            Write-Host ""
        }
    }
    
    Write-Host "CLI commands:"
    Write-Host "  vectorizer --help              # Show CLI help"
    Write-Host "  vectorizer --version           # Show version"
    Write-Host ""
    
    if (-not $NoPathUpdate) {
        Write-Host "💡 Tip: Restart your terminal or run 'refreshenv' to use vectorizer CLI immediately." -ForegroundColor Cyan
    } else {
        Write-Host "⚠️  Skipped adding to PATH (NoPathUpdate flag)" -ForegroundColor Yellow
        Write-Host "Please add $BinDir to your PATH environment variable manually."
    }
} else {
    Write-Host "❌ Build failed. Binary not found." -ForegroundColor Red
    exit 1
}

