# SDK Master/Replica Routing Architecture

## Overview

The Vectorizer SDKs implement automatic routing of operations to master and replica nodes based on operation type and configured read preferences. This provides a MongoDB-like developer experience where the SDK handles all connection management and routing logic transparently.

## Architecture Diagram

```
┌──────────────────────────────────────────────────────────────────────┐
│                      Application Layer                                │
│                                                                        │
│  const client = new VectorizerClient({                               │
│    hosts: {                                                           │
│      master: "http://master:15001",                                  │
│      replicas: ["http://r1:15001", "http://r2:15001"]               │
│    },                                                                 │
│    readPreference: "replica"                                         │
│  });                                                                  │
│                                                                        │
│  // Automatic routing!                                               │
│  await client.insertTexts(...)    // → Master                        │
│  await client.searchVectors(...)   // → Replica (round-robin)        │
└──────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌──────────────────────────────────────────────────────────────────────┐
│                      SDK Client Layer                                 │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │              VectorizerClient                                   │ │
│  │  - Hosts configuration (master + replicas)                     │ │
│  │  - Read preference (master/replica/nearest)                    │ │
│  │  - Connection pool management                                  │ │
│  │  - Per-operation routing                                       │ │
│  └────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌──────────────────────────────────────────────────────────────────────┐
│                   Operation Classifier                                │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │  Analyzes operation type:                                      │ │
│  │  • Write Operations (always → Master)                          │ │
│  │    - insertTexts, insertVectors                               │ │
│  │    - updateVector, deleteVector                               │ │
│  │    - createCollection, deleteCollection                       │ │
│  │    - batchInsert, batchUpdate, batchDelete                    │ │
│  │                                                                │ │
│  │  • Read Operations (→ Based on readPreference)                │ │
│  │    - searchVectors, hybridSearch                              │ │
│  │    - intelligentSearch, semanticSearch                        │ │
│  │    - getVector, listVectors                                   │ │
│  │    - listCollections, getCollectionInfo                       │ │
│  └────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────┘
                    │                              │
                    │                              │
              Write Operation                Read Operation
                    │                              │
                    ▼                              ▼
        ┌──────────────────────┐      ┌────────────────────────────┐
        │   Master Router      │      │  Read Preference Router    │
        │                      │      │                            │
        │  Always routes to:   │      │  Routes based on:          │
        │  • Master node       │      │  • readPreference config   │
        │  • No load balancing │      │  • Per-op override         │
        │                      │      │  • withMaster() context    │
        └──────────────────────┘      └────────────────────────────┘
                    │                              │
                    │                    ┌─────────┴──────────┐
                    │                    │                    │
                    │              readPreference:      readPreference:
                    │                 "master"              "replica"
                    │                    │                    │
                    ▼                    ▼                    ▼
        ┌──────────────────────┐  ┌──────────┐    ┌──────────────────────┐
        │       Master         │  │  Master  │    │  Round-Robin         │
        │   (Write Target)     │  │ (Strong  │    │  Replica Selector    │
        │                      │  │Consistency)│   │                      │
        │ http://master:15001  │  └──────────┘    │  Algorithm:          │
        └──────────────────────┘                   │  1. Get replicas[]   │
                                                   │  2. index = counter  │
                                                   │  3. Pick replicas[   │
                                                   │     index % len]     │
                                                   │  4. counter++        │
                                                   └──────────────────────┘
                                                              │
                    ┌─────────────────────────────────────────┼──────────────┐
                    │                                         │              │
                    ▼                                         ▼              ▼
        ┌──────────────────────┐              ┌──────────────────────┐  ┌──────────────────────┐
        │     Replica 1        │              │     Replica 2        │  │     Replica 3        │
        │ (Read-Only Copy)     │              │ (Read-Only Copy)     │  │ (Read-Only Copy)     │
        │                      │              │                      │  │                      │
        │ http://r1:15001      │◄─────────────┼─http://r2:15001      │  │ http://r3:15001      │
        └──────────────────────┘   Sequential └──────────────────────┘  └──────────────────────┘
                                   Round-Robin
                                   Distribution
```

## Request Flow Examples

### Example 1: Write Operation (Insert)

```
Application Code:
  client.insertTexts("docs", [...])
                │
                ▼
  Operation Classifier
    → Identifies: WRITE operation
                │
                ▼
  Master Router
    → Target: Master node only
                │
                ▼
  HTTP Request
    POST http://master:15001/api/v1/collections/docs/texts
```

### Example 2: Read Operation with Replica Preference

```
Application Code:
  client.searchVectors("docs", [0.1, 0.2, 0.3])
  (with readPreference: "replica")
                │
                ▼
  Operation Classifier
    → Identifies: READ operation
                │
                ▼
  Read Preference Router
    → Checks: readPreference = "replica"
                │
                ▼
  Round-Robin Selector
    → Counter = 0 → Replica 1
    → Counter = 1 → Replica 2  ◄─── Current request
    → Counter = 2 → Replica 3
    → Counter = 3 → Replica 1 (wraps around)
                │
                ▼
  HTTP Request
    POST http://r2:15001/api/v1/collections/docs/search
```

### Example 3: Read-Your-Writes Pattern

```
Application Code:
  // Insert new document
  await client.insertTexts("docs", [newDoc])
    → Routes to: Master
  
  // Immediately read it back with override
  const result = await client.searchVectors(
    "docs", 
    query, 
    { readPreference: "master" }  ◄─── Override to master
  )
    → Routes to: Master (not replica)
    → Guarantees: Document is visible
```

### Example 4: withMaster() Context

```
Application Code:
  await client.withMaster(async (masterClient) => {
    // Write operation
    await masterClient.insertTexts("docs", [newDoc])
      → Master
    
    // Read operation (forced to master by context)
    await masterClient.searchVectors("docs", query)
      → Master (instead of replica)
    
    // Another read
    await masterClient.getVector("docs", "id")
      → Master (instead of replica)
  })
  
  // Outside context - back to normal routing
  await client.searchVectors("docs", query)
    → Replica (based on preference)
```

## Connection Pool Management

```
┌────────────────────────────────────────────┐
│       Connection Pool Manager              │
├────────────────────────────────────────────┤
│                                            │
│  Master Connection:                        │
│  ┌──────────────────────────────────────┐ │
│  │ URL: http://master:15001             │ │
│  │ HTTP Client: axios/fetch/reqwest     │ │
│  │ Status: Active                       │ │
│  │ Health: OK                           │ │
│  └──────────────────────────────────────┘ │
│                                            │
│  Replica Connections:                      │
│  ┌──────────────────────────────────────┐ │
│  │ URL: http://r1:15001                 │ │
│  │ HTTP Client: axios/fetch/reqwest     │ │
│  │ Status: Active                       │ │
│  │ Health: OK                           │ │
│  └──────────────────────────────────────┘ │
│  ┌──────────────────────────────────────┐ │
│  │ URL: http://r2:15001                 │ │
│  │ HTTP Client: axios/fetch/reqwest     │ │
│  │ Status: Active                       │ │
│  │ Health: OK                           │ │
│  └──────────────────────────────────────┘ │
│  ┌──────────────────────────────────────┐ │
│  │ URL: http://r3:15001                 │ │
│  │ HTTP Client: axios/fetch/reqwest     │ │
│  │ Status: Degraded                     │ │
│  │ Health: Slow Response                │ │
│  └──────────────────────────────────────┘ │
│                                            │
│  Round-Robin Counter: 5 (atomic)           │
└────────────────────────────────────────────┘
```

## State Machine

```
┌─────────────────────────────────────────────────┐
│           Client Initialization                  │
│                                                  │
│  Input: HostConfig                              │
│  {                                              │
│    master: "http://master:15001",              │
│    replicas: ["http://r1:15001", ...]         │
│  }                                              │
└─────────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────┐
│         Parse and Validate Config                │
│  • Validate URLs                                │
│  • Check at least master is provided           │
│  • Initialize connection pool                  │
│  • Set initial round-robin counter = 0         │
└─────────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────┐
│           Ready State                           │
│  Waiting for operation...                       │
└─────────────────────────────────────────────────┘
                    │
        ┌───────────┴───────────┐
        │                       │
    Write Op                Read Op
        │                       │
        ▼                       ▼
┌──────────────┐      ┌────────────────────┐
│ Route to     │      │ Check preference:  │
│ Master       │      │ • master?          │
│              │      │ • replica?         │
│ No retry     │      │ • nearest?         │
│              │      │ • override?        │
└──────────────┘      └────────────────────┘
        │                       │
        │              ┌────────┴────────┐
        │              │                 │
        │         To Master         To Replica
        │              │                 │
        │              │      ┌──────────┴────────┐
        │              │      │ Round-Robin Select│
        │              │      │ Try replica[i]    │
        │              │      └───────────────────┘
        │              │                 │
        ▼              ▼                 ▼
┌──────────────────────────────────────────┐
│         Execute HTTP Request             │
│  • Send request                          │
│  • Await response                        │
│  • Handle errors                         │
└──────────────────────────────────────────┘
                    │
        ┌───────────┴───────────┐
        │                       │
    Success                   Error
        │                       │
        ▼                       ▼
┌──────────────┐      ┌────────────────────┐
│ Return       │      │ Retry logic:       │
│ Response     │      │ • Write: Fail fast │
│              │      │ • Read: Try next   │
│              │      │   replica          │
└──────────────┘      └────────────────────┘
```

## Sequence Diagrams

### Write Operation Sequence

```
App         SDK Client    Classifier    Master Router    Master Server
 │              │              │              │               │
 │─insertTexts─>│              │              │               │
 │              │──classify───>│              │               │
 │              │<─WRITE op────│              │               │
 │              │──route──────────────────────>│              │
 │              │                              │──POST req───>│
 │              │                              │<─200 OK──────│
 │              │<─response────────────────────│              │
 │<─result──────│              │              │               │
```

### Read Operation Sequence (Replica Preference)

```
App         SDK Client    Classifier    Pref Router    RR Selector    Replica
 │              │              │              │             │            │
 │─searchVectors>│              │              │             │            │
 │              │──classify───>│              │             │            │
 │              │<─READ op─────│              │             │            │
 │              │──check pref─────────────────>│            │            │
 │              │<─use replica─────────────────│            │            │
 │              │──select replica───────────────────────────>│           │
 │              │<─replica 2───────────────────────────────--│           │
 │              │──POST req──────────────────────────────────────────────>│
 │              │<─200 OK────────────────────────────────────────────────│
 │<─results─────│              │              │             │            │
```

## Performance Characteristics

### Routing Overhead

| Operation | Overhead | Description |
|-----------|----------|-------------|
| Operation Classification | O(1) | Constant time lookup in operation type map |
| Read Preference Check | O(1) | Simple variable comparison |
| Round-Robin Selection | O(1) | Atomic counter increment + modulo |
| **Total Routing Overhead** | **< 0.1ms** | Negligible compared to network latency |

### Load Distribution

With 3 replicas and 1000 read operations:
- Replica 1: ~333 requests (33.3%)
- Replica 2: ~333 requests (33.3%)
- Replica 3: ~334 requests (33.4%)

**Variance**: < 0.1% (near-perfect distribution)

## Error Handling and Failover

```
Read Request → Replica 1
                  │
                  ▼
            Connection Error?
                  │
         Yes ◄────┴────► No
          │              │
          ▼              ▼
    Try Replica 2    Return Success
          │
          ▼
    Connection Error?
          │
   Yes ◄──┴──► No
    │          │
    ▼          ▼
Try Replica 3  Return Success
    │
    ▼
All Failed?
    │
    ▼
Return Error
```

## Configuration Examples

### TypeScript/JavaScript

```typescript
// Full master/replica setup
const client = new VectorizerClient({
  hosts: {
    master: "http://master.example.com:15001",
    replicas: [
      "http://replica1.example.com:15001",
      "http://replica2.example.com:15001",
      "http://replica3.example.com:15001"
    ]
  },
  apiKey: "your-api-key",
  readPreference: ReadPreference.Replica  // or "master" or "nearest"
});

// Backward compatible single node
const client = new VectorizerClient({
  baseURL: "http://localhost:15001",
  apiKey: "your-api-key"
});
```

### Python

```python
# Full master/replica setup
client = VectorizerClient(
    hosts={
        "master": "http://master.example.com:15001",
        "replicas": [
            "http://replica1.example.com:15001",
            "http://replica2.example.com:15001",
            "http://replica3.example.com:15001"
        ]
    },
    api_key="your-api-key",
    read_preference=ReadPreference.REPLICA
)

# Backward compatible single node
client = VectorizerClient(
    base_url="http://localhost:15001",
    api_key="your-api-key"
)
```

### Rust

```rust
// Full master/replica setup
let client = VectorizerClient::builder()
    .master("http://master.example.com:15001")
    .replica("http://replica1.example.com:15001")
    .replica("http://replica2.example.com:15001")
    .replica("http://replica3.example.com:15001")
    .api_key("your-api-key")
    .read_preference(ReadPreference::Replica)
    .build()?;

// Backward compatible single node
let client = VectorizerClient::new("http://localhost:15001");
```

## Implementation Status

✅ **Implemented in all SDKs:**
- TypeScript
- JavaScript
- Python
- Rust
- Go
- C#

✅ **Features Complete:**
- Host configuration (master + replicas)
- Read preference (master/replica/nearest)
- Automatic write routing to master
- Automatic read routing based on preference
- Round-robin load balancing
- Per-operation preference override
- withMaster() context support
- Backward compatibility with single-node config

🔄 **In Progress:**
- Comprehensive test suite
- Performance benchmarks
- Production validation

## References

- [SDK Master/Slave Specification](./SDK_MASTER_SLAVE.md)
- [MongoDB Read Preference](https://www.mongodb.com/docs/manual/core/read-preference/)
- [Task: add-sdk-master-slave-abstraction](../../rulebook/tasks/add-sdk-master-slave-abstraction/)

