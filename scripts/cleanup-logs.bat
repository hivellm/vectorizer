@echo off
REM Log cleanup script for Vectorizer (Windows)
REM This script cleans up log files older than 1 day

echo 🧹 Starting log cleanup for Vectorizer...

REM Change to the project directory
cd /d "%~dp0\.."

REM Create .logs directory if it doesn't exist
if not exist ".logs" mkdir .logs

REM Count files before cleanup
for /f %%i in ('dir /b .logs\*.log 2^>nul ^| find /c /v ""') do set LOG_COUNT_BEFORE=%%i
echo 📊 Found %LOG_COUNT_BEFORE% log files before cleanup

REM Remove log files older than 1 day
set DELETED_COUNT=0
forfiles /p .logs /m *.log /d -1 /c "cmd /c echo 🗑️  Removing old log file: @path && del @path && set /a DELETED_COUNT+=1" 2>nul

REM Count files after cleanup
for /f %%i in ('dir /b .logs\*.log 2^>nul ^| find /c /v ""') do set LOG_COUNT_AFTER=%%i

echo ✅ Log cleanup completed!
echo 📈 Summary:
echo    - Files before cleanup: %LOG_COUNT_BEFORE%
echo    - Files deleted: %DELETED_COUNT%
echo    - Files remaining: %LOG_COUNT_AFTER%

REM Show remaining log files
if %LOG_COUNT_AFTER% gtr 0 (
    echo 📋 Remaining log files:
    for %%f in (.logs\*.log) do (
        echo    - %%f
    )
)

echo 🎉 Log cleanup script finished!
pause
