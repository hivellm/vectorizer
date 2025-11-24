#!/bin/bash
# Simple test script for graph functions

COLLECTION="graph-enabled-collection"
BASE_URL="http://127.0.0.1:15002"

echo "=== Testing Graph Functions ==="
echo "Collection: $COLLECTION"
echo ""

# Test 1: List graph nodes
echo "Test 1: List graph nodes"
curl -s -X GET "$BASE_URL/api/v1/graph/nodes/$COLLECTION" | jq -r '.count // "Error"'
echo ""

# Test 2: Get collection info
echo "Test 2: Get collection info"
curl -s -X GET "$BASE_URL/collections/$COLLECTION" | jq -r '.vector_count // "Error"'
echo ""

echo "=== Done ==="

