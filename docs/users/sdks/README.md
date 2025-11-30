---
title: SDKs
module: sdks
id: sdks-index
order: 0
description: Client SDKs for Vectorizer in multiple programming languages
tags: [sdks, client-libraries, python, typescript, javascript, rust, csharp, go]
---

# SDKs

Vectorizer provides official SDKs for multiple programming languages.

## Available SDKs

### [Python SDK](./PYTHON.md)

Complete Python SDK with async/await support:

- Installation: `pip install vectorizer-sdk`
- Full type hints
- Async operations
- Qdrant compatibility

### [TypeScript SDK](./TYPESCRIPT.md)

TypeScript/JavaScript SDK with full type safety:

- Installation: `npm install @hivellm/vectorizer-sdk`
- Complete TypeScript types
- Async operations
- Qdrant compatibility

### [JavaScript SDK](./JAVASCRIPT.md)

JavaScript SDK for Node.js:

- Installation: `npm install @hivellm/vectorizer-sdk-js`
- Simple API
- Async operations

### [Rust SDK](./RUST.md)

Rust SDK with full type safety:

- Installation: `cargo add vectorizer-sdk`
- Zero-cost abstractions
- Async operations
- Qdrant compatibility

### [C# SDK](./CSHARP.md)

C# SDK for .NET applications:

- Installation: `dotnet add package Vectorizer.Sdk`
- Full async/await support
- Dependency injection ready
- Qdrant compatibility

### [Go SDK](./GO.md)

Go SDK with idiomatic Go patterns:

- Installation: `go get github.com/hivellm/vectorizer-sdk-go`
- Context-based cancellation
- Concurrent operations
- Qdrant compatibility

## Quick Comparison

| Feature       | Python | TypeScript | JavaScript | Rust | C#  | Go  |
| ------------- | ------ | ---------- | ---------- | ---- | --- | --- |
| Type Safety   | ✅     | ✅         | ❌         | ✅   | ✅  | ✅  |
| Async/Await   | ✅     | ✅         | ✅         | ✅   | ✅  | ✅  |
| Qdrant Compat | ✅     | ✅         | ✅         | ✅   | ✅  | ✅  |
| Hybrid Search | ✅     | ✅         | ✅         | ✅   | ✅  | ✅  |
| Snapshots API | ✅     | ✅         | ✅         | ✅   | ✅  | ✅  |
| Sharding API  | ✅     | ✅         | ✅         | ✅   | ✅  | ✅  |
| Cluster API   | ✅     | ✅         | ✅         | ✅   | ✅  | ✅  |

## Common Operations

All SDKs support the same core operations:

- **Collections**: Create, list, get info, delete
- **Vectors**: Insert, get, update, delete
- **Search**: Basic, intelligent, semantic, hybrid
- **Batch**: Batch insert, update, delete
- **Qdrant**: Full Qdrant API compatibility
- **Snapshots**: Create, list, restore snapshots
- **Sharding**: Shard key management
- **Cluster**: Cluster status and management
- **Master/Replica Routing**: Automatic read/write routing

## Related Topics

- [Collections Guide](../collections/COLLECTIONS.md) - Collection operations
- [Search Guide](../search/SEARCH.md) - Search operations
- [Vectors Guide](../vectors/VECTORS.md) - Vector operations
- [Replication API](../api/REPLICATION.md) - Server-side replication
- [API Reference](../api/API_REFERENCE.md) - Complete REST API
