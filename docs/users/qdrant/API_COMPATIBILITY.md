# Qdrant API Compatibility Matrix

Complete compatibility matrix for Qdrant REST API endpoints, parameters, responses, and errors.

## Base URL

```
http://localhost:15002/qdrant
```

## Endpoint Compatibility Matrix

### Collection Management Endpoints

| Qdrant Endpoint       | Method | Vectorizer Endpoint          | Status  | Notes                 |
| --------------------- | ------ | ---------------------------- | ------- | --------------------- |
| `/collections`        | GET    | `/qdrant/collections`        | ✅ Full | Lists all collections |
| `/collections/{name}` | GET    | `/qdrant/collections/{name}` | ✅ Full | Get collection info   |
| `/collections/{name}` | PUT    | `/qdrant/collections/{name}` | ✅ Full | Create collection     |
| `/collections/{name}` | PATCH  | `/qdrant/collections/{name}` | ✅ Full | Update collection     |
| `/collections/{name}` | DELETE | `/qdrant/collections/{name}` | ✅ Full | Delete collection     |

### Vector Operations (Points) Endpoints

| Qdrant Endpoint                     | Method | Vectorizer Endpoint                        | Status  | Notes                 |
| ----------------------------------- | ------ | ------------------------------------------ | ------- | --------------------- |
| `/collections/{name}/points`        | GET    | `/qdrant/collections/{name}/points`        | ✅ Full | Retrieve points       |
| `/collections/{name}/points`        | PUT    | `/qdrant/collections/{name}/points`        | ✅ Full | Upsert points         |
| `/collections/{name}/points/delete` | POST   | `/qdrant/collections/{name}/points/delete` | ✅ Full | Delete points         |
| `/collections/{name}/points/scroll` | POST   | `/qdrant/collections/{name}/points/scroll` | ✅ Full | Scroll through points |
| `/collections/{name}/points/count`  | POST   | `/qdrant/collections/{name}/points/count`  | ✅ Full | Count points          |

### Search Operations Endpoints

| Qdrant Endpoint                              | Method | Vectorizer Endpoint                                 | Status  | Notes            |
| -------------------------------------------- | ------ | --------------------------------------------------- | ------- | ---------------- |
| `/collections/{name}/points/search`          | POST   | `/qdrant/collections/{name}/points/search`          | ✅ Full | Search points    |
| `/collections/{name}/points/search/batch`    | POST   | `/qdrant/collections/{name}/points/search/batch`    | ✅ Full | Batch search     |
| `/collections/{name}/points/recommend`       | POST   | `/qdrant/collections/{name}/points/recommend`       | ✅ Full | Recommend points |
| `/collections/{name}/points/recommend/batch` | POST   | `/qdrant/collections/{name}/points/recommend/batch` | ✅ Full | Batch recommend  |

### Alias Management Endpoints

| Qdrant Endpoint               | Method | Vectorizer Endpoint                  | Status  | Notes                   |
| ----------------------------- | ------ | ------------------------------------ | ------- | ----------------------- |
| `/collections/aliases`        | POST   | `/qdrant/collections/aliases`        | ✅ Full | Update aliases          |
| `/collections/{name}/aliases` | GET    | `/qdrant/collections/{name}/aliases` | ✅ Full | List collection aliases |
| `/aliases`                    | GET    | `/qdrant/aliases`                    | ✅ Full | List all aliases        |

### Unsupported Endpoints

| Qdrant Endpoint                 | Status | Reason                             |
| ------------------------------- | ------ | ---------------------------------- |
| `/collections/{name}/snapshots` | ❌     | Snapshots available via native API |
| `/collections/{name}/shards`    | ❌     | Sharding not supported             |
| `/cluster`                      | ❌     | Clustering not supported           |
| `/telemetry`                    | ❌     | Use native monitoring              |

## Parameter Compatibility

### Collection Creation Parameters

| Parameter                  | Qdrant | Vectorizer                    | Status        | Notes                      |
| -------------------------- | ------ | ----------------------------- | ------------- | -------------------------- |
| `vectors.size`             | ✅     | `dimension`                   | ✅ Compatible | Same functionality         |
| `vectors.distance`         | ✅     | `metric`                      | ✅ Compatible | Cosine, Dot, Euclidean     |
| `hnsw_config.m`            | ✅     | `hnsw_config.m`               | ✅ Compatible | HNSW parameter             |
| `hnsw_config.ef_construct` | ✅     | `hnsw_config.ef_construction` | ⚠️ Renamed    | Parameter name differs     |
| `hnsw_config.ef`           | ✅     | `hnsw_config.ef_search`       | ⚠️ Renamed    | Parameter name differs     |
| `optimizers_config`        | ✅     | Partial                       | ⚠️ Basic      | Limited optimizer support  |
| `quantization_config`      | ✅     | Partial                       | ⚠️ SQ8 only   | Scalar quantization only   |
| `replication_factor`       | ✅     | ❌                            | ❌            | Replication via native API |

### Search Parameters

| Parameter         | Qdrant | Vectorizer | Status        | Notes                       |
| ----------------- | ------ | ---------- | ------------- | --------------------------- |
| `vector`          | ✅     | ✅         | ✅ Compatible | Query vector                |
| `filter`          | ✅     | ✅         | ✅ Compatible | All filter types supported  |
| `limit`           | ✅     | ✅         | ✅ Compatible | Max results                 |
| `offset`          | ✅     | ✅         | ✅ Compatible | Pagination offset           |
| `with_payload`    | ✅     | ✅         | ✅ Compatible | Include payload             |
| `with_vector`     | ✅     | ✅         | ✅ Compatible | Include vector              |
| `score_threshold` | ✅     | ✅         | ✅ Compatible | Minimum score               |
| `using`           | ✅     | ❌         | ❌            | Named vectors not supported |
| `prefetch`        | ✅     | ❌         | ❌            | Prefetch not supported      |

### Filter Parameters

| Filter Type        | Qdrant | Vectorizer | Status        | Notes              |
| ------------------ | ------ | ---------- | ------------- | ------------------ |
| `must`             | ✅     | ✅         | ✅ Compatible | AND logic          |
| `must_not`         | ✅     | ✅         | ✅ Compatible | NOT logic          |
| `should`           | ✅     | ✅         | ✅ Compatible | OR logic           |
| `match`            | ✅     | ✅         | ✅ Compatible | Exact match        |
| `range`            | ✅     | ✅         | ✅ Compatible | Range queries      |
| `geo_bounding_box` | ✅     | ✅         | ✅ Compatible | Geo bounding box   |
| `geo_radius`       | ✅     | ✅         | ✅ Compatible | Geo radius         |
| `values_count`     | ✅     | ✅         | ✅ Compatible | Array/object count |

## Response Compatibility

### Collection Info Response

| Field                          | Qdrant | Vectorizer | Status        | Notes                |
| ------------------------------ | ------ | ---------- | ------------- | -------------------- |
| `result.status`                | ✅     | ✅         | ✅ Compatible | Collection status    |
| `result.optimizer_status`      | ✅     | ✅         | ✅ Compatible | Optimizer status     |
| `result.vectors_count`         | ✅     | ✅         | ✅ Compatible | Vector count         |
| `result.indexed_vectors_count` | ✅     | ✅         | ✅ Compatible | Indexed count        |
| `result.points_count`          | ✅     | ✅         | ✅ Compatible | Points count         |
| `result.segments_count`        | ✅     | ❌         | ❌            | Segments not exposed |
| `result.config`                | ✅     | ✅         | ✅ Compatible | Collection config    |
| `status`                       | ✅     | ✅         | ✅ Compatible | Response status      |
| `time`                         | ✅     | ✅         | ✅ Compatible | Processing time      |

### Search Response

| Field              | Qdrant | Vectorizer | Status        | Notes                      |
| ------------------ | ------ | ---------- | ------------- | -------------------------- |
| `result`           | ✅     | ✅         | ✅ Compatible | Search results array       |
| `result[].id`      | ✅     | ✅         | ✅ Compatible | Point ID                   |
| `result[].score`   | ✅     | ✅         | ✅ Compatible | Similarity score           |
| `result[].payload` | ✅     | ✅         | ✅ Compatible | Payload data               |
| `result[].vector`  | ✅     | ✅         | ✅ Compatible | Vector data (if requested) |
| `status`           | ✅     | ✅         | ✅ Compatible | Response status            |
| `time`             | ✅     | ✅         | ✅ Compatible | Processing time            |

## Error Compatibility

### Error Response Format

| Field          | Qdrant | Vectorizer | Status        | Notes         |
| -------------- | ------ | ---------- | ------------- | ------------- |
| `status.error` | ✅     | ✅         | ✅ Compatible | Error message |
| `status.code`  | ✅     | ✅         | ✅ Compatible | Error code    |

### Error Codes

| Error Code | Qdrant | Vectorizer | Status        | Notes                 |
| ---------- | ------ | ---------- | ------------- | --------------------- |
| `400`      | ✅     | ✅         | ✅ Compatible | Bad Request           |
| `404`      | ✅     | ✅         | ✅ Compatible | Not Found             |
| `409`      | ✅     | ✅         | ✅ Compatible | Conflict              |
| `500`      | ✅     | ✅         | ✅ Compatible | Internal Server Error |

### Common Error Messages

| Error                     | Qdrant | Vectorizer | Status        | Notes        |
| ------------------------- | ------ | ---------- | ------------- | ------------ |
| Collection not found      | ✅     | ✅         | ✅ Compatible | Same message |
| Invalid vector dimension  | ✅     | ✅         | ✅ Compatible | Same message |
| Collection already exists | ✅     | ✅         | ✅ Compatible | Same message |
| Invalid filter            | ✅     | ✅         | ✅ Compatible | Same message |

## Version Compatibility

| Qdrant Version | Vectorizer Version | Compatibility | Notes                     |
| -------------- | ------------------ | ------------- | ------------------------- |
| v1.14.x        | v1.3.0+            | ✅ Full       | REST API fully compatible |
| v1.13.x        | v1.3.0+            | ✅ Full       | Backward compatible       |
| v1.12.x        | v1.3.0+            | ⚠️ Partial    | Some features may differ  |
| v1.11.x        | v1.3.0+            | ⚠️ Partial    | Older API versions        |

## HTTP Status Codes

| Status Code                 | Qdrant | Vectorizer | Status        | Notes              |
| --------------------------- | ------ | ---------- | ------------- | ------------------ |
| `200 OK`                    | ✅     | ✅         | ✅ Compatible | Success            |
| `201 Created`               | ✅     | ✅         | ✅ Compatible | Created            |
| `400 Bad Request`           | ✅     | ✅         | ✅ Compatible | Invalid request    |
| `404 Not Found`             | ✅     | ✅         | ✅ Compatible | Resource not found |
| `409 Conflict`              | ✅     | ✅         | ✅ Compatible | Conflict           |
| `500 Internal Server Error` | ✅     | ✅         | ✅ Compatible | Server error       |

## Request/Response Format

### Request Format

All requests use JSON format matching Qdrant API:

```json
{
  "vector": [0.1, 0.2, ...],
  "filter": {
    "must": [...]
  },
  "limit": 10
}
```

### Response Format

All responses use Qdrant-compatible format:

```json
{
  "result": {
    ...
  },
  "status": "ok",
  "time": 0.001
}
```

## Compatibility Notes

### Fully Compatible

- ✅ All REST endpoints
- ✅ Request/response formats
- ✅ Error handling
- ✅ Filter system (all types)
- ✅ Batch operations
- ✅ Alias management

### Partially Compatible

- ⚠️ HNSW configuration (parameter names differ)
- ⚠️ Optimizer configuration (basic support)
- ⚠️ Quantization (SQ8 only)

### Not Compatible

- ❌ gRPC protocol
- ❌ Sharding endpoints
- ❌ Cluster management
- ❌ Named vectors (`using` parameter)
- ❌ Prefetch operations

## Migration Notes

When migrating from Qdrant:

1. **Change base URL**: `http://qdrant:6333` → `http://vectorizer:15002/qdrant`
2. **Update parameter names**: `ef_construct` → `ef_construction`, `ef` → `ef_search`
3. **Remove unsupported features**: gRPC, sharding, clustering
4. **Use native APIs**: For better performance and features

See [Migration Guide](../specs/QDRANT_MIGRATION.md) for detailed migration steps.
