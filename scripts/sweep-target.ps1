# sweep-target.ps1 — phase34 / issue #320
#
# Windows parity of scripts/sweep-target.sh. Garbage-collects stale
# artifacts in target\ without nuking the incremental hot set. Wraps
# cargo-sweep so the next build is still fast — only rlibs / object
# files / docs that haven't been touched in $Days days (default 14)
# are removed.
#
# Use this instead of `cargo clean` for routine hygiene. Reach for
# `cargo clean` only when you need a fully cold rebuild (debugging a
# build-script regression, switching toolchains, etc.).
#
# Usage:
#   pwsh scripts/sweep-target.ps1 [-Days 14]
#
# Schedules (Windows):
#   Task Scheduler -> Create Basic Task -> Daily 03:00
#   Action: pwsh.exe -File C:\path\to\vectorizer\scripts\sweep-target.ps1 -Days 14
#
# Full operator runbook: docs/development/rust-target-hygiene.md

[CmdletBinding()]
param(
    [int]$Days = 14
)

$ErrorActionPreference = 'Stop'

$Root = Resolve-Path (Join-Path $PSScriptRoot '..')
if (-not (Test-Path (Join-Path $Root 'Cargo.toml'))) {
    Write-Error "sweep-target.ps1: ${Root}\Cargo.toml not found — am I in a Cargo workspace?"
    exit 1
}

Set-Location $Root

if (-not (Get-Command cargo-sweep -ErrorAction SilentlyContinue)) {
    Write-Host "==> installing cargo-sweep (one-time)"
    cargo install --locked cargo-sweep
}

function Get-DirSize([string]$Path) {
    if (-not (Test-Path $Path)) { return 0L }
    $sum = (Get-ChildItem -Path $Path -Recurse -Force -ErrorAction SilentlyContinue |
        Where-Object { -not $_.PSIsContainer } |
        Measure-Object -Property Length -Sum).Sum
    if ($null -eq $sum) { return 0L }
    return [int64]$sum
}

function Format-Bytes([int64]$Bytes) {
    if ($Bytes -ge 1073741824) { return "{0:N2} GiB" -f ($Bytes / 1073741824) }
    if ($Bytes -ge 1048576)    { return "{0:N2} MiB" -f ($Bytes / 1048576) }
    if ($Bytes -ge 1024)       { return "{0:N2} KiB" -f ($Bytes / 1024) }
    return "$Bytes B"
}

$sizeBefore = Get-DirSize 'target'

Write-Host "==> sweeping artifacts not accessed in the last ${Days} day(s)"
cargo sweep --time $Days

$sizeAfter = Get-DirSize 'target'
$reclaimed = $sizeBefore - $sizeAfter

Write-Host ""
Write-Host ("target\  before: {0}" -f (Format-Bytes $sizeBefore))
Write-Host ("target\   after: {0}" -f (Format-Bytes $sizeAfter))
Write-Host ("      reclaimed: {0}" -f (Format-Bytes $reclaimed))
