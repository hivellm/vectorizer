@echo off
REM Fix Rollup Build Issues
REM This script fixes common rollup build problems in JavaScript SDK

echo ==============================================
echo     Rollup Build Fix Script
echo ==============================================
echo.

REM Check if JavaScript SDK directory exists
if not exist "javascript" (
    echo [ERROR] JavaScript SDK directory not found
    exit /b 1
)

cd javascript

echo [INFO] Cleaning existing build artifacts...
if exist "node_modules" rmdir /s /q "node_modules"
if exist "package-lock.json" del "package-lock.json"
if exist "dist" rmdir /s /q "dist"

echo [INFO] Reinstalling dependencies...
npm install

if %errorlevel% neq 0 (
    echo [ERROR] npm install failed
    cd ..
    exit /b 1
)

echo [INFO] Testing build...
npm run build

if %errorlevel% neq 0 (
    echo [ERROR] Build still failing
    cd ..
    exit /b 1
) else (
    echo [SUCCESS] Build successful!
)

echo [INFO] Testing publish preparation...
npm run prepublishOnly

if %errorlevel% neq 0 (
    echo [ERROR] Publish preparation failed
    cd ..
    exit /b 1
) else (
    echo [SUCCESS] Publish preparation successful!
)

cd ..

echo.
echo [SUCCESS] Rollup issues fixed successfully!
echo.
echo [INFO] You can now try publishing again:
echo   - npm publish (in javascript directory)
echo   - publish_sdks.ps1 -JavaScript
echo   - publish_sdks.ps1 -All
