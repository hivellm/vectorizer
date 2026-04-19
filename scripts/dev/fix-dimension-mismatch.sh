#!/bin/bash
# Fix Dimension Mismatch - Delete and Recreate Collections
# Usage: ./scripts/fix-dimension-mismatch.sh [YOUR_JWT_TOKEN]

set -e

TOKEN="${1:-}"
BASE_URL="http://127.0.0.1:15002"

if [ -z "$TOKEN" ]; then
    echo "❌ Error: JWT token required"
    echo "Usage: $0 <JWT_TOKEN>"
    echo ""
    echo "To get your token:"
    echo "1. Open browser DevTools (F12)"
    echo "2. Go to Application → Local Storage"
    echo "3. Find 'vectorizer_dashboard_token'"
    exit 1
fi

echo "🔍 Listing all collections..."
COLLECTIONS=$(curl -s -H "Authorization: Bearer $TOKEN" "$BASE_URL/api/v1/collections" | jq -r '.collections[]? | .name' 2>/dev/null)

if [ -z "$COLLECTIONS" ]; then
    echo "⚠️  No collections found or authentication failed"
    exit 1
fi

echo "📋 Found collections:"
echo "$COLLECTIONS"
echo ""

# Coleções do workspace
WORKSPACE_COLLECTIONS=(
    "vectorizer-source"
    "vectorizer-docs"
    "vectorizer-config"
    "vectorizer-sdk-typescript"
    "vectorizer-sdk-python"
    "vectorizer-sdk-rust"
    "vectorizer-dashboard"
    "vectorizer-scripts"
    "faq"
)

echo "🗑️  Deleting old collections with wrong dimension..."
for collection in "${WORKSPACE_COLLECTIONS[@]}"; do
    if echo "$COLLECTIONS" | grep -q "^${collection}$"; then
        echo "   Deleting: $collection"
        curl -s -X DELETE -H "Authorization: Bearer $TOKEN" "$BASE_URL/api/v1/collections/$collection" > /dev/null
        echo "   ✅ Deleted: $collection"
    fi
done

echo ""
echo "✨ Creating new collections with dimension=512..."
for collection in "${WORKSPACE_COLLECTIONS[@]}"; do
    echo "   Creating: $collection"
    curl -s -X PUT -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
        "$BASE_URL/api/v1/collections/$collection" \
        -d '{
            "dimension": 512,
            "metric": "cosine",
            "hnsw": {
                "m": 16,
                "ef_construction": 200,
                "ef_search": 64
            },
            "quantization": {
                "type": "sq",
                "sq": { "bits": 8 }
            },
            "embedding": {
                "model": "bm25"
            }
        }' > /dev/null
    echo "   ✅ Created: $collection"
done

echo ""
echo "🎉 All collections recreated successfully!"
echo ""
echo "📝 Next steps:"
echo "   1. Re-index your files through the dashboard"
echo "   2. Or run: rulebook task create reindex-workspace"


