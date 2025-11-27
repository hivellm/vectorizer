---
title: Rust SDK
module: sdks
id: rust-sdk
order: 4
description: Complete Rust SDK guide for Vectorizer
tags: [rust, sdk, client-library]
---

# Rust SDK

Complete guide to using the Vectorizer Rust SDK.

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
vectorizer-sdk = "1.3.0"
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

```rust
use vectorizer_sdk::VectorizerClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client
    let client = VectorizerClient::new("http://localhost:15002")?;

    // Create collection
    client.create_collection("my_docs", 384).await?;

    // Insert text
    client.insert_text("my_docs", "Hello, Vectorizer!").await?;

    // Search
    let results = client.search("my_docs", "hello", 5).await?;
    
    Ok(())
}
```

## Client Configuration

### Basic Client

```rust
let client = VectorizerClient::new("http://localhost:15002")?;
```

### With Custom Timeout

```rust
let client = VectorizerClient::builder()
    .base_url("http://localhost:15002")
    .timeout(std::time::Duration::from_secs(30))
    .build()?;
```

## Collection Operations

### Create Collection

```rust
client.create_collection("my_collection", 384).await?;
```

### List Collections

```rust
let collections = client.list_collections().await?;
for collection in collections {
    println!("Collection: {}", collection.name);
}
```

### Get Collection Info

```rust
let info = client.get_collection_info("my_collection").await?;
println!("Dimension: {}", info.dimension);
```

### Delete Collection

```rust
client.delete_collection("my_collection").await?;
```

## Vector Operations

### Insert Single Text

```rust
let vector_id = client.insert_text(
    "my_collection",
    "Vectorizer is awesome!",
    Some(std::collections::HashMap::from([
        ("source".to_string(), serde_json::json!("readme.md"))
    ]))
).await?;
```

### Batch Insert

```rust
let texts = vec!["Doc 1", "Doc 2", "Doc 3"];
let metadatas = vec![
    Some(std::collections::HashMap::from([("id".to_string(), serde_json::json!(1))])),
    Some(std::collections::HashMap::from([("id".to_string(), serde_json::json!(2))])),
    Some(std::collections::HashMap::from([("id".to_string(), serde_json::json!(3))]))
];
let vector_ids = client.batch_insert_text("my_collection", texts, metadatas).await?;
```

### Get Vector

```rust
let vector = client.get_vector("my_collection", "vector_id").await?;
```

### Update Vector

```rust
client.update_vector(
    "my_collection",
    "vector_id",
    Some("Updated content".to_string()),
    Some(std::collections::HashMap::from([
        ("updated_at".to_string(), serde_json::json!("2024-01-01"))
    ]))
).await?;
```

### Delete Vector

```rust
client.delete_vector("my_collection", "vector_id").await?;
```

## Search Operations

### Basic Search

```rust
let results = client.search("my_collection", "query", 10).await?;
```

### Intelligent Search

```rust
use vectorizer_sdk::IntelligentSearchRequest;

let results = client.intelligent_search(IntelligentSearchRequest {
    collection: "my_collection".to_string(),
    query: "neural networks".to_string(),
    max_results: 15,
    mmr_enabled: Some(true),
    mmr_lambda: Some(0.7),
    domain_expansion: Some(true),
    technical_focus: Some(true),
}).await?;
```

### Hybrid Search

```rust
use vectorizer_sdk::{HybridSearchRequest, SparseVector, HybridScoringAlgorithm};

let sparse = SparseVector::new(vec![0, 5, 10], vec![0.8, 0.6, 0.9])?;
let results = client.hybrid_search(HybridSearchRequest {
    collection: "my_collection".to_string(),
    query: "vector database".to_string(),
    query_sparse: Some(sparse),
    alpha: 0.7,
    algorithm: HybridScoringAlgorithm::ReciprocalRankFusion,
    dense_k: 20,
    sparse_k: 20,
    final_k: 10,
}).await?;
```

## Error Handling

```rust
use vectorizer_sdk::VectorizerError;

match client.create_collection("my_collection", 384).await {
    Ok(_) => println!("Collection created"),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Type Safety

The Rust SDK provides full type safety:

```rust
use vectorizer_sdk::{VectorizerClient, SearchResult};

async fn search_documents(
    client: &VectorizerClient,
    query: &str
) -> Result<Vec<SearchResult>, VectorizerError> {
    client.search("my_collection", query, 10).await
}
```

## Best Practices

1. **Use `?` operator**: For error propagation
2. **Use async/await**: All operations are async
3. **Use batch operations**: Much faster for multiple operations
4. **Handle errors**: Use `Result` types properly
5. **Reuse client**: Create client once and reuse it

## Related Topics

- [Python SDK](./PYTHON.md) - Python SDK
- [TypeScript SDK](./TYPESCRIPT.md) - TypeScript SDK
- [JavaScript SDK](./JAVASCRIPT.md) - JavaScript SDK

