#!/bin/bash
# Publish script for Hive Vectorizer Python SDK
# Usage: ./publish.sh [--test]

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Activate virtual environment if it exists
if [ -d "venv" ]; then
    echo -e "${YELLOW}🐍 Activating virtual environment...${NC}"
    source venv/bin/activate
elif [ -d ".venv" ]; then
    echo -e "${YELLOW}🐍 Activating virtual environment...${NC}"
    source .venv/bin/activate
fi

# Check for test flag
TEST_MODE=false
if [[ "$1" == "--test" ]]; then
    TEST_MODE=true
    REPO_URL="https://test.pypi.org/legacy/"
    REPO_NAME="testpypi"
    echo -e "${YELLOW}📤 Publishing to Test PyPI...${NC}"
else
    REPO_URL="https://upload.pypi.org/legacy/"
    REPO_NAME="pypi"
    echo -e "${BLUE}📤 Publishing to PyPI...${NC}"
fi

# Check if dist directory exists
if [ ! -d "dist" ] || [ -z "$(ls -A dist)" ]; then
    echo -e "${RED}❌ No distribution files found. Run ./build.sh first.${NC}"
    exit 1
fi

# Check if twine is installed
if ! python3 -c "import twine" 2>/dev/null; then
    echo -e "${RED}❌ 'twine' not found. Installing...${NC}"
    pip install twine
fi

# Verify package before upload
echo -e "${BLUE}✅ Verifying package...${NC}"
twine check dist/*

# Display what will be uploaded
echo ""
echo "Files to upload:"
ls -lh dist/
echo ""

# Get version from __init__.py
VERSION=$(grep "__version__" __init__.py | cut -d'"' -f2)
echo "Package version: $VERSION"
echo ""

# Confirm upload (unless in CI)
if [ -z "$CI" ]; then
    if [ "$TEST_MODE" = false ]; then
        echo -e "${YELLOW}⚠️  You are about to upload to PRODUCTION PyPI!${NC}"
        read -p "Are you sure you want to continue? (yes/no): " confirm
        if [ "$confirm" != "yes" ]; then
            echo "Upload cancelled."
            exit 0
        fi
    fi
fi

# Upload to PyPI
echo -e "${BLUE}📤 Uploading to $REPO_NAME...${NC}"
if [ "$TEST_MODE" = true ]; then
    twine upload --repository testpypi dist/*
else
    twine upload dist/*
fi

# Success message
echo ""
echo -e "${GREEN}✅ Package uploaded successfully!${NC}"
echo ""
if [ "$TEST_MODE" = true ]; then
    echo "View on Test PyPI: https://test.pypi.org/project/hive-vectorizer/"
    echo ""
    echo "Install from Test PyPI:"
    echo "  pip install --index-url https://test.pypi.org/simple/ hive-vectorizer"
else
    echo "View on PyPI: https://pypi.org/project/hive-vectorizer/"
    echo ""
    echo "Install from PyPI:"
    echo "  pip install hive-vectorizer"
fi
echo ""

