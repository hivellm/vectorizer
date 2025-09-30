@echo off
setlocal enabledelayedexpansion

REM Vectorizer Start Script for Windows
REM Supports both compiled binaries and cargo run (development mode)

echo 🚀 Starting Vectorizer Servers (GRPC Architecture)...
echo =====================================================

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

REM Start vzr orchestrator first (GRPC server)
echo Starting vzr orchestrator (GRPC server)...
if "%USE_COMPILED%"=="true" (
    start "vzr-orchestrator" /min ..\target\release\vzr.exe start --workspace "%WORKSPACE_FILE%"
) else (
    start "vzr-orchestrator" /min cargo run --bin vzr -- start --workspace "%WORKSPACE_FILE%"
)
echo ✅ vzr orchestrator started - Port 15003 (GRPC)

REM Wait for vzr to initialize
timeout /t 5 /nobreak >nul

REM Start MCP server (GRPC client)
echo Starting MCP server (GRPC client)...
if "%USE_COMPILED%"=="true" (
    start "mcp-server" /min ..\target\release\vectorizer-mcp-server.exe --workspace "%WORKSPACE_FILE%"
) else (
    start "mcp-server" /min cargo run --bin vectorizer-mcp-server -- --workspace "%WORKSPACE_FILE%"
)
echo ✅ MCP server started - Port 15002

REM Wait a moment for MCP server to initialize
timeout /t 3 /nobreak >nul

REM Start REST server (GRPC client)
echo Starting REST API server (GRPC client)...
if "%USE_COMPILED%"=="true" (
    start "rest-server" /min ..\target\release\vectorizer-server.exe --host 127.0.0.1 --port 15001 --workspace "%WORKSPACE_FILE%"
) else (
    start "rest-server" /min cargo run --bin vectorizer-server -- --host 127.0.0.1 --port 15001 --workspace "%WORKSPACE_FILE%"
)
echo ✅ REST API server started - Port 15001

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
