@echo off
REM Cargo Authentication Setup Script
REM This script helps set up authentication for publishing to crates.io

echo ==============================================
echo     Cargo Authentication Setup
echo ==============================================
echo.

REM Check if already logged in
if exist "%USERPROFILE%\.cargo\credentials" (
    echo [SUCCESS] Cargo credentials found
    goto :test_publish
)

echo [WARNING] Cargo authentication not configured
echo.
echo [INFO] To publish to crates.io, you need to:
echo   1. Create an account at https://crates.io
echo   2. Verify your email address at https://crates.io/settings/profile
echo   3. Get your API token from https://crates.io/settings/tokens
echo   4. Run 'cargo login' with your API token
echo.

set /p response="Do you want to run 'cargo login' now? (y/N): "
if /i not "%response%"=="y" (
    echo [WARNING] Skipping cargo login
    echo [INFO] You can run 'cargo login' later when you're ready
    goto :end
)

echo [INFO] Running 'cargo login'...
echo You will be prompted for your API token from crates.io
echo.

cargo login

if %errorlevel% == 0 (
    echo [SUCCESS] Successfully authenticated with Cargo!
    goto :test_publish
) else (
    echo [ERROR] Cargo login failed
    goto :end
)

:test_publish
echo.
echo [INFO] Testing cargo publishing setup...

if not exist "rust" (
    echo [ERROR] Rust SDK directory not found
    goto :end
)

cd rust

echo [INFO] Running cargo package --dry-run...
cargo package --dry-run

if %errorlevel% == 0 (
    echo [SUCCESS] Cargo package validation successful!
    echo [INFO] Your setup is ready for publishing
    cd ..
    echo.
    echo [SUCCESS] Cargo authentication setup completed successfully!
    echo [INFO] You can now publish the Rust SDK:
    echo   - cargo publish (in rust directory)
    echo   - publish_sdks.ps1 -Rust
    echo   - publish_sdks.ps1 -All
) else (
    echo [ERROR] Cargo package validation failed
    cd ..
    echo.
    echo [WARNING] Authentication setup completed, but package validation failed
    echo [INFO] Please check your Cargo.toml configuration
)

goto :end

:end








