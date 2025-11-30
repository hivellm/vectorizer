---
title: Master/Replica Routing
module: guides
id: master-replica-routing
order: 15
description: Automatic read/write routing for Master-Replica deployments
tags: [guides, replication, high-availability, master-replica, routing, load-balancing]
---

# Master/Replica Routing

All Vectorizer SDKs support **automatic read/write routing** for Master-Replica deployments, providing a MongoDB-like developer experience.

## Overview

When using Vectorizer with Master-Replica replication, the SDKs can automatically:

- **Route writes to master** - All write operations (insert, update, delete) go to the master node
- **Route reads based on preference** - Read operations distributed according to your configuration
- **Load balance across replicas** - Round-robin distribution for replica reads
- **Override per operation** - Force specific operations to master when needed

## Configuration

### TypeScript / JavaScript

```typescript
import { VectorizerClient } from '@hivellm/vectorizer-sdk';

const client = new VectorizerClient({
  hosts: {
    master: 'http://master-node:15002',
    replicas: [
      'http://replica1:15002',
      'http://replica2:15002',
      'http://replica3:15002'
    ]
  },
  apiKey: 'your-api-key',
  readPreference: 'replica'  // 'master' | 'replica' | 'nearest'
});
```

### Python

```python
from vectorizer import VectorizerClient

client = VectorizerClient(
    hosts={
        "master": "http://master-node:15002",
        "replicas": [
            "http://replica1:15002",
            "http://replica2:15002",
            "http://replica3:15002"
        ]
    },
    api_key="your-api-key",
    read_preference="replica"  # "master" | "replica" | "nearest"
)
```

### Rust

```rust
use vectorizer_sdk::{VectorizerClient, ReadPreference};

let client = VectorizerClient::builder()
    .master("http://master-node:15002")
    .replica("http://replica1:15002")
    .replica("http://replica2:15002")
    .replica("http://replica3:15002")
    .api_key("your-api-key")
    .read_preference(ReadPreference::Replica)
    .build()?;
```

### Go

```go
import "github.com/hivellm/vectorizer-sdk-go"

client := vectorizer.NewClient(&vectorizer.Config{
    Hosts: vectorizer.HostConfig{
        Master: "http://master-node:15002",
        Replicas: []string{
            "http://replica1:15002",
            "http://replica2:15002",
            "http://replica3:15002",
        },
    },
    APIKey: "your-api-key",
    ReadPreference: vectorizer.ReadPreferenceReplica,
})
```

### C#

```csharp
using Vectorizer.Sdk;

var client = new VectorizerClient(new ClientConfig
{
    Hosts = new HostConfig
    {
        Master = "http://master-node:15002",
        Replicas = new[] {
            "http://replica1:15002",
            "http://replica2:15002",
            "http://replica3:15002"
        }
    },
    ApiKey = "your-api-key",
    ReadPreference = ReadPreference.Replica
});
```

## Read Preferences

| Preference | Behavior | Use Case |
|------------|----------|----------|
| `master` | All reads go to master | Strong consistency, read-your-writes |
| `replica` | Round-robin across replicas | High read throughput, eventual consistency |
| `nearest` | Route to lowest latency node | Geo-distributed deployments |

## Operation Classification

### Write Operations (Always Master)

These operations are **always** routed to the master node:

- `insertTexts()` / `insertVectors()`
- `updateVector()`
- `deleteVector()` / `deleteVectors()`
- `createCollection()` / `deleteCollection()`
- `batchInsert()` / `batchUpdate()` / `batchDelete()`

### Read Operations (Based on Preference)

These operations respect the `readPreference` setting:

- `search()` / `searchVectors()`
- `hybridSearch()` / `semanticSearch()` / `intelligentSearch()`
- `getVector()` / `listVectors()`
- `listCollections()` / `getCollectionInfo()`
- `healthCheck()`

## Per-Operation Override

### Read-Your-Writes Pattern

After writing, you may need to immediately read from master to ensure consistency:

**TypeScript:**

```typescript
// Write to master
await client.insertTexts('docs', [{ id: 'doc1', text: 'Hello World' }]);

// Read from master to ensure we see the write
const result = await client.getVector('docs', 'doc1', { 
  readPreference: 'master' 
});
```

**Python:**

```python
# Write to master
await client.insert_texts('docs', [{'id': 'doc1', 'text': 'Hello World'}])

# Read from master to ensure we see the write
result = await client.get_vector('docs', 'doc1', read_preference='master')
```

### withMaster() Context

Execute multiple operations with master routing:

**TypeScript:**

```typescript
await client.withMaster(async (masterClient) => {
  // All operations in this block go to master
  await masterClient.insertTexts('docs', [newDoc]);
  const result = await masterClient.search('docs', 'query');
  return result;
});
```

**Python:**

```python
async with client.with_master() as master_client:
    # All operations in this block go to master
    await master_client.insert_texts('docs', [new_doc])
    result = await master_client.search('docs', 'query')
```

**Rust:**

```rust
client.with_master(|master_client| async {
    master_client.insert_texts("docs", &[new_doc]).await?;
    master_client.search("docs", "query").await
}).await?;
```

## Backward Compatibility

Single-node configurations continue to work unchanged:

```typescript
// Single node - no replication
const client = new VectorizerClient({
  baseURL: 'http://localhost:15002',
  apiKey: 'your-api-key'
});
```

## Load Balancing

When `readPreference` is `replica`, read operations are distributed using **round-robin** load balancing:

```
Request 1 → replica1
Request 2 → replica2
Request 3 → replica3
Request 4 → replica1
...
```

This ensures even distribution across all healthy replicas.

## Error Handling

If a replica is unavailable, the SDK will:

1. Skip the unavailable replica
2. Try the next replica in the rotation
3. Fall back to master if all replicas fail (configurable)

## Best Practices

### High Read Throughput

```typescript
const client = new VectorizerClient({
  hosts: { master: '...', replicas: ['...', '...', '...'] },
  readPreference: 'replica'  // Distribute reads across replicas
});
```

### Strong Consistency

```typescript
const client = new VectorizerClient({
  hosts: { master: '...', replicas: ['...'] },
  readPreference: 'master'  // All operations go to master
});
```

### Read-Your-Writes

```typescript
// Write with immediate read
await client.insertTexts('docs', [doc]);
const result = await client.getVector('docs', doc.id, { 
  readPreference: 'master' 
});

// Or use withMaster for multiple operations
await client.withMaster(async (c) => {
  await c.insertTexts('docs', [doc]);
  return c.search('docs', 'query');
});
```

## Related Topics

- [Replication API](../api/REPLICATION.md) - Server-side replication configuration
- [High Availability](../configuration/CLUSTER.md) - Cluster setup guide
- [SDKs Overview](./README.md) - All available SDKs

