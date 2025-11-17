#!/bin/bash
# Publish script for Vectorizer C# SDK to NuGet
# Usage: ./publish.sh [API_KEY]

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

echo -e "${BLUE}📤 Publishing Vectorizer C# SDK to NuGet...${NC}"

# Check if dotnet is installed
if ! command -v dotnet &> /dev/null; then
    echo -e "${RED}❌ .NET SDK not found.${NC}"
    exit 1
fi

# Check if artifacts directory exists
if [ ! -d "artifacts" ] || [ -z "$(ls -A artifacts/*.nupkg 2>/dev/null)" ]; then
    echo -e "${RED}❌ No NuGet packages found in artifacts/. Run ./build.sh first.${NC}"
    exit 1
fi

# Get package files
PACKAGE_FILE=$(ls artifacts/*.nupkg 2>/dev/null | grep -v ".symbols.nupkg" | head -n 1)
SYMBOLS_FILE=$(ls artifacts/*.snupkg 2>/dev/null | head -n 1)

if [ -z "$PACKAGE_FILE" ]; then
    echo -e "${RED}❌ No .nupkg file found.${NC}"
    exit 1
fi

echo ""
echo "Files to upload:"
ls -lh "$PACKAGE_FILE"
[ -n "$SYMBOLS_FILE" ] && ls -lh "$SYMBOLS_FILE"
echo ""

# Get API key
API_KEY="$1"
if [ -z "$API_KEY" ]; then
    echo -e "${YELLOW}⚠️  No API key provided as argument.${NC}"
    echo -e "${YELLOW}Looking for NUGET_API_KEY environment variable...${NC}"
    API_KEY="$NUGET_API_KEY"
fi

if [ -z "$API_KEY" ]; then
    echo -e "${RED}❌ No API key found!${NC}"
    echo ""
    echo "Usage:"
    echo "  ./publish.sh YOUR_API_KEY"
    echo "  or set NUGET_API_KEY environment variable"
    echo ""
    echo "Get your API key from: https://www.nuget.org/account/apikeys"
    exit 1
fi

# Confirm upload (unless in CI)
if [ -z "$CI" ]; then
    echo -e "${YELLOW}⚠️  You are about to upload to PRODUCTION NuGet.org!${NC}"
    read -p "Are you sure you want to continue? (yes/no): " confirm
    if [ "$confirm" != "yes" ]; then
        echo "Upload cancelled."
        exit 0
    fi
fi

# Push to NuGet
echo -e "${BLUE}📤 Uploading to NuGet.org...${NC}"
dotnet nuget push "$PACKAGE_FILE" \
    --api-key "$API_KEY" \
    --source https://api.nuget.org/v3/index.json \
    --skip-duplicate

# Push symbols if exists
if [ -n "$SYMBOLS_FILE" ]; then
    echo -e "${BLUE}📤 Uploading symbols package...${NC}"
    dotnet nuget push "$SYMBOLS_FILE" \
        --api-key "$API_KEY" \
        --source https://api.nuget.org/v3/index.json \
        --skip-duplicate
fi

# Success message
VERSION=$(basename "$PACKAGE_FILE" | sed 's/Vectorizer.Sdk.\(.*\).nupkg/\1/')

echo ""
echo -e "${GREEN}✅ Package uploaded successfully!${NC}"
echo ""
echo "View on NuGet: https://www.nuget.org/packages/Vectorizer.Sdk/$VERSION"
echo ""
echo "Install with:"
echo "  dotnet add package Vectorizer.Sdk --version $VERSION"
echo ""
echo "Note: It may take a few minutes for the package to be indexed and available."
echo ""

