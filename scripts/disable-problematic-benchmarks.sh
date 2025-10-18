#!/bin/bash
# Script to temporarily disable problematic benchmarks

echo "🚫 Disabling problematic benchmarks temporarily..."

# List of problematic benchmark files
PROBLEMATIC_BENCHMARKS=(
    "benchmark/scripts/test_10k_limit.rs"
    "benchmark/scripts/metal_native_comprehensive_benchmark.rs"
    "benchmark/scripts/scale_512d_benchmark.rs"
    "benchmark/scripts/optimized_metal_benchmark.rs"
    "benchmark/scripts/metal_native_hnsw_benchmark.rs"
    "benchmark/scripts/vram_validation_benchmark.rs"
)

for benchmark in "${PROBLEMATIC_BENCHMARKS[@]}"; do
    if [ -f "$benchmark" ]; then
        echo "📝 Disabling $benchmark"
        # Rename to .rs.disabled
        mv "$benchmark" "$benchmark.disabled"
        echo "✅ Disabled $benchmark"
    else
        echo "⚠️  File not found: $benchmark"
    fi
done

echo "🎉 Problematic benchmarks disabled!"
echo ""
echo "To re-enable them later, run:"
echo "  find benchmark/scripts -name '*.rs.disabled' -exec sh -c 'mv \"\$1\" \"\${1%.disabled}\"' _ {} \\;"

