#!/bin/bash

# Create Release Script for Vectorizer
# This script helps create GitHub releases with proper versioning

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if git is clean
check_git_status() {
    print_status "Checking git status..."
    
    if [ -n "$(git status --porcelain)" ]; then
        print_error "Working directory is not clean. Please commit or stash your changes."
        git status --short
        exit 1
    fi
    
    print_success "Working directory is clean"
}

# Function to get current version
get_current_version() {
    # Try to get version from Cargo.toml
    if [ -f "Cargo.toml" ]; then
        grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/'
    else
        echo "0.0.0"
    fi
}

# Function to get next version
get_next_version() {
    local current_version=$1
    local version_type=$2
    
    # Parse version
    local major=$(echo $current_version | cut -d. -f1)
    local minor=$(echo $current_version | cut -d. -f2)
    local patch=$(echo $current_version | cut -d. -f3)
    
    case $version_type in
        "major")
            echo "$((major + 1)).0.0"
            ;;
        "minor")
            echo "$major.$((minor + 1)).0"
            ;;
        "patch")
            echo "$major.$minor.$((patch + 1))"
            ;;
        *)
            print_error "Invalid version type: $version_type"
            exit 1
            ;;
    esac
}

# Function to update version in files
update_version() {
    local new_version=$1
    
    print_status "Updating version to $new_version..."
    
    # Update Cargo.toml
    if [ -f "Cargo.toml" ]; then
        sed -i "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
        print_success "Updated Cargo.toml"
    fi
    
    # Update client SDKs
    for sdk in client-sdks/*/package.json; do
        if [ -f "$sdk" ]; then
            sed -i "s/\"version\": \".*\"/\"version\": \"$new_version\"/" "$sdk"
            print_success "Updated $sdk"
        fi
    done
    
    for sdk in client-sdks/*/setup.py; do
        if [ -f "$sdk" ]; then
            sed -i "s/version = \".*\"/version = \"$new_version\"/" "$sdk"
            print_success "Updated $sdk"
        fi
    done
    
    for sdk in client-sdks/*/Cargo.toml; do
        if [ -f "$sdk" ]; then
            sed -i "s/^version = \".*\"/version = \"$new_version\"/" "$sdk"
            print_success "Updated $sdk"
        fi
    done
}

# Function to create changelog entry
create_changelog_entry() {
    local new_version=$1
    local version_type=$2
    
    print_status "Creating changelog entry..."
    
    # Create temporary changelog entry
    local temp_file=$(mktemp)
    
    cat > "$temp_file" << EOF
## [$new_version] - $(date +%Y-%m-%d)

### 🚀 Release Notes

#### Changes
- Version bump to $new_version
- Automated release build
- Multi-platform binaries available

#### Platform Support
- ✅ Linux x86_64
- ✅ Linux aarch64  
- ✅ Windows x86_64
- ✅ Windows aarch64
- ✅ macOS x86_64
- ✅ macOS aarch64

#### Installation
\`\`\`bash
# Download for your platform from GitHub Releases
wget https://github.com/hivellm/vectorizer/releases/download/v$new_version/vectorizer-linux-x86_64.tar.gz
tar -xzf vectorizer-linux-x86_64.tar.gz
./vectorizer-server --config config.yml
\`\`\`

EOF
    
    # Insert at the beginning of CHANGELOG.md
    if [ -f "CHANGELOG.md" ]; then
        cp CHANGELOG.md CHANGELOG.md.bak
        cat "$temp_file" CHANGELOG.md.bak > CHANGELOG.md
        rm CHANGELOG.md.bak
        print_success "Updated CHANGELOG.md"
    fi
    
    rm "$temp_file"
}

# Function to commit and tag
commit_and_tag() {
    local new_version=$1
    
    print_status "Committing changes..."
    
    git add .
    git commit -m "chore: bump version to $new_version"
    
    print_status "Creating tag v$new_version..."
    git tag -a "v$new_version" -m "Release v$new_version"
    
    print_success "Created tag v$new_version"
}

# Function to push to remote
push_to_remote() {
    local new_version=$1
    
    print_status "Pushing to remote..."
    
    git push origin main
    git push origin "v$new_version"
    
    print_success "Pushed to remote"
}

# Function to display help
show_help() {
    echo "Vectorizer Release Creator"
    echo ""
    echo "Usage: $0 [OPTIONS] VERSION_TYPE"
    echo ""
    echo "Options:"
    echo "  -h, --help     Show this help message"
    echo "  -d, --dry-run  Show what would be done without making changes"
    echo "  -v, --version  Specify exact version instead of auto-increment"
    echo ""
    echo "Version Types:"
    echo "  patch         Increment patch version (0.0.1 -> 0.0.2)"
    echo "  minor         Increment minor version (0.1.0 -> 0.2.0)"
    echo "  major         Increment major version (1.0.0 -> 2.0.0)"
    echo ""
    echo "Examples:"
    echo "  $0 patch                    # Create patch release"
    echo "  $0 minor                    # Create minor release"
    echo "  $0 --version 1.5.0          # Create release with specific version"
    echo "  $0 --dry-run patch          # Preview patch release"
}

# Main function
main() {
    local version_type=""
    local dry_run=false
    local specific_version=""
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -d|--dry-run)
                dry_run=true
                shift
                ;;
            -v|--version)
                specific_version="$2"
                shift 2
                ;;
            patch|minor|major)
                version_type="$1"
                shift
                ;;
            *)
                print_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    # Validate arguments
    if [ -z "$version_type" ] && [ -z "$specific_version" ]; then
        print_error "Version type or specific version required"
        show_help
        exit 1
    fi
    
    if [ -n "$specific_version" ] && [ -n "$version_type" ]; then
        print_error "Cannot specify both version type and specific version"
        exit 1
    fi
    
    # Get current version
    local current_version=$(get_current_version)
    print_status "Current version: $current_version"
    
    # Determine new version
    local new_version
    if [ -n "$specific_version" ]; then
        new_version="$specific_version"
    else
        new_version=$(get_next_version "$current_version" "$version_type")
    fi
    
    print_status "New version: $new_version"
    
    if [ "$dry_run" = true ]; then
        print_warning "DRY RUN - No changes will be made"
        echo ""
        echo "Would perform the following actions:"
        echo "1. Check git status"
        echo "2. Update version to $new_version in:"
        echo "   - Cargo.toml"
        echo "   - client-sdks/*/package.json"
        echo "   - client-sdks/*/setup.py"
        echo "   - client-sdks/*/Cargo.toml"
        echo "3. Update CHANGELOG.md"
        echo "4. Commit changes"
        echo "5. Create tag v$new_version"
        echo "6. Push to remote"
        echo "7. Trigger GitHub Actions release workflow"
        exit 0
    fi
    
    # Confirm release
    echo ""
    print_warning "This will create a release for version $new_version"
    read -p "Continue? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_error "Release cancelled"
        exit 1
    fi
    
    # Execute release process
    echo ""
    print_status "Starting release process for v$new_version..."
    
    check_git_status
    update_version "$new_version"
    create_changelog_entry "$new_version" "$version_type"
    commit_and_tag "$new_version"
    push_to_remote "$new_version"
    
    echo ""
    print_success "Release v$new_version created successfully!"
    echo ""
    print_status "Next steps:"
    echo "1. GitHub Actions will automatically build and create the release"
    echo "2. Monitor the workflow at: https://github.com/hivellm/vectorizer/actions"
    echo "3. Download links will be available at: https://github.com/hivellm/vectorizer/releases/tag/v$new_version"
    echo ""
    print_status "The release includes:"
    echo "- Linux x86_64 and aarch64 binaries"
    echo "- Windows x86_64 and aarch64 binaries"
    echo "- macOS x86_64 and aarch64 binaries"
    echo "- Installation scripts for each platform"
    echo "- Configuration files and documentation"
}

# Run main function
main "$@"
