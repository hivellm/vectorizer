@echo off
setlocal enabledelayedexpansion

REM Vectorizer Start Script for Windows
REM Supports both compiled binaries and cargo run (development mode)

echo 🚀 Starting Vectorizer Server (Unified Architecture)...
echo ======================================================

REM Set default values
set DAEMON_MODE=false

REM Parse command line arguments
:parse_args
if "%~1"=="" goto :start_server
if "%~1"=="--daemon" (
    set DAEMON_MODE=true
    shift
    goto :parse_args
)
if "%~1"=="--help" goto :usage
if "%~1"=="-h" goto :usage
goto :parse_args

:usage
echo Usage: %0 [OPTIONS]
echo.
echo Options:
echo   --daemon                      Run as daemon/service (background)
echo   --help, -h                    Show this help message
echo.
echo Examples:
echo   %0                            # Start in foreground
echo   %0 --daemon                   # Start in background
exit /b 1

:start_server
REM Check if binary exists
set USE_COMPILED=false
if exist "..\target\release\vectorizer.exe" (
    set USE_COMPILED=true
    echo ✅ Using compiled binary from ..\target\release
) else (
    echo ⚠️  Compiled binary not found, using cargo run (development mode)
    echo    To build binary: cargo build --release
)

echo 🖥️  Operating System: Windows
echo 🔧 Binary Mode: %USE_COMPILED%
echo 👻 Daemon Mode: %DAEMON_MODE%

REM Start vectorizer server
echo Starting vectorizer server...
if "%USE_COMPILED%"=="true" (
    if "%DAEMON_MODE%"=="true" (
        REM In daemon mode, start in background and exit
        start "vectorizer-server" /min ..\target\release\vectorizer.exe
        echo ✅ Vectorizer server started in daemon mode - Port 15002
        echo 📄 Logs: .logs\vectorizer-*.log
        echo 🛑 Use 'scripts\stop.bat' to stop the server
        exit /b 0
    ) else (
        start "vectorizer-server" /min ..\target\release\vectorizer.exe
    )
) else (
    if "%DAEMON_MODE%"=="true" (
        REM In daemon mode, start in background and exit
        start "vectorizer-server" /min cargo run --bin vectorizer
        echo ✅ Vectorizer server started in daemon mode - Port 15002
        echo 📄 Logs: .logs\vectorizer-*.log
        echo 🛑 Use 'scripts\stop.bat' to stop the server
        exit /b 0
    ) else (
        start "vectorizer-server" /min cargo run --bin vectorizer
    )
)

echo ✅ Vectorizer server started - Port 15002

echo.
echo 🎉 Server is running!
echo ======================================================
echo 📡 REST API: http://127.0.0.1:15002
echo 🔧 MCP Server: http://127.0.0.1:15002/mcp/sse
echo 📊 Dashboard: http://127.0.0.1:15002/
echo.
echo 🏗️  Architecture:
echo    Client → REST/MCP → Vector Store
echo.
echo 💡 Press Ctrl+C to stop the server
echo 💡 Use stop.bat to stop the server

REM Keep the script running
pause