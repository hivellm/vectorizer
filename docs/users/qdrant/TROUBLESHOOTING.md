# Qdrant Compatibility Troubleshooting Guide

Common issues, solutions, and debugging tips for Qdrant compatibility.

## Table of Contents

1. [Common Issues](#common-issues)
2. [Error Resolution](#error-resolution)
3. [Performance Tuning](#performance-tuning)
4. [Debugging Guide](#debugging-guide)
5. [FAQ](#faq)

## Common Issues

### Collection Not Found

**Symptoms**:

- Error: `Collection not found`
- HTTP 404 status

**Diagnosis**:

```bash
# Check if collection exists
curl http://localhost:15002/qdrant/collections

# Check collection name spelling
curl http://localhost:15002/qdrant/collections/{name}
```

**Solutions**:

1. Verify collection name is correct
2. Check if collection was created via Qdrant API (not native API)
3. List all collections to see available names
4. Create collection if it doesn't exist

**Prevention**:

- Use consistent naming conventions
- Document collection names
- Use collection aliases for flexibility

### Invalid Vector Dimension

**Symptoms**:

- Error: `Invalid vector dimension`
- HTTP 400 status
- Dimension mismatch errors

**Diagnosis**:

```bash
# Check collection dimension
curl http://localhost:15002/qdrant/collections/{name}

# Verify vector dimension matches
# Collection dimension: 384
# Your vector: [0.1, 0.2, ...] (must be 384 elements)
```

**Solutions**:

1. Verify vector dimension matches collection config
2. Check vector length before sending
3. Use correct embedding model
4. Validate vectors before insertion

**Prevention**:

- Document collection dimensions
- Validate vectors before API calls
- Use consistent embedding models

### Filter Not Working

**Symptoms**:

- Filters return no results
- Filters return unexpected results
- Filter syntax errors

**Diagnosis**:

```bash
# Test filter syntax
curl -X POST http://localhost:15002/qdrant/collections/{name}/points/search \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [...],
    "filter": {
      "must": [{"type": "match", "key": "category", "match_value": "electronics"}]
    }
  }'
```

**Solutions**:

1. Verify filter syntax matches Qdrant format
2. Check payload field names match filter keys
3. Verify filter types are correct (match, range, geo, etc.)
4. Test filters individually before combining

**Common Filter Issues**:

- Field name typos
- Wrong filter type
- Nested field access (use dot notation: `user.name`)
- Case sensitivity in string matches

**Prevention**:

- Document payload schema
- Use consistent field names
- Test filters incrementally

### Performance Issues

**Symptoms**:

- Slow search queries
- High latency
- Timeout errors

**Diagnosis**:

```bash
# Check query time
curl -X POST http://localhost:15002/qdrant/collections/{name}/points/search \
  -H "Content-Type: application/json" \
  -d '{"vector": [...], "limit": 10}' \
  -w "\nTime: %{time_total}s\n"

# Check collection stats
curl http://localhost:15002/qdrant/collections/{name}
```

**Solutions**:

1. **Reduce limit**: Lower `limit` parameter
2. **Optimize filters**: Use indexed fields
3. **Enable quantization**: Reduce memory usage
4. **Use native API**: 30-50% faster
5. **Tune HNSW**: Adjust `ef_search` parameter

**Performance Tips**:

- Use payload indexing for filtered searches
- Enable quantization for large collections
- Use batch operations when possible
- Consider migrating to native API

### Collection Already Exists

**Symptoms**:

- Error: `Collection already exists`
- HTTP 409 status

**Solutions**:

1. Delete existing collection first
2. Use different collection name
3. Update existing collection instead

**Prevention**:

- Check collection existence before creation
- Use unique naming conventions
- Document collection lifecycle

## Error Resolution

### Error Codes

| Code  | Meaning      | Solution                                   |
| ----- | ------------ | ------------------------------------------ |
| `400` | Bad Request  | Check request format and parameters        |
| `404` | Not Found    | Verify resource exists                     |
| `409` | Conflict     | Resolve conflict (e.g., collection exists) |
| `500` | Server Error | Check server logs, retry request           |

### Common Error Messages

#### "Collection not found"

- **Cause**: Collection doesn't exist or wrong name
- **Solution**: List collections, verify name, create if needed

#### "Invalid vector dimension"

- **Cause**: Vector size doesn't match collection config
- **Solution**: Check collection dimension, fix vector size

#### "Invalid filter"

- **Cause**: Filter syntax error or unsupported filter type
- **Solution**: Verify filter format, check filter types

#### "Collection already exists"

- **Cause**: Trying to create existing collection
- **Solution**: Delete first or use different name

#### "Internal server error"

- **Cause**: Server-side issue
- **Solution**: Check server logs, restart if needed

### Error Debugging Steps

1. **Check HTTP Status**: Verify status code
2. **Read Error Message**: Check error details
3. **Validate Request**: Verify request format
4. **Check Logs**: Review server logs
5. **Test Incrementally**: Simplify request to isolate issue

## Performance Tuning

### Search Performance

**Optimize Query Parameters**:

```json
{
  "vector": [...],
  "limit": 10,           // Lower = faster
  "ef": 50,             // Lower = faster (if supported)
  "score_threshold": 0.5 // Filter early
}
```

**Enable Payload Indexing**:

- Index frequently filtered fields
- Use keyword index for exact matches
- Use integer/float index for ranges

**Use Quantization**:

- Enable SQ8 quantization
- Reduces memory usage
- May slightly reduce accuracy

### Write Performance

**Batch Operations**:

- Use batch upsert instead of single inserts
- Optimal batch size: 100-500 vectors
- Reduces API overhead

**Disable Auto-Indexing**:

- For bulk inserts, disable payload indexing temporarily
- Re-enable after bulk insert completes

### Configuration Tuning

**HNSW Parameters**:

```json
{
  "hnsw_config": {
    "m": 16, // Lower = faster indexing
    "ef_construction": 100, // Lower = faster indexing
    "ef_search": 50 // Lower = faster search
  }
}
```

**Memory Optimization**:

- Enable quantization
- Use compression
- Set memory limits

## Debugging Guide

### Enable Debug Logging

**Server Configuration**:

```yaml
logging:
  level: "debug" # Enable debug logs
  log_requests: true
  log_responses: true
```

### Check Request/Response

**Enable Request Logging**:

```bash
# Verbose curl output
curl -v -X POST http://localhost:15002/qdrant/collections/{name}/points/search \
  -H "Content-Type: application/json" \
  -d @request.json
```

### Verify Endpoint Availability

```bash
# Check server status
curl http://localhost:15002/api/status

# List Qdrant endpoints
curl http://localhost:15002/qdrant/collections
```

### Test Filter Logic

```bash
# Test simple filter
curl -X POST http://localhost:15002/qdrant/collections/{name}/points/search \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [...],
    "filter": {
      "must": [{"type": "match", "key": "status", "match_value": "active"}]
    }
  }'
```

### Monitor Performance

**Check Query Times**:

- Response includes `time` field
- Monitor for slow queries
- Compare with native API performance

**Check Collection Stats**:

```bash
curl http://localhost:15002/qdrant/collections/{name}
# Check: vectors_count, indexed_vectors_count, points_count
```

## FAQ

### Q: Why is Qdrant API slower than native API?

**A**: Compatibility layer adds overhead. Native API is optimized for Vectorizer's architecture and is 30-50% faster.

### Q: Can I use Qdrant client libraries?

**A**: Yes, but only REST API clients. gRPC clients won't work. See [Client Compatibility](../clients/README.md).

### Q: Are all Qdrant filters supported?

**A**: Yes, all filter types are supported: match, range, geo_bounding_box, geo_radius, values_count, and nested filters.

### Q: Can I use Qdrant snapshots?

**A**: Snapshots are available via native API only. Use `/api/backups` endpoints for backup/restore.

### Q: How do I migrate from Qdrant to Vectorizer?

**A**: See [Migration Guide](../../specs/QDRANT_MIGRATION.md) for step-by-step instructions.

### Q: Is gRPC supported?

**A**: No, only REST API is supported. Use REST API or migrate to native APIs.

### Q: Can I use sharding/clustering?

**A**: No, sharding and clustering are not supported. Use native replication API for high availability.

### Q: How do I migrate my data from Qdrant?

**A**: Use the migration tools provided in `vectorizer::migration::qdrant`:

```rust
// Export from Qdrant
let exported = QdrantDataExporter::export_collection("http://qdrant:6333", "my_collection").await?;

// Import into Vectorizer
let result = QdrantDataImporter::import_collection(&store, &exported).await?;
```

See [Migration Guide](../../specs/QDRANT_MIGRATION.md) for detailed instructions.

### Q: Can I migrate sparse vectors?

**A**: No, sparse vectors are not supported in migration. You'll need to convert them to dense vectors first.

### Q: What happens to named vectors during migration?

**A**: Named vectors are not supported. Only the first named vector will be used during migration. Consider converting to single vector format before migrating.

### Q: What's the difference between Qdrant API and native API?

**A**: Native API offers better performance, more features (intelligent search, etc.), and better integration (MCP, WebSocket).

### Q: How do I improve search performance?

**A**:

1. Use native API (30-50% faster)
2. Enable payload indexing
3. Optimize HNSW parameters
4. Use quantization
5. Reduce query limit

### Q: Can I use Qdrant filters with native API?

**A**: Native API uses different filter format. Use Qdrant API for Qdrant filters, or convert to native filter format.

### Q: What Qdrant version is compatible?

**A**: Vectorizer is compatible with Qdrant v1.14.x REST API. Older versions may have some differences.

## Support Resources

- **Documentation**: [API Compatibility](./API_COMPATIBILITY.md)
- **Migration Guide**: [Migration Guide](../../specs/QDRANT_MIGRATION.md)
- **Examples**: [Examples](./EXAMPLES.md)
- **GitHub Issues**: https://github.com/hivellm/vectorizer/issues
- **Community**: [Discord](https://discord.gg/vectorizer)

## Escalation

If issues persist:

1. **Check Logs**: Review server logs for detailed errors
2. **Simplify Request**: Test with minimal request to isolate issue
3. **Compare with Native API**: Test same operation with native API
4. **File Issue**: Create GitHub issue with:
   - Error message
   - Request/response
   - Server logs
   - Steps to reproduce
