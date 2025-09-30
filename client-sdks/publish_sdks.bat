@echo off
REM Hive Vectorizer SDK Publisher Script (Windows Batch)
REM This script publishes all client SDKs to their respective package registries

setlocal enabledelayedexpansion

REM Check if PowerShell is available
powershell -Command "Get-Command powershell" >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] PowerShell is required but not found
    echo Please install PowerShell and try again
    pause
    exit /b 1
)

REM Check arguments
set "SDK=all"
set "TEST_ONLY=false"
set "FORCE=false"
set "NO_TEST=false"

:parse_args
if "%~1"=="" goto :main
if "%~1"=="--help" goto :show_help
if "%~1"=="-h" goto :show_help
if "%~1"=="--test" (
    set "TEST_ONLY=true"
    shift
    goto :parse_args
)
if "%~1"=="-t" (
    set "TEST_ONLY=true"
    shift
    goto :parse_args
)
if "%~1"=="--force" (
    set "FORCE=true"
    shift
    goto :parse_args
)
if "%~1"=="-f" (
    set "FORCE=true"
    shift
    goto :parse_args
)
if "%~1"=="--no-test" (
    set "NO_TEST=true"
    shift
    goto :parse_args
)
if "%~1"=="typescript" (
    set "SDK=typescript"
    shift
    goto :parse_args
)
if "%~1"=="javascript" (
    set "SDK=javascript"
    shift
    goto :parse_args
)
if "%~1"=="python" (
    set "SDK=python"
    shift
    goto :parse_args
)
if "%~1"=="rust" (
    set "SDK=rust"
    shift
    goto :parse_args
)
if "%~1"=="all" (
    set "SDK=all"
    shift
    goto :parse_args
)

echo [ERROR] Unknown option: %~1
goto :show_help

:show_help
echo Hive Vectorizer SDK Publisher
echo.
echo Usage: publish_sdks.bat [OPTIONS] [SDK]
echo.
echo Options:
echo   -h, --help     Show this help message
echo   -t, --test     Run tests only (don't publish)
echo   -f, --force    Skip confirmation prompts
echo   --no-test      Skip running tests before publishing
echo.
echo SDKs:
echo   typescript     Publish only TypeScript SDK
echo   javascript     Publish only JavaScript SDK
echo   python         Publish only Python SDK
echo   rust           Publish only Rust SDK
echo   all            Publish all SDKs (default)
echo.
echo Examples:
echo   publish_sdks.bat                    # Publish all SDKs
echo   publish_sdks.bat --test             # Run tests for all SDKs
echo   publish_sdks.bat typescript         # Publish only TypeScript SDK
echo   publish_sdks.bat --force python     # Publish Python SDK without prompts
pause
exit /b 0

:main
echo ==================================================
echo     Hive Vectorizer SDK Publisher
echo ==================================================
echo.

REM Build PowerShell command arguments
set "PS_ARGS="
if "%TEST_ONLY%"=="true" set "PS_ARGS=%PS_ARGS% -Test"
if "%FORCE%"=="true" set "PS_ARGS=%PS_ARGS% -Force"
if "%NO_TEST%"=="true" set "PS_ARGS=%PS_ARGS% -NoTest"
set "PS_ARGS=%PS_ARGS% -SDK %SDK%"

REM Execute PowerShell script
powershell -ExecutionPolicy Bypass -File "%~dp0publish_sdks.ps1" %PS_ARGS%

if %errorlevel% neq 0 (
    echo.
    echo [ERROR] Publishing failed with exit code %errorlevel%
    pause
    exit /b %errorlevel%
)

echo.
echo [SUCCESS] Publishing completed successfully!
pause
exit /b 0
