@echo off
setlocal enabledelayedexpansion

REM Vectorizer Development Start Script for Windows
REM Always uses cargo run for development (never uses compiled binaries)

echo 🚀 Starting Vectorizer Servers (Development Mode)...
echo ====================================================

REM Set default workspace file
set WORKSPACE_FILE=config\vectorize-workspace.yml

REM Parse command line arguments
:parse_args
if "%~1"=="" goto :start_servers
if "%~1"=="--workspace" (
    set WORKSPACE_FILE=%~2
    shift
    shift
    goto :parse_args
)
if "%~1"=="--help" goto :usage
if "%~1"=="-h" goto :usage
if "%~1"=="" goto :start_servers
set WORKSPACE_FILE=%~1
shift
goto :parse_args

:usage
echo Usage: %0 [--workspace WORKSPACE_FILE]
echo        %0 WORKSPACE_FILE
echo.
echo Options:
echo   --workspace WORKSPACE_FILE    Path to vectorize-workspace.yml file
echo   WORKSPACE_FILE                Path to vectorize-workspace.yml file (positional)
echo.
echo Examples:
echo   %0 --workspace vectorize-workspace.yml
echo   %0 ..\my-project\vectorize-workspace.yml
echo   %0                             # Uses default: vectorize-workspace.yml
echo.
echo Note: This script always uses cargo run (development mode)
exit /b 1

:start_servers
REM Check if workspace file exists
if not exist "%WORKSPACE_FILE%" (
    echo Error: Workspace file '%WORKSPACE_FILE%' does not exist
    exit /b 1
)

echo 📁 Workspace File: %WORKSPACE_FILE%
echo 🖥️  Operating System: Windows
echo 🔧 Mode: Development (cargo run)
echo ⚡ Hot reloading enabled

REM Start vzr orchestrator first (GRPC server)
echo Starting vzr orchestrator (GRPC server)...
start "vzr-orchestrator-dev" /min cargo run --bin vzr -- start --workspace "%WORKSPACE_FILE%"
echo ✅ vzr orchestrator started - Port 15003 (GRPC)

REM Wait for vzr to initialize
timeout /t 5 /nobreak >nul

REM Start MCP server (GRPC client)
echo Starting MCP server (GRPC client)...
start "mcp-server-dev" /min cargo run --bin vectorizer-mcp-server -- --workspace "%WORKSPACE_FILE%"
echo ✅ MCP server started - Port 15002

REM Wait a moment for MCP server to initialize
timeout /t 3 /nobreak >nul

REM Start REST server (GRPC client)
echo Starting REST API server (GRPC client)...
start "rest-server-dev" /min cargo run --bin vectorizer-server -- --host 127.0.0.1 --port 15001 --workspace "%WORKSPACE_FILE%"
echo ✅ REST API server started - Port 15001

echo.
echo 🎉 All development servers are running!
echo ====================================================
echo 📡 REST API: http://127.0.0.1:15001
echo 🔧 MCP Server: ws://127.0.0.1:15002/mcp
echo ⚡ GRPC Orchestrator: http://127.0.0.1:15003
echo.
echo 🏗️  Architecture:
echo    Client → REST/MCP → GRPC → vzr → Vector Store
echo.
echo 💡 Development Features:
echo    - Hot reloading enabled
echo    - Debug logging active
echo    - Source code changes trigger rebuilds
echo.
echo 💡 Press Ctrl+C to stop all servers
echo 💡 Use stop-dev.bat to stop all servers

REM Keep the script running
pause
