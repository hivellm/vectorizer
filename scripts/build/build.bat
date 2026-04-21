@echo off
setlocal enabledelayedexpansion

REM Vectorizer Build Script for Windows
REM Builds optimized binaries for production deployment

echo 🔨 Building Vectorizer Binaries...
echo ==================================

echo 🖥️  Operating System: Windows

REM Build release binaries
echo Building release binaries...
cargo build --release

if !errorlevel! equ 0 (
    echo ✅ Build successful!
    echo.
    echo 📦 Built binaries:
    echo    vectorizer.exe (Main server with REST API and MCP)
    echo.
    echo 📁 Location: target\release\
    echo.
    echo 🚀 Ready for production deployment!
    echo    Use scripts\start.bat to run with compiled binaries
) else (
    echo ❌ Build failed!
    exit /b 1
)
