#!/bin/bash

echo "🧪 Testing Vectorizer API Endpoints"
echo "===================================="

BASE_URL="http://localhost:15002"

# Test Health
echo -e "\n1️⃣ Testing Health Endpoint:"
curl -s "$BASE_URL/health" | head -3

# Test List Collections
echo -e "\n2️⃣ Testing List Collections:"
curl -s "$BASE_URL/collections" | jq '.collections | length' 2>/dev/null || echo "Failed to parse response"

# Test Get Collection Info
echo -e "\n3️⃣ Testing Get Collection Info:"
curl -s "$BASE_URL/collections/gov-bips" | head -5

# Test Search
echo -e "\n4️⃣ Testing Search:"
SEARCH_RESPONSE=$(curl -s -X POST "$BASE_URL/collections/gov-bips/search/text" \
  -H "Content-Type: application/json" \
  -d '{"query":"bitcoin","limit":2}')
echo "Response keys:" $(echo "$SEARCH_RESPONSE" | jq 'keys' 2>/dev/null || echo "Not JSON")
echo "Results count:" $(echo "$SEARCH_RESPONSE" | jq '.results | length' 2>/dev/null || echo "Failed")

# Test Create Collection
echo -e "\n5️⃣ Testing Create Collection:"
CREATE_RESPONSE=$(curl -s -X POST "$BASE_URL/collections" \
  -H "Content-Type: application/json" \
  -d '{"name":"test_sdk_collection_new","dimension":384,"metric":"cosine"}')
echo "Response keys:" $(echo "$CREATE_RESPONSE" | jq 'keys' 2>/dev/null || echo "Not JSON")
echo "Response:" $(echo "$CREATE_RESPONSE" | head -3)

# Test Embed
echo -e "\n6️⃣ Testing Embed:"
curl -s -X POST "$BASE_URL/embed" \
  -H "Content-Type: application/json" \
  -d '{"text":"test text"}' | head -3

echo -e "\n🎯 Endpoint testing completed!"
