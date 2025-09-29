#!/bin/bash

echo "🔍 Testing Pure Rust SDK"
echo "========================"

cd /mnt/f/Node/hivellm/vectorizer/client-sdks/rust

echo "📁 Current directory: $(pwd)"
echo "📋 Files:"
ls -la

echo ""
echo "🔧 Rust version:"
rustc --version
cargo --version

echo ""
echo "📦 Checking Cargo.toml:"
cat Cargo.toml

echo ""
echo "🏗️ Building SDK:"
cargo build

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    
    echo ""
    echo "🧪 Running comprehensive test:"
    cargo run --example comprehensive_test
    
    echo ""
    echo "🔬 Running integration tests:"
    cargo test --test integration_tests
    
    echo ""
    echo "🎯 Running working example:"
    cargo run --example test_working
    
else
    echo "❌ Build failed!"
fi
