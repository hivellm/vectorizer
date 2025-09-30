# Create Release Script for Vectorizer (PowerShell)
# This script helps create GitHub releases with proper versioning

param(
    [Parameter(Position=0)]
    [ValidateSet("patch", "minor", "major")]
    [string]$VersionType,
    
    [string]$Version,
    
    [switch]$DryRun,
    
    [switch]$Help
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

# Function to show help
function Show-Help {
    Write-Host "Vectorizer Release Creator (PowerShell)"
    Write-Host ""
    Write-Host "Usage: .\create_release.ps1 [OPTIONS] VERSION_TYPE"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -Help          Show this help message"
    Write-Host "  -DryRun        Show what would be done without making changes"
    Write-Host "  -Version       Specify exact version instead of auto-increment"
    Write-Host ""
    Write-Host "Version Types:"
    Write-Host "  patch         Increment patch version (0.0.1 -> 0.0.2)"
    Write-Host "  minor         Increment minor version (0.1.0 -> 0.2.0)"
    Write-Host "  major         Increment major version (1.0.0 -> 2.0.0)"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\create_release.ps1 patch                    # Create patch release"
    Write-Host "  .\create_release.ps1 minor                    # Create minor release"
    Write-Host "  .\create_release.ps1 -Version '1.5.0'        # Create release with specific version"
    Write-Host "  .\create_release.ps1 -DryRun patch           # Preview patch release"
}

# Function to check git status
function Test-GitStatus {
    Write-Status "Checking git status..."
    
    $gitStatus = git status --porcelain
    if ($gitStatus) {
        Write-Error "Working directory is not clean. Please commit or stash your changes."
        git status --short
        exit 1
    }
    
    Write-Success "Working directory is clean"
}

# Function to get current version
function Get-CurrentVersion {
    if (Test-Path "Cargo.toml") {
        $line = Select-String -Path "Cargo.toml" -Pattern '^version = '
        if ($line) {
            return ($line.Line -replace 'version = "([^"]*)".*', '$1')
        }
    }
    return "0.0.0"
}

# Function to get next version
function Get-NextVersion {
    param(
        [string]$CurrentVersion,
        [string]$VersionType
    )
    
    $parts = $CurrentVersion.Split('.')
    $major = [int]$parts[0]
    $minor = [int]$parts[1]
    $patch = [int]$parts[2]
    
    switch ($VersionType) {
        "major" { return "$($major + 1).0.0" }
        "minor" { return "$major.$($minor + 1).0" }
        "patch" { return "$major.$minor.$($patch + 1)" }
        default {
            Write-Error "Invalid version type: $VersionType"
            exit 1
        }
    }
}

# Function to update version in files
function Update-Version {
    param([string]$NewVersion)
    
    Write-Status "Updating version to $NewVersion..."
    
    # Update Cargo.toml
    if (Test-Path "Cargo.toml") {
        (Get-Content "Cargo.toml") -replace '^version = ".*"', "version = `"$NewVersion`"" | Set-Content "Cargo.toml"
        Write-Success "Updated Cargo.toml"
    }
    
    # Update client SDKs package.json files
    Get-ChildItem -Path "client-sdks" -Recurse -Name "package.json" | ForEach-Object {
        $file = "client-sdks\$_"
        (Get-Content $file) -replace '"version": ".*"', "`"version`": `"$NewVersion`"" | Set-Content $file
        Write-Success "Updated $file"
    }
    
    # Update client SDKs setup.py files
    Get-ChildItem -Path "client-sdks" -Recurse -Name "setup.py" | ForEach-Object {
        $file = "client-sdks\$_"
        (Get-Content $file) -replace 'version = ".*"', "version = `"$NewVersion`"" | Set-Content $file
        Write-Success "Updated $file"
    }
    
    # Update client SDKs Cargo.toml files
    Get-ChildItem -Path "client-sdks" -Recurse -Name "Cargo.toml" | ForEach-Object {
        $file = "client-sdks\$_"
        (Get-Content $file) -replace '^version = ".*"', "version = `"$NewVersion`"" | Set-Content $file
        Write-Success "Updated $file"
    }
}

# Function to create changelog entry
function New-ChangelogEntry {
    param(
        [string]$NewVersion,
        [string]$VersionType
    )
    
    Write-Status "Creating changelog entry..."
    
    $changelogEntry = @"
## [$NewVersion] - $(Get-Date -Format 'yyyy-MM-dd')

### 🚀 Release Notes

#### Changes
- Version bump to $NewVersion
- Automated release build
- Multi-platform binaries available

#### Platform Support
- ✅ Linux x86_64
- ✅ Linux aarch64  
- ✅ Windows x86_64
- ✅ Windows aarch64
- ✅ macOS x86_64
- ✅ macOS aarch64

#### Installation
``````bash
# Download for your platform from GitHub Releases
wget https://github.com/hivellm/vectorizer/releases/download/v$NewVersion/vectorizer-linux-x86_64.tar.gz
tar -xzf vectorizer-linux-x86_64.tar.gz
./vectorizer-server --config config.yml
``````

"@
    
    if (Test-Path "CHANGELOG.md") {
        $originalContent = Get-Content "CHANGELOG.md"
        $changelogEntry + "`n" + ($originalContent -join "`n") | Set-Content "CHANGELOG.md"
        Write-Success "Updated CHANGELOG.md"
    }
}

# Function to commit and tag
function New-GitCommitAndTag {
    param([string]$NewVersion)
    
    Write-Status "Committing changes..."
    
    git add .
    git commit -m "chore: bump version to $NewVersion"
    
    Write-Status "Creating tag v$NewVersion..."
    git tag -a "v$NewVersion" -m "Release v$NewVersion"
    
    Write-Success "Created tag v$NewVersion"
}

# Function to push to remote
function Push-ToRemote {
    param([string]$NewVersion)
    
    Write-Status "Pushing to remote..."
    
    git push origin main
    git push origin "v$NewVersion"
    
    Write-Success "Pushed to remote"
}

# Main execution
function Main {
    if ($Help) {
        Show-Help
        return
    }
    
    # Validate arguments
    if (-not $VersionType -and -not $Version) {
        Write-Error "Version type or specific version required"
        Show-Help
        exit 1
    }
    
    if ($Version -and $VersionType) {
        Write-Error "Cannot specify both version type and specific version"
        exit 1
    }
    
    # Get current version
    $currentVersion = Get-CurrentVersion
    Write-Status "Current version: $currentVersion"
    
    # Determine new version
    $newVersion = if ($Version) { $Version } else { Get-NextVersion $currentVersion $VersionType }
    Write-Status "New version: $newVersion"
    
    if ($DryRun) {
        Write-Warning "DRY RUN - No changes will be made"
        Write-Host ""
        Write-Host "Would perform the following actions:"
        Write-Host "1. Check git status"
        Write-Host "2. Update version to $newVersion in:"
        Write-Host "   - Cargo.toml"
        Write-Host "   - client-sdks/*/package.json"
        Write-Host "   - client-sdks/*/setup.py"
        Write-Host "   - client-sdks/*/Cargo.toml"
        Write-Host "3. Update CHANGELOG.md"
        Write-Host "4. Commit changes"
        Write-Host "5. Create tag v$newVersion"
        Write-Host "6. Push to remote"
        Write-Host "7. Trigger GitHub Actions release workflow"
        return
    }
    
    # Confirm release
    Write-Host ""
    Write-Warning "This will create a release for version $newVersion"
    $response = Read-Host "Continue? (y/N)"
    if ($response -notmatch "^[Yy]$") {
        Write-Error "Release cancelled"
        exit 1
    }
    
    # Execute release process
    Write-Host ""
    Write-Status "Starting release process for v$newVersion..."
    
    Test-GitStatus
    Update-Version $newVersion
    New-ChangelogEntry $newVersion $VersionType
    New-GitCommitAndTag $newVersion
    Push-ToRemote $newVersion
    
    Write-Host ""
    Write-Success "Release v$newVersion created successfully!"
    Write-Host ""
    Write-Status "Next steps:"
    Write-Host "1. GitHub Actions will automatically build and create the release"
    Write-Host "2. Monitor the workflow at: https://github.com/hivellm/vectorizer/actions"
    Write-Host "3. Download links will be available at: https://github.com/hivellm/vectorizer/releases/tag/v$newVersion"
    Write-Host ""
    Write-Status "The release includes:"
    Write-Host "- Linux x86_64 and aarch64 binaries"
    Write-Host "- Windows x86_64 and aarch64 binaries"
    Write-Host "- macOS x86_64 and aarch64 binaries"
    Write-Host "- Installation scripts for each platform"
    Write-Host "- Configuration files and documentation"
}

# Run main function
Main

