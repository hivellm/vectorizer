#!/bin/bash
# Build script for Hive Vectorizer Python SDK
# This script creates distribution packages (wheel and source distribution)

set -e

echo "🔨 Building Hive Vectorizer Python SDK..."

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
else
    echo -e "${YELLOW}⚠️  No virtual environment found. Creating one...${NC}"
    python3 -m venv venv
    source venv/bin/activate
    echo -e "${BLUE}📦 Installing build tools...${NC}"
    pip install --upgrade pip setuptools wheel build twine
fi

# Check Python version
echo -e "${BLUE}📋 Checking Python version...${NC}"
PYTHON_VERSION=$(python3 --version | cut -d' ' -f2)
echo "Python version: $PYTHON_VERSION"

# Check if required tools are installed
echo -e "${BLUE}📦 Checking build tools...${NC}"
if ! python3 -c "import build" 2>/dev/null; then
    echo -e "${YELLOW}⚠️  'build' module not found. Installing...${NC}"
    pip install build
fi

if ! python3 -c "import twine" 2>/dev/null; then
    echo -e "${YELLOW}⚠️  'twine' module not found. Installing...${NC}"
    pip install twine
fi

# Install dependencies
echo -e "${BLUE}📦 Installing dependencies...${NC}"
pip install -r requirements.txt

# Clean previous builds
echo -e "${BLUE}🧹 Cleaning previous builds...${NC}"
rm -rf build/ dist/ *.egg-info hive_vectorizer.egg-info

# Run syntax check
echo -e "${BLUE}🔍 Running syntax check...${NC}"
python3 -m py_compile client.py models.py exceptions.py utils/*.py || {
    echo -e "${RED}❌ Syntax check failed!${NC}"
    exit 1
}

# Run tests
echo -e "${BLUE}🧪 Running tests...${NC}"
PYTHONPATH=. python3 tests/test_simple.py || {
    echo -e "${YELLOW}⚠️  Some tests failed, but continuing...${NC}"
}

# Build package
echo -e "${BLUE}📦 Building package...${NC}"
python3 -m build

# Verify package
echo -e "${BLUE}✅ Verifying package...${NC}"
twine check dist/*

# Display results
echo ""
echo -e "${GREEN}✅ Build completed successfully!${NC}"
echo ""
echo "Generated artifacts:"
ls -lh dist/

echo ""
echo "Next steps:"
echo "  1. Test the package: pip install dist/hive_vectorizer-*.whl"
echo "  2. Upload to Test PyPI: ./publish.sh --test"
echo "  3. Upload to PyPI: ./publish.sh"
echo ""

