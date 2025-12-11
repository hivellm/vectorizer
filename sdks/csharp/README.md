# Vectorizer C# SDK

[![NuGet version](https://img.shields.io/nuget/v/Vectorizer.Sdk.svg)](https://www.nuget.org/packages/Vectorizer.Sdk)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

High-performance C# SDK for Vectorizer vector database.

**Package**: `Vectorizer.Sdk`  
**Version**: 2.2.0  
**NuGet**: https://www.nuget.org/packages/Vectorizer.Sdk

## Features

- ✅ **Async/Await Support**: Full async/await support for high performance
- ✅ **.NET 8.0+**: Modern C# with latest language features
- ✅ **Type Safety**: Strong typing with comprehensive models
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
- ✅ **Error Handling**: Comprehensive exception handling
- ✅ **IDisposable**: Proper resource management

## Installation

```bash
dotnet add package Vectorizer.Sdk

# Or via NuGet Package Manager
Install-Package Vectorizer.Sdk

# Or specific version
dotnet add package Vectorizer.Sdk --version 2.2.0
```

## Quick Start

```csharp
using Vectorizer;
using Vectorizer.Models;

// Create client
var client = new VectorizerClient(new ClientConfig
{
    BaseUrl = "http://localhost:15002",
    ApiKey = "your-api-key"
});

try
{
    // Health check
    await client.HealthAsync();
    Console.WriteLine("✓ Server is healthy");

    // Create collection
    var collection = await client.CreateCollectionAsync(new CreateCollectionRequest
    {
        Name = "documents",
        Config = new CollectionConfig
        {
            Dimension = 384,
            Metric = DistanceMetric.Cosine
        }
    });
    Console.WriteLine($"✓ Created collection: {collection.Name}");

    // Insert text
    var result = await client.InsertTextAsync("documents", "Hello, world!", null);
    Console.WriteLine($"✓ Inserted vector ID: {result.Id}");

    // Search
    var results = await client.SearchTextAsync("documents", "hello", new SearchOptions
    {
        Limit = 10
    });
    Console.WriteLine($"✓ Found {results.Count} results");

    // Intelligent search
    var intelligentResults = await client.IntelligentSearchAsync(new IntelligentSearchRequest
    {
        Query = "machine learning algorithms",
        Collections = new List<string> { "documents" },
        MaxResults = 15,
        DomainExpansion = true,
        TechnicalFocus = true,
        MMREnabled = true,
        MMRLambda = 0.7
    });
    Console.WriteLine($"✓ Intelligent search found {intelligentResults.Count} results");

    // Graph Operations (requires graph enabled in collection config)
    // List all graph nodes
    var nodes = await client.ListGraphNodesAsync("documents");
    Console.WriteLine($"✓ Graph has {nodes.Count} nodes");

    // Get neighbors of a node
    var neighbors = await client.GetGraphNeighborsAsync("documents", "document1");
    Console.WriteLine($"✓ Node has {neighbors.Neighbors.Count} neighbors");

    // Find related nodes within 2 hops
    var related = await client.FindRelatedNodesAsync("documents", "document1", new FindRelatedRequest
    {
        MaxHops = 2,
        RelationshipType = "SIMILAR_TO"
    });
    Console.WriteLine($"✓ Found {related.Related.Count} related nodes");

    // Find shortest path between two nodes
    var path = await client.FindGraphPathAsync(new FindPathRequest
    {
        Collection = "documents",
        Source = "document1",
        Target = "document2"
    });
    if (path.Found)
    {
        Console.WriteLine($"✓ Path found: {string.Join(" -> ", path.Path.Select(n => n.Id))}");
    }

    // Create explicit relationship
    var edge = await client.CreateGraphEdgeAsync(new CreateEdgeRequest
    {
        Collection = "documents",
        Source = "document1",
        Target = "document2",
        RelationshipType = "REFERENCES",
        Weight = 0.9f
    });
    Console.WriteLine($"✓ Created edge: {edge.EdgeId}");

    // Semantic search
    var semanticResults = await client.SemanticSearchAsync(
        "documents",
        "neural networks",
        maxResults: 10,
        semanticReranking: true
    );
    Console.WriteLine($"✓ Semantic search found {semanticResults.Count} results");
}
catch (VectorizerException ex)
{
    Console.WriteLine($"Error: {ex.ErrorType} - {ex.Message}");
}
finally
{
    client.Dispose();
}
```

## Configuration

### Basic Configuration

```csharp
var client = new VectorizerClient(new ClientConfig
{
    BaseUrl = "http://localhost:15002",
    ApiKey = "your-api-key",
    TimeoutSeconds = 30
});
```

### Custom HTTP Client

```csharp
var httpClient = new HttpClient
{
    Timeout = TimeSpan.FromSeconds(60)
};

var client = new VectorizerClient(new ClientConfig
{
    BaseUrl = "http://localhost:15002",
    ApiKey = "your-api-key",
    HttpClient = httpClient
});
```

### Master/Slave Configuration (Read/Write Separation)

Vectorizer supports **Master-Replica replication** for high availability and read scaling. The SDK provides **automatic routing** - writes go to master, reads are distributed across replicas.

#### Basic Setup

```csharp
using Vectorizer.Sdk;

// Configure with master and replicas - SDK handles routing automatically
var client = new VectorizerClient(new ClientConfig
{
    Hosts = new HostConfig
    {
        Master = "http://master-node:15001",
        Replicas = new[] { "http://replica1:15001", "http://replica2:15001" }
    },
    ApiKey = "your-api-key",
    ReadPreference = ReadPreference.Replica // Master | Replica | Nearest
});

// Writes automatically go to master
await client.CreateCollectionAsync(new CreateCollectionRequest
{
    Name = "documents",
    Config = new CollectionConfig
    {
        Dimension = 768,
        Metric = DistanceMetric.Cosine
    }
});

await client.InsertTextAsync("documents", "Sample document", new Dictionary<string, object>
{
    ["source"] = "api"
});

// Reads automatically go to replicas (load balanced)
var results = await client.SearchTextAsync("documents", "sample", new SearchOptions
{
    Limit = 10
});

var collections = await client.ListCollectionsAsync();
```

#### Read Preferences

| Preference | Description | Use Case |
|------------|-------------|----------|
| `ReadPreference.Replica` | Route reads to replicas (round-robin) | Default for high read throughput |
| `ReadPreference.Master` | Route all reads to master | When you need read-your-writes consistency |
| `ReadPreference.Nearest` | Route to the node with lowest latency | Geo-distributed deployments |

#### Read-Your-Writes Consistency

For operations that need to immediately read what was just written:

```csharp
// Option 1: Override read preference for specific operation
await client.InsertTextAsync("docs", "New document", null);
var result = await client.GetVectorAsync("docs", "doc_id", ReadPreference.Master);

// Option 2: Use options with read preference
var options = new GetOptions { ReadPreference = ReadPreference.Master };
var result = await client.GetVectorAsync("docs", "doc_id", options);
```

#### Automatic Operation Routing

The SDK automatically classifies operations:

| Operation Type | Routed To | Methods |
|---------------|-----------|---------|
| **Writes** | Always Master | `InsertTextAsync`, `InsertVectorAsync`, `UpdateVectorAsync`, `DeleteVectorAsync`, `CreateCollectionAsync`, `DeleteCollectionAsync` |
| **Reads** | Based on `ReadPreference` | `SearchAsync`, `SearchTextAsync`, `GetVectorAsync`, `ListCollectionsAsync`, `IntelligentSearchAsync`, `SemanticSearchAsync` |

#### Standalone Mode (Single Node)

For development or single-node deployments:

```csharp
// Single node - no replication
var client = new VectorizerClient(new ClientConfig
{
    BaseUrl = "http://localhost:15001",
    ApiKey = "your-api-key"
});
```

## API Reference

### Collection Management

```csharp
// List collections
var collections = await client.ListCollectionsAsync();

// Get collection info
var info = await client.GetCollectionInfoAsync("documents");

// Create collection
var collection = await client.CreateCollectionAsync(new CreateCollectionRequest
{
    Name = "documents",
    Config = new CollectionConfig
    {
        Dimension = 384,
        Metric = DistanceMetric.Cosine
    }
});

// Delete collection
await client.DeleteCollectionAsync("documents");
```

### Vector Operations

```csharp
// Insert text (with automatic embedding)
var result = await client.InsertTextAsync("documents", "Hello, world!", new Dictionary<string, object>
{
    ["source"] = "example.txt"
});

// Get vector
var vector = await client.GetVectorAsync("documents", "vector-id");

// Update vector
await client.UpdateVectorAsync("documents", "vector-id", new Vector
{
    Id = "vector-id",
    Data = new float[] { 0.1f, 0.2f, 0.3f },
    Payload = new Dictionary<string, object>
    {
        ["updated"] = true
    }
});

// Delete vector
await client.DeleteVectorAsync("documents", "vector-id");

// Vector search
var results = await client.SearchAsync("documents", new float[] { 0.1f, 0.2f, 0.3f }, new SearchOptions
{
    Limit = 10
});

// Text search
var textResults = await client.SearchTextAsync("documents", "query", new SearchOptions
{
    Limit = 10,
    Filter = new Dictionary<string, object>
    {
        ["category"] = "AI"
    }
});
```

### Intelligent Search

```csharp
// Intelligent search with multi-query expansion
var results = await client.IntelligentSearchAsync(new IntelligentSearchRequest
{
    Query = "machine learning algorithms",
    Collections = new List<string> { "documents", "research" },
    MaxResults = 15,
    DomainExpansion = true,
    TechnicalFocus = true,
    MMREnabled = true,
    MMRLambda = 0.7
});
```

### Semantic Search

```csharp
// Semantic search with reranking
var results = await client.SemanticSearchAsync(
    collectionName: "documents",
    query: "neural networks",
    maxResults: 10,
    semanticReranking: true,
    similarityThreshold: 0.6
);
```

### Contextual Search

```csharp
// Context-aware search with metadata filtering
var results = await client.ContextualSearchAsync(new ContextualSearchRequest
{
    Collection = "docs",
    Query = "API documentation",
    ContextFilters = new Dictionary<string, object>
    {
        ["category"] = "backend",
        ["language"] = "csharp"
    },
    MaxResults = 10
});
```

### Multi-Collection Search

```csharp
// Cross-collection search with intelligent aggregation
var results = await client.MultiCollectionSearchAsync(new MultiCollectionSearchRequest
{
    Query = "authentication",
    Collections = new List<string> { "docs", "code", "tickets" },
    MaxTotalResults = 20,
    MaxPerCollection = 5,
    CrossCollectionReranking = true
});
```

### Discovery Operations

```csharp
// Filter collections based on query relevance
var filtered = await client.FilterCollectionsAsync(new FilterCollectionsRequest
{
    Query = "machine learning",
    MinScore = 0.5
});

// Expand queries with related terms
var expanded = await client.ExpandQueriesAsync(new ExpandQueriesRequest
{
    Query = "neural networks",
    MaxExpansions = 5
});

// Intelligent discovery across collections
var discovery = await client.DiscoverAsync(new DiscoverRequest
{
    Query = "authentication methods",
    MaxResults = 10
});
```

### File Operations

```csharp
// Get file content from collection
var content = await client.GetFileContentAsync("docs", "src/Client.cs");

// List all files in a collection
var files = await client.ListFilesInCollectionAsync("docs");

// Get ordered chunks of a file
var chunks = await client.GetFileChunksOrderedAsync("docs", "README.md", 1000);

// Get project structure outline
var outline = await client.GetProjectOutlineAsync("codebase");

// Find files related to a specific file
var related = await client.GetRelatedFilesAsync("codebase", "src/Client.cs", 5);
```

### Summarization Operations

```csharp
// Summarize text using various methods
var summary = await client.SummarizeTextAsync(new SummarizeTextRequest
{
    Text = "Long document text...",
    Method = "extractive", // "extractive", "abstractive", "hybrid"
    MaxLength = 200
});

// Summarize context with metadata
var summary = await client.SummarizeContextAsync(new SummarizeContextRequest
{
    Context = "Document context...",
    Method = "abstractive",
    Focus = "key_points"
});
```

### Workspace Management

```csharp
// Add a new workspace
await client.AddWorkspaceAsync(new AddWorkspaceRequest
{
    Name = "my-project",
    Path = "/path/to/project"
});

// List all workspaces
var workspaces = await client.ListWorkspacesAsync();

// Remove a workspace
await client.RemoveWorkspaceAsync("my-project");
```

### Backup Operations

```csharp
// Create a backup of collections
var backup = await client.CreateBackupAsync(new CreateBackupRequest
{
    Name = "backup-2024-11-24"
});

// List all available backups
var backups = await client.ListBackupsAsync();

// Restore from a backup
await client.RestoreBackupAsync(new RestoreBackupRequest
{
    Filename = "backup-2024-11-24.vecdb"
});
```

### Batch Operations

```csharp
// Batch insert
var batchResult = await client.BatchInsertAsync("documents", new BatchInsertRequest
{
    Texts = new List<string>
    {
        "Machine learning algorithms",
        "Deep learning neural networks",
        "Natural language processing"
    }
});

// Batch search
var batchSearchResult = await client.BatchSearchAsync("documents", new BatchSearchRequest
{
    Queries = new List<string>
    {
        "machine learning",
        "neural networks",
        "NLP techniques"
    },
    Limit = 5
});
```

## Error Handling

```csharp
try
{
    var collection = await client.CreateCollectionAsync(new CreateCollectionRequest
    {
        Name = "documents",
        Config = new CollectionConfig
        {
            Dimension = 384,
            Metric = DistanceMetric.Cosine
        }
    });
}
catch (VectorizerException ex)
{
    if (ex.IsNotFound)
    {
        Console.WriteLine("Collection not found");
    }
    else if (ex.IsUnauthorized)
    {
        Console.WriteLine("Authentication failed");
    }
    else if (ex.IsValidationError)
    {
        Console.WriteLine($"Validation error: {ex.Message}");
    }
    else
    {
        Console.WriteLine($"Error: {ex.ErrorType} - {ex.Message} (status: {ex.StatusCode})");
    }
}
```

## Examples

See [Examples](./Examples/) directory for more usage examples:

- [Basic Example](./Examples/BasicExample.cs) - Basic operations
- More examples coming soon

## Development

```bash
# Restore dependencies
dotnet restore

# Build
dotnet build

# Run tests
dotnet test

# Build release
dotnet build -c Release

# Pack NuGet package
dotnet pack -c Release
```

## Requirements

- .NET 8.0 or later
- System.Text.Json 9.0.0+

## License

Apache License 2.0 - see [LICENSE](./LICENSE) for details.

## Support

- **Documentation**: [Vectorizer Documentation](../../docs/)
- **Issues**: [GitHub Issues](https://github.com/hivellm/vectorizer/issues)
- **Discussions**: [GitHub Discussions](https://github.com/hivellm/vectorizer/discussions)
