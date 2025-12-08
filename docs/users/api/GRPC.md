# gRPC API

Vectorizer provides native gRPC APIs for high-performance, strongly-typed communication. This includes both the Vectorizer-native gRPC service and full Qdrant gRPC API compatibility.

## Overview

Vectorizer exposes two gRPC services:

1. **VectorizerService**: Native Vectorizer API with hybrid search and advanced features
2. **Qdrant-Compatible Services**: Full Qdrant gRPC API compatibility for drop-in replacement

### Default Ports

| Service | Port | Protocol |
|---------|------|----------|
| Vectorizer gRPC | 15003 | HTTP/2 |
| Qdrant gRPC (compatible) | 6334 | HTTP/2 |

## Vectorizer Native gRPC API

### Service Definition

```protobuf
service VectorizerService {
    // Collection management
    rpc ListCollections(ListCollectionsRequest) returns (ListCollectionsResponse);
    rpc CreateCollection(CreateCollectionRequest) returns (CreateCollectionResponse);
    rpc GetCollectionInfo(GetCollectionInfoRequest) returns (GetCollectionInfoResponse);
    rpc DeleteCollection(DeleteCollectionRequest) returns (DeleteCollectionResponse);

    // Vector operations
    rpc InsertVector(InsertVectorRequest) returns (InsertVectorResponse);
    rpc InsertVectors(stream InsertVectorRequest) returns (InsertVectorsResponse);
    rpc GetVector(GetVectorRequest) returns (GetVectorResponse);
    rpc UpdateVector(UpdateVectorRequest) returns (UpdateVectorResponse);
    rpc DeleteVector(DeleteVectorRequest) returns (DeleteVectorResponse);

    // Search operations
    rpc Search(SearchRequest) returns (SearchResponse);
    rpc BatchSearch(BatchSearchRequest) returns (BatchSearchResponse);
    rpc HybridSearch(HybridSearchRequest) returns (HybridSearchResponse);

    // Health and stats
    rpc HealthCheck(HealthCheckRequest) returns (HealthCheckResponse);
    rpc GetStats(GetStatsRequest) returns (GetStatsResponse);
}
```

### Collection Management

#### List Collections

```protobuf
message ListCollectionsRequest {}

message ListCollectionsResponse {
    repeated string collection_names = 1;
}
```

**Example (grpcurl):**

```bash
grpcurl -plaintext localhost:15003 vectorizer.VectorizerService/ListCollections
```

#### Create Collection

```protobuf
message CreateCollectionRequest {
    string name = 1;
    CollectionConfig config = 2;
}

message CollectionConfig {
    uint32 dimension = 1;
    DistanceMetric metric = 2;
    HnswConfig hnsw_config = 3;
    QuantizationConfig quantization = 4;
    StorageType storage_type = 5;
}
```

**Example:**

```bash
grpcurl -plaintext -d '{
  "name": "my_collection",
  "config": {
    "dimension": 384,
    "metric": "COSINE",
    "hnsw_config": {
      "m": 16,
      "ef_construction": 200,
      "ef": 100
    }
  }
}' localhost:15003 vectorizer.VectorizerService/CreateCollection
```

#### Get Collection Info

```protobuf
message GetCollectionInfoRequest {
    string collection_name = 1;
}

message GetCollectionInfoResponse {
    CollectionInfo info = 1;
}

message CollectionInfo {
    string name = 1;
    CollectionConfig config = 2;
    uint64 vector_count = 3;
    int64 created_at = 4;
    int64 updated_at = 5;
}
```

### Vector Operations

#### Insert Vector

```protobuf
message InsertVectorRequest {
    string collection_name = 1;
    string vector_id = 2;
    repeated float data = 3;
    map<string, string> payload = 4;
}
```

**Example:**

```bash
grpcurl -plaintext -d '{
  "collection_name": "my_collection",
  "vector_id": "vec_001",
  "data": [0.1, 0.2, 0.3, ...],
  "payload": {
    "title": "Document 1",
    "category": "tech"
  }
}' localhost:15003 vectorizer.VectorizerService/InsertVector
```

#### Streaming Insert (Batch)

The `InsertVectors` method accepts a stream of `InsertVectorRequest` messages and returns a summary:

```protobuf
message InsertVectorsResponse {
    uint32 inserted_count = 1;
    uint32 failed_count = 2;
    repeated string errors = 3;
}
```

### Search Operations

#### Basic Search

```protobuf
message SearchRequest {
    string collection_name = 1;
    repeated float query_vector = 2;
    uint32 limit = 3;
    double threshold = 4;
    map<string, string> filter = 5;
}

message SearchResponse {
    repeated SearchResult results = 1;
}

message SearchResult {
    string id = 1;
    double score = 2;
    repeated float vector = 3;
    map<string, string> payload = 4;
}
```

**Example:**

```bash
grpcurl -plaintext -d '{
  "collection_name": "my_collection",
  "query_vector": [0.1, 0.2, 0.3, ...],
  "limit": 10,
  "threshold": 0.7
}' localhost:15003 vectorizer.VectorizerService/Search
```

#### Batch Search

Execute multiple searches in a single request:

```protobuf
message BatchSearchRequest {
    string collection_name = 1;
    repeated SearchRequest queries = 2;
}

message BatchSearchResponse {
    repeated SearchResponse results = 1;
}
```

#### Hybrid Search

Combine dense and sparse vectors using RRF (Reciprocal Rank Fusion):

```protobuf
message HybridSearchRequest {
    string collection_name = 1;
    repeated float dense_query = 2;
    SparseVector sparse_query = 3;
    HybridSearchConfig config = 4;
}

message SparseVector {
    repeated uint32 indices = 1;
    repeated float values = 2;
}

message HybridSearchConfig {
    uint32 dense_k = 1;    // Top-k for dense search
    uint32 sparse_k = 2;   // Top-k for sparse search
    uint32 final_k = 3;    // Final result count
    double alpha = 4;      // Dense/sparse weight (0-1)
    HybridScoringAlgorithm algorithm = 5;
}

message HybridSearchResult {
    string id = 1;
    double hybrid_score = 2;
    double dense_score = 3;
    double sparse_score = 4;
    repeated float vector = 5;
    map<string, string> payload = 6;
}
```

**Scoring Algorithms:**

```protobuf
enum HybridScoringAlgorithm {
    RRF = 0;           // Reciprocal Rank Fusion (default)
    WEIGHTED = 1;      // Weighted sum
    ALPHA_BLEND = 2;   // Alpha blending
}
```

### Health and Stats

#### Health Check

```protobuf
message HealthCheckResponse {
    string status = 1;      // "healthy" or "unhealthy"
    string version = 2;     // Vectorizer version
    int64 timestamp = 3;    // Unix timestamp
}
```

#### Get Stats

```protobuf
message GetStatsResponse {
    uint32 collections_count = 1;
    uint64 total_vectors = 2;
    int64 uptime_seconds = 3;   // Server uptime
    string version = 4;
}
```

### Enums

```protobuf
enum DistanceMetric {
    COSINE = 0;
    EUCLIDEAN = 1;
    DOT_PRODUCT = 2;
}

enum StorageType {
    MEMORY = 0;
    MMAP = 1;
}
```

### Quantization Configuration

```protobuf
message QuantizationConfig {
    oneof config {
        ScalarQuantization scalar = 1;
        ProductQuantization product = 2;
        BinaryQuantization binary = 3;
    }
}

message ScalarQuantization {
    uint32 bits = 1;  // 4, 8, or 16
}

message ProductQuantization {
    uint32 subvectors = 1;
    uint32 centroids = 2;
}

message BinaryQuantization {}
```

## Cluster gRPC API

For distributed deployments, Vectorizer provides a cluster service for inter-node communication:

```protobuf
service ClusterService {
    // Cluster state management
    rpc GetClusterState(GetClusterStateRequest) returns (GetClusterStateResponse);
    rpc UpdateClusterState(UpdateClusterStateRequest) returns (UpdateClusterStateResponse);

    // Remote vector operations
    rpc RemoteInsertVector(RemoteInsertVectorRequest) returns (RemoteInsertVectorResponse);
    rpc RemoteUpdateVector(RemoteUpdateVectorRequest) returns (RemoteUpdateVectorResponse);
    rpc RemoteDeleteVector(RemoteDeleteVectorRequest) returns (RemoteDeleteVectorResponse);
    rpc RemoteSearchVectors(RemoteSearchVectorsRequest) returns (RemoteSearchVectorsResponse);

    // Remote collection operations
    rpc RemoteCreateCollection(RemoteCreateCollectionRequest) returns (RemoteCreateCollectionResponse);
    rpc RemoteGetCollectionInfo(RemoteGetCollectionInfoRequest) returns (RemoteGetCollectionInfoResponse);
    rpc RemoteDeleteCollection(RemoteDeleteCollectionRequest) returns (RemoteDeleteCollectionResponse);

    // Health and quota
    rpc HealthCheck(HealthCheckRequest) returns (HealthCheckResponse);
    rpc CheckQuota(CheckQuotaRequest) returns (CheckQuotaResponse);
}
```

### Multi-Tenant Support

All cluster operations support tenant context for isolation:

```protobuf
message TenantContext {
    string tenant_id = 1;          // Tenant/user ID (UUID)
    optional string username = 2;   // For logging
    repeated string permissions = 3; // read, write, admin
    optional string trace_id = 4;   // Distributed tracing
}
```

### Node Status

```protobuf
enum NodeStatus {
    ACTIVE = 0;
    JOINING = 1;
    LEAVING = 2;
    UNAVAILABLE = 3;
}

message NodeMetadata {
    optional string version = 1;
    repeated string capabilities = 2;
    uint64 vector_count = 3;
    uint64 memory_usage = 4;
    float cpu_usage = 5;
}
```

## Qdrant-Compatible gRPC API

Vectorizer implements the complete Qdrant gRPC API for drop-in compatibility. Connect Qdrant clients to port 6334.

### Supported Services

| Service | Methods |
|---------|---------|
| CollectionsService | Get, List, Create, Update, Delete, UpdateAliases, CollectionClusterInfo, CollectionExists |
| PointsService | Upsert, Delete, Get, UpdateVectors, DeleteVectors, SetPayload, OverwritePayload, DeletePayload, ClearPayload, Search, SearchBatch, SearchGroups, Scroll, Recommend, RecommendBatch, RecommendGroups, Count, Query, QueryBatch, Facet |
| SnapshotsService | Create, List, Delete, CreateFull, ListFull, DeleteFull |

### Example (Qdrant Python Client)

```python
from qdrant_client import QdrantClient

# Connect to Vectorizer's Qdrant-compatible gRPC port
client = QdrantClient(host="localhost", port=6334, prefer_grpc=True)

# Use exactly like Qdrant
client.upsert(
    collection_name="my_collection",
    points=[
        {
            "id": 1,
            "vector": [0.1, 0.2, 0.3, ...],
            "payload": {"title": "Document 1"}
        }
    ]
)

results = client.search(
    collection_name="my_collection",
    query_vector=[0.1, 0.2, 0.3, ...],
    limit=10
)
```

## Client Libraries

### Rust (tonic)

```rust
use vectorizer::vectorizer_client::VectorizerServiceClient;
use vectorizer::{SearchRequest, CreateCollectionRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = VectorizerServiceClient::connect("http://localhost:15003").await?;

    // Create collection
    let request = tonic::Request::new(CreateCollectionRequest {
        name: "my_collection".to_string(),
        config: Some(CollectionConfig {
            dimension: 384,
            metric: DistanceMetric::Cosine as i32,
            ..Default::default()
        }),
    });

    let response = client.create_collection(request).await?;
    println!("Created: {}", response.get_ref().success);

    Ok(())
}
```

### Python (grpcio)

```python
import grpc
import vectorizer_pb2
import vectorizer_pb2_grpc

channel = grpc.insecure_channel('localhost:15003')
stub = vectorizer_pb2_grpc.VectorizerServiceStub(channel)

# Search
request = vectorizer_pb2.SearchRequest(
    collection_name="my_collection",
    query_vector=[0.1, 0.2, 0.3, ...],
    limit=10,
    threshold=0.7
)

response = stub.Search(request)
for result in response.results:
    print(f"ID: {result.id}, Score: {result.score}")
```

### Go

```go
package main

import (
    "context"
    "log"

    pb "vectorizer/proto"
    "google.golang.org/grpc"
)

func main() {
    conn, err := grpc.Dial("localhost:15003", grpc.WithInsecure())
    if err != nil {
        log.Fatal(err)
    }
    defer conn.Close()

    client := pb.NewVectorizerServiceClient(conn)

    resp, err := client.Search(context.Background(), &pb.SearchRequest{
        CollectionName: "my_collection",
        QueryVector:    []float32{0.1, 0.2, 0.3, ...},
        Limit:          10,
    })

    for _, result := range resp.Results {
        log.Printf("ID: %s, Score: %f", result.Id, result.Score)
    }
}
```

## Configuration

### Enable gRPC

```yaml
# config.yml
grpc:
  enabled: true
  port: 15003
  max_message_size: 16777216  # 16MB
  reflection: true  # Enable gRPC reflection for grpcurl

qdrant:
  grpc_port: 6334
```

### TLS Configuration

```yaml
grpc:
  enabled: true
  port: 15003
  tls:
    enabled: true
    cert_path: /path/to/cert.pem
    key_path: /path/to/key.pem
    ca_path: /path/to/ca.pem  # Optional, for mTLS
```

## Performance Tips

1. **Use streaming**: For batch inserts, use `InsertVectors` streaming RPC
2. **Connection pooling**: Reuse gRPC channels across requests
3. **Compression**: Enable gRPC compression for large payloads
4. **Keep-alive**: Configure keep-alive for long-lived connections

```yaml
grpc:
  keep_alive_time: 60s
  keep_alive_timeout: 20s
  max_concurrent_streams: 100
```

## Error Handling

gRPC errors follow standard status codes:

| Code | Meaning |
|------|---------|
| `OK` (0) | Success |
| `INVALID_ARGUMENT` (3) | Invalid request parameters |
| `NOT_FOUND` (5) | Collection or vector not found |
| `ALREADY_EXISTS` (6) | Collection already exists |
| `PERMISSION_DENIED` (7) | Authentication/authorization failed |
| `RESOURCE_EXHAUSTED` (8) | Rate limit exceeded |
| `INTERNAL` (13) | Internal server error |
| `UNAVAILABLE` (14) | Service temporarily unavailable |

## Related Topics

- [REST API Reference](./API_REFERENCE.md) - HTTP REST API
- [Qdrant Compatibility](../qdrant/API_COMPATIBILITY.md) - Qdrant API compatibility
- [Cluster Configuration](../configuration/CLUSTER.md) - Distributed deployment
- [Authentication](./AUTHENTICATION.md) - Security configuration
