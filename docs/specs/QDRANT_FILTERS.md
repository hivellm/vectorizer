# Qdrant Filter System - Complete Guide

This guide covers all filter types supported by the Vectorizer Qdrant compatibility layer.

## Table of Contents

1. [Overview](#overview)
2. [Filter Structure](#filter-structure)
3. [Match Filters](#match-filters)
4. [Range Filters](#range-filters)
5. [Geo Filters](#geo-filters)
6. [Values Count Filters](#values-count-filters)
7. [Nested Filters](#nested-filters)
8. [Complex Filter Examples](#complex-filter-examples)
9. [Performance Tips](#performance-tips)

## Overview

Filters allow you to refine search results based on payload metadata. All filters are applied **after** vector similarity search but **before** returning results to the client.

### Filter Logic

- **MUST**: All conditions must be true (AND logic)
- **MUST NOT**: All conditions must be false (NOT logic)
- **SHOULD**: At least one condition must be true (OR logic)

## Filter Structure

```json
{
  "filter": {
    "must": [
      // All conditions here must match
    ],
    "must_not": [
      // All conditions here must NOT match
    ],
    "should": [
      // At least ONE condition here must match
    ]
  }
}
```

## Match Filters

Match filters compare exact values in your payload.

### String Match

```json
{
  "filter": {
    "must": [
      {
        "type": "match",
        "key": "category",
        "match_value": "electronics"
      }
    ]
  }
}
```

### Integer Match

```json
{
  "filter": {
    "must": [
      {
        "type": "match",
        "key": "year",
        "match_value": 2024
      }
    ]
  }
}
```

### Boolean Match

```json
{
  "filter": {
    "must": [
      {
        "type": "match",
        "key": "in_stock",
        "match_value": true
      }
    ]
  }
}
```

### Text Match (Advanced)

Text match supports different matching strategies:

#### Contains

```json
{
  "type": "match",
  "key": "description",
  "match_value": {
    "text": "rust",
    "type": "contains"
  }
}
```

#### Prefix

```json
{
  "type": "match",
  "key": "title",
  "match_value": {
    "text": "Introduction to",
    "type": "prefix"
  }
}
```

#### Suffix

```json
{
  "type": "match",
  "key": "filename",
  "match_value": {
    "text": ".rs",
    "type": "suffix"
  }
}
```

#### Exact

```json
{
  "type": "match",
  "key": "status",
  "match_value": {
    "text": "active",
    "type": "exact"
  }
}
```

## Range Filters

Range filters work with numeric values.

### Greater Than (gt)

```json
{
  "type": "range",
  "key": "price",
  "range": {
    "gt": 100.0
  }
}
```

### Greater Than or Equal (gte)

```json
{
  "type": "range",
  "key": "age",
  "range": {
    "gte": 18
  }
}
```

### Less Than (lt)

```json
{
  "type": "range",
  "key": "score",
  "range": {
    "lt": 50.0
  }
}
```

### Less Than or Equal (lte)

```json
{
  "type": "range",
  "key": "temperature",
  "range": {
    "lte": 25.0
  }
}
```

### Between (Exclusive Max)

```json
{
  "type": "range",
  "key": "price",
  "range": {
    "gte": 10.0,
    "lt": 100.0
  }
}
```

### Between (Inclusive)

```json
{
  "type": "range",
  "key": "rating",
  "range": {
    "gte": 3.0,
    "lte": 5.0
  }
}
```

## Geo Filters

Geo filters work with geographic coordinates.

### Geo Bounding Box

Find all points within a rectangular area.

```json
{
  "type": "geo_bounding_box",
  "key": "location",
  "geo_bounding_box": {
    "top_right": {
      "lat": 40.8,
      "lon": -73.9
    },
    "bottom_left": {
      "lat": 40.6,
      "lon": -74.1
    }
  }
}
```

**Use Case**: Find all stores in Manhattan, NY

### Geo Radius

Find all points within a certain distance from a center point.

```json
{
  "type": "geo_radius",
  "key": "location",
  "geo_radius": {
    "center": {
      "lat": 40.7128,
      "lon": -74.0060
    },
    "radius": 5000.0
  }
}
```

**Radius**: Distance in meters
**Use Case**: Find all restaurants within 5km of Times Square

### Supported Location Formats

Your payload can store locations in two formats:

#### Object Format (Recommended)

```json
{
  "location": {
    "lat": 40.7128,
    "lon": -74.0060
  }
}
```

#### Array Format

```json
{
  "location": [40.7128, -74.0060]
}
```

## Values Count Filters

Count the number of items in arrays or objects.

### Array Count

```json
{
  "type": "values_count",
  "key": "tags",
  "values_count": {
    "gte": 3
  }
}
```

**Use Case**: Products with at least 3 tags

### Object Count

```json
{
  "type": "values_count",
  "key": "features",
  "values_count": {
    "lt": 10
  }
}
```

### Between

```json
{
  "type": "values_count",
  "key": "images",
  "values_count": {
    "gte": 2,
    "lte": 5
  }
}
```

**Use Case**: Products with 2 to 5 images

## Nested Filters

Filters support nested JSON structures using dot notation.

### Nested Key Access

```json
{
  "type": "match",
  "key": "user.profile.age",
  "match_value": 30
}
```

For this payload:

```json
{
  "user": {
    "profile": {
      "age": 30,
      "name": "John"
    }
  }
}
```

### Deep Nesting

```json
{
  "type": "range",
  "key": "stats.performance.cpu.usage",
  "range": {
    "lt": 80.0
  }
}
```

## Complex Filter Examples

### E-commerce Product Search

Find electronics priced $50-$200, in stock, with good ratings:

```json
{
  "filter": {
    "must": [
      {
        "type": "match",
        "key": "category",
        "match_value": "electronics"
      },
      {
        "type": "range",
        "key": "price",
        "range": {
          "gte": 50.0,
          "lte": 200.0
        }
      },
      {
        "type": "match",
        "key": "in_stock",
        "match_value": true
      },
      {
        "type": "range",
        "key": "rating",
        "range": {
          "gte": 4.0
        }
      }
    ]
  }
}
```

### Restaurant Finder

Find Italian restaurants within 2km, open now, with ratings > 4.0:

```json
{
  "filter": {
    "must": [
      {
        "type": "match",
        "key": "cuisine",
        "match_value": "italian"
      },
      {
        "type": "geo_radius",
        "key": "location",
        "geo_radius": {
          "center": {
            "lat": 40.7128,
            "lon": -74.0060
          },
          "radius": 2000.0
        }
      },
      {
        "type": "match",
        "key": "open_now",
        "match_value": true
      },
      {
        "type": "range",
        "key": "rating",
        "range": {
          "gt": 4.0
        }
      }
    ]
  }
}
```

### Job Search with Exclusions

Find remote jobs, NOT in sales, with 3+ benefits:

```json
{
  "filter": {
    "must": [
      {
        "type": "match",
        "key": "remote",
        "match_value": true
      },
      {
        "type": "values_count",
        "key": "benefits",
        "values_count": {
          "gte": 3
        }
      }
    ],
    "must_not": [
      {
        "type": "match",
        "key": "department",
        "match_value": "sales"
      }
    ]
  }
}
```

### Real Estate with OR Logic

Find properties that are either luxury OR newly renovated:

```json
{
  "filter": {
    "must": [
      {
        "type": "range",
        "key": "price",
        "range": {
          "lte": 1000000.0
        }
      }
    ],
    "should": [
      {
        "type": "match",
        "key": "luxury",
        "match_value": true
      },
      {
        "type": "range",
        "key": "renovation_year",
        "range": {
          "gte": 2020
        }
      }
    ]
  }
}
```

### Content Moderation

Find user-generated content that needs review:

```json
{
  "filter": {
    "must": [
      {
        "type": "match",
        "key": "status",
        "match_value": "pending"
      }
    ],
    "should": [
      {
        "type": "match",
        "key": "flagged_by_users",
        "match_value": true
      },
      {
        "type": "range",
        "key": "ai_confidence_score",
        "range": {
          "lt": 0.7
        }
      },
      {
        "type": "values_count",
        "key": "reported_issues",
        "values_count": {
          "gte": 1
        }
      }
    ]
  }
}
```

## Performance Tips

### 1. Filter Before Search (When Possible)

If your filter drastically reduces the search space, consider pre-filtering data.

### 2. Use Indexed Fields

Fields frequently used in filters should be indexed for better performance.

### 3. Combine Multiple Conditions Efficiently

Order your MUST conditions from most restrictive to least restrictive:

```json
{
  "must": [
    {"type": "match", "key": "rare_category", "match_value": "X"},  // Very restrictive
    {"type": "range", "key": "price", "range": {"gte": 100}},       // Moderately restrictive
    {"type": "match", "key": "in_stock", "match_value": true}       // Less restrictive
  ]
}
```

### 4. Avoid Deep Nesting

While supported, deeply nested keys (`a.b.c.d.e.f`) are slower than top-level keys.

### 5. Use Geo Filters Wisely

Geo radius calculations are expensive. Combine with bounding box first:

```json
{
  "must": [
    {
      "type": "geo_bounding_box",
      "key": "location",
      "geo_bounding_box": {
        "top_right": {"lat": 41.0, "lon": -73.0},
        "bottom_left": {"lat": 40.0, "lon": -75.0}
      }
    },
    {
      "type": "geo_radius",
      "key": "location",
      "geo_radius": {
        "center": {"lat": 40.7128, "lon": -74.0060},
        "radius": 10000.0
      }
    }
  ]
}
```

### 6. Batch Operations

Use batch search/recommend endpoints for multiple queries with similar filters.

## API Integration

### Search with Filters

```bash
curl -X POST "http://localhost:6333/collections/my_collection/points/search" \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [0.1, 0.2, 0.3, 0.4],
    "limit": 10,
    "filter": {
      "must": [
        {
          "type": "range",
          "key": "price",
          "range": {"gte": 50, "lte": 200}
        }
      ]
    }
  }'
```

### Recommend with Filters

```bash
curl -X POST "http://localhost:6333/collections/my_collection/points/recommend" \
  -H "Content-Type: application/json" \
  -d '{
    "positive": ["id-1", "id-2"],
    "negative": ["id-3"],
    "limit": 10,
    "filter": {
      "must": [
        {
          "type": "match",
          "key": "category",
          "match_value": "similar"
        }
      ]
    }
  }'
```

## Rust API Examples

### Using FilterBuilder

```rust
use vectorizer::models::qdrant::{
    QdrantFilterBuilder, QdrantCondition, QdrantRange, QdrantGeoPoint
};

// Simple filter
let filter = QdrantFilterBuilder::new()
    .must(QdrantCondition::match_string("category", "tech"))
    .must(QdrantCondition::range("price", QdrantRange::between(10.0, 100.0)))
    .build();

// Complex filter
let filter = QdrantFilterBuilder::new()
    .must(QdrantCondition::geo_radius(
        "location",
        QdrantGeoPoint::new(40.7128, -74.0060),
        5000.0
    ))
    .must(QdrantCondition::values_count("tags", QdrantValuesCount::gte(3)))
    .must_not(QdrantCondition::match_string("status", "archived"))
    .should(QdrantCondition::match_bool("featured", true))
    .should(QdrantCondition::match_bool("on_sale", true))
    .build();
```

## Error Handling

### Invalid Filter Structure

- Missing required fields
- Invalid key paths
- Type mismatches

### Unsupported Operations

- Regex matching (use text match instead)
- Full-text search (use semantic search)
- Complex aggregations

## Migration from Qdrant

All filter types are fully compatible with Qdrant's API. Simply change the base URL and your filters will work identically.

### Differences from Qdrant

1. **Performance**: Filters are applied post-search rather than pre-search
2. **Geo Calculations**: Uses Haversine formula (same as Qdrant)
3. **Text Match**: Supports all 4 types (exact, prefix, suffix, contains)

## Filter-Based Operations

Beyond search, filters can be used for bulk operations on vectors and payloads.

### Filter-Based Deletion

Delete all vectors matching a filter:

```bash
POST /collections/{collection}/points/delete
{
  "filter": {
    "must": [
      {"type": "match", "key": "status", "match_value": "archived"}
    ]
  }
}
```

**Use Case**: Clean up old or archived data

### Filter-Based Payload Update

Update payloads on vectors matching a filter:

```bash
POST /collections/{collection}/points/payload
{
  "filter": {
    "must": [
      {"type": "match", "key": "category", "match_value": "electronics"}
    ]
  },
  "payload": {
    "on_sale": true,
    "discount": 0.2
  }
}
```

**Use Case**: Apply bulk updates to matching vectors

### Filter-Based Payload Overwrite

Completely replace payloads on matching vectors:

```bash
PUT /collections/{collection}/points/payload
{
  "filter": {
    "must": [
      {"type": "range", "key": "updated_at", "range": {"lt": 1700000000}}
    ]
  },
  "payload": {
    "migrated": true,
    "version": 2
  }
}
```

**Use Case**: Migration or schema updates

### Filter-Based Payload Delete

Remove specific payload fields from matching vectors:

```bash
POST /collections/{collection}/points/payload/delete
{
  "filter": {
    "must": [
      {"type": "match", "key": "pii_data", "match_value": true}
    ]
  },
  "keys": ["email", "phone", "address"]
}
```

**Use Case**: GDPR compliance, PII removal

### Filter-Based Payload Clear

Remove all payload data from matching vectors:

```bash
POST /collections/{collection}/points/payload/clear
{
  "filter": {
    "must": [
      {"type": "match", "key": "cleanup_required", "match_value": true}
    ]
  }
}
```

**Use Case**: Reset payload data

### Best Practices for Filter Operations

1. **Test filters first**: Use search with the same filter to see what will be affected
2. **Backup data**: Before bulk operations, ensure you have backups
3. **Use specific filters**: Avoid overly broad filters that affect too many vectors
4. **Monitor progress**: Large operations may take time; monitor logs for progress
5. **Batch wisely**: For very large datasets, consider batching operations

## See Also

- [Qdrant Collections Management](./QDRANT_COLLECTIONS.md)
- [Search API Guide](./SEARCH_API.md)
- [Performance Optimization](../BENCHMARKING.md)

