# Add SDK Master/Slave Abstraction

## Status: pending

## Why

Currently, when using Vectorizer with Master-Replica replication, developers need to manually manage separate client instances for write operations (master) and read operations (replicas). This approach is error-prone, requires boilerplate code, and doesn't match the developer experience of modern databases like MongoDB.

MongoDB provides a clean abstraction where:
- You configure once with connection string(s) and a `readPreference`
- The SDK automatically routes writes to primary and reads to secondaries
- Developers use a single client interface without worrying about connection management

Vectorizer SDKs should provide the same level of abstraction for a better developer experience.

## What Changes

### 1. Configuration API Changes (All SDKs)

Add new configuration options to support master/replica topology:

```typescript
// TypeScript/JavaScript
const client = new VectorizerClient({
  hosts: {
    master: "http://master-node:15001",
    replicas: ["http://replica1:15001", "http://replica2:15001"]
  },
  apiKey: "your-api-key",
  readPreference: "replica"  // "master" | "replica" | "nearest"
});
```

```python
# Python
client = VectorizerClient(
    hosts={
        "master": "http://master-node:15001",
        "replicas": ["http://replica1:15001", "http://replica2:15001"]
    },
    api_key="your-api-key",
    read_preference="replica"
)
```

```rust
// Rust
let client = VectorizerClient::builder()
    .master("http://master-node:15001")
    .replica("http://replica1:15001")
    .replica("http://replica2:15001")
    .api_key("your-api-key")
    .read_preference(ReadPreference::Replica)
    .build()?;
```

### 2. Read Preferences

Implement three read preferences:

| Preference | Behavior |
|------------|----------|
| `master` | Route all reads to master (strong consistency) |
| `replica` | Route reads to replicas with round-robin load balancing |
| `nearest` | Route to the node with lowest latency |

### 3. Automatic Operation Routing

The SDK should internally classify operations:

**Write Operations** (always routed to master):
- `insertTexts`, `insertVectors`
- `updateVector`, `deleteVector`
- `createCollection`, `deleteCollection`
- `batchInsert`, `batchUpdate`, `batchDelete`

**Read Operations** (routed based on `readPreference`):
- `searchVectors`, `hybridSearch`, `intelligentSearch`, `semanticSearch`
- `getVector`, `listVectors`
- `listCollections`, `getCollectionInfo`

### 4. Read Preference Override

Allow per-operation override for read-your-writes scenarios:

```typescript
// Override for single operation
const result = await client.getVector("docs", "id", { readPreference: "master" });

// Context manager pattern
await client.withMaster(async (masterClient) => {
  await masterClient.insertTexts("docs", [newDoc]);
  return await masterClient.getVector("docs", newDoc.id);
});
```

### 5. Backward Compatibility

The existing single-node configuration MUST continue to work:

```typescript
// Single node (no replication) - unchanged
const client = new VectorizerClient({
  baseURL: "http://localhost:15001",
  apiKey: "your-api-key"
});
```

### 6. Internal Connection Pool

Implement internal connection management:
- Maintain HTTP client pool for each node
- Implement round-robin for replica selection
- Handle connection failures with automatic retry to next replica
- Implement health checking for nodes (optional)

## Impact

- **Developer Experience**: Clean, MongoDB-like API for master/replica deployments
- **Backward Compatibility**: Existing single-node configurations unchanged
- **Code Reduction**: No more boilerplate for managing multiple clients
- **Production Ready**: Built-in load balancing and failover support

## Files to Modify

### TypeScript SDK
- `sdks/typescript/src/client.ts` - Add hosts config and routing logic
- `sdks/typescript/src/types.ts` - Add HostConfig, ReadPreference types
- `sdks/typescript/README.md` - Update documentation

### JavaScript SDK
- `sdks/javascript/src/client.js` - Add hosts config and routing logic
- `sdks/javascript/README.md` - Update documentation

### Python SDK
- `sdks/python/client.py` - Add hosts config and routing logic
- `sdks/python/models.py` - Add HostConfig, ReadPreference types
- `sdks/python/README.md` - Update documentation

### Rust SDK
- `sdks/rust/src/client.rs` - Add builder pattern with hosts
- `sdks/rust/src/types.rs` - Add HostConfig, ReadPreference
- `sdks/rust/README.md` - Update documentation

### Go SDK
- `sdks/go/client.go` - Add HostConfig and routing
- `sdks/go/README.md` - Update documentation

### C# SDK
- `sdks/csharp/VectorizerClient.cs` - Add HostConfig and routing
- `sdks/csharp/README.md` - Update documentation

## Testing

1. Unit tests for operation classification (write vs read)
2. Unit tests for read preference routing
3. Integration tests with mock master/replica topology
4. Backward compatibility tests with single-node config
5. Round-robin load balancing verification
6. Read preference override tests
