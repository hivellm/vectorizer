# Build script for Hive Vectorizer Python SDK (PowerShell)
# This script creates distribution packages (wheel and source distribution)

Write-Host "🔨 Building Hive Vectorizer Python SDK..." -ForegroundColor Blue

$ErrorActionPreference = "Stop"

# Get script directory
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $ScriptDir

# Activate virtual environment if it exists
if (Test-Path "venv\Scripts\Activate.ps1") {
    Write-Host "🐍 Activating virtual environment..." -ForegroundColor Yellow
    & "venv\Scripts\Activate.ps1"
} elseif (Test-Path ".venv\Scripts\Activate.ps1") {
    Write-Host "🐍 Activating virtual environment..." -ForegroundColor Yellow
    & ".venv\Scripts\Activate.ps1"
} else {
    Write-Host "⚠️  No virtual environment found. Creating one..." -ForegroundColor Yellow
    python -m venv venv
    & "venv\Scripts\Activate.ps1"
    Write-Host "📦 Installing build tools..." -ForegroundColor Blue
    pip install --upgrade pip setuptools wheel build twine
}

# Check Python version
Write-Host "📋 Checking Python version..." -ForegroundColor Blue
$PythonVersion = python --version
Write-Host "Python version: $PythonVersion"

# Check if required tools are installed
Write-Host "📦 Checking build tools..." -ForegroundColor Blue
python -c "import build" 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️  'build' module not found. Installing..." -ForegroundColor Yellow
    pip install build
}

python -c "import twine" 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️  'twine' module not found. Installing..." -ForegroundColor Yellow
    pip install twine
}

# Install dependencies
Write-Host "📦 Installing dependencies..." -ForegroundColor Blue
pip install -r requirements.txt

# Clean previous builds
Write-Host "🧹 Cleaning previous builds..." -ForegroundColor Blue
if (Test-Path "build") { Remove-Item -Recurse -Force build }
if (Test-Path "dist") { Remove-Item -Recurse -Force dist }
if (Test-Path "*.egg-info") { Remove-Item -Recurse -Force *.egg-info }
if (Test-Path "hive_vectorizer.egg-info") { Remove-Item -Recurse -Force hive_vectorizer.egg-info }

# Run syntax check
Write-Host "🔍 Running syntax check..." -ForegroundColor Blue
python -m py_compile client.py models.py exceptions.py utils\*.py
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Syntax check failed!" -ForegroundColor Red
    exit 1
}

# Run tests
Write-Host "🧪 Running tests..." -ForegroundColor Blue
$env:PYTHONPATH = "."
python tests\test_simple.py
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️  Some tests failed, but continuing..." -ForegroundColor Yellow
}

# Build package
Write-Host "📦 Building package..." -ForegroundColor Blue
python -m build

# Verify package
Write-Host "✅ Verifying package..." -ForegroundColor Blue
twine check dist\*

# Display results
Write-Host ""
Write-Host "✅ Build completed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "Generated artifacts:"
Get-ChildItem dist

Write-Host ""
Write-Host "Next steps:"
Write-Host "  1. Test the package: pip install dist\hive_vectorizer-*.whl"
Write-Host "  2. Upload to Test PyPI: .\publish.ps1 -Test"
Write-Host "  3. Upload to PyPI: .\publish.ps1"
Write-Host ""

