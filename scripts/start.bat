@echo off
setlocal enabledelayedexpansion

REM Vectorizer Start Script for Windows
REM Supports both compiled binaries and cargo run (development mode)

echo 🚀 Starting Vectorizer Servers (GRPC Architecture)...
echo =====================================================

REM Set default workspace file
set WORKSPACE_FILE=config\vectorize-workspace.yml
set DAEMON_MODE=false

REM Parse command line arguments
:parse_args
if "%~1"=="" goto :start_servers
if "%~1"=="--workspace" (
    if "%~2"=="" (
        echo Error: --workspace requires a file argument
        goto :usage
    )
    set WORKSPACE_FILE=%~2
    shift
    shift
    goto :parse_args
)
if "%~1"=="--daemon" (
    set DAEMON_MODE=true
    shift
    goto :parse_args
)
if "%~1"=="--help" goto :usage
if "%~1"=="-h" goto :usage
if "%~1"=="" goto :start_servers
REM Positional argument (workspace file)
set WORKSPACE_FILE=%~1
shift
goto :parse_args

:usage
echo Usage: %0 [OPTIONS] [WORKSPACE_FILE]
echo.
echo Options:
echo   --workspace WORKSPACE_FILE    Path to vectorize-workspace.yml file
echo   --daemon                      Run as daemon/service (background)
echo   --help, -h                    Show this help message
echo   WORKSPACE_FILE                Path to vectorize-workspace.yml file (positional)
echo.
echo Examples:
echo   %0 --workspace vectorize-workspace.yml
echo   %0 --workspace vectorize-workspace.yml --daemon
echo   %0 ..\my-project\vectorize-workspace.yml
echo   %0 --daemon                   # Uses default: vectorize-workspace.yml
echo   %0                            # Uses default: vectorize-workspace.yml
exit /b 1

:start_servers
REM Check if workspace file exists
if not exist "%WORKSPACE_FILE%" (
    echo Error: Workspace file '%WORKSPACE_FILE%' does not exist
    exit /b 1
)

REM Check if binaries exist
set USE_COMPILED=false
if exist "..\target\release\vzr.exe" if exist "..\target\release\vectorizer-mcp-server.exe" if exist "..\target\release\vectorizer-server.exe" (
    set USE_COMPILED=true
    echo ✅ Using compiled binaries from ..\target\release
) else (
    echo ⚠️  Compiled binaries not found, using cargo run (development mode)
    echo    To build binaries: cargo build --release
)

echo 📁 Workspace File: %WORKSPACE_FILE%
echo 🖥️  Operating System: Windows
echo 🔧 Binary Mode: %USE_COMPILED%
echo 👻 Daemon Mode: %DAEMON_MODE%

REM Build vzr command with daemon option if requested
set VZR_CMD_ARGS=start --workspace "%WORKSPACE_FILE%"
if "%DAEMON_MODE%"=="true" (
    set VZR_CMD_ARGS=%VZR_CMD_ARGS% --daemon
)

REM Start vzr orchestrator (handles all servers internally in workspace mode)
echo Starting vzr orchestrator (GRPC server)...
if "%USE_COMPILED%"=="true" (
    if "%DAEMON_MODE%"=="true" (
        REM In daemon mode, start in background and exit
        start "vzr-orchestrator" /min ..\target\release\vzr.exe %VZR_CMD_ARGS%
        echo ✅ vzr orchestrator started in daemon mode - Port 15003 (GRPC)
        echo 📄 Logs: vectorizer-workspace.log
        echo 🛑 Use 'vectorizer stop' to stop all services
        exit /b 0
    ) else (
        start "vzr-orchestrator" /min ..\target\release\vzr.exe %VZR_CMD_ARGS%
    )
) else (
    if "%DAEMON_MODE%"=="true" (
        REM In daemon mode, start in background and exit
        start "vzr-orchestrator" /min cargo run --bin vzr -- %VZR_CMD_ARGS%
        echo ✅ vzr orchestrator started in daemon mode - Port 15003 (GRPC)
        echo 📄 Logs: vectorizer-workspace.log
        echo 🛑 Use 'vectorizer stop' to stop all services
        exit /b 0
    ) else (
        start "vzr-orchestrator" /min cargo run --bin vzr -- %VZR_CMD_ARGS%
    )
)

echo ✅ vzr orchestrator started - Port 15003 (GRPC)

REM In workspace mode, vzr handles all servers internally
REM No need to start MCP and REST servers separately

echo.
echo 🎉 All servers are running!
echo =====================================================
echo 📡 REST API: http://127.0.0.1:15001
echo 🔧 MCP Server: ws://127.0.0.1:15002/mcp
echo ⚡ GRPC Orchestrator: http://127.0.0.1:15003
echo.
echo 🏗️  Architecture:
echo    Client → REST/MCP → GRPC → vzr → Vector Store
echo.
echo 💡 Press Ctrl+C to stop all servers
echo 💡 Use stop.bat to stop all servers

REM Keep the script running
pause
