# Qdrant Compatibility & Migration Guide

## Overview

Vectorizer provides **Qdrant API compatibility** for users migrating from Qdrant. Both REST and gRPC protocols are supported. However, **we strongly recommend using Vectorizer's native APIs** for better performance, features, and long-term support.

## ⚠️ Important Notice

**Qdrant compatibility is available via REST API and gRPC.**

- ✅ **REST API**: Available at `/qdrant/*` endpoints
- ✅ **gRPC API**: Available on port `<REST_PORT + 1>` (e.g., if REST is on 7777, gRPC is on 7778)
- ❌ **MCP Protocol**: Qdrant tools removed (use native Vectorizer MCP tools instead)
- ⚠️ **Compatibility Layer**: Provided for migration purposes only
- 🎯 **Recommendation**: Migrate to native Vectorizer APIs

## Qdrant REST API Compatibility

### Available Endpoints

#### Collection Management
```
GET    /qdrant/collections              - List all collections
GET    /qdrant/collections/{name}       - Get collection info
POST   /qdrant/collections/{name}       - Create collection
PUT    /qdrant/collections/{name}       - Update collection
DELETE /qdrant/collections/{name}       - Delete collection
```

#### Vector Operations (Points)
```
POST   /qdrant/collections/{name}/points        - Upsert points
GET    /qdrant/collections/{name}/points        - Retrieve points
DELETE /qdrant/collections/{name}/points        - Delete points
POST   /qdrant/collections/{name}/points/scroll - Scroll through points
POST   /qdrant/collections/{name}/points/count  - Count points
```

#### Search Operations
```
POST   /qdrant/collections/{name}/points/search            - Search points
POST   /qdrant/collections/{name}/points/recommend         - Recommend points
POST   /qdrant/collections/{name}/points/search/batch      - Batch search
POST   /qdrant/collections/{name}/points/recommend/batch   - Batch recommend
POST   /qdrant/collections/{name}/points/search/groups     - Search with grouping
POST   /qdrant/collections/{name}/points/search/matrix/pairs   - Similarity matrix (pairs)
POST   /qdrant/collections/{name}/points/search/matrix/offsets - Similarity matrix (offsets)
```

#### Query API
```
POST   /qdrant/collections/{name}/points/query        - Query points (universal search)
POST   /qdrant/collections/{name}/points/query/batch  - Batch query
POST   /qdrant/collections/{name}/points/query/groups - Query with grouping
```

#### Snapshots
```
GET    /qdrant/collections/{name}/snapshots           - List collection snapshots
POST   /qdrant/collections/{name}/snapshots           - Create collection snapshot
DELETE /qdrant/collections/{name}/snapshots/{snap}    - Delete collection snapshot
POST   /qdrant/collections/{name}/snapshots/recover   - Recover from snapshot
GET    /qdrant/snapshots                              - List all snapshots
POST   /qdrant/snapshots                              - Create full snapshot
```

#### Sharding
```
GET    /qdrant/collections/{name}/shards              - List shard keys
PUT    /qdrant/collections/{name}/shards              - Create shard key
POST   /qdrant/collections/{name}/shards/delete       - Delete shard key
```

#### Cluster Management
```
GET    /qdrant/cluster                        - Get cluster status
POST   /qdrant/cluster/recover                - Recover current peer
DELETE /qdrant/cluster/peer/{peer_id}         - Remove peer from cluster
GET    /qdrant/cluster/metadata/keys          - List metadata keys
GET    /qdrant/cluster/metadata/keys/{key}    - Get metadata key value
PUT    /qdrant/cluster/metadata/keys/{key}    - Update metadata key value
```

### Example Usage

#### Create Collection (Qdrant Format)
```bash
curl -X POST http://localhost:15002/qdrant/collections/my_collection \
  -H "Content-Type: application/json" \
  -d '{
    "vectors": {
      "size": 384,
      "distance": "Cosine"
    },
    "hnsw_config": {
      "m": 16,
      "ef_construct": 100
    }
  }'
```

#### Upsert Points (Qdrant Format)
```bash
curl -X POST http://localhost:15002/qdrant/collections/my_collection/points \
  -H "Content-Type: application/json" \
  -d '{
    "points": [
      {
        "id": "1",
        "vector": [0.1, 0.2, ...],
        "payload": {"text": "example"}
      }
    ]
  }'
```

#### Search Points (Qdrant Format)
```bash
curl -X POST http://localhost:15002/qdrant/collections/my_collection/points/search \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [0.1, 0.2, ...],
    "limit": 10,
    "with_payload": true
  }'
```

## Migration to Native Vectorizer API

### Why Migrate?

**Native Vectorizer APIs offer significant advantages:**

#### 🚀 Performance
- **Faster**: Optimized for Vectorizer's architecture
- **Efficient**: Lower overhead without compatibility layer
- **Caching**: Advanced query result caching (10-100x speedup)

#### 🎯 Features
- **Intelligent Search**: AI-powered search with query expansion
- **Semantic Search**: Cross-encoder reranking
- **Multi-Collection**: Search across multiple collections
- **File Operations**: Direct file indexing and search
- **Hybrid Search**: BM25 + vector search combination

#### 🔧 Better Integration
- **MCP Protocol**: Full support for Model Context Protocol
- **UMICP**: Universal Model Interaction & Control Protocol
- **Streaming**: WebSocket support for real-time updates
- **Monitoring**: Built-in metrics and observability

### Migration Examples

#### Collection Creation

**Qdrant API:**
```bash
POST /qdrant/collections/{name}
{
  "vectors": {"size": 384, "distance": "Cosine"}
}
```

**Native Vectorizer API:**
```bash
POST /collections
{
  "name": "my_collection",
  "dimension": 384,
  "metric": "Cosine"
}
```

**Native Vectorizer MCP:**
```javascript
await mcp.call_tool('create_collection', {
  name: 'my_collection',
  dimension: 384,
  metric: 'Cosine'
});
```

#### Vector Insert

**Qdrant API:**
```bash
POST /qdrant/collections/{name}/points
{
  "points": [
    {"id": "1", "vector": [...], "payload": {...}}
  ]
}
```

**Native Vectorizer API:**
```bash
POST /insert
{
  "collection": "my_collection",
  "vectors": [
    {"id": "1", "data": [...], "payload": {...}}
  ]
}
```

**Native Vectorizer MCP:**
```javascript
await mcp.call_tool('insert_text', {
  collection: 'my_collection',
  text: 'Your content here',
  metadata: {...}
});
```

#### Vector Search

**Qdrant API:**
```bash
POST /qdrant/collections/{name}/points/search
{
  "vector": [...],
  "limit": 10
}
```

**Native Vectorizer API:**
```bash
POST /search
{
  "collection": "my_collection",
  "query": [...],
  "limit": 10
}
```

**Native Vectorizer MCP (Intelligent Search):**
```javascript
await mcp.call_tool('search_intelligent', {
  query: 'search by text',
  collections: ['my_collection'],
  max_results: 10
});
```

## Feature Comparison

| Feature | Qdrant REST | Qdrant gRPC | Native Vectorizer | Native MCP |
|---------|-------------|-------------|-------------------|------------|
| Collection CRUD | ✅ | ✅ | ✅ | ✅ |
| Vector CRUD | ✅ | ✅ | ✅ | ✅ |
| Basic Search | ✅ | ✅ | ✅ | ✅ |
| Query API | ✅ | ✅ | ✅ | ✅ |
| Search Groups | ✅ | ✅ | ✅ | ✅ |
| Search Matrix | ✅ | ✅ | ✅ | ✅ |
| Snapshots | ✅ | ✅ | ✅ | ✅ |
| Sharding API | ✅ | ✅ | ✅ | ✅ |
| Cluster API | ✅ | ⚠️ | ✅ | ✅ |
| Quantization | ✅ | ✅ | ✅ | ✅ |
| Intelligent Search | ❌ | ❌ | ✅ | ✅ |
| Semantic Search | ❌ | ❌ | ✅ | ✅ |
| Multi-Collection | ❌ | ❌ | ✅ | ✅ |
| Text Embedding | ❌ | ❌ | ✅ | ✅ |
| File Indexing | ❌ | ❌ | ✅ | ✅ |
| Query Caching | ❌ | ❌ | ✅ | ✅ |
| Reranking | ❌ | ❌ | ✅ | ✅ |
| Hybrid Search | ❌ | ❌ | ✅ | ✅ |
| Workspace Management | ❌ | ❌ | ✅ | ✅ |
| Performance | ⚠️ Slower | ⚠️ Better | ✅ Optimized | ✅ Best |

## Qdrant gRPC API Compatibility

Vectorizer now supports Qdrant-compatible gRPC API on port `REST_PORT + 1`. This enables migration from Qdrant clients using gRPC protocol.

### Available gRPC Services

#### Collections Service (`qdrant.Collections`)
```protobuf
rpc Get(GetCollectionInfoRequest) returns (GetCollectionInfoResponse)
rpc List(ListCollectionsRequest) returns (ListCollectionsResponse)
rpc Create(CreateCollection) returns (CollectionOperationResponse)
rpc Update(UpdateCollection) returns (CollectionOperationResponse)
rpc Delete(DeleteCollection) returns (CollectionOperationResponse)
rpc UpdateAliases(ChangeAliases) returns (CollectionOperationResponse)
rpc ListCollectionAliases(ListCollectionAliasesRequest) returns (ListAliasesResponse)
rpc ListAliases(ListAliasesRequest) returns (ListAliasesResponse)
rpc CollectionClusterInfo(CollectionClusterInfoRequest) returns (CollectionClusterInfoResponse)
rpc CollectionExists(CollectionExistsRequest) returns (CollectionExistsResponse)
rpc UpdateCollectionClusterSetup(UpdateCollectionClusterSetupRequest) returns (UpdateCollectionClusterSetupResponse)
rpc CreateShardKey(CreateShardKeyRequest) returns (CreateShardKeyResponse)
rpc DeleteShardKey(DeleteShardKeyRequest) returns (DeleteShardKeyResponse)
```

#### Points Service (`qdrant.Points`)
```protobuf
rpc Upsert(UpsertPoints) returns (PointsOperationResponse)
rpc Delete(DeletePoints) returns (PointsOperationResponse)
rpc Get(GetPoints) returns (GetResponse)
rpc UpdateVectors(UpdatePointVectors) returns (PointsOperationResponse)
rpc DeleteVectors(DeletePointVectors) returns (PointsOperationResponse)
rpc SetPayload(SetPayloadPoints) returns (PointsOperationResponse)
rpc OverwritePayload(SetPayloadPoints) returns (PointsOperationResponse)
rpc DeletePayload(DeletePayloadPoints) returns (PointsOperationResponse)
rpc ClearPayload(ClearPayloadPoints) returns (PointsOperationResponse)
rpc CreateFieldIndex(CreateFieldIndexCollection) returns (PointsOperationResponse)
rpc DeleteFieldIndex(DeleteFieldIndexCollection) returns (PointsOperationResponse)
rpc Search(SearchPoints) returns (SearchResponse)
rpc SearchBatch(SearchBatchPoints) returns (SearchBatchResponse)
rpc SearchGroups(SearchPointGroups) returns (SearchGroupsResponse)
rpc Scroll(ScrollPoints) returns (ScrollResponse)
rpc Recommend(RecommendPoints) returns (RecommendResponse)
rpc RecommendBatch(RecommendBatchPoints) returns (RecommendBatchResponse)
rpc RecommendGroups(RecommendPointGroups) returns (RecommendGroupsResponse)
rpc Discover(DiscoverPoints) returns (DiscoverResponse)
rpc DiscoverBatch(DiscoverBatchPoints) returns (DiscoverBatchResponse)
rpc Count(CountPoints) returns (CountResponse)
rpc UpdateBatch(UpdateBatchPoints) returns (UpdateBatchResponse)
rpc Query(QueryPoints) returns (QueryResponse)
rpc QueryBatch(QueryBatchPoints) returns (QueryBatchResponse)
rpc QueryGroups(QueryPointGroups) returns (QueryGroupsResponse)
rpc Facet(FacetCounts) returns (FacetResponse)
rpc SearchMatrixPairs(SearchMatrixPoints) returns (SearchMatrixPairsResponse)
rpc SearchMatrixOffsets(SearchMatrixPoints) returns (SearchMatrixOffsetsResponse)
```

#### Snapshots Service (`qdrant.Snapshots`)
```protobuf
rpc Create(CreateSnapshotRequest) returns (CreateSnapshotResponse)
rpc List(ListSnapshotsRequest) returns (ListSnapshotsResponse)
rpc Delete(DeleteSnapshotRequest) returns (DeleteSnapshotResponse)
rpc CreateFull(CreateFullSnapshotRequest) returns (CreateSnapshotResponse)
rpc ListFull(ListFullSnapshotsRequest) returns (ListSnapshotsResponse)
rpc DeleteFull(DeleteFullSnapshotRequest) returns (DeleteSnapshotResponse)
```

### Example gRPC Usage

#### Python (qdrant-client)
```python
from qdrant_client import QdrantClient

# Connect to Vectorizer's Qdrant-compatible gRPC
client = QdrantClient(host="localhost", port=7778, prefer_grpc=True)

# Create collection
client.create_collection(
    collection_name="my_collection",
    vectors_config={"size": 128, "distance": "Cosine"}
)

# Insert points
client.upsert(
    collection_name="my_collection",
    points=[
        {"id": 1, "vector": [0.1] * 128, "payload": {"text": "hello"}},
        {"id": 2, "vector": [0.2] * 128, "payload": {"text": "world"}}
    ]
)

# Search
results = client.search(
    collection_name="my_collection",
    query_vector=[0.1] * 128,
    limit=10
)
```

#### Rust (qdrant-client)
```rust
use qdrant_client::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Vectorizer's Qdrant-compatible gRPC
    let client = QdrantClient::new(Some(QdrantClientConfig::from_url(
        "http://localhost:7778"
    )))?;

    // Create collection
    client.create_collection(&CreateCollection {
        collection_name: "my_collection".into(),
        vectors_config: Some(VectorsConfig {
            config: Some(vectors_config::Config::Params(VectorParams {
                size: 128,
                distance: Distance::Cosine.into(),
                ..Default::default()
            })),
        }),
        ..Default::default()
    }).await?;

    // Search
    let results = client.search_points(&SearchPoints {
        collection_name: "my_collection".into(),
        vector: vec![0.1; 128],
        limit: 10,
        ..Default::default()
    }).await?;

    Ok(())
}
```

## Compatibility Limitations

### Not Supported
- ❌ Collection aliases (API exists but not implemented)
- ❌ Named vectors storage (API accepts but stores single vector)
- ❌ Snapshot upload (download and recover supported)

### Partial Support
- ⚠️ HNSW configuration (mapped to Vectorizer's HNSW)
- ⚠️ Optimizer configuration (basic support)
- ⚠️ Quantization (Scalar int8, Product, Binary supported)
- ⚠️ Recommend API (basic strategies only)
- ⚠️ Filtering by payload conditions (basic support)
- ⚠️ Cluster operations (simulated for single-node)
- ⚠️ Sharding (API compatible, logical sharding)

### Fully Supported
- ✅ Collection management (create, read, update, delete) - REST + gRPC
- ✅ Vector operations (upsert, retrieve, delete, count) - REST + gRPC
- ✅ Basic vector search - REST + gRPC
- ✅ Scroll operations - REST + gRPC
- ✅ Batch operations - REST + gRPC
- ✅ Query API (query, batch, groups with prefetch) - REST + gRPC
- ✅ Search groups (group results by payload field) - REST + gRPC
- ✅ Search matrix (pairs and offsets format) - REST + gRPC
- ✅ Snapshots (list, create, delete, recover) - REST + gRPC
- ✅ Sharding API (create, delete, list shard keys) - REST + gRPC
- ✅ Cluster management API (status, recover, metadata) - REST only

## Migration Tools

Vectorizer provides migration tools to help you migrate from Qdrant:

### Configuration Migration

Parse and convert Qdrant configuration files:

```rust
use vectorizer::migration::qdrant::{QdrantConfigParser, ConfigFormat};

// Parse Qdrant config file
let qdrant_config = QdrantConfigParser::parse_file("qdrant_config.yaml")?;

// Validate configuration
let validation = QdrantConfigParser::validate(&qdrant_config)?;
if !validation.is_valid {
    eprintln!("Validation errors: {:?}", validation.errors);
}

// Convert to Vectorizer format
let vectorizer_configs = QdrantConfigParser::convert_to_vectorizer(&qdrant_config)?;

// Create collections in Vectorizer
for (name, config) in vectorizer_configs {
    store.create_collection(&name, config)?;
}
```

### Data Migration

Export data from Qdrant and import into Vectorizer:

```rust
use vectorizer::migration::qdrant::{QdrantDataExporter, QdrantDataImporter};

// Export collection from Qdrant
let exported = QdrantDataExporter::export_collection(
    "http://localhost:6333",
    "my_collection"
).await?;

// Save to file for backup
QdrantDataExporter::export_to_file(&exported, "backup.json")?;

// Import into Vectorizer
let result = QdrantDataImporter::import_collection(&store, &exported).await?;
println!("Imported {} points", result.imported_count);
```

### Migration Validation

Validate exported data before importing:

```rust
use vectorizer::migration::qdrant::MigrationValidator;

// Validate export
let validation = MigrationValidator::validate_export(&exported)?;
if !validation.is_valid {
    eprintln!("Validation errors: {:?}", validation.errors);
}

// Check compatibility
let compatibility = MigrationValidator::validate_compatibility(&exported);
if !compatibility.is_compatible {
    eprintln!("Incompatible features: {:?}", compatibility.incompatible_features);
}

// Validate integrity after import
let integrity = MigrationValidator::validate_integrity(&exported, result.imported_count)?;
println!("Integrity: {:.2}%", integrity.integrity_percentage);
```

## Migration Path

### Phase 1: Assessment (Week 1)
1. Identify which Qdrant features you use
2. Check compatibility table above
3. Review native Vectorizer alternatives
4. Use migration tools to validate your data
5. Plan migration timeline

### Phase 2: Dual Mode (Weeks 2-4)
1. Keep existing Qdrant API calls working via `/qdrant/*`
2. Export your collections using migration tools
3. Import into Vectorizer for testing
4. Start using native APIs for new features
5. Test native APIs in parallel
6. Compare performance and results

### Phase 3: Migration (Weeks 5-8)
1. Use migration tools to export all collections
2. Import into Vectorizer
3. Validate data integrity
4. Replace Qdrant API calls with native equivalents
5. Update client code to use MCP or native REST
6. Test thoroughly
7. Monitor performance improvements

### Phase 4: Completion (Week 9+)
1. Remove Qdrant API dependency
2. Use only native Vectorizer APIs
3. Enjoy improved performance and features

## Support

For help with migration:
- 📚 **Qdrant Compatibility Docs**: `/docs/users/qdrant/` - Complete compatibility documentation
- 🔍 **API Compatibility Matrix**: `/docs/users/qdrant/API_COMPATIBILITY.md` - Detailed compatibility matrix
- 📊 **Feature Parity**: `/docs/users/qdrant/FEATURE_PARITY.md` - Feature comparison
- 🛠️ **Troubleshooting**: `/docs/users/qdrant/TROUBLESHOOTING.md` - Common issues and solutions
- 💻 **Examples**: `/docs/users/qdrant/EXAMPLES.md` - Code examples and tutorials
- 🔍 **MCP Tools**: See `/docs/specs/MCP.md`
- 🚀 **Native API**: See `/docs/specs/SPECIFICATIONS_INDEX.md`
- 💬 **Issues**: https://github.com/hivellm/vectorizer/issues

## Deprecation Timeline

- **v1.2.x**: Qdrant REST API available for compatibility
- **v1.3.x**: Qdrant API marked as deprecated (still working)
- **v2.0.0**: Qdrant API may be removed (native API only)

**Start migrating now to avoid breaking changes in v2.0!**

