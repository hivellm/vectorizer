# Vectorizer Go SDK

[![Go Reference](https://pkg.go.dev/badge/github.com/hivellm/vectorizer-sdk-go.svg)](https://pkg.go.dev/github.com/hivellm/vectorizer-sdk-go)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

High-performance Go SDK for Vectorizer vector database.

**Package**: `github.com/hivellm/vectorizer-sdk-go`  
**Version**: 1.5.1

## Features

- ✅ **Simple API**: Clean and intuitive Go interface
- ✅ **High Performance**: Optimized for production workloads
- ✅ **Collection Management**: CRUD operations for collections
- ✅ **Vector Operations**: Insert, search, update, delete vectors
- ✅ **Semantic Search**: Text and vector similarity search
- ✅ **Intelligent Search**: AI-powered search with query expansion, MMR diversification, and domain expansion
- ✅ **Semantic Search**: Advanced semantic search with reranking and similarity thresholds
- ✅ **Contextual Search**: Context-aware search with metadata filtering
- ✅ **Multi-Collection Search**: Cross-collection search with intelligent aggregation
- ✅ **Hybrid Search**: Combine dense and sparse vectors for improved search quality
- ✅ **Discovery Operations**: Collection filtering, query expansion, and intelligent discovery
- ✅ **File Operations**: File content retrieval, chunking, project outlines, and related files
- ✅ **Graph Relationships**: Automatic relationship discovery, path finding, and edge management
- ✅ **Summarization**: Text and context summarization with multiple methods
- ✅ **Workspace Management**: Multi-workspace support for project organization
- ✅ **Backup & Restore**: Collection backup and restore operations
- ✅ **Batch Operations**: Efficient bulk insert, update, delete, and search
- ✅ **Qdrant Compatibility**: Full Qdrant 1.14.x REST API compatibility for easy migration
  - Snapshots API (create, list, delete, recover)
  - Sharding API (create shard keys, distribute data)
  - Cluster Management API (status, recovery, peer management, metadata)
  - Query API (query, batch query, grouped queries with prefetch)
  - Search Groups and Matrix API (grouped results, similarity matrices)
  - Named Vectors support (partial)
  - Quantization configuration (PQ and Binary)
- ✅ **Error Handling**: Comprehensive error handling with typed errors
- ✅ **Type Safety**: Strong typing with Go's type system

## Installation

```bash
go get github.com/hivellm/vectorizer-sdk-go

# Or specific version
go get github.com/hivellm/vectorizer-sdk-go@v1.5.1
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

### Master/Slave Configuration (Read/Write Separation)

Vectorizer supports **Master-Replica replication** for high availability and read scaling. The SDK provides **automatic routing** - writes go to master, reads are distributed across replicas.

#### Basic Setup

```go
package main

import (
    "context"
    "github.com/hivellm/vectorizer-sdk-go"
)

func main() {
    ctx := context.Background()

    // Configure with master and replicas - SDK handles routing automatically
    client := vectorizer.NewClient(&vectorizer.Config{
        Hosts: vectorizer.HostConfig{
            Master:   "http://master-node:15001",
            Replicas: []string{"http://replica1:15001", "http://replica2:15001"},
        },
        APIKey:         "your-api-key",
        ReadPreference: vectorizer.ReadPreferenceReplica, // Master | Replica | Nearest
    })

    // Writes automatically go to master
    client.CreateCollection(&vectorizer.CreateCollectionRequest{
        Name: "documents",
        Config: &vectorizer.CollectionConfig{
            Dimension: 768,
            Metric:    vectorizer.MetricCosine,
        },
    })

    client.InsertText(ctx, "documents", "Sample document", map[string]interface{}{
        "source": "api",
    })

    // Reads automatically go to replicas (load balanced)
    results, _ := client.SearchText(ctx, "documents", "sample", &vectorizer.SearchOptions{
        Limit: 10,
    })

    collections, _ := client.ListCollections(ctx)
}
```

#### Read Preferences

| Preference | Description | Use Case |
|------------|-------------|----------|
| `ReadPreferenceReplica` | Route reads to replicas (round-robin) | Default for high read throughput |
| `ReadPreferenceMaster` | Route all reads to master | When you need read-your-writes consistency |
| `ReadPreferenceNearest` | Route to the node with lowest latency | Geo-distributed deployments |

#### Read-Your-Writes Consistency

For operations that need to immediately read what was just written:

```go
// Option 1: Override read preference for specific operation
client.InsertText(ctx, "docs", "New document", nil)
result, _ := client.GetVectorWithPreference(ctx, "docs", "doc_id", vectorizer.ReadPreferenceMaster)

// Option 2: Use options struct
opts := &vectorizer.GetOptions{ReadPreference: vectorizer.ReadPreferenceMaster}
result, _ := client.GetVector(ctx, "docs", "doc_id", opts)
```

#### Automatic Operation Routing

The SDK automatically classifies operations:

| Operation Type | Routed To | Methods |
|---------------|-----------|---------|
| **Writes** | Always Master | `InsertText`, `InsertVector`, `UpdateVector`, `DeleteVector`, `CreateCollection`, `DeleteCollection` |
| **Reads** | Based on `ReadPreference` | `Search`, `SearchText`, `GetVector`, `ListCollections`, `IntelligentSearch`, `SemanticSearch` |

#### Standalone Mode (Single Node)

For development or single-node deployments:

```go
// Single node - no replication
client := vectorizer.NewClient(&vectorizer.Config{
    BaseURL: "http://localhost:15001",
    APIKey:  "your-api-key",
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

### Contextual Search

```go
// Context-aware search with metadata filtering
results, err := client.ContextualSearch(&vectorizer.ContextualSearchRequest{
    Collection: "docs",
    Query:      "API documentation",
    ContextFilters: map[string]interface{}{
        "category": "backend",
        "language": "go",
    },
    MaxResults: 10,
})
```

### Multi-Collection Search

```go
// Cross-collection search with intelligent aggregation
results, err := client.MultiCollectionSearch(&vectorizer.MultiCollectionSearchRequest{
    Query:              "authentication",
    Collections:        []string{"docs", "code", "tickets"},
    MaxTotalResults:    20,
    MaxPerCollection:   5,
    CrossCollectionReranking: true,
})
```

### Discovery Operations

```go
// Filter collections based on query relevance
filtered, err := client.FilterCollections(&vectorizer.FilterCollectionsRequest{
    Query:    "machine learning",
    MinScore: 0.5,
})

// Expand queries with related terms
expanded, err := client.ExpandQueries(&vectorizer.ExpandQueriesRequest{
    Query:         "neural networks",
    MaxExpansions: 5,
})

// Intelligent discovery across collections
discovery, err := client.Discover(&vectorizer.DiscoverRequest{
    Query:      "authentication methods",
    MaxResults: 10,
})
```

### File Operations

```go
// Get file content from collection
content, err := client.GetFileContent("docs", "src/client.go")

// List all files in a collection
files, err := client.ListFilesInCollection("docs")

// Get ordered chunks of a file
chunks, err := client.GetFileChunksOrdered("docs", "README.md", 1000)

// Get project structure outline
outline, err := client.GetProjectOutline("codebase")

// Find files related to a specific file
related, err := client.GetRelatedFiles("codebase", "src/client.go", 5)
```

### Summarization Operations

```go
// Summarize text using various methods
summary, err := client.SummarizeText(&vectorizer.SummarizeTextRequest{
    Text:      "Long document text...",
    Method:    "extractive", // "extractive", "abstractive", "hybrid"
    MaxLength: 200,
})

// Summarize context with metadata
summary, err := client.SummarizeContext(&vectorizer.SummarizeContextRequest{
    Context: "Document context...",
    Method: "abstractive",
    Focus:  "key_points",
})
```

### Workspace Management

```go
// Add a new workspace
err := client.AddWorkspace(&vectorizer.AddWorkspaceRequest{
    Name: "my-project",
    Path: "/path/to/project",
})

// List all workspaces
workspaces, err := client.ListWorkspaces()

// Remove a workspace
err := client.RemoveWorkspace("my-project")
```

### Backup Operations

```go
// Create a backup of collections
backup, err := client.CreateBackup(&vectorizer.CreateBackupRequest{
    Name: "backup-2024-11-24",
})

// List all available backups
backups, err := client.ListBackups()

// Restore from a backup
err := client.RestoreBackup(&vectorizer.RestoreBackupRequest{
    Filename: "backup-2024-11-24.vecdb",
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
