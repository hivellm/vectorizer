# SDK Multi-Tenant Usage Guide

This guide covers how to use Vectorizer SDKs in a HiveHub Cloud multi-tenant environment.

## Overview

When Vectorizer operates in cluster mode with HiveHub Cloud, each tenant (user) has isolated data and resources. SDKs need to authenticate using API keys and respect tenant boundaries.

## Authentication

All SDKs support API key authentication via the `Authorization: Bearer <api_key>` header.

### API Key Types

- **Live Keys** (`hh_live_...`): Production keys for live environments
- **Test Keys** (`hh_test_...`): Test keys for development/staging

## SDK Examples

### TypeScript/JavaScript

```typescript
import { VectorizerClient } from '@vectorizer/sdk';

// Initialize with API key
const client = new VectorizerClient({
  baseUrl: 'https://vectorizer.hivehub.cloud',
  apiKey: 'hh_live_your_api_key_here',
  timeout: 30000,
});

// Create a collection (automatically scoped to your tenant)
const collection = await client.createCollection({
  name: 'my_documents',
  dimension: 384,
  metric: 'cosine',
});

// Insert vectors
await client.insertVectors('my_documents', [
  {
    id: 'doc1',
    vector: [...], // 384-dimensional vector
    payload: { title: 'My Document', source: 'file.pdf' },
  },
]);

// Search (only returns results from your collections)
const results = await client.search('my_documents', {
  vector: queryVector,
  limit: 10,
});
```

### Python

```python
from vectorizer import VectorizerClient

# Initialize with API key
client = VectorizerClient(
    base_url="https://vectorizer.hivehub.cloud",
    api_key="hh_live_your_api_key_here",
    timeout=30
)

# Create collection
collection = client.create_collection(
    name="my_documents",
    dimension=384,
    metric="cosine"
)

# Insert vectors
client.insert_vectors("my_documents", [
    {
        "id": "doc1",
        "vector": [...],  # 384-dimensional vector
        "payload": {"title": "My Document", "source": "file.pdf"}
    }
])

# Search
results = client.search("my_documents", query_vector, limit=10)
```

### Go

```go
package main

import (
    "github.com/hivellm/vectorizer-go"
)

func main() {
    // Initialize with API key
    client := vectorizer.NewClient(&vectorizer.Config{
        BaseURL: "https://vectorizer.hivehub.cloud",
        APIKey:  "hh_live_your_api_key_here",
        Timeout: 30 * time.Second,
    })

    // Create collection
    _, err := client.CreateCollection(&vectorizer.CreateCollectionRequest{
        Name:      "my_documents",
        Dimension: 384,
        Metric:    vectorizer.MetricCosine,
    })
    if err != nil {
        log.Fatal(err)
    }

    // Search
    results, err := client.Search("my_documents", queryVector, &vectorizer.SearchOptions{
        Limit: 10,
    })
}
```

### C#

```csharp
using Vectorizer;

// Initialize with API key
var client = new VectorizerClient(new ClientConfig
{
    BaseUrl = "https://vectorizer.hivehub.cloud",
    ApiKey = "hh_live_your_api_key_here",
    TimeoutSeconds = 30
});

// Create collection
var collection = await client.CreateCollectionAsync(new CreateCollectionRequest
{
    Name = "my_documents",
    Dimension = 384,
    Metric = "cosine"
});

// Search
var results = await client.SearchAsync("my_documents", queryVector, new SearchOptions
{
    Limit = 10
});
```

### Rust

```rust
use vectorizer_sdk::{VectorizerClient, VectorizerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize with API key
    let client = VectorizerClient::new_with_api_key(
        "https://vectorizer.hivehub.cloud",
        "hh_live_your_api_key_here",
    )?;

    // Create collection
    client.create_collection(CreateCollectionRequest {
        name: "my_documents".to_string(),
        dimension: 384,
        metric: Some("cosine".to_string()),
        ..Default::default()
    }).await?;

    // Search
    let results = client.search("my_documents", &query_vector, Some(10)).await?;

    Ok(())
}
```

## Multi-Tenant Best Practices

### 1. Collection Naming

Collections are automatically scoped to your tenant, so you can use simple names:

```typescript
// Good - simple names work
await client.createCollection({ name: 'documents' });
await client.createCollection({ name: 'images' });

// Not needed - no tenant prefix required
// await client.createCollection({ name: 'tenant_123:documents' });
```

### 2. Error Handling

Handle authentication and quota errors appropriately:

```typescript
try {
  await client.createCollection({ name: 'my_collection' });
} catch (error) {
  if (error.status === 401) {
    console.error('Invalid API key');
  } else if (error.status === 403) {
    console.error('Permission denied');
  } else if (error.status === 429) {
    console.error('Quota exceeded or rate limited');
    // Implement backoff/retry logic
  }
}
```

### 3. API Key Security

- Never commit API keys to version control
- Use environment variables or secret management
- Rotate keys periodically using the HiveHub Dashboard

```typescript
// Good - use environment variables
const client = new VectorizerClient({
  apiKey: process.env.VECTORIZER_API_KEY,
});

// Bad - hardcoded keys
const client = new VectorizerClient({
  apiKey: 'hh_live_actual_key_here', // DON'T DO THIS
});
```

### 4. Master/Replica Topology

For high-availability setups, configure master and replica hosts:

```typescript
const client = new VectorizerClient({
  apiKey: process.env.VECTORIZER_API_KEY,
  hosts: {
    master: 'https://master.vectorizer.hivehub.cloud',
    replicas: [
      'https://replica1.vectorizer.hivehub.cloud',
      'https://replica2.vectorizer.hivehub.cloud',
    ],
  },
  readPreference: 'replica', // 'master' | 'replica' | 'nearest'
});

// Writes go to master
await client.insertVectors('collection', vectors);

// Reads can go to replicas for better performance
const results = await client.search('collection', queryVector);

// Force read from master for consistency
const masterClient = client.withMaster();
const freshResults = await masterClient.search('collection', queryVector);
```

### 5. Quota Management

Monitor your usage and respect quotas:

```typescript
// Check collection count before creating
const collections = await client.listCollections();
if (collections.length >= 100) {
  console.warn('Approaching collection limit');
}

// Handle quota errors gracefully
try {
  await client.insertVectors('collection', vectors);
} catch (error) {
  if (error.type === 'quota_exceeded') {
    // Upgrade plan or clean up old data
    console.error(`Quota exceeded: ${error.message}`);
  }
}
```

## Request Signing (Optional)

For enhanced security, enable request signing:

```typescript
import { createRequestSigner } from '@vectorizer/sdk';

const signer = createRequestSigner({
  secretKey: process.env.VECTORIZER_SIGNING_SECRET,
});

const client = new VectorizerClient({
  apiKey: process.env.VECTORIZER_API_KEY,
  requestSigner: signer,
});
```

## IP Whitelisting

If IP whitelisting is enabled for your tenant, ensure requests come from allowed IPs:

1. Configure allowed IPs in the HiveHub Dashboard
2. Use a static IP or configure your proxy/load balancer
3. For development, add your local IP or VPN

## Troubleshooting

### "Invalid API key format"
- Ensure key starts with `hh_live_` or `hh_test_`
- Check for extra whitespace or truncation

### "Collection not found"
- Collections are tenant-scoped; you can only access your own
- Verify collection name spelling

### "Permission denied"
- Check your API key permissions in HiveHub Dashboard
- Some operations require higher permission levels

### "Rate limited"
- Implement exponential backoff
- Consider upgrading your plan for higher limits

### "Quota exceeded"
- Review usage in HiveHub Dashboard
- Clean up unused collections/vectors
- Upgrade plan for more storage

## Support

- Documentation: https://docs.hivehub.cloud/vectorizer
- Issues: https://github.com/hivellm/vectorizer/issues
- Community: https://discord.gg/hivellm
