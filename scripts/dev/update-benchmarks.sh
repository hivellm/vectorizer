#!/bin/bash
# Script to update all benchmarks to use hive-gpu instead of local GPU module

set -e

echo "🔄 Updating benchmarks to use hive-gpu..."

# Find all benchmark files that import vectorizer::gpu
BENCHMARK_FILES=$(find benches/scripts -name "*.rs" -exec grep -l "vectorizer::gpu" {} \;)

for file in $BENCHMARK_FILES; do
    echo "📝 Updating $file"
    
    # Replace imports
    sed -i '' 's/use vectorizer::gpu::/use hive_gpu::/g' "$file"
    sed -i '' 's/use vectorizer::gpu::{/use hive_gpu::{/g' "$file"
    
    # Add missing imports
    if ! grep -q "use hive_gpu::" "$file"; then
        # Add after the first use statement
        sed -i '' '/^use /a\
use hive_gpu::{GpuVector, GpuDistanceMetric, GpuContext};
' "$file"
    fi
    
    # Replace MetalNativeCollection with MetalNativeVectorStorage
    sed -i '' 's/MetalNativeCollection/MetalNativeVectorStorage/g' "$file"
    
    # Replace vectorizer::gpu:: with hive_gpu::
    sed -i '' 's/vectorizer::gpu::/hive_gpu::/g' "$file"
    
    echo "✅ Updated $file"
done

echo "🎉 All benchmarks updated!"
echo ""
echo "Note: You may need to manually update the implementation details"
echo "to match the new hive-gpu API."

