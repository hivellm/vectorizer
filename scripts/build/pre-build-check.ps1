#!/usr/bin/env pwsh
# Pre-build safety checks for Windows
# Run this before building to verify system is ready

param(
    [switch]$Force,
    [switch]$Verbose
)

Write-Host "=" -ForegroundColor Cyan -NoNewline
Write-Host "=".PadRight(79, "=") -ForegroundColor Cyan
Write-Host "  Vectorizer Pre-Build Safety Checks" -ForegroundColor Yellow
Write-Host "=" -ForegroundColor Cyan -NoNewline
Write-Host "=".PadRight(79, "=") -ForegroundColor Cyan
Write-Host ""

$issues = @()
$warnings = @()

# 1. Check Windows version
Write-Host "[1/10] Checking Windows version..." -ForegroundColor Cyan
$osVersion = [System.Environment]::OSVersion.Version
$buildNumber = (Get-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion").CurrentBuild

if ($osVersion.Major -lt 10) {
    $issues += "Windows 10 or later required (detected: Windows $($osVersion.Major))"
} else {
    Write-Host "      ✅ Windows $($osVersion.Major).$($osVersion.Minor) Build $buildNumber" -ForegroundColor Green
}

# 2. Check available memory
Write-Host ""
Write-Host "[2/10] Checking system memory..." -ForegroundColor Cyan
$totalMemoryGB = [math]::Round((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory / 1GB, 2)
$freeMemoryGB = [math]::Round((Get-CimInstance Win32_OperatingSystem).FreePhysicalMemory / 1MB, 2)

Write-Host "      Total: $totalMemoryGB GB | Free: $freeMemoryGB GB" -ForegroundColor Gray

if ($totalMemoryGB -lt 8) {
    $warnings += "8GB RAM recommended (detected: $totalMemoryGB GB)"
}

if ($freeMemoryGB -lt 2) {
    $issues += "Insufficient free memory (need 2GB+, have $freeMemoryGB GB)"
}

# 3. Check disk space
Write-Host ""
Write-Host "[3/10] Checking disk space..." -ForegroundColor Cyan
$drive = (Get-Location).Drive
$freeSpaceGB = [math]::Round((Get-PSDrive $drive.Name).Free / 1GB, 2)

Write-Host "      Free space on $($drive.Name): $freeSpaceGB GB" -ForegroundColor Gray

if ($freeSpaceGB -lt 10) {
    $issues += "Insufficient disk space (need 10GB+, have $freeSpaceGB GB)"
}

# 4. Check Rust toolchain
Write-Host ""
Write-Host "[4/10] Checking Rust toolchain..." -ForegroundColor Cyan
try {
    $rustVersion = & rustc --version 2>&1
    Write-Host "      ✅ $rustVersion" -ForegroundColor Green
} catch {
    $issues += "Rust not installed. Install from https://rustup.rs"
}

try {
    $cargoVersion = & cargo --version 2>&1
    Write-Host "      ✅ $cargoVersion" -ForegroundColor Green
} catch {
    $issues += "Cargo not found"
}

# 5. Check nightly toolchain
Write-Host ""
Write-Host "[5/10] Checking nightly toolchain..." -ForegroundColor Cyan
try {
    $nightlyVersion = & rustc +nightly --version 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "      ✅ Nightly installed" -ForegroundColor Green
    } else {
        $warnings += "Nightly toolchain not found (will install automatically)"
    }
} catch {
    $warnings += "Nightly toolchain not found"
}

# 6. Check GPU drivers (if GPU features will be used)
Write-Host ""
Write-Host "[6/10] Checking GPU drivers..." -ForegroundColor Cyan
$gpuInfo = Get-WmiObject Win32_VideoController | Select-Object -First 1

if ($gpuInfo) {
    $gpuName = $gpuInfo.Name
    $driverVersion = $gpuInfo.DriverVersion
    $driverDate = $gpuInfo.DriverDate
    
    Write-Host "      GPU: $gpuName" -ForegroundColor Gray
    Write-Host "      Driver: $driverVersion" -ForegroundColor Gray
    
    if ($driverDate) {
        $driverAge = (Get-Date) - [Management.ManagementDateTimeConverter]::ToDateTime($driverDate)
        if ($driverAge.Days -gt 180) {
            $warnings += "GPU drivers are old ($([math]::Round($driverAge.Days / 30)) months). Update recommended."
        }
    }
} else {
    Write-Host "      No GPU detected" -ForegroundColor Gray
}

# 7. Check for conflicting processes
Write-Host ""
Write-Host "[7/10] Checking for conflicting processes..." -ForegroundColor Cyan
$rustProcesses = Get-Process | Where-Object { $_.ProcessName -match "rust|cargo" }

if ($rustProcesses) {
    Write-Host "      ⚠️  $($rustProcesses.Count) Rust processes running" -ForegroundColor Yellow
    if ($Verbose) {
        $rustProcesses | ForEach-Object {
            Write-Host "         - $($_.ProcessName) (PID: $($_.Id))" -ForegroundColor Gray
        }
    }
    $warnings += "Other Rust builds may be running. Consider stopping them."
}

# 8. Check antivirus exclusions
Write-Host ""
Write-Host "[8/10] Checking antivirus exclusions..." -ForegroundColor Cyan
try {
    $exclusions = Get-MpPreference -ErrorAction SilentlyContinue | Select-Object -ExpandProperty ExclusionPath
    $targetExcluded = $exclusions | Where-Object { $_ -match "vectorizer\\target" }
    
    if ($targetExcluded) {
        Write-Host "      ✅ target/ directory excluded" -ForegroundColor Green
    } else {
        $warnings += "target/ directory not in antivirus exclusions. This can slow builds."
        Write-Host "      ⚠️  target/ not excluded from Windows Defender" -ForegroundColor Yellow
        Write-Host "      Run as admin: Add-MpPreference -ExclusionPath 'F:\Node\hivellm\vectorizer\target'" -ForegroundColor Gray
    }
} catch {
    Write-Host "      Could not check antivirus settings" -ForegroundColor Gray
}

# 9. Check virtual memory
Write-Host ""
Write-Host "[9/10] Checking virtual memory (pagefile)..." -ForegroundColor Cyan
$pagefile = Get-CimInstance Win32_PageFileUsage
if ($pagefile) {
    $pagefileSizeGB = [math]::Round($pagefile.AllocatedBaseSize / 1024, 2)
    Write-Host "      Pagefile size: $pagefileSizeGB GB" -ForegroundColor Gray
    
    if ($pagefileSizeGB -lt 8) {
        $warnings += "Small pagefile ($pagefileSizeGB GB). 16GB+ recommended for Rust builds."
    }
} else {
    $warnings += "No pagefile detected. Virtual memory recommended for large builds."
}

# 10. Check recent BSODs
Write-Host ""
Write-Host "[10/10] Checking for recent BSODs..." -ForegroundColor Cyan
try {
    $recentBSODs = Get-EventLog -LogName System -Source "BugCheck" -Newest 5 -ErrorAction SilentlyContinue
    
    if ($recentBSODs) {
        Write-Host "      ⚠️  Found $($recentBSODs.Count) recent BSODs" -ForegroundColor Yellow
        $issues += "Recent BSODs detected. Review Event Viewer before building."
        
        if ($Verbose) {
            $recentBSODs | ForEach-Object {
                Write-Host "         - $($_.TimeGenerated): $($_.Message -split "`n" | Select-Object -First 1)" -ForegroundColor Gray
            }
        }
    } else {
        Write-Host "      ✅ No recent BSODs detected" -ForegroundColor Green
    }
} catch {
    Write-Host "      Could not check BSOD history" -ForegroundColor Gray
}

# Summary
Write-Host ""
Write-Host "=" -ForegroundColor Cyan -NoNewline
Write-Host "=".PadRight(79, "=") -ForegroundColor Cyan
Write-Host "  Summary" -ForegroundColor Yellow
Write-Host "=" -ForegroundColor Cyan -NoNewline
Write-Host "=".PadRight(79, "=") -ForegroundColor Cyan
Write-Host ""

if ($issues.Count -eq 0 -and $warnings.Count -eq 0) {
    Write-Host "✅ All checks passed! System is ready for safe build." -ForegroundColor Green
    Write-Host ""
    Write-Host "Recommended build command:" -ForegroundColor Cyan
    Write-Host "   .\scripts\build-windows-safe.ps1" -ForegroundColor Gray
    exit 0
}

if ($issues.Count -gt 0) {
    Write-Host "❌ Critical issues found:" -ForegroundColor Red
    $issues | ForEach-Object {
        Write-Host "   - $_" -ForegroundColor Red
    }
    Write-Host ""
}

if ($warnings.Count -gt 0) {
    Write-Host "⚠️  Warnings:" -ForegroundColor Yellow
    $warnings | ForEach-Object {
        Write-Host "   - $_" -ForegroundColor Yellow
    }
    Write-Host ""
}

if ($issues.Count -gt 0) {
    Write-Host "Please resolve critical issues before building." -ForegroundColor Red
    Write-Host ""
    
    if (!$Force) {
        exit 1
    } else {
        Write-Host "⚠️  -Force specified, continuing anyway..." -ForegroundColor Yellow
        Write-Host ""
    }
}

if ($warnings.Count -gt 0) {
    Write-Host "Warnings detected. Build may be slower or less stable." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Continue anyway? [Y/n]" -ForegroundColor Cyan -NoNewline
    $response = Read-Host " "
    
    if ($response -ne "" -and $response -ne "Y" -and $response -ne "y") {
        Write-Host "Build cancelled." -ForegroundColor Yellow
        exit 1
    }
}

Write-Host "✅ Pre-build checks complete. Ready to build." -ForegroundColor Green
exit 0


