---
title: Recommendation System
module: use-cases
id: recommendation-system
order: 2
description: Build a recommendation system with Vectorizer
tags: [use-cases, recommendations, similarity, examples]
---

# Recommendation System

Build a content-based recommendation system using Vectorizer.

## Overview

This use case demonstrates how to build a recommendation system that:

- Recommends similar items based on content
- Handles user preferences
- Scales to millions of items
- Provides real-time recommendations

## Architecture

```
Items → Feature Extraction → Embedding → Vectorizer → Similarity Search → Recommendations
```

## Implementation

### Step 1: Create Collection for Items

```python
from vectorizer_sdk import VectorizerClient

client = VectorizerClient("http://localhost:15002")

# Create collection optimized for recommendations
await client.create_collection(
    "items",
    dimension=384,
    metric="cosine",  # Best for similarity
    quantization={"enabled": True, "type": "scalar", "bits": 8},
    hnsw_config={"m": 16, "ef_search": 64}  # Fast search
)
```

### Step 2: Index Items

```python
async def index_item(item_id, features, metadata):
    """Index an item with its features."""
    # Features can be text description, tags, or pre-computed embeddings
    await client.insert_text(
        "items",
        features,  # Text description or combined features
        id=item_id,
        metadata={
            "category": metadata["category"],
            "tags": metadata.get("tags", []),
            "rating": metadata.get("rating", 0),
            "popularity": metadata.get("popularity", 0)
        }
    )

# Example: Index products
await index_item(
    "product_001",
    "Wireless Bluetooth headphones with noise cancellation",
    {
        "category": "electronics",
        "tags": ["audio", "wireless", "bluetooth"],
        "rating": 4.5,
        "popularity": 1000
    }
)
```

### Step 3: Find Similar Items

```python
async def recommend_similar(item_id, limit=10, min_score=0.7):
    """Find similar items to a given item."""
    # Get the item's vector
    item = await client.get_vector("items", item_id)

    # Search for similar items
    results = await client.search(
        "items",
        item["vector"],  # Use the item's vector
        limit=limit + 1,  # +1 to exclude the original item
        similarity_threshold=min_score,
        with_payload=True
    )

    # Filter out the original item
    recommendations = [
        {
            "id": r["id"],
            "score": r["score"],
            "category": r["payload"]["category"],
            "tags": r["payload"]["tags"]
        }
        for r in results if r["id"] != item_id
    ][:limit]

    return recommendations

recommendations = await recommend_similar("product_001", limit=5)
```

### Step 4: User-Based Recommendations

```python
async def recommend_for_user(user_id, user_preferences, limit=10):
    """Recommend items based on user preferences."""
    # Combine user preferences into a query
    query = " ".join(user_preferences.get("interests", []))

    # Search with filters
    results = await client.search(
        "items",
        query,
        limit=limit,
        filter={
            "category": {"$in": user_preferences.get("categories", [])}
        } if user_preferences.get("categories") else None,
        with_payload=True
    )

    return [
        {
            "id": r["id"],
            "score": r["score"],
            "title": r["payload"].get("title", ""),
            "category": r["payload"]["category"]
        }
        for r in results
    ]

# Example usage
user_prefs = {
    "interests": ["wireless", "bluetooth", "audio"],
    "categories": ["electronics"]
}
recommendations = await recommend_for_user("user_123", user_prefs)
```

### Step 5: Hybrid Recommendations

```python
async def hybrid_recommendations(item_id, user_preferences, limit=10):
    """Combine item similarity with user preferences."""
    # Get similar items
    similar = await recommend_similar(item_id, limit=limit * 2)

    # Filter by user preferences
    filtered = [
        item for item in similar
        if any(tag in user_preferences.get("interests", [])
               for tag in item["tags"])
    ]

    return filtered[:limit]
```

## Advanced: Collaborative Filtering

```python
async def collaborative_recommendations(user_id, user_items, limit=10):
    """Recommend items based on what similar users liked."""
    # Get embeddings for user's liked items
    item_vectors = []
    for item_id in user_items:
        item = await client.get_vector("items", item_id)
        if item and "vector" in item:
            item_vectors.append(item["vector"])

    if not item_vectors:
        return []

    # Average the vectors to create user profile
    import numpy as np
    user_vector = np.mean(item_vectors, axis=0).tolist()

    # Find similar items
    results = await client.search(
        "items",
        user_vector,
        limit=limit,
        with_payload=True
    )

    # Exclude items user already has
    return [
        r for r in results
        if r["id"] not in user_items
    ]
```

## Performance Tips

1. **Pre-compute embeddings**: Generate embeddings offline for faster indexing
2. **Use batch operations**: Index multiple items at once
3. **Cache recommendations**: Store recommendations for frequently accessed items
4. **Filter early**: Use metadata filters to reduce search space

## Real-World Example

```python
import asyncio
from vectorizer_sdk import VectorizerClient

async def main():
    client = VectorizerClient("http://localhost:15002")

    # Create collection
    await client.create_collection("products", dimension=384)

    # Index products
    products = [
        {
            "id": "p1",
            "description": "Wireless headphones with noise cancellation",
            "category": "electronics",
            "tags": ["audio", "wireless"]
        },
        {
            "id": "p2",
            "description": "Bluetooth speaker with bass boost",
            "category": "electronics",
            "tags": ["audio", "bluetooth"]
        }
    ]

    for p in products:
        await index_item(p["id"], p["description"], {
            "category": p["category"],
            "tags": p["tags"]
        })

    # Get recommendations
    recs = await recommend_similar("p1", limit=5)
    print(f"Recommendations for p1: {recs}")

asyncio.run(main())
```

## Related Topics

- [Search Guide](../search/SEARCH.md) - Similarity search
- [Collections Guide](../collections/COLLECTIONS.md) - Collection setup
- [Performance Guide](../performance/PERFORMANCE.md) - Optimization
