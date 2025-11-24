# Vectorizer Go SDK

[![Go Reference](https://pkg.go.dev/badge/github.com/hivellm/vectorizer-sdk-go.svg)](https://pkg.go.dev/github.com/hivellm/vectorizer-sdk-go)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

High-performance Go SDK for Vectorizer vector database.

**Package**: `github.com/hivellm/vectorizer-sdk-go`  
**Version**: 1.5.0

## Features

- ✅ **Simple API**: Clean and intuitive Go interface
- ✅ **High Performance**: Optimized for production workloads
- ✅ **Collection Management**: CRUD operations for collections
- ✅ **Vector Operations**: Insert, search, update, delete vectors
- ✅ **Semantic Search**: Text and vector similarity search
- ✅ **Intelligent Search**: Advanced multi-query search with domain expansion
- ✅ **Semantic Search**: High-precision semantic search with reranking
- ✅ **Hybrid Search**: Combine dense and sparse vectors for improved search quality
- ✅ **Batch Operations**: Efficient bulk operations
- ✅ **Error Handling**: Comprehensive error handling with typed errors
- ✅ **Type Safety**: Strong typing with Go's type system

## Installation

```bash
go get github.com/hivellm/vectorizer-sdk-go

# Or specific version
go get github.com/hivellm/vectorizer-sdk-go@v1.5.0
```

## Quick Start

```go
package main

import (
    "fmt"
    "log"
    "github.com/hivellm/vectorizer-sdk-go"
)

func main() {
    // Create client
    client := vectorizer.NewClient(&vectorizer.Config{
        BaseURL: "http://localhost:15002",
        APIKey:  "your-api-key",
    })

    // Health check
    if err := client.Health(); err != nil {
        log.Fatalf("Health check failed: %v", err)
    }
    fmt.Println("✓ Server is healthy")

    // Create collection
    collection, err := client.CreateCollection(&vectorizer.CreateCollectionRequest{
        Name: "documents",
        Config: &vectorizer.CollectionConfig{
            Dimension: 384,
            Metric:    vectorizer.MetricCosine,
        },
    })
    if err != nil {
        log.Fatalf("Failed to create collection: %v", err)
    }
    fmt.Printf("✓ Created collection: %s\n", collection.Name)

    // Insert text
    result, err := client.InsertText("documents", "Hello, world!", nil)
    if err != nil {
        log.Fatalf("Failed to insert text: %v", err)
    }
    fmt.Printf("✓ Inserted vector ID: %s\n", result.ID)

    // Search
    results, err := client.SearchText("documents", "hello", &vectorizer.SearchOptions{
        Limit: 10,
    })
    if err != nil {
        log.Fatalf("Failed to search: %v", err)
    }
    fmt.Printf("✓ Found %d results\n", len(results))

    // Intelligent search
    intelligentResults, err := client.IntelligentSearch(&vectorizer.IntelligentSearchRequest{
        Query:       "machine learning algorithms",
        Collections: []string{"documents"},
        MaxResults:  15,
        DomainExpansion: true,
        TechnicalFocus: true,
        MMREnabled:  true,
        MMRLambda:   0.7,
    })
    if err != nil {
        log.Fatalf("Failed intelligent search: %v", err)
    }
    fmt.Printf("✓ Intelligent search found %d results\n", len(intelligentResults))

    // Graph Operations (requires graph enabled in collection config)
    // List all graph nodes
    nodes, err := client.ListGraphNodes("documents")
    if err != nil {
        log.Fatalf("Failed to list graph nodes: %v", err)
    }
    fmt.Printf("✓ Graph has %d nodes\n", nodes.Count)

    // Get neighbors of a node
    neighbors, err := client.GetGraphNeighbors("documents", "document1")
    if err != nil {
        log.Fatalf("Failed to get neighbors: %v", err)
    }
    fmt.Printf("✓ Node has %d neighbors\n", len(neighbors.Neighbors))

    // Find related nodes within 2 hops
    related, err := client.FindRelatedNodes("documents", "document1", &vectorizer.FindRelatedRequest{
        MaxHops:          2,
        RelationshipType: "SIMILAR_TO",
    })
    if err != nil {
        log.Fatalf("Failed to find related nodes: %v", err)
    }
    fmt.Printf("✓ Found %d related nodes\n", len(related.Related))

    // Find shortest path between two nodes
    path, err := client.FindGraphPath(&vectorizer.FindPathRequest{
        Collection: "documents",
        Source:     "document1",
        Target:     "document2",
    })
    if err != nil {
        log.Fatalf("Failed to find path: %v", err)
    }
    if path.Found {
        fmt.Printf("✓ Path found: %v\n", path.Path)
    }

    // Create explicit relationship
    edge, err := client.CreateGraphEdge(&vectorizer.CreateEdgeRequest{
        Collection:       "documents",
        Source:           "document1",
        Target:           "document2",
        RelationshipType: "REFERENCES",
        Weight:           0.9,
    })
    if err != nil {
        log.Fatalf("Failed to create edge: %v", err)
    }
    fmt.Printf("✓ Created edge: %s\n", edge.EdgeID)

    // Semantic search
    semanticResults, err := client.SemanticSearch(&vectorizer.SemanticSearchRequest{
        Collection:         "documents",
        Query:              "neural networks",
        MaxResults:         10,
        SemanticReranking:  true,
        SimilarityThreshold: 0.6,
    })
    if err != nil {
        log.Fatalf("Failed semantic search: %v", err)
    }
    fmt.Printf("✓ Semantic search found %d results\n", len(semanticResults))
}
```

## Configuration

### Basic Configuration

```go
client := vectorizer.NewClient(&vectorizer.Config{
    BaseURL: "http://localhost:15002",
    APIKey:  "your-api-key",
    Timeout: 30 * time.Second,
})
```

### Custom HTTP Client

```go
httpClient := &http.Client{
    Timeout: 60 * time.Second,
    Transport: &http.Transport{
        MaxIdleConns: 100,
        IdleConnTimeout: 90 * time.Second,
    },
}

client := vectorizer.NewClient(&vectorizer.Config{
    BaseURL:    "http://localhost:15002",
    APIKey:     "your-api-key",
    HTTPClient: httpClient,
})
```

## API Reference

### Collection Management

```go
// List collections
collections, err := client.ListCollections()

// Get collection info
info, err := client.GetCollectionInfo("documents")

// Create collection
collection, err := client.CreateCollection(&vectorizer.CreateCollectionRequest{
    Name: "documents",
    Config: &vectorizer.CollectionConfig{
        Dimension: 384,
        Metric:    vectorizer.MetricCosine,
    },
})

// Delete collection
err := client.DeleteCollection("documents")
```

### Vector Operations

```go
// Insert text (with automatic embedding)
result, err := client.InsertText("documents", "Hello, world!", map[string]interface{}{
    "source": "example.txt",
})

// Get vector
vector, err := client.GetVector("documents", "vector-id")

// Update vector
err := client.UpdateVector("documents", "vector-id", &vectorizer.Vector{
    Data: []float32{0.1, 0.2, 0.3},
    Payload: map[string]interface{}{
        "updated": true,
    },
})

// Delete vector
err := client.DeleteVector("documents", "vector-id")

// Vector search
results, err := client.Search("documents", []float32{0.1, 0.2, 0.3}, &vectorizer.SearchOptions{
    Limit: 10,
})

// Text search
results, err := client.SearchText("documents", "query", &vectorizer.SearchOptions{
    Limit: 10,
    Filter: map[string]interface{}{
        "category": "AI",
    },
})
```

### Intelligent Search

```go
// Intelligent search with multi-query expansion
results, err := client.IntelligentSearch(&vectorizer.IntelligentSearchRequest{
    Query:           "machine learning algorithms",
    Collections:     []string{"documents", "research"},
    MaxResults:      15,
    DomainExpansion: true,
    TechnicalFocus: true,
    MMREnabled:      true,
    MMRLambda:       0.7,
})
```

### Semantic Search

```go
// Semantic search with reranking
results, err := client.SemanticSearch(&vectorizer.SemanticSearchRequest{
    Collection:         "documents",
    Query:              "neural networks",
    MaxResults:         10,
    SemanticReranking:  true,
    SimilarityThreshold: 0.6,
})
```

### Batch Operations

```go
// Batch insert
batchResult, err := client.BatchInsert("documents", &vectorizer.BatchInsertRequest{
    Texts: []string{
        "Machine learning algorithms",
        "Deep learning neural networks",
        "Natural language processing",
    },
})

// Batch search
batchSearchResult, err := client.BatchSearch("documents", &vectorizer.BatchSearchRequest{
    Queries: []string{
        "machine learning",
        "neural networks",
        "NLP techniques",
    },
    Limit: 5,
})
```

## Error Handling

```go
result, err := client.CreateCollection(&vectorizer.CreateCollectionRequest{
    Name: "documents",
    Config: &vectorizer.CollectionConfig{
        Dimension: 384,
        Metric:    vectorizer.MetricCosine,
    },
})

if err != nil {
    if vectorizerErr, ok := err.(*vectorizer.VectorizerError); ok {
        if vectorizerErr.IsNotFound() {
            fmt.Println("Collection not found")
        } else if vectorizerErr.IsUnauthorized() {
            fmt.Println("Authentication failed")
        } else if vectorizerErr.IsValidationError() {
            fmt.Println("Validation error:", vectorizerErr.Message)
        } else {
            fmt.Printf("Error: %s (status: %d)\n", vectorizerErr.Message, vectorizerErr.Status)
        }
    } else {
        fmt.Printf("Unexpected error: %v\n", err)
    }
    return
}
```

## Examples

See [examples](./examples/) directory for more usage examples:

- [Basic Usage](./examples/basic.go) - Basic operations
- More examples coming soon

## Development

```bash
# Run tests
go test ./...

# Run tests with coverage
go test -cover ./...

# Build
go build ./...

# Format code
go fmt ./...

# Lint
golangci-lint run
```

## License

Apache License 2.0 - see [LICENSE](./LICENSE) for details.

## Support

- **Documentation**: [Vectorizer Documentation](../../docs/)
- **Issues**: [GitHub Issues](https://github.com/hivellm/vectorizer/issues)
- **Discussions**: [GitHub Discussions](https://github.com/hivellm/vectorizer/discussions)
