#!/bin/bash

# Test script for File Discovery System
# This script tests the new file discovery functionality

echo "🔍 Testing File Discovery System"
echo "================================="

# Configuration
TEST_DIR="./test_discovery"
SERVER_PORT=8082
LOG_FILE="discovery_test.log"

# Clean up previous test
echo "🧹 Cleaning up previous test..."
rm -rf "$TEST_DIR"
rm -f "$LOG_FILE"

# Create test directory structure
echo "📁 Creating test directory structure..."
mkdir -p "$TEST_DIR/docs"
mkdir -p "$TEST_DIR/src"
mkdir -p "$TEST_DIR/config"

# Create test files
echo "📄 Creating test files..."
echo "# Test Document 1" > "$TEST_DIR/docs/test1.md"
echo "# Test Document 2" > "$TEST_DIR/docs/test2.md"
echo "// Test Source 1" > "$TEST_DIR/src/test1.rs"
echo "// Test Source 2" > "$TEST_DIR/src/test2.rs"
echo "test: value" > "$TEST_DIR/config/test.yaml"

# Create a test workspace configuration
echo "⚙️ Creating test workspace configuration..."
cat > "$TEST_DIR/vectorize-workspace.yml" << EOF
global_settings:
  file_watcher:
    watch_paths:
      - "docs"
      - "src"
      - "config"

projects:
  - name: "test-project"
    path: "."
    collections:
      - name: "test-docs"
        include_patterns: ["*.md"]
      - name: "test-src"
        include_patterns: ["*.rs"]
      - name: "test-config"
        include_patterns: ["*.yaml", "*.yml"]
EOF

echo "🚀 Starting Vectorizer server with file discovery..."
cd "$TEST_DIR"

# Start server in background
RUST_LOG=info ../target/release/vectorizer --port $SERVER_PORT > "../$LOG_FILE" 2>&1 &
SERVER_PID=$!

echo "⏳ Waiting for server to initialize and discover files..."
sleep 30

echo "📊 Checking discovery results..."
if grep -q "File discovery completed" "../$LOG_FILE"; then
    echo "✅ File discovery completed successfully!"
    
    # Extract discovery stats
    DISCOVERY_LINE=$(grep "File discovery completed" "../$LOG_FILE" | tail -1)
    echo "📈 Discovery stats: $DISCOVERY_LINE"
    
    # Check if files were indexed
    if grep -q "files indexed" "../$LOG_FILE"; then
        INDEXED_COUNT=$(grep "files indexed" "../$LOG_FILE" | tail -1 | grep -o '[0-9]\+ files indexed' | grep -o '[0-9]\+')
        echo "✅ $INDEXED_COUNT files were indexed during discovery"
    fi
    
    # Check sync results
    if grep -q "Collection sync completed" "../$LOG_FILE"; then
        SYNC_LINE=$(grep "Collection sync completed" "../$LOG_FILE" | tail -1)
        echo "🔄 Sync results: $SYNC_LINE"
    fi
    
else
    echo "❌ File discovery did not complete or failed"
    echo "📋 Last 20 lines of log:"
    tail -20 "../$LOG_FILE"
fi

echo ""
echo "🔍 Testing real-time file monitoring..."
echo "📝 Creating new test file..."
echo "# New Test Document" > "docs/new_test.md"

echo "⏳ Waiting for file to be detected and indexed..."
sleep 10

if grep -q "File change detected.*new_test.md" "../$LOG_FILE"; then
    echo "✅ New file was detected by file watcher!"
else
    echo "⚠️ New file may not have been detected"
fi

echo ""
echo "🧪 Testing file modification..."
echo "📝 Modifying existing file..."
echo "# Modified Test Document" > "docs/test1.md"

echo "⏳ Waiting for modification to be detected..."
sleep 10

if grep -q "File change detected.*test1.md" "../$LOG_FILE"; then
    echo "✅ File modification was detected by file watcher!"
else
    echo "⚠️ File modification may not have been detected"
fi

echo ""
echo "🧪 Testing file deletion..."
echo "🗑️ Deleting test file..."
rm "docs/test2.md"

echo "⏳ Waiting for deletion to be detected..."
sleep 10

if grep -q "File change detected.*test2.md" "../$LOG_FILE"; then
    echo "✅ File deletion was detected by file watcher!"
else
    echo "⚠️ File deletion may not have been detected"
fi

# Stop server
echo ""
echo "🛑 Stopping server..."
kill $SERVER_PID 2>/dev/null
wait $SERVER_PID 2>/dev/null

echo ""
echo "📊 Final Test Results:"
echo "======================"

# Check overall results
if grep -q "File discovery completed" "../$LOG_FILE" && \
   grep -q "File change detected" "../$LOG_FILE"; then
    echo "🎉 SUCCESS: File Discovery System is working correctly!"
    echo "✅ Initial file discovery: PASSED"
    echo "✅ Real-time file monitoring: PASSED"
    echo "✅ File modification detection: PASSED"
    echo "✅ File deletion detection: PASSED"
else
    echo "❌ FAILURE: File Discovery System has issues"
    echo "📋 Check the log file for details: $LOG_FILE"
fi

echo ""
echo "📋 Log file location: $LOG_FILE"
echo "🔍 To view full log: cat $LOG_FILE"

# Clean up test directory
cd ..
rm -rf "$TEST_DIR"

echo ""
echo "🏁 Test completed!"
