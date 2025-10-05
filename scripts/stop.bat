@echo off
setlocal enabledelayedexpansion

REM Vectorizer Stop Script for Windows
REM Stops Vectorizer unified server

echo 🛑 Stopping Vectorizer Unified Server...
echo ========================================

set STOPPED=0

REM Kill processes using Vectorizer ports (primary method)
echo Checking for processes using Vectorizer ports...
for %%p in (15002) do (
    for /f "tokens=5" %%a in ('netstat -ano ^| findstr :%%p 2^>nul') do (
        echo Stopping server on port %%p (PID: %%a)
        taskkill /pid %%a 2>nul
        timeout /t 1 /nobreak >nul
        REM Force kill if still running
        taskkill /f /pid %%a 2>nul
        set STOPPED=1
    )
)

REM Stop vectorizer binary processes
echo Stopping vectorizer processes...
taskkill /im vectorizer.exe 2>nul
if !errorlevel! equ 0 (
    set STOPPED=1
    timeout /t 1 /nobreak >nul
    REM Force kill if still running
    taskkill /f /im vectorizer.exe 2>nul
)

REM Stop cargo processes running vectorizer
echo Stopping cargo processes...
for /f "tokens=2" %%a in ('tasklist /fi "imagename eq cargo.exe" /fo list ^| findstr PID') do (
    wmic process where "ProcessId=%%a" get CommandLine 2>nul | findstr /i "vectorizer" >nul
    if !errorlevel! equ 0 (
        echo Stopping cargo process (PID: %%a)
        taskkill /pid %%a 2>nul
        timeout /t 1 /nobreak >nul
        taskkill /f /pid %%a 2>nul
        set STOPPED=1
    )
)

echo.
if !STOPPED! equ 1 (
    echo 🎉 Vectorizer server stopped successfully!
) else (
    echo ℹ️  No Vectorizer server was running
)
echo 🏗️  Architecture: Unified Server (REST/MCP on single process)
