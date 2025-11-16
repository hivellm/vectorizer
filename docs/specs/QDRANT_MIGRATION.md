# Qdrant Compatibility & Migration Guide

## Overview

Vectorizer provides **limited Qdrant REST API compatibility** for users migrating from Qdrant. However, **we strongly recommend using Vectorizer's native APIs** for better performance, features, and long-term support.

## ⚠️ Important Notice

**Qdrant compatibility is ONLY available via REST API.**

- ✅ **REST API**: Available at `/qdrant/*` endpoints
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

| Feature | Qdrant API | Native Vectorizer | Native MCP |
|---------|------------|-------------------|------------|
| Collection CRUD | ✅ | ✅ | ✅ |
| Vector CRUD | ✅ | ✅ | ✅ |
| Basic Search | ✅ | ✅ | ✅ |
| Intelligent Search | ❌ | ✅ | ✅ |
| Semantic Search | ❌ | ✅ | ✅ |
| Multi-Collection | ❌ | ✅ | ✅ |
| Text Embedding | ❌ | ✅ | ✅ |
| File Indexing | ❌ | ✅ | ✅ |
| Query Caching | ❌ | ✅ | ✅ |
| Reranking | ❌ | ✅ | ✅ |
| Hybrid Search | ❌ | ✅ | ✅ |
| Workspace Management | ❌ | ✅ | ✅ |
| Performance | ⚠️ Slower | ✅ Optimized | ✅ Best |

## Compatibility Limitations

### Not Supported
- ❌ gRPC protocol (only REST)
- ❌ Filtering by payload conditions
- ❌ Snapshots and backups via Qdrant API
- ❌ Cluster operations
- ❌ Sharding
- ❌ Full-text search via Qdrant format
- ❌ Collection aliases
- ❌ Custom sharding keys

### Partial Support
- ⚠️ HNSW configuration (mapped to Vectorizer's HNSW)
- ⚠️ Optimizer configuration (basic support)
- ⚠️ Quantization (SQ8 only)
- ⚠️ Recommend API (basic strategies only)

### Fully Supported
- ✅ Collection management (create, read, update, delete)
- ✅ Vector operations (upsert, retrieve, delete, count)
- ✅ Basic vector search
- ✅ Scroll operations
- ✅ Batch operations

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

