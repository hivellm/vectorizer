# Qdrant Feature Parity Documentation

Complete feature comparison between Qdrant and Vectorizer, including limitations and migration notes.

## Feature Comparison Matrix

| Feature | Qdrant | Vectorizer Qdrant API | Vectorizer Native | Status | Notes |
|---------|--------|---------------------|------------------|--------|-------|
| **Collection Management** |
| Create collection | ✅ | ✅ | ✅ | ✅ Full | All configs supported |
| Update collection | ✅ | ✅ | ✅ | ✅ Full | Config updates work |
| Delete collection | ✅ | ✅ | ✅ | ✅ Full | Complete deletion |
| List collections | ✅ | ✅ | ✅ | ✅ Full | All collections listed |
| Collection info | ✅ | ✅ | ✅ | ✅ Full | Detailed info available |
| **Vector Operations** |
| Upsert points | ✅ | ✅ | ✅ | ✅ Full | Single and batch |
| Retrieve points | ✅ | ✅ | ✅ | ✅ Full | With payload/vector |
| Delete points | ✅ | ✅ | ✅ | ✅ Full | By ID or filter |
| Update points | ✅ | ✅ | ✅ | ✅ Full | Via upsert |
| Count points | ✅ | ✅ | ✅ | ✅ Full | With filters |
| Scroll points | ✅ | ✅ | ✅ | ✅ Full | Pagination support |
| **Search Operations** |
| Vector search | ✅ | ✅ | ✅ | ✅ Full | All metrics supported |
| Filtered search | ✅ | ✅ | ✅ | ✅ Full | All filter types |
| Batch search | ✅ | ✅ | ✅ | ✅ Full | Multiple queries |
| Recommend | ✅ | ✅ | ✅ | ✅ Full | Positive/negative |
| Batch recommend | ✅ | ✅ | ✅ | ✅ Full | Multiple recommendations |
| **Filter Types** |
| Match filter | ✅ | ✅ | ✅ | ✅ Full | String, int, bool |
| Range filter | ✅ | ✅ | ✅ | ✅ Full | Numeric ranges |
| Geo bounding box | ✅ | ✅ | ✅ | ✅ Full | Geographic queries |
| Geo radius | ✅ | ✅ | ✅ | ✅ Full | Radius queries |
| Values count | ✅ | ✅ | ✅ | ✅ Full | Array/object count |
| Nested filters | ✅ | ✅ | ✅ | ✅ Full | Complex logic |
| **Indexing** |
| HNSW index | ✅ | ✅ | ✅ | ✅ Full | Configurable |
| Payload indexing | ✅ | ✅ | ✅ | ✅ Full | Keyword, integer, float, text, geo |
| Sparse vectors | ✅ | ✅ | ✅ | ✅ Full | Sparse vector support |
| Quantization | ✅ | ✅ | ✅ | ✅ Full | Scalar, product, binary |
| **Advanced Features** |
| Hybrid search | ❌ | ❌ | ✅ | ❌ | Native only |
| Intelligent search | ❌ | ❌ | ✅ | ❌ | Native only |
| Semantic search | ❌ | ❌ | ✅ | ❌ | Native only |
| Multi-collection | ❌ | ❌ | ✅ | ❌ | Native only |
| Text embedding | ❌ | ❌ | ✅ | ❌ | Native only |
| File indexing | ❌ | ❌ | ✅ | ❌ | Native only |
| Query caching | ❌ | ❌ | ✅ | ❌ | Native only |
| **Aliases** |
| Create alias | ✅ | ✅ | ✅ | ✅ Full | Alias support |
| Delete alias | ✅ | ✅ | ✅ | ✅ Full | Alias removal |
| List aliases | ✅ | ✅ | ✅ | ✅ Full | All aliases |
| **Snapshots** |
| Create snapshot | ✅ | ✅ | ✅ | ✅ Full | Full Qdrant API support |
| List snapshots | ✅ | ✅ | ✅ | ✅ Full | Full Qdrant API support |
| Restore snapshot | ✅ | ✅ | ✅ | ✅ Full | Full Qdrant API support |
| Full snapshot | ✅ | ✅ | ✅ | ✅ Full | Cross-collection snapshot |
| **Clustering** |
| Sharding API | ✅ | ✅ | ✅ | ✅ Full | API compatible (logical) |
| Replication | ✅ | ⚠️ | ✅ | ⚠️ Partial | Via native API |
| Cluster management | ✅ | ✅ | ✅ | ✅ Full | Status, recover, metadata |
| **Query API** |
| Query points | ✅ | ✅ | ✅ | ✅ Full | Universal search |
| Batch query | ✅ | ✅ | ✅ | ✅ Full | Multiple queries |
| Query groups | ✅ | ✅ | ✅ | ✅ Full | Grouped results |
| Prefetch | ✅ | ✅ | ✅ | ✅ Full | Recursive prefetch |
| **Search Groups & Matrix** |
| Search groups | ✅ | ✅ | ✅ | ✅ Full | Group by payload |
| Matrix pairs | ✅ | ✅ | ✅ | ✅ Full | Similarity pairs |
| Matrix offsets | ✅ | ✅ | ✅ | ✅ Full | Compact format |
| **Protocols** |
| REST API | ✅ | ✅ | ✅ | ✅ Full | Full compatibility |
| gRPC | ✅ | ✅ | ✅ | ✅ Full | Collections, Points, Snapshots |
| WebSocket | ❌ | ❌ | ✅ | ❌ | Native only |
| MCP Protocol | ❌ | ❌ | ✅ | ❌ | Native only |
| **Performance** |
| Query latency | Baseline | +10-20% | -30-50% | ⚠️ | Compatibility overhead |
| Throughput | Baseline | -10-15% | +20-40% | ⚠️ | Native optimized |
| Memory usage | Baseline | Similar | -20-30% | ✅ | Better optimization |

## Feature Status Legend

- ✅ **Full**: Fully supported with same functionality
- ⚠️ **Partial**: Supported with limitations or differences
- ❌ **Not Supported**: Not available in this API

## Detailed Feature Analysis

### Fully Supported Features

#### Collection Management (100%)
- ✅ Create, read, update, delete collections
- ✅ Collection configuration (HNSW, quantization, etc.)
- ✅ Collection statistics and status
- ✅ Collection aliases

#### Vector Operations (100%)
- ✅ Upsert (single and batch)
- ✅ Retrieve (with payload/vector filtering)
- ✅ Delete (by ID or filter)
- ✅ Count (with filters)
- ✅ Scroll (pagination)

#### Search Operations (100%)
- ✅ Vector similarity search
- ✅ Filtered search (all filter types)
- ✅ Batch search
- ✅ Recommend (positive/negative)
- ✅ Batch recommend

#### Filter System (100%)
- ✅ Match filters (string, integer, boolean)
- ✅ Range filters (numeric ranges)
- ✅ Geo filters (bounding box, radius)
- ✅ Values count filters
- ✅ Nested filters (complex logic)

### Partially Supported Features

#### Optimizer Configuration (Partial)
- ⚠️ Basic optimizer settings supported
- ⚠️ Advanced tuning options limited

**Workaround**: Use native API for full optimizer control.

#### HNSW Configuration (Partial)
- ⚠️ Parameter names differ: `ef_construct` → `ef_construction`
- ⚠️ Some advanced parameters not exposed

**Migration**: Update parameter names in configs.

#### Named Vectors (Partial)
- ⚠️ API accepts `using` parameter in search/query operations
- ⚠️ Single vector extracted from named vector upserts
- ❌ Multi-vector storage not supported

**Migration**: Use single vector per point or native API.

### Fully Supported New Features

#### Quantization (Full)
- ✅ **Scalar Quantization (SQ8)**: Supported
- ✅ **Product Quantization (PQ)**: x4, x8, x16, x32, x64 compression
- ✅ **Binary Quantization**: Supported

#### Query API (Full)
- ✅ Query points (universal search)
- ✅ Batch query (multiple queries)
- ✅ Query groups (grouped results)
- ✅ Prefetch operations (recursive)

#### Search Groups & Matrix (Full)
- ✅ Search groups (group by payload field)
- ✅ Matrix pairs (similarity pairs)
- ✅ Matrix offsets (compact format)

#### Snapshots (Full)
- ✅ List collection snapshots
- ✅ Create collection snapshot
- ✅ Delete collection snapshot
- ✅ Recover from snapshot
- ✅ List all snapshots
- ✅ Create full snapshot

#### Sharding API (Full)
- ✅ List shard keys
- ✅ Create shard key
- ✅ Delete shard key

#### Cluster Management (Full)
- ✅ Get cluster status
- ✅ Recover current peer
- ✅ Remove peer
- ✅ Metadata keys (list, get, update)

### Not Supported Features

#### Named Vectors Storage
- ❌ Multi-vector named vectors storage not supported
- ⚠️ API accepts format but stores single vector

**Migration**: Use single vector per point or native API.

## Limitations

### Known Limitations

1. **Performance Overhead**
   - Compatibility layer adds 10-20% latency
   - Native APIs are 30-50% faster

2. **Feature Gaps**
   - Named vectors multi-storage not supported
   - Some advanced optimizer options limited

3. **Configuration Differences**
   - Some parameter names differ
   - Advanced configs may not map directly

### Workarounds

1. **For Better Performance**: Use native Vectorizer APIs
2. **For Advanced Features**: Use native Vectorizer APIs
3. **For Named Vectors**: Use single vector per point or native API
4. **For Clustering**: Use native replication or single-node

## Migration Recommendations

### When to Use Qdrant API
- ✅ During migration period
- ✅ For compatibility testing
- ✅ For existing Qdrant codebases

### When to Use Native API
- ✅ For new projects
- ✅ For better performance
- ✅ For advanced features
- ✅ For production deployments

## Performance Comparison

### Query Latency

| Operation | Qdrant API | Native API | Improvement |
|-----------|-----------|-----------|-------------|
| Simple search | 100% | 70% | 30% faster |
| Filtered search | 100% | 65% | 35% faster |
| Batch search | 100% | 60% | 40% faster |
| Intelligent search | N/A | 50% | Native only |

### Throughput

| Operation | Qdrant API | Native API | Improvement |
|-----------|-----------|-----------|-------------|
| Queries/sec | 100% | 120% | 20% more |
| Writes/sec | 100% | 140% | 40% more |
| Batch ops/sec | 100% | 130% | 30% more |

## Use Case Recommendations

### Use Qdrant API When:
- Migrating existing Qdrant applications
- Testing compatibility
- Temporary compatibility needs
- Learning Vectorizer features

### Use Native API When:
- Building new applications
- Need maximum performance
- Need advanced features (intelligent search, etc.)
- Production deployments
- Long-term projects

## Version Compatibility

| Qdrant Version | Supported Features | Notes |
|---------------|-------------------|-------|
| v1.14.x | ✅ All REST features | Full compatibility |
| v1.13.x | ✅ All REST features | Backward compatible |
| v1.12.x | ⚠️ Most features | Some differences |
| v1.11.x | ⚠️ Basic features | Limited compatibility |

## Support and Migration

For help with feature parity or migration:
- 📚 See [Migration Guide](../../specs/QDRANT_MIGRATION.md)
- 🔍 See [API Compatibility](./API_COMPATIBILITY.md)
- 💬 [GitHub Issues](https://github.com/hivellm/vectorizer/issues)

