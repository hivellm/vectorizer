# Script to replace all println! and eprintln! with tracing macros
# Usage: .\scripts\replace-println.ps1

Write-Host "`n🔍 Replacing all println! and eprintln! with tracing..." -ForegroundColor Cyan

$files = Get-ChildItem -Path . -Recurse -Filter "*.rs" -Exclude "*.bak" | Where-Object {
    # Exclude target and qdrant directories
    if ($_.FullName -match "\\target\\" -or $_.FullName -match "\\qdrant\\") {
        return $false
    }
    $content = Get-Content $_.FullName -Raw -ErrorAction SilentlyContinue
    if ($content) {
        $content -match "println!" -or $content -match "eprintln!"
    } else {
        $false
    }
}

$totalFiles = $files.Count
$processed = 0
$errors = 0

foreach ($file in $files) {
    $processed++
    $relativePath = $file.FullName.Replace((Get-Location).Path + "\", "")
    
    try {
        $content = Get-Content $file.FullName -Raw
        
        # Skip if file doesn't need tracing import
        $needsTracingImport = $content -match "println!" -or $content -match "eprintln!"
        if (-not $needsTracingImport) {
            continue
        }
        
        # Check if already has tracing import
        $hasTracingImport = $content -match "use tracing::"
        
        # Skip build.rs - build scripts cannot use tracing
        if ($file.Name -eq "build.rs") {
            Write-Host "  [$processed/$totalFiles] ⏭️  $relativePath (skipped - build script)" -ForegroundColor Yellow
            continue
        }
        
        # Replace println! with tracing::info!
        $content = $content -replace 'println!\s*\(', 'tracing::info!('
        
        # Replace eprintln! with tracing::error!
        $content = $content -replace 'eprintln!\s*\(', 'tracing::error!('
        
        # Skip if no replacements were made
        if ($content -notmatch "tracing::info!" -and $content -notmatch "tracing::error!") {
            continue
        }
        
        # Add tracing import if needed and not present
        if (-not $hasTracingImport -and ($content -match "tracing::info!" -or $content -match "tracing::error!")) {
            # Find the last use statement
            if ($content -match "(?s)(.*?)(\n\s*use\s+.*?;.*?)(\n)") {
                $beforeUses = $matches[1]
                $uses = $matches[2]
                $afterUses = $matches[3]
                
                # Add tracing import after other use statements
                $content = $beforeUses + $uses + "`nuse tracing::{info, error, warn, debug};" + $afterUses + $content.Substring($beforeUses.Length + $uses.Length + $afterUses.Length)
            } else {
                # Add at the beginning if no use statements found
                $content = "use tracing::{info, error, warn, debug};`n" + $content
            }
        }
        
        # Write back
        Set-Content -Path $file.FullName -Value $content -NoNewline
        Write-Host "  [$processed/$totalFiles] ✅ $relativePath" -ForegroundColor Green
    }
    catch {
        $errors++
        Write-Host "  [$processed/$totalFiles] ❌ $relativePath : $($_.Exception.Message)" -ForegroundColor Red
    }
}

Write-Host "`n✅ Done! Processed $processed files, $errors errors" -ForegroundColor Green

