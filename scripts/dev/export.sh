#!/bin/bash

# Vectorizer Export Script
# Cria pacote de distribuição com arquivos necessários para produção
# Usage: ./scripts/export.sh [version] [--minimal|--full]

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_ROOT/target/release"
EXPORT_DIR="$PROJECT_ROOT/dist"

# Default values
VERSION=""
PACKAGE_TYPE="full"  # full or minimal

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --version=*)
            VERSION="${1#*=}"
            shift
            ;;
        --minimal)
            PACKAGE_TYPE="minimal"
            shift
            ;;
        --full)
            PACKAGE_TYPE="full"
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [--version=VERSION] [--minimal|--full]"
            echo ""
            echo "Options:"
            echo "  --version=VERSION    Set export version (default: auto-detect from Cargo.toml)"
            echo "  --minimal            Create minimal package (binary + config only)"
            echo "  --full               Create full package (binary + configs + scripts + docs)"
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

# Detect OS
case "$(uname -s)" in
    Linux*)     OS="linux";;
    Darwin*)    OS="macos";;
    CYGWIN*|MINGW*|MSYS*) OS="windows";;
    *)          OS="unknown";;
esac

PACKAGE_NAME="vectorizer-${VERSION}-${OS}-x86_64"
PACKAGE_DIR="$EXPORT_DIR/$PACKAGE_NAME"

echo -e "${GREEN}🚀 Vectorizer Export Utility${NC}"
echo -e "${BLUE}Version: $VERSION${NC}"
echo -e "${BLUE}OS: $OS${NC}"
echo -e "${BLUE}Package Type: $PACKAGE_TYPE${NC}"
echo -e "${BLUE}Export Directory: $EXPORT_DIR${NC}"
echo ""

# Check if release binary exists
if [[ ! -f "$BUILD_DIR/vectorizer" ]]; then
    echo -e "${RED}❌ Release binary not found!${NC}"
    echo -e "${YELLOW}Run 'cargo build --release' first${NC}"
    exit 1
fi

# Create export directory
echo -e "${BLUE}📁 Creating export directory...${NC}"
rm -rf "$PACKAGE_DIR"
mkdir -p "$PACKAGE_DIR"

# Copy main binary
echo -e "${BLUE}📦 Copying binaries...${NC}"
cp "$BUILD_DIR/vectorizer" "$PACKAGE_DIR/"
chmod +x "$PACKAGE_DIR/vectorizer"
echo -e "${GREEN}  ✅ vectorizer${NC}"

if [[ "$PACKAGE_TYPE" == "full" ]]; then
    # Copy CLI tools
    if [[ -f "$BUILD_DIR/vectorizer-cli" ]]; then
        cp "$BUILD_DIR/vectorizer-cli" "$PACKAGE_DIR/"
        chmod +x "$PACKAGE_DIR/vectorizer-cli"
        echo -e "${GREEN}  ✅ vectorizer-cli${NC}"
    fi
    
    if [[ -f "$BUILD_DIR/vzr" ]]; then
        cp "$BUILD_DIR/vzr" "$PACKAGE_DIR/"
        chmod +x "$PACKAGE_DIR/vzr"
        echo -e "${GREEN}  ✅ vzr${NC}"
    fi
fi

# Copy configuration files
echo -e "${BLUE}📄 Copying configuration files...${NC}"
mkdir -p "$PACKAGE_DIR/config"

if [[ -f "$PROJECT_ROOT/config.exemple.yml" ]]; then
    cp "$PROJECT_ROOT/config.exemple.yml" "$PACKAGE_DIR/config/config.yml"
    echo -e "${GREEN}  ✅ config.yml${NC}"
fi

if [[ "$PACKAGE_TYPE" == "full" ]]; then
    if [[ -f "$PROJECT_ROOT/config/workspace.example.yml" ]]; then
        cp "$PROJECT_ROOT/config/workspace.example.yml" "$PACKAGE_DIR/config/workspace.yml"
        echo -e "${GREEN}  ✅ workspace.yml${NC}"
    fi
fi

# Copy scripts
if [[ "$PACKAGE_TYPE" == "full" ]]; then
    echo -e "${BLUE}📜 Copying management scripts...${NC}"
    mkdir -p "$PACKAGE_DIR/scripts"
    
    for script in start.sh stop.sh status.sh; do
        if [[ -f "$PROJECT_ROOT/scripts/$script" ]]; then
            cp "$PROJECT_ROOT/scripts/$script" "$PACKAGE_DIR/scripts/"
            chmod +x "$PACKAGE_DIR/scripts/$script"
            echo -e "${GREEN}  ✅ $script${NC}"
        fi
    done
fi

# Copy dashboard
echo -e "${BLUE}🖥️  Copying dashboard...${NC}"
if [[ -d "$PROJECT_ROOT/dashboard" ]]; then
    mkdir -p "$PACKAGE_DIR/dashboard"
    cp -r "$PROJECT_ROOT/dashboard/"* "$PACKAGE_DIR/dashboard/"
    echo -e "${GREEN}  ✅ dashboard/ (web interface)${NC}"
fi

# Copy documentation
echo -e "${BLUE}📚 Copying documentation...${NC}"
if [[ -f "$PROJECT_ROOT/README.md" ]]; then
    cp "$PROJECT_ROOT/README.md" "$PACKAGE_DIR/"
    echo -e "${GREEN}  ✅ README.md${NC}"
fi

if [[ "$PACKAGE_TYPE" == "full" ]]; then
    if [[ -f "$PROJECT_ROOT/CHANGELOG.md" ]]; then
        cp "$PROJECT_ROOT/CHANGELOG.md" "$PACKAGE_DIR/"
        echo -e "${GREEN}  ✅ CHANGELOG.md${NC}"
    fi
    
    if [[ -f "$PROJECT_ROOT/LICENSE" ]]; then
        cp "$PROJECT_ROOT/LICENSE" "$PACKAGE_DIR/"
        echo -e "${GREEN}  ✅ LICENSE${NC}"
    fi
    
    if [[ -f "$PROJECT_ROOT/GUIA_EXPORTACAO_DEPLOY.md" ]]; then
        cp "$PROJECT_ROOT/GUIA_EXPORTACAO_DEPLOY.md" "$PACKAGE_DIR/"
        echo -e "${GREEN}  ✅ GUIA_EXPORTACAO_DEPLOY.md${NC}"
    fi
fi

# Create quick start script
echo -e "${BLUE}🚀 Creating quick start script...${NC}"
cat > "$PACKAGE_DIR/start-vectorizer.sh" << 'EOF'
#!/bin/bash

# Vectorizer Quick Start Script

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "🚀 Starting Vectorizer..."

# Check if config exists
if [[ ! -f "config/config.yml" ]]; then
    echo "❌ Configuration file not found!"
    echo "Please copy and configure config/config.yml"
    exit 1
fi

# Start server
./vectorizer --config config/config.yml

EOF
chmod +x "$PACKAGE_DIR/start-vectorizer.sh"
echo -e "${GREEN}  ✅ start-vectorizer.sh${NC}"

# Create installation guide
echo -e "${BLUE}📝 Creating installation guide...${NC}"
cat > "$PACKAGE_DIR/INSTALL.md" << EOF
# Vectorizer Installation Guide

## Quick Start

### 1. Configure
Edit the configuration file:
\`\`\`bash
nano config/config.yml
\`\`\`

### 2. Start Server
\`\`\`bash
./start-vectorizer.sh
\`\`\`

### 3. Verify
\`\`\`bash
curl http://localhost:15002/health
\`\`\`

## Services

- **REST API**: http://localhost:15002
- **MCP Server**: http://localhost:15002/mcp/sse
- **Dashboard**: http://localhost:15002/dashboard
- **Health Check**: http://localhost:15002/health

## Configuration

Edit \`config/config.yml\` to change:
- Server host and port
- Logging level
- Collection defaults
- Performance settings

## Advanced Options

### Option 1: Manual Start
\`\`\`bash
./vectorizer --config config/config.yml
\`\`\`

### Option 2: With Workspace
\`\`\`bash
./vectorizer --config config/config.yml --workspace config/workspace.yml
\`\`\`

### Option 3: Systemd Service (Linux)
\`\`\`bash
# See GUIA_EXPORTACAO_DEPLOY.md for systemd setup
\`\`\`

## Troubleshooting

### Port Already in Use
\`\`\`bash
# Check what's using port 15002
lsof -i :15002

# Change port in config/config.yml
\`\`\`

### Permission Denied
\`\`\`bash
chmod +x vectorizer
chmod +x start-vectorizer.sh
\`\`\`

## Documentation

- Full documentation: README.md
- Deployment guide: GUIA_EXPORTACAO_DEPLOY.md
- Changelog: CHANGELOG.md

## Support

For issues and questions:
- GitHub: https://github.com/hivellm/vectorizer
- Documentation: https://vectorizer.dev
EOF
echo -e "${GREEN}  ✅ INSTALL.md${NC}"

# Calculate package size
PACKAGE_SIZE=$(du -sh "$PACKAGE_DIR" | cut -f1)

# Create tarball
echo -e "${BLUE}📦 Creating tarball...${NC}"
cd "$EXPORT_DIR"

if [[ "$OS" == "windows" ]]; then
    # Create ZIP for Windows
    if command -v zip &> /dev/null; then
        zip -r "${PACKAGE_NAME}.zip" "$PACKAGE_NAME" > /dev/null
        ARCHIVE_SIZE=$(du -sh "${PACKAGE_NAME}.zip" | cut -f1)
        echo -e "${GREEN}✅ Created: ${PACKAGE_NAME}.zip (${ARCHIVE_SIZE})${NC}"
    else
        echo -e "${YELLOW}⚠️  zip not found, skipping archive creation${NC}"
    fi
else
    # Create tar.gz for Linux/macOS
    tar -czf "${PACKAGE_NAME}.tar.gz" "$PACKAGE_NAME"
    ARCHIVE_SIZE=$(du -sh "${PACKAGE_NAME}.tar.gz" | cut -f1)
    echo -e "${GREEN}✅ Created: ${PACKAGE_NAME}.tar.gz (${ARCHIVE_SIZE})${NC}"
fi

cd "$PROJECT_ROOT"

# Generate checksum
echo -e "${BLUE}🔐 Generating checksums...${NC}"
cd "$EXPORT_DIR"

if [[ "$OS" == "windows" ]]; then
    if [[ -f "${PACKAGE_NAME}.zip" ]]; then
        sha256sum "${PACKAGE_NAME}.zip" > "${PACKAGE_NAME}.zip.sha256"
        echo -e "${GREEN}  ✅ SHA256 checksum created${NC}"
    fi
else
    if [[ -f "${PACKAGE_NAME}.tar.gz" ]]; then
        sha256sum "${PACKAGE_NAME}.tar.gz" > "${PACKAGE_NAME}.tar.gz.sha256"
        echo -e "${GREEN}  ✅ SHA256 checksum created${NC}"
    fi
fi

cd "$PROJECT_ROOT"

# Summary
echo ""
echo -e "${GREEN}═══════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Export Complete!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════${NC}"
echo ""
echo -e "${BLUE}📦 Package Details:${NC}"
echo -e "  Name: ${PACKAGE_NAME}"
echo -e "  Type: ${PACKAGE_TYPE}"
echo -e "  Size: ${PACKAGE_SIZE} (uncompressed)"
if [[ -n "$ARCHIVE_SIZE" ]]; then
    echo -e "  Archive: ${ARCHIVE_SIZE} (compressed)"
fi
echo ""
echo -e "${BLUE}📁 Location:${NC}"
echo -e "  Directory: $PACKAGE_DIR"
if [[ "$OS" == "windows" ]]; then
    echo -e "  Archive: $EXPORT_DIR/${PACKAGE_NAME}.zip"
else
    echo -e "  Archive: $EXPORT_DIR/${PACKAGE_NAME}.tar.gz"
fi
echo ""
echo -e "${BLUE}📋 Package Contents:${NC}"
echo -e "  ✅ vectorizer (main binary)"

if [[ "$PACKAGE_TYPE" == "full" ]]; then
    echo -e "  ✅ vectorizer-cli (CLI tool)"
    echo -e "  ✅ vzr (CLI alias)"
fi

echo -e "  ✅ config/config.yml"

if [[ "$PACKAGE_TYPE" == "full" ]]; then
    echo -e "  ✅ config/workspace.yml"
    echo -e "  ✅ scripts/ (management scripts)"
fi

echo -e "  ✅ start-vectorizer.sh (quick start)"
echo -e "  ✅ INSTALL.md (installation guide)"
echo -e "  ✅ README.md (documentation)"

if [[ "$PACKAGE_TYPE" == "full" ]]; then
    echo -e "  ✅ CHANGELOG.md"
    echo -e "  ✅ LICENSE"
    echo -e "  ✅ GUIA_EXPORTACAO_DEPLOY.md"
fi

echo ""
echo -e "${YELLOW}📤 Next Steps:${NC}"
echo "  1. Test the package on a clean system"
echo "  2. Extract and run: ./start-vectorizer.sh"
echo "  3. Verify health: curl http://localhost:15002/health"
echo "  4. Distribute via GitHub Releases or your preferred method"
echo ""
echo -e "${BLUE}🔗 GitHub Release Command:${NC}"
echo "  gh release create v${VERSION} \\"
if [[ "$OS" == "windows" ]]; then
    echo "    $EXPORT_DIR/${PACKAGE_NAME}.zip \\"
    echo "    $EXPORT_DIR/${PACKAGE_NAME}.zip.sha256"
else
    echo "    $EXPORT_DIR/${PACKAGE_NAME}.tar.gz \\"
    echo "    $EXPORT_DIR/${PACKAGE_NAME}.tar.gz.sha256"
fi
echo ""

