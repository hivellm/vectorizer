@echo off
REM NPM Authentication Script - OTP Only
REM This script simplifies npm authentication to only request OTP

echo ==============================================
echo     NPM Authentication - OTP Only
echo ==============================================
echo.

REM Check if already logged in
npm whoami >nul 2>&1
if %errorlevel% == 0 (
    echo [SUCCESS] Already logged in to npm
    npm whoami
    goto :success
)

echo [WARNING] Not logged in to npm. Setting up authentication...

REM Check if NPM_TOKEN is available
if defined NPM_TOKEN (
    echo [INFO] Using NPM_TOKEN for authentication...
    echo //registry.npmjs.org/:_authToken=%NPM_TOKEN% > "%USERPROFILE%\.npmrc"
    
    npm whoami >nul 2>&1
    if %errorlevel% == 0 (
        echo [SUCCESS] Authenticated with NPM_TOKEN
        npm whoami
        goto :success
    ) else (
        echo [ERROR] NPM_TOKEN authentication failed
        goto :error
    )
)

REM Interactive login with OTP only
echo [INFO] Starting npm login process...
echo.
echo [WARNING] You will be prompted for:
echo   1. Username
echo   2. Password
echo   3. Email
echo   4. OTP (One-Time Password) - This is the main step
echo.
echo [INFO] Setting browser to 'wslview' for WSL environment...

REM Set browser for WSL environment
set BROWSER=wslview

REM Attempt npm login
echo [INFO] Running 'npm login'...
npm login

if %errorlevel% == 0 (
    echo [SUCCESS] Successfully logged in to npm
    npm whoami
    goto :success
) else (
    echo [ERROR] npm login failed
    goto :error
)

:success
echo.
echo [SUCCESS] Authentication completed successfully!
echo [INFO] You can now publish packages using:
echo   - npm publish
echo   - publish_sdks.ps1 -TypeScript
echo   - publish_sdks.ps1 -JavaScript
echo   - publish_sdks.ps1 -All
goto :end

:error
echo.
echo [ERROR] Authentication failed!
echo [INFO] You can try:
echo   1. Run this script again
echo   2. Set NPM_TOKEN environment variable
echo   3. Run 'npm login' manually
exit /b 1

:end
