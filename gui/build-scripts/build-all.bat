@echo off
setlocal enabledelayedexpansion

REM Build script for Vectorizer GUI + Vectorizer backend (Windows)

echo 🔨 Building Vectorizer Complete Package
echo ==========================================

REM Step 1: Build Vectorizer Rust binary
echo.
echo 📦 Step 1: Building Vectorizer binary...
cd ..
cargo build --release

if !errorlevel! neq 0 (
    echo ❌ Vectorizer build failed!
    exit /b 1
)

echo ✅ Vectorizer binary built successfully

REM Step 2: Build GUI
echo.
echo 📦 Step 2: Building Electron GUI...
cd gui

REM Install dependencies if needed
if not exist "node_modules" (
    echo Installing dependencies...
    call pnpm install
)

REM Build frontend and main process
echo Building frontend...
call pnpm run build

if !errorlevel! neq 0 (
    echo ❌ GUI build failed!
    exit /b 1
)

echo ✅ GUI built successfully

REM Step 3: Package for Windows
echo.
echo 📦 Step 3: Packaging application for Windows...
call pnpm run electron:build:win

if !errorlevel! neq 0 (
    echo ❌ Packaging failed!
    exit /b 1
)

echo.
echo ✅ Build Complete!
echo 📁 Output directory: gui\dist-release\
echo.
echo Package includes:
echo   - Vectorizer GUI application
echo   - Vectorizer binary (embedded)
echo   - Default configuration
echo   - Windows Service installer
echo.
echo 🚀 Ready for distribution!

endlocal

