# C# SDK

Official C# SDK for Vectorizer with full Qdrant API compatibility.

## Installation

### NuGet Package

```bash
dotnet add package Vectorizer.Sdk
```

### Package Manager

```powershell
Install-Package Vectorizer.Sdk
```

## Quick Start

```csharp
using Vectorizer.Sdk;

// Create client
var client = new VectorizerClient("http://localhost:15002");

// Create collection
await client.CreateCollectionAsync("my_collection", new CreateCollectionRequest
{
    Dimension = 384,
    Metric = DistanceMetric.Cosine
});

// Insert vectors
await client.InsertVectorsAsync("my_collection", new[]
{
    new VectorData
    {
        Id = "doc1",
        Vector = new float[] { 0.1f, 0.2f, 0.3f, /* ... */ },
        Payload = new Dictionary<string, object>
        {
            ["title"] = "Document Title",
            ["category"] = "tech"
        }
    }
});

// Search
var results = await client.SearchAsync("my_collection", new SearchRequest
{
    Vector = new float[] { 0.1f, 0.2f, 0.3f, /* ... */ },
    Limit = 10
});

foreach (var result in results)
{
    Console.WriteLine($"ID: {result.Id}, Score: {result.Score}");
}
```

## API Reference

### Client Initialization

```csharp
// Basic initialization
var client = new VectorizerClient("http://localhost:15002");

// With options
var client = new VectorizerClient("http://localhost:15002", new VectorizerClientOptions
{
    Timeout = TimeSpan.FromSeconds(30),
    ApiKey = "your-api-key" // Optional
});
```

### Collection Operations

```csharp
// Create collection
await client.CreateCollectionAsync("collection_name", new CreateCollectionRequest
{
    Dimension = 384,
    Metric = DistanceMetric.Cosine,
    HnswConfig = new HnswConfig
    {
        M = 16,
        EfConstruction = 100
    }
});

// List collections
var collections = await client.ListCollectionsAsync();

// Get collection info
var info = await client.GetCollectionInfoAsync("collection_name");

// Delete collection
await client.DeleteCollectionAsync("collection_name");
```

### Vector Operations

```csharp
// Insert single vector
await client.InsertVectorAsync("collection_name", new VectorData
{
    Id = "doc1",
    Vector = vectorArray,
    Payload = new Dictionary<string, object> { ["key"] = "value" }
});

// Insert batch
await client.InsertVectorsAsync("collection_name", vectors);

// Get vector by ID
var vector = await client.GetVectorAsync("collection_name", "doc1");

// Delete vector
await client.DeleteVectorAsync("collection_name", "doc1");
```

### Search Operations

```csharp
// Basic search
var results = await client.SearchAsync("collection_name", new SearchRequest
{
    Vector = queryVector,
    Limit = 10
});

// Search with filter
var results = await client.SearchAsync("collection_name", new SearchRequest
{
    Vector = queryVector,
    Limit = 10,
    Filter = new Filter
    {
        Must = new[]
        {
            new Condition { Match = new MatchCondition { Key = "category", Value = "tech" } }
        }
    }
});

// Intelligent search
var results = await client.IntelligentSearchAsync("collection_name", new IntelligentSearchRequest
{
    Query = "How does authentication work?",
    MaxResults = 10,
    TechnicalFocus = true
});
```

## Qdrant API Compatibility

The C# SDK includes full Qdrant API compatibility:

```csharp
// Create Qdrant-compatible client
var qdrantClient = client.AsQdrant();

// Use Qdrant API
await qdrantClient.UpsertAsync("collection_name", new QdrantUpsertRequest
{
    Points = new[]
    {
        new QdrantPoint
        {
            Id = 1,
            Vector = vectorArray,
            Payload = new Dictionary<string, object> { ["key"] = "value" }
        }
    }
});

// Qdrant search
var results = await qdrantClient.SearchAsync("collection_name", new QdrantSearchRequest
{
    Vector = queryVector,
    Limit = 10,
    WithPayload = true,
    WithVector = false
});

// Snapshots
var snapshots = await qdrantClient.ListSnapshotsAsync("collection_name");
await qdrantClient.CreateSnapshotAsync("collection_name");

// Sharding
var shardKeys = await qdrantClient.ListShardKeysAsync("collection_name");
await qdrantClient.CreateShardKeyAsync("collection_name", "tenant_1");

// Cluster management
var clusterStatus = await qdrantClient.GetClusterStatusAsync();
```

## Advanced Features

### Async/Await Support

All operations are fully async:

```csharp
// Parallel operations
var tasks = collections.Select(async c =>
{
    var info = await client.GetCollectionInfoAsync(c);
    return new { Collection = c, Info = info };
});

var results = await Task.WhenAll(tasks);
```

### Error Handling

```csharp
try
{
    await client.CreateCollectionAsync("existing_collection", request);
}
catch (VectorizerException ex) when (ex.StatusCode == 409)
{
    Console.WriteLine("Collection already exists");
}
catch (VectorizerException ex)
{
    Console.WriteLine($"Error: {ex.Message}");
}
```

### Cancellation Support

```csharp
using var cts = new CancellationTokenSource(TimeSpan.FromSeconds(10));

try
{
    var results = await client.SearchAsync("collection_name", request, cts.Token);
}
catch (OperationCanceledException)
{
    Console.WriteLine("Operation timed out");
}
```

## Configuration

### Connection Options

```csharp
var options = new VectorizerClientOptions
{
    BaseUrl = "http://localhost:15002",
    Timeout = TimeSpan.FromSeconds(30),
    MaxRetries = 3,
    RetryDelay = TimeSpan.FromMilliseconds(500)
};

var client = new VectorizerClient(options);
```

### Dependency Injection

```csharp
// In Startup.cs or Program.cs
services.AddVectorizerClient(options =>
{
    options.BaseUrl = "http://localhost:15002";
    options.Timeout = TimeSpan.FromSeconds(30);
});

// In your service
public class MyService
{
    private readonly IVectorizerClient _client;
    
    public MyService(IVectorizerClient client)
    {
        _client = client;
    }
}
```

## Requirements

- .NET 8.0 or later
- System.Text.Json 9.0+

## Related Topics

- [Python SDK](./PYTHON.md) - Python client library
- [TypeScript SDK](./TYPESCRIPT.md) - TypeScript client library
- [Rust SDK](./RUST.md) - Rust client library
- [Qdrant Compatibility](../qdrant/API_COMPATIBILITY.md) - Qdrant API reference

