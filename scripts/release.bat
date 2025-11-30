@echo off
REM Vectorizer Release Script for Windows
REM Builds and packages Vectorizer for Windows distributions
REM Usage: scripts\release.bat [version] [--skip-tests] [--skip-docs]

setlocal enabledelayedexpansion

REM Configuration
set PROJECT_NAME=vectorizer
set PROJECT_ROOT=%~dp0..
set RELEASE_DIR=%PROJECT_ROOT%\releases
set BUILD_DIR=%PROJECT_ROOT%\target\release

REM Default values
set VERSION=
set SKIP_TESTS=false
set SKIP_DOCS=false
set BUILD_WINDOWS=true

REM Parse command line arguments
:parse_args
if "%~1"=="" goto :args_done
if "%~1"=="--skip-tests" (
    set SKIP_TESTS=true
    shift
    goto :parse_args
)
if "%~1"=="--skip-docs" (
    set SKIP_DOCS=true
    shift
    goto :parse_args
)
if "%~1"=="--help" (
    goto :show_help
)
if "%~1"=="-h" (
    goto :show_help
)
if "%VERSION%"=="" (
    set VERSION=%~1
)
shift
goto :parse_args

:args_done

REM Auto-detect version if not provided
if "%VERSION%"=="" (
    for /f "tokens=2 delims==" %%i in ('findstr "version = " "%PROJECT_ROOT%\Cargo.toml"') do (
        set VERSION=%%i
        set VERSION=!VERSION:"=!
    )
    echo 📦 Auto-detected version: !VERSION!
)

if "%VERSION%"=="" (
    echo ❌ Could not detect version from Cargo.toml
    echo Please specify version manually: scripts\release.bat 1.0.0
    exit /b 1
)

echo 🚀 Starting Vectorizer Release Build
echo Version: %VERSION%
echo Project Root: %PROJECT_ROOT%
echo Release Directory: %RELEASE_DIR%
echo.

REM Create release directory
if not exist "%RELEASE_DIR%" mkdir "%RELEASE_DIR%"

REM Check prerequisites
echo.
echo ================================================
echo 🔍 Checking Prerequisites
echo ================================================

REM Check if cargo exists
cargo --version >nul 2>&1
if errorlevel 1 (
    echo ❌ Rust/Cargo not found
    echo Please install Rust from https://rustup.rs/
    exit /b 1
) else (
    for /f "tokens=2" %%i in ('cargo --version') do echo ✅ Rust: %%i
)

REM Check if zip exists
where zip >nul 2>&1
if errorlevel 1 (
    echo ❌ zip command not found
    echo Please install zip utility
    exit /b 1
) else (
    echo ✅ zip
)

echo ✅ All prerequisites satisfied

REM Run tests
if "%SKIP_TESTS%"=="true" (
    echo.
    echo ⏭️  Skipping tests
) else (
    echo.
    echo ================================================
    echo 🧪 Running Tests
    echo ================================================
    
    cd /d "%PROJECT_ROOT%"
    echo Running cargo test...
    cargo test --release
    if errorlevel 1 (
        echo ❌ Tests failed
        exit /b 1
    )
    echo ✅ All tests passed
)

REM Generate documentation
if "%SKIP_DOCS%"=="true" (
    echo.
    echo ⏭️  Skipping documentation generation
) else (
    echo.
    echo ================================================
    echo 📚 Generating Documentation
    echo ================================================
    
    cd /d "%PROJECT_ROOT%"
    echo Generating Rust documentation...
    cargo doc --release --no-deps
    if errorlevel 1 (
        echo ⚠️  Documentation generation failed, continuing...
    ) else (
        echo ✅ Documentation generated
    )
)

REM Build Windows binaries
echo.
echo ================================================
echo 🪟 Building Windows Binaries
echo ================================================

cd /d "%PROJECT_ROOT%"
echo Building release binaries for Windows...

REM Add Windows target if not already added
cargo target list --installed | findstr "x86_64-pc-windows-gnu" >nul
if errorlevel 1 (
    echo Adding Windows target...
    cargo target add x86_64-pc-windows-gnu
)

REM Build main binaries for Windows
set BINARIES=vectorizer vectorizer-cli
for %%b in (%BINARIES%) do (
    echo Building %%b for Windows...
    cargo build --release --bin %%b
    if errorlevel 1 (
        echo ❌ Failed to build %%b for Windows
        exit /b 1
    )
    echo ✅ %%b (Windows) built successfully
)

echo ✅ All Windows binaries built successfully

REM Create Windows package
echo.
echo ================================================
echo 📦 Creating Windows Package
echo ================================================

set PACKAGE_NAME=%PROJECT_NAME%-%VERSION%-windows-x86_64
set PACKAGE_DIR=%RELEASE_DIR%\%PACKAGE_NAME%

REM Create package directory
if not exist "%PACKAGE_DIR%" mkdir "%PACKAGE_DIR%"

echo Creating Windows package: %PACKAGE_NAME%

REM Copy Windows binaries
copy "%PROJECT_ROOT%\target\release\vectorizer.exe" "%PACKAGE_DIR%\"
copy "%PROJECT_ROOT%\target\release\vectorizer-cli.exe" "%PACKAGE_DIR%\"

REM Copy configuration files
if not exist "%PACKAGE_DIR%\config" mkdir "%PACKAGE_DIR%\config"
copy "%PROJECT_ROOT%\config.example.yml" "%PACKAGE_DIR%\config\"
copy "%PROJECT_ROOT%\workspace.example.yml" "%PACKAGE_DIR%\config\"

REM Copy scripts
if not exist "%PACKAGE_DIR%\scripts" mkdir "%PACKAGE_DIR%\scripts"
copy "%PROJECT_ROOT%\scripts\start.bat" "%PACKAGE_DIR%\scripts\"
copy "%PROJECT_ROOT%\scripts\stop.bat" "%PACKAGE_DIR%\scripts\"
copy "%PROJECT_ROOT%\scripts\status.bat" "%PACKAGE_DIR%\scripts\"
copy "%PROJECT_ROOT%\scripts\start-dev.bat" "%PACKAGE_DIR%\scripts\"
copy "%PROJECT_ROOT%\scripts\stop-dev.bat" "%PACKAGE_DIR%\scripts\"
copy "%PROJECT_ROOT%\scripts\build.bat" "%PACKAGE_DIR%\scripts\"

REM Copy documentation
copy "%PROJECT_ROOT%\README.md" "%PACKAGE_DIR%\"
copy "%PROJECT_ROOT%\CHANGELOG.md" "%PACKAGE_DIR%\"
if exist "%PROJECT_ROOT%\LICENSE" copy "%PROJECT_ROOT%\LICENSE" "%PACKAGE_DIR%\"

REM Copy Python SDK
if exist "%PROJECT_ROOT%\client-sdks\python" (
    xcopy /E /I "%PROJECT_ROOT%\client-sdks\python" "%PACKAGE_DIR%\client-sdks\python"
)

REM Create Windows installation script
echo @echo off > "%PACKAGE_DIR%\install.bat"
echo REM Vectorizer Installation Script for Windows >> "%PACKAGE_DIR%\install.bat"
echo. >> "%PACKAGE_DIR%\install.bat"
echo echo 🚀 Installing Vectorizer... >> "%PACKAGE_DIR%\install.bat"
echo. >> "%PACKAGE_DIR%\install.bat"
echo REM Create installation directory >> "%PACKAGE_DIR%\install.bat"
echo if not exist "C:\Program Files\Vectorizer" mkdir "C:\Program Files\Vectorizer" >> "%PACKAGE_DIR%\install.bat"
echo. >> "%PACKAGE_DIR%\install.bat"
echo REM Copy files >> "%PACKAGE_DIR%\install.bat"
echo xcopy /E /I /Y . "C:\Program Files\Vectorizer" >> "%PACKAGE_DIR%\install.bat"
echo. >> "%PACKAGE_DIR%\install.bat"
echo REM Add to PATH (requires admin) >> "%PACKAGE_DIR%\install.bat"
echo echo Adding Vectorizer to PATH... >> "%PACKAGE_DIR%\install.bat"
echo setx PATH "%%PATH%%;C:\Program Files\Vectorizer" /M >> "%PACKAGE_DIR%\install.bat"
echo. >> "%PACKAGE_DIR%\install.bat"
echo echo ✅ Vectorizer installed successfully! >> "%PACKAGE_DIR%\install.bat"
echo echo. >> "%PACKAGE_DIR%\install.bat"
echo echo To start manually: >> "%PACKAGE_DIR%\install.bat"
echo echo   cd "C:\Program Files\Vectorizer" >> "%PACKAGE_DIR%\install.bat"
echo echo   vectorizer.exe >> "%PACKAGE_DIR%\install.bat"
echo echo. >> "%PACKAGE_DIR%\install.bat"
echo echo Services: >> "%PACKAGE_DIR%\install.bat"
echo echo   REST API: http://localhost:15002 >> "%PACKAGE_DIR%\install.bat"
echo echo   MCP Server: http://localhost:15002/mcp/sse >> "%PACKAGE_DIR%\install.bat"
echo echo. >> "%PACKAGE_DIR%\install.bat"
echo pause >> "%PACKAGE_DIR%\install.bat"

REM Create README for package
echo # Vectorizer %VERSION% - Windows Package > "%PACKAGE_DIR%\PACKAGE_README.md"
echo. >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo ## Quick Start >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo. >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo 1. **Installation:** >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo    - Run `install.bat` as Administrator >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo    - Or manually copy files to desired location >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo. >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo 2. **Start Service:** >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo    ```cmd >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo    vectorizer.exe >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo    ``` >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo. >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo 3. **Development Mode:** >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo    ```cmd >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo    scripts\start.bat >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo    ``` >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo. >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo ## Services >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo. >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo - **REST API**: http://localhost:15002 >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo - **MCP Server**: http://localhost:15002/mcp/sse >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo. >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo ## Configuration >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo. >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo - Edit `config\config.yml` for server settings >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo - Edit `config\workspace.yml` for workspace configuration >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo. >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo ## Python SDK >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo. >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo The Python SDK is included in `client-sdks\python\`. See the README in that directory for usage instructions. >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo. >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo ## Support >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo. >> "%PACKAGE_DIR%\PACKAGE_README.md"
echo For issues and questions, please visit: https://github.com/hivellm/vectorizer >> "%PACKAGE_DIR%\PACKAGE_README.md"

REM Create ZIP archive
cd /d "%RELEASE_DIR%"
zip -r "%PACKAGE_NAME%.zip" "%PACKAGE_NAME%"

echo ✅ Windows package created: %PACKAGE_NAME%.zip

REM Create release notes
echo.
echo ================================================
echo 📝 Creating Release Notes
echo ================================================

set RELEASE_NOTES_FILE=%RELEASE_DIR%\RELEASE_NOTES_%VERSION%.md

echo # Vectorizer Release %VERSION% > "%RELEASE_NOTES_FILE%"
echo. >> "%RELEASE_NOTES_FILE%"
echo ## 📦 Packages >> "%RELEASE_NOTES_FILE%"
echo. >> "%RELEASE_NOTES_FILE%"
echo ### Windows (x86_64) >> "%RELEASE_NOTES_FILE%"
echo - **File**: `%PROJECT_NAME%-%VERSION%-windows-x86_64.zip` >> "%RELEASE_NOTES_FILE%"
echo - **Installation**: Extract and run `install.bat` as Administrator >> "%RELEASE_NOTES_FILE%"
echo - **Manual Start**: `vectorizer.exe` >> "%RELEASE_NOTES_FILE%"
echo. >> "%RELEASE_NOTES_FILE%"
echo ## 🚀 Quick Start >> "%RELEASE_NOTES_FILE%"
echo. >> "%RELEASE_NOTES_FILE%"
echo 1. **Download** the Windows package >> "%RELEASE_NOTES_FILE%"
echo 2. **Extract** the ZIP archive >> "%RELEASE_NOTES_FILE%"
echo 3. **Install** using the provided installation script >> "%RELEASE_NOTES_FILE%"
echo 4. **Start** the service >> "%RELEASE_NOTES_FILE%"
echo. >> "%RELEASE_NOTES_FILE%"
echo ## 🔧 Services >> "%RELEASE_NOTES_FILE%"
echo. >> "%RELEASE_NOTES_FILE%"
echo - **REST API**: http://localhost:15002 >> "%RELEASE_NOTES_FILE%"
echo - **MCP Server**: http://localhost:15002/mcp/sse >> "%RELEASE_NOTES_FILE%"
echo. >> "%RELEASE_NOTES_FILE%"
echo ## 📚 Documentation >> "%RELEASE_NOTES_FILE%"
echo. >> "%RELEASE_NOTES_FILE%"
echo - **README**: Complete documentation included in package >> "%RELEASE_NOTES_FILE%"
echo - **Configuration**: Edit `config\config.yml` and `config\workspace.yml` >> "%RELEASE_NOTES_FILE%"
echo - **Python SDK**: Included in `client-sdks\python\` >> "%RELEASE_NOTES_FILE%"
echo. >> "%RELEASE_NOTES_FILE%"
echo ## 🐛 Support >> "%RELEASE_NOTES_FILE%"
echo. >> "%RELEASE_NOTES_FILE%"
echo - **Issues**: https://github.com/hivellm/vectorizer/issues >> "%RELEASE_NOTES_FILE%"
echo - **Documentation**: https://github.com/hivellm/vectorizer/blob/main/README.md >> "%RELEASE_NOTES_FILE%"
echo. >> "%RELEASE_NOTES_FILE%"
echo ## 📋 System Requirements >> "%RELEASE_NOTES_FILE%"
echo. >> "%RELEASE_NOTES_FILE%"
echo ### Windows >> "%RELEASE_NOTES_FILE%"
echo - Windows 10+ (x64) >> "%RELEASE_NOTES_FILE%"
echo - Visual C++ Redistributable >> "%RELEASE_NOTES_FILE%"
echo - 2GB+ RAM recommended >> "%RELEASE_NOTES_FILE%"
echo - 1GB+ disk space >> "%RELEASE_NOTES_FILE%"

echo ✅ Release notes created: %RELEASE_NOTES_FILE%

REM Show final summary
echo.
echo ================================================
echo 🎉 Release Build Complete
echo ================================================

echo ✅ Vectorizer %VERSION% release packages created successfully!
echo.
echo 📦 Packages created:
echo   🪟 Windows: %PROJECT_NAME%-%VERSION%-windows-x86_64.zip
echo.
echo 📁 Release directory: %RELEASE_DIR%
echo.
echo 📋 Next steps:
echo   1. Test the packages on clean systems
echo   2. Upload to GitHub Releases
echo   3. Update documentation
echo   4. Announce the release
echo.
echo 🔗 GitHub Release URL:
echo   https://github.com/hivellm/vectorizer/releases/new

goto :end

:show_help
echo Usage: scripts\release.bat [version] [--skip-tests] [--skip-docs]
echo.
echo Options:
echo   version              Set release version (default: auto-detect from Cargo.toml)
echo   --skip-tests         Skip running tests before build
echo   --skip-docs          Skip generating documentation
echo   -h, --help          Show this help message
goto :end

:end
