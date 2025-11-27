# Go SDK

Official Go SDK for Vectorizer with full Qdrant API compatibility.

## Installation

```bash
go get github.com/hivellm/vectorizer-sdk-go
```

## Quick Start

```go
package main

import (
    "context"
    "fmt"
    "log"

    vectorizer "github.com/hivellm/vectorizer-sdk-go"
)

func main() {
    // Create client
    client := vectorizer.NewClient("http://localhost:15002")

    ctx := context.Background()

    // Create collection
    err := client.CreateCollection(ctx, "my_collection", &vectorizer.CreateCollectionRequest{
        Dimension: 384,
        Metric:    vectorizer.MetricCosine,
    })
    if err != nil {
        log.Fatal(err)
    }

    // Insert vectors
    err = client.InsertVectors(ctx, "my_collection", []vectorizer.VectorData{
        {
            ID:     "doc1",
            Vector: []float32{0.1, 0.2, 0.3 /* ... */},
            Payload: map[string]interface{}{
                "title":    "Document Title",
                "category": "tech",
            },
        },
    })
    if err != nil {
        log.Fatal(err)
    }

    // Search
    results, err := client.Search(ctx, "my_collection", &vectorizer.SearchRequest{
        Vector: []float32{0.1, 0.2, 0.3 /* ... */},
        Limit:  10,
    })
    if err != nil {
        log.Fatal(err)
    }

    for _, result := range results {
        fmt.Printf("ID: %s, Score: %.4f\n", result.ID, result.Score)
    }
}
```

## API Reference

### Client Initialization

```go
// Basic initialization
client := vectorizer.NewClient("http://localhost:15002")

// With options
client := vectorizer.NewClient("http://localhost:15002",
    vectorizer.WithTimeout(30*time.Second),
    vectorizer.WithAPIKey("your-api-key"),
)
```

### Collection Operations

```go
// Create collection
err := client.CreateCollection(ctx, "collection_name", &vectorizer.CreateCollectionRequest{
    Dimension: 384,
    Metric:    vectorizer.MetricCosine,
    HnswConfig: &vectorizer.HnswConfig{
        M:              16,
        EfConstruction: 100,
    },
})

// List collections
collections, err := client.ListCollections(ctx)

// Get collection info
info, err := client.GetCollectionInfo(ctx, "collection_name")

// Delete collection
err = client.DeleteCollection(ctx, "collection_name")
```

### Vector Operations

```go
// Insert single vector
err := client.InsertVector(ctx, "collection_name", vectorizer.VectorData{
    ID:      "doc1",
    Vector:  vectorArray,
    Payload: map[string]interface{}{"key": "value"},
})

// Insert batch
err := client.InsertVectors(ctx, "collection_name", vectors)

// Get vector by ID
vector, err := client.GetVector(ctx, "collection_name", "doc1")

// Delete vector
err = client.DeleteVector(ctx, "collection_name", "doc1")
```

### Search Operations

```go
// Basic search
results, err := client.Search(ctx, "collection_name", &vectorizer.SearchRequest{
    Vector: queryVector,
    Limit:  10,
})

// Search with filter
results, err := client.Search(ctx, "collection_name", &vectorizer.SearchRequest{
    Vector: queryVector,
    Limit:  10,
    Filter: &vectorizer.Filter{
        Must: []vectorizer.Condition{
            {Match: &vectorizer.MatchCondition{Key: "category", Value: "tech"}},
        },
    },
})

// Intelligent search
results, err := client.IntelligentSearch(ctx, "collection_name", &vectorizer.IntelligentSearchRequest{
    Query:          "How does authentication work?",
    MaxResults:     10,
    TechnicalFocus: true,
})
```

## Qdrant API Compatibility

The Go SDK includes full Qdrant API compatibility:

```go
// Create Qdrant-compatible client
qdrantClient := client.Qdrant()

// Use Qdrant API
err := qdrantClient.Upsert(ctx, "collection_name", &vectorizer.QdrantUpsertRequest{
    Points: []vectorizer.QdrantPoint{
        {
            ID:      1,
            Vector:  vectorArray,
            Payload: map[string]interface{}{"key": "value"},
        },
    },
})

// Qdrant search
results, err := qdrantClient.Search(ctx, "collection_name", &vectorizer.QdrantSearchRequest{
    Vector:      queryVector,
    Limit:       10,
    WithPayload: true,
    WithVector:  false,
})

// Snapshots
snapshots, err := qdrantClient.ListSnapshots(ctx, "collection_name")
snapshot, err := qdrantClient.CreateSnapshot(ctx, "collection_name")

// Sharding
shardKeys, err := qdrantClient.ListShardKeys(ctx, "collection_name")
err = qdrantClient.CreateShardKey(ctx, "collection_name", "tenant_1")

// Cluster management
status, err := qdrantClient.GetClusterStatus(ctx)
```

## Advanced Features

### Concurrent Operations

```go
// Parallel search across collections
var wg sync.WaitGroup
results := make(chan SearchResult, len(collections))

for _, collection := range collections {
    wg.Add(1)
    go func(c string) {
        defer wg.Done()
        result, err := client.Search(ctx, c, request)
        if err == nil {
            results <- SearchResult{Collection: c, Results: result}
        }
    }(collection)
}

wg.Wait()
close(results)
```

### Error Handling

```go
result, err := client.CreateCollection(ctx, "existing_collection", request)
if err != nil {
    var apiErr *vectorizer.APIError
    if errors.As(err, &apiErr) {
        switch apiErr.StatusCode {
        case 409:
            log.Println("Collection already exists")
        case 404:
            log.Println("Not found")
        default:
            log.Printf("API error: %s", apiErr.Message)
        }
    }
    return err
}
```

### Context with Timeout

```go
ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
defer cancel()

results, err := client.Search(ctx, "collection_name", request)
if err != nil {
    if errors.Is(err, context.DeadlineExceeded) {
        log.Println("Operation timed out")
    }
    return err
}
```

## Configuration

### Client Options

```go
client := vectorizer.NewClient("http://localhost:15002",
    vectorizer.WithTimeout(30*time.Second),
    vectorizer.WithMaxRetries(3),
    vectorizer.WithRetryDelay(500*time.Millisecond),
    vectorizer.WithHTTPClient(customHTTPClient),
)
```

### Custom HTTP Client

```go
httpClient := &http.Client{
    Timeout: 30 * time.Second,
    Transport: &http.Transport{
        MaxIdleConns:        100,
        MaxIdleConnsPerHost: 100,
        IdleConnTimeout:     90 * time.Second,
    },
}

client := vectorizer.NewClient("http://localhost:15002",
    vectorizer.WithHTTPClient(httpClient),
)
```

## Types

### Distance Metrics

```go
const (
    MetricCosine    = "cosine"
    MetricEuclidean = "euclidean"
    MetricDot       = "dot"
)
```

### Search Result

```go
type SearchResult struct {
    ID      string                 `json:"id"`
    Score   float32                `json:"score"`
    Vector  []float32              `json:"vector,omitempty"`
    Payload map[string]interface{} `json:"payload,omitempty"`
}
```

### Filter Types

```go
type Filter struct {
    Must    []Condition `json:"must,omitempty"`
    MustNot []Condition `json:"must_not,omitempty"`
    Should  []Condition `json:"should,omitempty"`
}

type Condition struct {
    Match *MatchCondition `json:"match,omitempty"`
    Range *RangeCondition `json:"range,omitempty"`
}
```

## Requirements

- Go 1.21 or later

## Related Topics

- [Python SDK](./PYTHON.md) - Python client library
- [TypeScript SDK](./TYPESCRIPT.md) - TypeScript client library
- [Rust SDK](./RUST.md) - Rust client library
- [Qdrant Compatibility](../qdrant/API_COMPATIBILITY.md) - Qdrant API reference

