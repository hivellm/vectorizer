#!/bin/bash
# Script to migrate tasks from openspec to rulebook

set -e

cd "$(dirname "$0")/.."

echo "Migrating tasks from openspec/changes to rulebook/tasks..."

# List of tasks to migrate
tasks=(
    "add-production-documentation"
    "add-qdrant-advanced-features"
    "add-qdrant-clients"
    "add-qdrant-clustering"
    "add-qdrant-collections"
    "add-qdrant-compatibility"
    "add-qdrant-grpc"
    "add-qdrant-migration"
    "add-qdrant-testing"
    "add-query-caching"
    "nexus-integration"
)

for task in "${tasks[@]}"; do
    src_dir="openspec/changes/$task"
    dst_dir="rulebook/tasks/$task"
    
    if [ -d "$src_dir" ]; then
        echo "Processing $task..."
        
        # Create destination directory if it doesn't exist
        mkdir -p "$dst_dir"
        
        # Copy all files (will overwrite if they exist)
        cp -r "$src_dir"/* "$dst_dir/" 2>/dev/null || true
        
        echo "  ✓ Copied files for $task"
    else
        echo "  ⚠ Source directory $src_dir not found"
    fi
done

echo ""
echo "Migration complete!"
echo ""
echo "Files in openspec/changes: $(find openspec/changes -type f -name '*.md' 2>/dev/null | wc -l)"
echo "Files in rulebook/tasks: $(find rulebook/tasks -type f -name '*.md' 2>/dev/null | wc -l)"

