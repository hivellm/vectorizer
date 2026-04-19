@echo off
setlocal enabledelayedexpansion

REM Vectorizer Status Script for Windows
REM Shows status of all Vectorizer servers

echo 📊 Vectorizer Server Status (REST + MCP Architecture)
echo ==================================================

REM Check vzr orchestrator (internal server)
tasklist /fi "imagename eq vzr.exe" 2>nul | find /i "vzr.exe" >nul
if !errorlevel! equ 0 (
    echo ✅ vzr Orchestrator (Internal): RUNNING
    echo    Port: 15003 (Internal)
    
    REM Test internal server health
    curl -s --max-time 2 http://127.0.0.1:15003/health >nul 2>&1
    if !errorlevel! equ 0 (
        echo    Health: 🟢 OK
    ) else (
        echo    Health: 🟡 UNREACHABLE
    )
) else (
    echo ❌ vzr Orchestrator (Internal): NOT RUNNING
)

echo.

REM Check MCP server
tasklist /fi "imagename eq vectorizer-mcp-server.exe" 2>nul | find /i "vectorizer-mcp-server.exe" >nul
if !errorlevel! equ 0 (
    echo ✅ MCP Server: RUNNING
    echo    Port: 15002 (WebSocket endpoint: /mcp)
    
    REM Test MCP server health
    curl -s --max-time 2 http://127.0.0.1:15002/health >nul 2>&1
    if !errorlevel! equ 0 (
        echo    Health: 🟢 OK
    ) else (
        echo    Health: 🟡 UNREACHABLE
    )
) else (
    echo ❌ MCP Server: NOT RUNNING
)

echo.

REM Check REST server
tasklist /fi "imagename eq vectorizer.exe" 2>nul | find /i "vectorizer.exe" >nul
if !errorlevel! equ 0 (
    echo ✅ Vectorizer Server: RUNNING
    echo    Port: 15002
    
    REM Test REST server health
    curl -s --max-time 2 http://127.0.0.1:15002/health >nul 2>&1
    if !errorlevel! equ 0 (
        echo    Health: 🟢 OK
        
        REM Get collection stats (if jq is available)
        curl -s --max-time 2 http://127.0.0.1:15002/api/v1/collections > temp_collections.json 2>nul
        if exist temp_collections.json (
            echo    Collections: Available
            del temp_collections.json
        ) else (
            echo    Collections: ?
        )
    ) else (
        echo    Health: 🟡 UNREACHABLE
    )
) else (
    echo ❌ REST API Server: NOT RUNNING
)

echo.
echo 🏗️  Architecture:
echo    Client → REST/MCP → Internal Server → Vector Store
echo.
echo 💡 Commands:
echo    Start all servers: scripts\start.bat
echo    Stop all servers: scripts\stop.bat
echo    Check status: scripts\status.bat
echo    Build binaries: cargo build --release
