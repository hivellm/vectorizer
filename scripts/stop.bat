@echo off
setlocal enabledelayedexpansion

REM Vectorizer Stop Script for Windows
REM Stops all Vectorizer servers and processes

echo 🛑 Stopping Vectorizer Servers (GRPC Architecture)...
echo =====================================================

REM Stop vzr orchestrator processes
echo Stopping vzr orchestrator...
taskkill /f /im vzr.exe 2>nul
if !errorlevel! equ 0 (
    echo ✅ vzr orchestrator stopped
) else (
    echo ℹ️  No vzr orchestrator running
)

REM Stop vectorizer-mcp-server processes
echo Stopping MCP servers...
taskkill /f /im vectorizer-mcp-server.exe 2>nul
if !errorlevel! equ 0 (
    echo ✅ MCP servers stopped
) else (
    echo ℹ️  No MCP servers running
)

REM Stop vectorizer-server processes
echo Stopping REST servers...
taskkill /f /im vectorizer-server.exe 2>nul
if !errorlevel! equ 0 (
    echo ✅ REST servers stopped
) else (
    echo ℹ️  No REST servers running
)

REM Stop cargo processes
echo Stopping cargo processes...
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
echo 🎉 All Vectorizer servers stopped successfully!
echo 🏗️  Architecture: vzr (GRPC) + MCP + REST servers
