#!/bin/bash
# Test script for Bend integration

echo "Testing Bend Integration for Vectorizer"
echo "========================================"

# Check if Bend is installed
echo "Checking Bend installation..."
if command -v bend &> /dev/null; then
    echo "✅ Bend is installed"
    bend --version
else
    echo "❌ Bend not found in PATH"
    echo "Please install Bend first:"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/HigherOrderCO/Bend/main/install.sh | bash"
    exit 1
fi

echo ""
echo "Testing simple Bend program..."
if [ -f "examples/bend/simple_test.bend" ]; then
    echo "Running simple_test.bend..."
    bend run-c examples/bend/simple_test.bend
    echo "✅ Simple test completed"
else
    echo "❌ simple_test.bend not found"
fi

echo ""
echo "Testing vector search program..."
if [ -f "examples/bend/vector_search.bend" ]; then
    echo "Running vector_search.bend (CPU)..."
    bend run-c examples/bend/vector_search.bend
    echo "✅ Vector search test completed"
    
    # Try CUDA if available
    echo ""
    echo "Testing CUDA acceleration..."
    bend run-cu examples/bend/vector_search.bend
    echo "✅ CUDA test completed"
else
    echo "❌ vector_search.bend not found"
fi

echo ""
echo "Bend integration test completed!"
