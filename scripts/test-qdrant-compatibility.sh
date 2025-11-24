#!/bin/bash
# Qdrant Compatibility Test Script
# Tests all Qdrant REST API endpoints for compatibility

set -e

BASE_URL="${BASE_URL:-http://localhost:15002/qdrant}"
COLLECTION_NAME="test_compatibility_$(date +%s)"
VERBOSE="${VERBOSE:-false}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counters
TESTS_PASSED=0
TESTS_FAILED=0

# Helper functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Generate a vector array JSON string
generate_vector() {
    local value=$1
    local dimension=$2
    local vector="["
    for i in $(seq 1 $dimension); do
        if [ $i -gt 1 ]; then
            vector+=","
        fi
        vector+="$value"
    done
    vector+="]"
    echo "$vector"
}

test_endpoint() {
    local method=$1
    local endpoint=$2
    local data=$3
    local expected_status=${4:-200}
    local description=$5

    if [ "$VERBOSE" = "true" ]; then
        echo "Testing: $method $endpoint"
    fi

    if [ -n "$data" ]; then
        response=$(curl -s -w "\n%{http_code}" -X "$method" \
            "$BASE_URL$endpoint" \
            -H "Content-Type: application/json" \
            -d "$data")
    else
        response=$(curl -s -w "\n%{http_code}" -X "$method" \
            "$BASE_URL$endpoint")
    fi

    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')

    if [ "$http_code" -eq "$expected_status" ]; then
        log_info "✓ $description"
        ((TESTS_PASSED++))
        return 0
    else
        log_error "✗ $description (expected $expected_status, got $http_code)"
        if [ "$VERBOSE" = "true" ]; then
            echo "Response: $body"
        fi
        ((TESTS_FAILED++))
        return 1
    fi
}

# Cleanup function
cleanup() {
    log_info "Cleaning up test collection..."
    curl -s -X DELETE "$BASE_URL/collections/$COLLECTION_NAME" > /dev/null 2>&1 || true
}

trap cleanup EXIT

# Test 1: List Collections
log_info "Testing Collection Management..."
test_endpoint "GET" "/collections" "" 200 "List collections"

# Test 2: Create Collection
log_info "Creating test collection: $COLLECTION_NAME"
test_endpoint "PUT" "/collections/$COLLECTION_NAME" \
    '{"vectors": {"size": 384, "distance": "Cosine"}}' \
    200 "Create collection"

# Test 3: Get Collection Info
test_endpoint "GET" "/collections/$COLLECTION_NAME" "" 200 "Get collection info"

# Test 4: Collection Already Exists
test_endpoint "PUT" "/collections/$COLLECTION_NAME" \
    '{"vectors": {"size": 384, "distance": "Cosine"}}' \
    409 "Collection already exists (409)"

# Test 5: Upsert Points
log_info "Testing Vector Operations..."
VECTOR_384=$(generate_vector "0.1" 384)
test_endpoint "PUT" "/collections/$COLLECTION_NAME/points" \
    "{\"points\": [{\"id\": \"1\", \"vector\": $VECTOR_384, \"payload\": {\"text\": \"test\"}}]}" \
    200 "Upsert points"

# Test 6: Retrieve Points
test_endpoint "GET" "/collections/$COLLECTION_NAME/points?ids=1" "" 200 "Retrieve points"

# Test 7: Count Points
test_endpoint "POST" "/collections/$COLLECTION_NAME/points/count" \
    '{}' \
    200 "Count points"

# Test 8: Search Points
log_info "Testing Search Operations..."
test_endpoint "POST" "/collections/$COLLECTION_NAME/points/search" \
    "{\"vector\": $VECTOR_384, \"limit\": 10}" \
    200 "Search points"

# Test 9: Filtered Search
test_endpoint "POST" "/collections/$COLLECTION_NAME/points/search" \
    "{\"vector\": $VECTOR_384, \"filter\": {\"must\": [{\"type\": \"match\", \"key\": \"text\", \"match_value\": \"test\"}]}, \"limit\": 10}" \
    200 "Filtered search"

# Test 10: Batch Search
test_endpoint "POST" "/collections/$COLLECTION_NAME/points/search/batch" \
    "{\"searches\": [{\"vector\": $VECTOR_384, \"limit\": 5}]}" \
    200 "Batch search"

# Test 11: Scroll Points
test_endpoint "POST" "/collections/$COLLECTION_NAME/points/scroll" \
    '{"limit": 10}' \
    200 "Scroll points"

# Test 12: Invalid Collection
log_info "Testing Error Handling..."
test_endpoint "GET" "/collections/nonexistent_collection" "" 404 "Collection not found (404)"

# Test 13: Invalid Vector Dimension
VECTOR_100=$(generate_vector "0.1" 100)
test_endpoint "PUT" "/collections/$COLLECTION_NAME/points" \
    "{\"points\": [{\"id\": \"2\", \"vector\": $VECTOR_100, \"payload\": {}}]}" \
    400 "Invalid vector dimension (400)"

# Summary
echo ""
log_info "Test Summary:"
echo "  Passed: $TESTS_PASSED"
echo "  Failed: $TESTS_FAILED"
echo "  Total:  $((TESTS_PASSED + TESTS_FAILED))"

if [ $TESTS_FAILED -eq 0 ]; then
    log_info "All tests passed! ✓"
    exit 0
else
    log_error "Some tests failed!"
    exit 1
fi

