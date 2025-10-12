#!/bin/bash

# Vectorizer Release Script
# Builds and packages Vectorizer for Linux and Windows distributions
# Usage: ./scripts/release.sh [version] [--skip-tests] [--skip-docs]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_NAME="vectorizer"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RELEASE_DIR="$PROJECT_ROOT/releases"
BUILD_DIR="$PROJECT_ROOT/target/release"

# Default values
VERSION=""
SKIP_TESTS=false
SKIP_DOCS=false
BUILD_LINUX=true
BUILD_WINDOWS=true

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --version=*)
            VERSION="${1#*=}"
            shift
            ;;
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        --skip-docs)
            SKIP_DOCS=true
            shift
            ;;
        --linux-only)
            BUILD_WINDOWS=false
            shift
            ;;
        --windows-only)
            BUILD_LINUX=false
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [--version=VERSION] [--skip-tests] [--skip-docs] [--linux-only] [--windows-only]"
            echo ""
            echo "Options:"
            echo "  --version=VERSION    Set release version (default: auto-detect from Cargo.toml)"
            echo "  --skip-tests         Skip running tests before build"
            echo "  --skip-docs          Skip generating documentation"
            echo "  --linux-only         Build only Linux packages"
            echo "  --windows-only       Build only Windows packages"
            echo "  -h, --help          Show this help message"
            exit 0
            ;;
        *)
            if [[ -z "$VERSION" ]]; then
                VERSION="$1"
            fi
            shift
            ;;
    esac
done

# Auto-detect version if not provided
if [[ -z "$VERSION" ]]; then
    VERSION=$(grep '^version = ' "$PROJECT_ROOT/Cargo.toml" | sed 's/version = "\(.*\)"/\1/')
    echo -e "${BLUE}📦 Auto-detected version: $VERSION${NC}"
fi

# Validate version format
if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+ ]]; then
    echo -e "${RED}❌ Invalid version format: $VERSION${NC}"
    echo "Version must be in format: X.Y.Z (e.g., 1.0.0)"
    exit 1
fi

echo -e "${GREEN}🚀 Starting Vectorizer Release Build${NC}"
echo -e "${BLUE}Version: $VERSION${NC}"
echo -e "${BLUE}Project Root: $PROJECT_ROOT${NC}"
echo -e "${BLUE}Release Directory: $RELEASE_DIR${NC}"
echo ""

# Function to print section headers
print_section() {
    echo -e "\n${YELLOW}================================================${NC}"
    echo -e "${YELLOW}$1${NC}"
    echo -e "${YELLOW}================================================${NC}"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check prerequisites
check_prerequisites() {
    print_section "🔍 Checking Prerequisites"
    
    local missing_deps=()
    
    # Check Rust
    if ! command_exists cargo; then
        missing_deps+=("rust")
    else
        local rust_version=$(cargo --version | cut -d' ' -f2)
        echo -e "${GREEN}✅ Rust: $rust_version${NC}"
    fi
    
    # Check cross-compilation tools for Windows
    if [[ "$BUILD_WINDOWS" == true ]]; then
        if ! command_exists x86_64-w64-mingw32-gcc; then
            echo -e "${YELLOW}⚠️  Windows cross-compilation tools not found${NC}"
            echo -e "${YELLOW}   Install with: sudo apt-get install gcc-mingw-w64-x86-64${NC}"
            missing_deps+=("mingw-w64")
        else
            echo -e "${GREEN}✅ Windows cross-compilation tools${NC}"
        fi
    fi
    
    # Check additional tools
    if ! command_exists zip; then
        missing_deps+=("zip")
    else
        echo -e "${GREEN}✅ zip${NC}"
    fi
    
    if ! command_exists tar; then
        missing_deps+=("tar")
    else
        echo -e "${GREEN}✅ tar${NC}"
    fi
    
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        echo -e "${RED}❌ Missing dependencies: ${missing_deps[*]}${NC}"
        echo -e "${YELLOW}Please install missing dependencies and try again${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✅ All prerequisites satisfied${NC}"
}

# Function to run tests
run_tests() {
    if [[ "$SKIP_TESTS" == true ]]; then
        echo -e "${YELLOW}⏭️  Skipping tests${NC}"
        return
    fi
    
    print_section "🧪 Running Tests"
    
    cd "$PROJECT_ROOT"
    
    echo -e "${BLUE}Running cargo test with Metal Native support...${NC}"
    if cargo test --release --features metal-native; then
        echo -e "${GREEN}✅ All tests passed${NC}"
    else
        echo -e "${RED}❌ Tests failed${NC}"
        exit 1
    fi
}

# Function to generate documentation
generate_docs() {
    if [[ "$SKIP_DOCS" == true ]]; then
        echo -e "${YELLOW}⏭️  Skipping documentation generation${NC}"
        return
    fi
    
    print_section "📚 Generating Documentation"
    
    cd "$PROJECT_ROOT"
    
    echo -e "${BLUE}Generating Rust documentation...${NC}"
    if cargo doc --release --no-deps; then
        echo -e "${GREEN}✅ Documentation generated${NC}"
    else
        echo -e "${YELLOW}⚠️  Documentation generation failed, continuing...${NC}"
    fi
}

# Function to build Linux binaries
build_linux() {
    print_section "🐧 Building Linux Binaries"
    
    cd "$PROJECT_ROOT"
    
    echo -e "${BLUE}Building release binaries for Linux...${NC}"
    
    # Build main binaries
    local binaries=("vzr" "vectorizer-server" "vectorizer-mcp-server")
    
    for binary in "${binaries[@]}"; do
        echo -e "${BLUE}Building $binary...${NC}"
        if cargo build --release --bin "$binary" --features metal-native; then
            echo -e "${GREEN}✅ $binary built successfully${NC}"
        else
            echo -e "${RED}❌ Failed to build $binary${NC}"
            exit 1
        fi
    done
    
    echo -e "${GREEN}✅ All Linux binaries built successfully${NC}"
}

# Function to build Windows binaries
build_windows() {
    print_section "🪟 Building Windows Binaries"
    
    cd "$PROJECT_ROOT"
    
    echo -e "${BLUE}Building release binaries for Windows...${NC}"
    
    # Add Windows target if not already added
    if ! rustup target list --installed | grep -q "x86_64-pc-windows-gnu"; then
        echo -e "${BLUE}Adding Windows target...${NC}"
        rustup target add x86_64-pc-windows-gnu
    fi
    
    # Build main binaries for Windows
    local binaries=("vzr" "vectorizer-server" "vectorizer-mcp-server")
    
    for binary in "${binaries[@]}"; do
        echo -e "${BLUE}Building $binary for Windows...${NC}"
        if cargo build --release --bin "$binary" --target x86_64-pc-windows-gnu --features metal-native; then
            echo -e "${GREEN}✅ $binary (Windows) built successfully${NC}"
        else
            echo -e "${RED}❌ Failed to build $binary for Windows${NC}"
            exit 1
        fi
    done
    
    echo -e "${GREEN}✅ All Windows binaries built successfully${NC}"
}

# Function to create Linux package
create_linux_package() {
    print_section "📦 Creating Linux Package"
    
    local package_name="${PROJECT_NAME}-${VERSION}-linux-x86_64"
    local package_dir="$RELEASE_DIR/$package_name"
    
    # Create package directory
    mkdir -p "$package_dir"
    
    echo -e "${BLUE}Creating Linux package: $package_name${NC}"
    
    # Copy binaries
    cp "$BUILD_DIR/vzr" "$package_dir/"
    cp "$BUILD_DIR/vectorizer-server" "$package_dir/"
    cp "$BUILD_DIR/vectorizer-mcp-server" "$package_dir/"
    
    # Copy configuration files
    mkdir -p "$package_dir/config"
    cp "$PROJECT_ROOT/config.example.yml" "$package_dir/config/"
    cp "$PROJECT_ROOT/vectorize-workspace.example.yml" "$package_dir/config/"
    
    # Copy scripts
    mkdir -p "$package_dir/scripts"
    cp "$PROJECT_ROOT/scripts/start.sh" "$package_dir/scripts/"
    cp "$PROJECT_ROOT/scripts/stop.sh" "$package_dir/scripts/"
    cp "$PROJECT_ROOT/scripts/status.sh" "$package_dir/scripts/"
    cp "$PROJECT_ROOT/scripts/start-dev.sh" "$package_dir/scripts/"
    cp "$PROJECT_ROOT/scripts/stop-dev.sh" "$package_dir/scripts/"
    cp "$PROJECT_ROOT/scripts/build.sh" "$package_dir/scripts/"
    
    # Make scripts executable
    chmod +x "$package_dir"/*.sh
    chmod +x "$package_dir/scripts"/*.sh
    
    # Copy documentation
    mkdir -p "$package_dir/docs"
    cp "$PROJECT_ROOT/README.md" "$package_dir/"
    cp "$PROJECT_ROOT/CHANGELOG.md" "$package_dir/"
    cp "$PROJECT_ROOT/LICENSE" "$package_dir/" 2>/dev/null || true
    
    # Copy Python SDK
    if [[ -d "$PROJECT_ROOT/client-sdks/python" ]]; then
        cp -r "$PROJECT_ROOT/client-sdks/python" "$package_dir/client-sdks/"
    fi
    
    # Create installation script
    cat > "$package_dir/install.sh" << 'EOF'
#!/bin/bash

# Vectorizer Installation Script for Linux

set -e

INSTALL_DIR="/opt/vectorizer"
SYSTEMD_DIR="/etc/systemd/system"
BIN_DIR="/usr/local/bin"

echo "🚀 Installing Vectorizer..."

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo "❌ This script must be run as root (use sudo)"
   exit 1
fi

# Create installation directory
mkdir -p "$INSTALL_DIR"

# Copy files
cp -r ./* "$INSTALL_DIR/"

# Create symlinks for easy access
ln -sf "$INSTALL_DIR/vzr" "$BIN_DIR/vzr"
ln -sf "$INSTALL_DIR/vectorizer-server" "$BIN_DIR/vectorizer-server"
ln -sf "$INSTALL_DIR/vectorizer-mcp-server" "$BIN_DIR/vectorizer-mcp-server"

# Create systemd service
cat > "$SYSTEMD_DIR/vectorizer.service" << 'SERVICE_EOF'
[Unit]
Description=Vectorizer Vector Database Server
After=network.target

[Service]
Type=simple
User=vectorizer
Group=vectorizer
WorkingDirectory=/opt/vectorizer
ExecStart=/opt/vectorizer/vzr start --workspace config/vectorize-workspace.yml
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
SERVICE_EOF

# Create vectorizer user
if ! id "vectorizer" &>/dev/null; then
    useradd -r -s /bin/false -d "$INSTALL_DIR" vectorizer
fi

# Set permissions
chown -R vectorizer:vectorizer "$INSTALL_DIR"
chmod +x "$INSTALL_DIR"/*.sh
chmod +x "$INSTALL_DIR/scripts"/*.sh

echo "✅ Vectorizer installed successfully!"
echo ""
echo "To start the service:"
echo "  sudo systemctl enable vectorizer"
echo "  sudo systemctl start vectorizer"
echo ""
echo "To check status:"
echo "  sudo systemctl status vectorizer"
echo ""
echo "Manual start:"
echo "  vzr start --workspace config/vectorize-workspace.yml"
EOF
    
    chmod +x "$package_dir/install.sh"
    
    # Create README for package
    cat > "$package_dir/PACKAGE_README.md" << EOF
# Vectorizer $VERSION - Linux Package

## Quick Start

1. **Installation:**
   \`\`\`bash
   sudo ./install.sh
   \`\`\`

2. **Start Service:**
   \`\`\`bash
   sudo systemctl enable vectorizer
   sudo systemctl start vectorizer
   \`\`\`

3. **Manual Start:**
   \`\`\`bash
   ./scripts/start.sh --workspace config/vectorize-workspace.yml
   \`\`\`

## Services

- **REST API**: http://localhost:15001
- **MCP Server**: ws://localhost:15002/mcp

## Configuration

- Edit \`config/config.yml\` for server settings
- Edit \`config/vectorize-workspace.yml\` for workspace configuration

## Python SDK

The Python SDK is included in \`client-sdks/python/\`. See the README in that directory for usage instructions.

## Support

For issues and questions, please visit: https://github.com/hivellm/vectorizer
EOF
    
    # Create tarball
    cd "$RELEASE_DIR"
    tar -czf "${package_name}.tar.gz" "$package_name"
    
    echo -e "${GREEN}✅ Linux package created: ${package_name}.tar.gz${NC}"
}

# Function to create Windows package
create_windows_package() {
    print_section "📦 Creating Windows Package"
    
    local package_name="${PROJECT_NAME}-${VERSION}-windows-x86_64"
    local package_dir="$RELEASE_DIR/$package_name"
    
    # Create package directory
    mkdir -p "$package_dir"
    
    echo -e "${BLUE}Creating Windows package: $package_name${NC}"
    
    # Copy Windows binaries
    cp "$PROJECT_ROOT/target/x86_64-pc-windows-gnu/release/vzr.exe" "$package_dir/"
    cp "$PROJECT_ROOT/target/x86_64-pc-windows-gnu/release/vectorizer-server.exe" "$package_dir/"
    cp "$PROJECT_ROOT/target/x86_64-pc-windows-gnu/release/vectorizer-mcp-server.exe" "$package_dir/"
    
    # Copy configuration files
    mkdir -p "$package_dir/config"
    cp "$PROJECT_ROOT/config.example.yml" "$package_dir/config/"
    cp "$PROJECT_ROOT/vectorize-workspace.example.yml" "$package_dir/config/"
    
    # Copy scripts
    mkdir -p "$package_dir/scripts"
    cp "$PROJECT_ROOT/scripts/start.bat" "$package_dir/scripts/"
    cp "$PROJECT_ROOT/scripts/stop.bat" "$package_dir/scripts/"
    cp "$PROJECT_ROOT/scripts/status.bat" "$package_dir/scripts/"
    cp "$PROJECT_ROOT/scripts/start-dev.bat" "$package_dir/scripts/"
    cp "$PROJECT_ROOT/scripts/stop-dev.bat" "$package_dir/scripts/"
    cp "$PROJECT_ROOT/scripts/build.bat" "$package_dir/scripts/"
    
    # Copy documentation
    cp "$PROJECT_ROOT/README.md" "$package_dir/"
    cp "$PROJECT_ROOT/CHANGELOG.md" "$package_dir/"
    cp "$PROJECT_ROOT/LICENSE" "$package_dir/" 2>/dev/null || true
    
    # Copy Python SDK
    if [[ -d "$PROJECT_ROOT/client-sdks/python" ]]; then
        cp -r "$PROJECT_ROOT/client-sdks/python" "$package_dir/client-sdks/"
    fi
    
    # Create Windows installation script
    cat > "$package_dir/install.bat" << 'EOF'
@echo off
REM Vectorizer Installation Script for Windows

echo 🚀 Installing Vectorizer...

REM Create installation directory
if not exist "C:\Program Files\Vectorizer" mkdir "C:\Program Files\Vectorizer"

REM Copy files
xcopy /E /I /Y . "C:\Program Files\Vectorizer"

REM Add to PATH (requires admin)
echo Adding Vectorizer to PATH...
setx PATH "%PATH%;C:\Program Files\Vectorizer" /M

REM Create Windows service (optional)
echo.
echo ✅ Vectorizer installed successfully!
echo.
echo To start manually:
echo   cd "C:\Program Files\Vectorizer"
echo   scripts\start.bat --workspace config\vectorize-workspace.yml
echo.
echo Services:
echo   REST API: http://localhost:15001
echo   MCP Server: ws://localhost:15002/mcp
echo.
pause
EOF
    
    # Create README for package
    cat > "$package_dir/PACKAGE_README.md" << EOF
# Vectorizer $VERSION - Windows Package

## Quick Start

1. **Installation:**
   - Run \`install.bat\` as Administrator
   - Or manually copy files to desired location

2. **Start Service:**
   \`\`\`cmd
   scripts\\start.bat --workspace config\\vectorize-workspace.yml
   \`\`\`

3. **Development Mode:**
   \`\`\`cmd
   scripts\\start-dev.bat --workspace config\\vectorize-workspace.yml
   \`\`\`

## Services

- **REST API**: http://localhost:15001
- **MCP Server**: ws://localhost:15002/mcp

## Configuration

- Edit \`config\\config.yml\` for server settings
- Edit \`config\\vectorize-workspace.yml\` for workspace configuration

## Python SDK

The Python SDK is included in \`client-sdks\\python\\\`. See the README in that directory for usage instructions.

## Support

For issues and questions, please visit: https://github.com/hivellm/vectorizer
EOF
    
    # Create ZIP archive
    cd "$RELEASE_DIR"
    zip -r "${package_name}.zip" "$package_name"
    
    echo -e "${GREEN}✅ Windows package created: ${package_name}.zip${NC}"
}

# Function to create release notes
create_release_notes() {
    print_section "📝 Creating Release Notes"
    
    local release_notes_file="$RELEASE_DIR/RELEASE_NOTES_${VERSION}.md"
    
    cat > "$release_notes_file" << EOF
# Vectorizer Release $VERSION

## 📦 Packages

### Linux (x86_64)
- **File**: \`${PROJECT_NAME}-${VERSION}-linux-x86_64.tar.gz\`
- **Installation**: Extract and run \`sudo ./install.sh\`
- **Manual Start**: \`./scripts/start.sh --workspace config/vectorize-workspace.yml\`

### Windows (x86_64)
- **File**: \`${PROJECT_NAME}-${VERSION}-windows-x86_64.zip\`
- **Installation**: Extract and run \`install.bat\` as Administrator
- **Manual Start**: \`scripts\\start.bat --workspace config\\vectorize-workspace.yml\`

## 🚀 Quick Start

1. **Download** the appropriate package for your platform
2. **Extract** the archive
3. **Install** using the provided installation script
4. **Start** the service

## 🔧 Services

- **REST API**: http://localhost:15001
- **MCP Server**: ws://localhost:15002/mcp  

## 📚 Documentation

- **README**: Complete documentation included in package
- **Configuration**: Edit \`config/config.yml\` and \`config/vectorize-workspace.yml\`
- **Python SDK**: Included in \`client-sdks/python/\`

## 🐛 Support

- **Issues**: https://github.com/hivellm/vectorizer/issues
- **Documentation**: https://github.com/hivellm/vectorizer/blob/main/README.md

## 📋 System Requirements

### Linux
- x86_64 architecture
- glibc 2.17+ or musl libc
- 2GB+ RAM recommended
- 1GB+ disk space

### Windows
- Windows 10+ (x64)
- Visual C++ Redistributable
- 2GB+ RAM recommended
- 1GB+ disk space

## 🔄 Upgrade Notes

If upgrading from a previous version:
1. Stop the current service
2. Backup your configuration files
3. Install the new version
4. Restore your configuration files
5. Start the service

EOF
    
    echo -e "${GREEN}✅ Release notes created: $release_notes_file${NC}"
}

# Function to show final summary
show_summary() {
    print_section "🎉 Release Build Complete"
    
    echo -e "${GREEN}✅ Vectorizer $VERSION release packages created successfully!${NC}"
    echo ""
    echo -e "${BLUE}📦 Packages created:${NC}"
    
    if [[ "$BUILD_LINUX" == true ]]; then
        echo -e "  🐧 Linux: ${PROJECT_NAME}-${VERSION}-linux-x86_64.tar.gz"
    fi
    
    if [[ "$BUILD_WINDOWS" == true ]]; then
        echo -e "  🪟 Windows: ${PROJECT_NAME}-${VERSION}-windows-x86_64.zip"
    fi
    
    echo ""
    echo -e "${BLUE}📁 Release directory: $RELEASE_DIR${NC}"
    echo ""
    echo -e "${YELLOW}📋 Next steps:${NC}"
    echo "  1. Test the packages on clean systems"
    echo "  2. Upload to GitHub Releases"
    echo "  3. Update documentation"
    echo "  4. Announce the release"
    echo ""
    echo -e "${BLUE}🔗 GitHub Release URL:${NC}"
    echo "  https://github.com/hivellm/vectorizer/releases/new"
}

# Main execution
main() {
    # Create release directory
    mkdir -p "$RELEASE_DIR"
    
    # Run build process
    check_prerequisites
    run_tests
    generate_docs
    
    if [[ "$BUILD_LINUX" == true ]]; then
        build_linux
        create_linux_package
    fi
    
    if [[ "$BUILD_WINDOWS" == true ]]; then
        build_windows
        create_windows_package
    fi
    
    create_release_notes
    show_summary
}

# Run main function
main "$@"
