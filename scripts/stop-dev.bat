@echo off
setlocal enabledelayedexpansion

REM Vectorizer Development Stop Script for Windows
REM Stops all development servers (cargo run processes)

echo 🛑 Stopping Vectorizer Development Servers...
echo ==============================================

REM Stop cargo processes running vzr
echo Stopping vzr orchestrator (development)...
for /f "tokens=2" %%a in ('tasklist /fi "imagename eq cargo.exe" /fo csv ^| findstr /i "cargo"') do (
    wmic process where "name='cargo.exe' and commandline like '%%vzr%%'" get processid /value 2>nul | findstr ProcessId
    if !errorlevel! equ 0 (
        echo Stopping cargo vzr process
        taskkill /f /im cargo.exe /fi "commandline like '%%vzr%%'" 2>nul
    )
)

REM Stop cargo processes running vectorizer-mcp-server
echo Stopping MCP servers (development)...
for /f "tokens=2" %%a in ('tasklist /fi "imagename eq cargo.exe" /fo csv ^| findstr /i "cargo"') do (
    wmic process where "name='cargo.exe' and commandline like '%%vectorizer-mcp-server%%'" get processid /value 2>nul | findstr ProcessId
    if !errorlevel! equ 0 (
        echo Stopping cargo MCP process
        taskkill /f /im cargo.exe /fi "commandline like '%%vectorizer-mcp-server%%'" 2>nul
    )
)

REM Stop cargo processes running vectorizer-server
echo Stopping REST servers (development)...
for /f "tokens=2" %%a in ('tasklist /fi "imagename eq cargo.exe" /fo csv ^| findstr /i "cargo"') do (
    wmic process where "name='cargo.exe' and commandline like '%%vectorizer-server%%'" get processid /value 2>nul | findstr ProcessId
    if !errorlevel! equ 0 (
        echo Stopping cargo REST process
        taskkill /f /im cargo.exe /fi "commandline like '%%vectorizer-server%%'" 2>nul
    )
)

REM Stop all cargo processes (simpler approach)
echo Stopping all cargo processes...
taskkill /f /im cargo.exe 2>nul
if !errorlevel! equ 0 (
    echo ✅ Cargo processes stopped
) else (
    echo ℹ️  No cargo processes running
)

REM Kill processes using Vectorizer ports
echo Checking for processes using Vectorizer ports...
for %%p in (15001 15002 15003) do (
    for /f "tokens=5" %%a in ('netstat -ano ^| findstr :%%p') do (
        echo Killing process using port %%p (PID: %%a)
        taskkill /f /pid %%a 2>nul
    )
)

echo.
echo 🎉 All development servers stopped successfully!
echo 🏗️  Architecture: vzr (GRPC) + MCP + REST servers (development mode)
