#!/bin/bash
# Fix GPU Collections - Force CPU Collections for Persistence
# This script deletes all GPU collections and recreates them as CPU collections
# so they can be properly saved and persist across restarts

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

# Lista de coleções conhecidas do workspace
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
    "workspace-files"
)

echo "🗑️  Deleting all collections (will recreate as CPU)..."
for collection in "${WORKSPACE_COLLECTIONS[@]}"; do
    if echo "$COLLECTIONS" | grep -q "^${collection}$"; then
        echo "   Deleting: $collection"
        curl -s -X DELETE -H "Authorization: Bearer $TOKEN" "$BASE_URL/api/v1/collections/$collection" > /dev/null
        echo "   ✅ Deleted: $collection"
    fi
done

echo ""
echo "✨ Creating new CPU-only collections (GPU disabled in config)..."
for collection in "${WORKSPACE_COLLECTIONS[@]}"; do
    echo "   Creating: $collection (CPU mode)"
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
    echo "   ✅ Created: $collection (CPU)"
done

echo ""
echo "🎉 All collections recreated as CPU collections!"
echo ""
echo "📝 Next steps:"
echo "   1. Re-index your files through the dashboard or API"
echo "   2. Collections will now persist across restarts"
echo "   3. Check config.yml: gpu.enabled should be false"

