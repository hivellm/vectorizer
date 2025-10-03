#!/bin/bash

# Script to test Bend integration with Vectorizer
# This script tests the complete Bend integration

echo "🧪 Testing Bend Integration with Vectorizer"
echo "=========================================="

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Please run this script from the Vectorizer root directory"
    exit 1
fi

# Check if Bend is installed
if ! command -v bend &> /dev/null; then
    echo "❌ Bend not found in PATH"
    echo "Please install Bend first:"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/HigherOrderCO/Bend/main/install.sh | bash"
    exit 1
fi

# Check if HVM is installed
if ! command -v hvm &> /dev/null; then
    echo "❌ HVM not found in PATH"
    echo "Please install HVM first:"
    echo "cargo install hvm"
    exit 1
fi

echo "✅ Bend and HVM are available"

# Test basic Bend programs
echo ""
echo "🔧 Testing basic Bend programs..."

# Test simple factorial
echo "Testing simple_test.bend..."
RESULT_SIMPLE=$(bend --hvm-bin $(which hvm) run-rs examples/bend/simple_test.bend)
if echo "$RESULT_SIMPLE" | grep -q "3628800"; then
    echo "✅ simple_test.bend: $RESULT_SIMPLE"
else
    echo "❌ simple_test.bend failed: $RESULT_SIMPLE"
    exit 1
fi

# Test vector similarity
echo "Testing fixed_size_similarity.bend..."
RESULT_SIMILARITY=$(bend --hvm-bin $(which hvm) run-rs examples/bend/fixed_size_similarity.bend)
if echo "$RESULT_SIMILARITY" | grep -q "999"; then
    echo "✅ fixed_size_similarity.bend: $RESULT_SIMILARITY"
else
    echo "❌ fixed_size_similarity.bend failed: $RESULT_SIMILARITY"
    exit 1
fi

# Test Rust compilation
echo ""
echo "🔨 Testing Rust compilation with Bend integration..."

if cargo check --quiet; then
    echo "✅ Rust code compiles successfully"
else
    echo "❌ Rust compilation failed"
    echo "Running cargo check to see errors:"
    cargo check
    exit 1
fi

# Test Bend module compilation
echo ""
echo "🧪 Testing Bend module compilation..."

if cargo check --lib --quiet; then
    echo "✅ Bend module compiles successfully"
else
    echo "❌ Bend module compilation failed"
    echo "Running cargo check --lib to see errors:"
    cargo check --lib
    exit 1
fi

# Test integration tests
echo ""
echo "🧪 Running integration tests..."

if cargo test bend --quiet; then
    echo "✅ Bend integration tests passed"
else
    echo "❌ Bend integration tests failed"
    echo "Running cargo test bend to see errors:"
    cargo test bend
    exit 1
fi

echo ""
echo "🎉 All Bend integration tests passed successfully!"
echo ""
echo "📊 Integration Summary:"
echo "  ✅ Bend installation verified"
echo "  ✅ HVM installation verified"
echo "  ✅ Basic Bend programs working"
echo "  ✅ Vector similarity operations working"
echo "  ✅ Rust compilation successful"
echo "  ✅ Bend module integration successful"
echo "  ✅ Integration tests passing"
echo ""
echo "🚀 Bend is ready for production use with Vectorizer!"
