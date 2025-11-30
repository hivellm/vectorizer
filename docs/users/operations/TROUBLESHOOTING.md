---
title: Troubleshooting Guide
module: troubleshooting
id: troubleshooting-guide
order: 1
description: Common issues and solutions for Vectorizer
tags: [troubleshooting, problems, issues, solutions]
---

# Troubleshooting Guide

Common issues and their solutions.

## Service Issues

### Service Won't Start

**Symptoms**: Service fails to start or immediately stops.

**Solutions**:

1. Check logs: `sudo journalctl -u vectorizer -n 50` (Linux) or `Get-EventLog -LogName Application -Source Vectorizer -Newest 50` (Windows)
2. Verify port 15002 is not in use: `netstat -tuln | grep 15002` (Linux) or `netstat -ano | findstr 15002` (Windows)
3. Check binary permissions: `ls -l /usr/local/bin/vectorizer` (Linux)
4. Verify user exists: `id vectorizer` (Linux)

### Service Keeps Restarting

**Symptoms**: Service starts but immediately restarts in a loop.

**Solutions**:

1. Check logs for crash reasons
2. Verify configuration is valid
3. Check system resources (memory, disk space)
4. Review recent changes to configuration

## Connection Issues

### Cannot Connect to API

**Symptoms**: `curl http://localhost:15002/health` fails.

**Solutions**:

1. Verify service is running: `sudo systemctl status vectorizer` (Linux) or `Get-Service Vectorizer` (Windows)
2. Check firewall settings
3. Verify host binding: Should be `0.0.0.0` or `127.0.0.1`
4. Check port availability

### Connection Refused

**Symptoms**: Connection refused errors.

**Solutions**:

1. Service may not be running
2. Port may be blocked by firewall
3. Service may be binding to wrong interface

## Performance Issues

### Slow Searches

**Symptoms**: Search operations take too long (> 20ms for small collections).

**Solutions**:

1. **Check collection size and dimension**:

   - Large collections (>100K vectors) naturally take longer
   - Higher dimensions (768+) are slower than lower (384)

2. **Verify HNSW index is built**:

   ```bash
   curl http://localhost:15002/collections/my_collection
   # Check "indexed_count" matches "vector_count"
   ```

3. **Optimize HNSW parameters**:

   ```json
   {
     "hnsw_config": {
       "ef_search": 32 // Lower = faster, but less accurate
     }
   }
   ```

4. **Enable quantization**:

   ```json
   {
     "quantization": {
       "enabled": true,
       "type": "scalar",
       "bits": 8
     }
   }
   ```

5. **Check system resources**:

   ```bash
   # CPU usage
   top -p $(pgrep vectorizer)

   # Memory usage
   ps aux | grep vectorizer
   ```

6. **Reduce search limit**: Request fewer results if possible

### High Memory Usage

**Symptoms**: Vectorizer uses too much memory (>2GB for <1M vectors).

**Solutions**:

1. **Enable quantization** (4x memory reduction):

   ```json
   {
     "quantization": {
       "enabled": true,
       "type": "scalar",
       "bits": 8
     }
   }
   ```

2. **Reduce collection size**: Delete unused collections or vectors

3. **Check for memory leaks**:

   ```bash
   # Monitor memory over time
   watch -n 1 'ps aux | grep vectorizer | awk "{print \$6/1024\" MB\"}"'
   ```

4. **Use compression** for payloads:

   ```json
   {
     "compression": {
       "enabled": true,
       "threshold_bytes": 1024
     }
   }
   ```

5. **Lower HNSW M parameter**: Reduces index memory usage

### Slow Indexing

**Symptoms**: Vector insertion is slow (<500 vectors/second).

**Solutions**:

1. **Use batch operations**:

   ```python
   # ✅ Good: Batch insert
   await client.batch_insert_text("collection", texts)

   # ❌ Bad: Individual inserts
   for text in texts:
       await client.insert_text("collection", text)
   ```

2. **Optimize batch size**: 500-1000 vectors per batch is optimal

3. **Lower ef_construction**: Faster indexing, slightly lower quality:

   ```json
   {
     "hnsw_config": {
       "ef_construction": 100 // Lower = faster indexing
     }
   }
   ```

4. **Disable quantization during indexing**: Enable after indexing is complete

5. **Check disk I/O**: Slow disk can bottleneck indexing

## Data Issues

### Collections Not Found

**Error Message**: `Collection not found: {collection_name}`

**Symptoms**: "Collection not found" errors when accessing collections.

**Solutions**:

1. **Verify collection exists**:

   ```bash
   curl http://localhost:15002/collections
   ```

2. **Check collection name spelling**: Collection names are case-sensitive

3. **Verify data directory permissions**:

   ```bash
   # Linux
   ls -la /var/lib/vectorizer

   # Windows
   dir %ProgramData%\Vectorizer
   ```

4. **Check if collection was deleted**: Review logs for deletion events

### Collection Already Exists

**Error Message**: `Collection already exists: {collection_name}`

**Symptoms**: Cannot create collection with existing name.

**Solutions**:

1. **Delete existing collection** (if you want to recreate it):

   ```bash
   curl -X DELETE http://localhost:15002/collections/my_collection
   ```

2. **Use a different name**:

   ```bash
   curl -X POST http://localhost:15002/collections \
     -H "Content-Type: application/json" \
     -d '{"name": "my_collection_v2", "dimension": 384}'
   ```

3. **Update existing collection** instead of creating new one

### Invalid Dimension

**Error Message**: `Invalid dimension: expected {expected}, got {got}`

**Symptoms**: Vector insertion fails due to dimension mismatch.

**Solutions**:

1. **Check collection dimension**:

   ```bash
   curl http://localhost:15002/collections/my_collection
   ```

2. **Verify vector dimension matches**:

   ```python
   # Python example
   collection_info = await client.get_collection_info("my_collection")
   expected_dim = collection_info["dimension"]

   # Ensure your vector has this exact dimension
   vector = [0.1] * expected_dim  # Correct dimension
   ```

3. **Use correct embedding model**: Ensure your embedding model outputs the expected dimension

### Dimension Mismatch

**Error Message**: `Dimension mismatch: expected {expected}, got {actual}`

**Symptoms**: Vector operations fail due to dimension inconsistency.

**Solutions**:

1. **Verify all vectors have same dimension**:

   ```python
   # Check vector dimensions before insertion
   for vector in vectors:
       assert len(vector) == expected_dimension, \
           f"Vector has {len(vector)} dimensions, expected {expected_dimension}"
   ```

2. **Normalize vector dimensions**: Ensure consistent preprocessing

3. **Check batch operations**: All vectors in a batch must have same dimension

### Vector Not Found

**Error Message**: `Vector not found: {vector_id}`

**Symptoms**: Cannot retrieve or update a vector.

**Solutions**:

1. **Verify vector exists**:

   ```bash
   curl http://localhost:15002/collections/my_collection/vectors/vector_id
   ```

2. **Check vector ID spelling**: IDs are case-sensitive

3. **List all vectors** (if collection is small):
   ```python
   # Search with empty query to get all vectors
   results = await client.search("my_collection", "", limit=1000)
   vector_ids = [r["id"] for r in results]
   ```

### Data Loss

**Symptoms**: Collections or vectors disappear.

**Solutions**:

1. **Check data directory**: `/var/lib/vectorizer` (Linux) or `%ProgramData%\Vectorizer` (Windows)

2. **Verify disk space**:

   ```bash
   # Linux
   df -h /var/lib/vectorizer

   # Windows
   dir %ProgramData%\Vectorizer
   ```

3. **Check logs for errors**:

   ```bash
   sudo journalctl -u vectorizer | grep -i error
   ```

4. **Review backup procedures**: Ensure regular backups are configured

5. **Check for accidental deletion**: Review recent operations

## API Errors

### Validation Errors

**Error Message**: `Invalid {field}: {reason}`

**Symptoms**: API requests fail with validation errors.

**Common Issues**:

1. **Invalid dimension**:

   ```json
   // ❌ Wrong: dimension must be positive integer
   {"dimension": -1}

   // ✅ Correct
   {"dimension": 384}
   ```

2. **Invalid metric**:

   ```json
   // ❌ Wrong: metric must be one of: cosine, euclidean, dot_product
   {"metric": "manhattan"}

   // ✅ Correct
   {"metric": "cosine"}
   ```

3. **Invalid vector data**:

   ```json
   // ❌ Wrong: vector must be array of numbers
   {"vector": "not an array"}

   // ✅ Correct
   {"vector": [0.1, 0.2, 0.3]}
   ```

**Solutions**:

1. **Validate input before sending**: Check all required fields
2. **Check API documentation**: Verify field types and constraints
3. **Use SDKs**: SDKs provide automatic validation

### Rate Limit Exceeded

**Error Message**: `Rate limit exceeded: {limit_type} limit of {limit}`

**Symptoms**: Requests are rejected due to rate limiting.

**Solutions**:

1. **Reduce request frequency**: Add delays between requests
2. **Use batch operations**: Combine multiple operations into batches
3. **Implement retry logic**: Retry with exponential backoff
4. **Contact administrator**: Request rate limit increase if needed

### Serialization Errors

**Error Message**: `Serialization error: {details}` or `JSON error: {details}`

**Symptoms**: Request/response parsing fails.

**Solutions**:

1. **Validate JSON format**: Ensure valid JSON syntax
2. **Check data types**: Numbers must be numbers, not strings
3. **Verify encoding**: Use UTF-8 encoding
4. **Check payload size**: Very large payloads may fail

## Installation Issues

### Binary Not Found

**Symptoms**: `vectorizer: command not found`.

**Solutions**:

1. **Verify installation**:

   ```bash
   # Linux
   which vectorizer

   # Windows
   Get-Command vectorizer
   ```

2. **Check PATH environment variable**:

   ```bash
   # Linux
   echo $PATH
   # Should include /usr/local/bin or $HOME/.cargo/bin

   # Windows
   $env:PATH
   # Should include %USERPROFILE%\.cargo\bin
   ```

3. **Reinstall if necessary**:

   ```bash
   # Re-run installation script
   curl -fsSL https://raw.githubusercontent.com/hivellm/vectorizer/main/scripts/install.sh | bash
   ```

4. **Manual PATH addition** (if needed):

   ```bash
   # Linux - Add to ~/.bashrc or ~/.zshrc
   export PATH="$HOME/.cargo/bin:$PATH"

   # Windows - Add to PATH environment variable
   ```

### Permission Denied

**Symptoms**: Permission errors during installation or operation.

**Solutions**:

1. **Use sudo for Linux** (when required):

   ```bash
   sudo systemctl start vectorizer
   ```

2. **Run PowerShell as Administrator** on Windows:

   - Right-click PowerShell → "Run as Administrator"

3. **Check file permissions**:

   ```bash
   # Linux
   ls -l /usr/local/bin/vectorizer
   # Should be executable: -rwxr-xr-x

   chmod +x /usr/local/bin/vectorizer  # If not executable
   ```

4. **Check data directory permissions**:
   ```bash
   # Linux
   sudo chown -R vectorizer:vectorizer /var/lib/vectorizer
   sudo chmod 755 /var/lib/vectorizer
   ```

### Build Failures

**Symptoms**: Installation fails during compilation.

**Solutions**:

1. **Check Rust version**: Requires Rust 1.9+ with edition 2024

   ```bash
   rustc --version
   ```

2. **Update Rust toolchain**:

   ```bash
   rustup update stable
   rustup default stable
   ```

3. **Install build dependencies**:

   ```bash
   # Ubuntu/Debian
   sudo apt-get install build-essential pkg-config libssl-dev

   # macOS
   xcode-select --install
   ```

4. **Check available memory**: Build requires at least 2GB RAM

5. **Review build logs**: Look for specific error messages

## Common Error Codes

### HTTP Status Codes

| Code | Meaning               | Common Causes                                            |
| ---- | --------------------- | -------------------------------------------------------- |
| 200  | Success               | -                                                        |
| 400  | Bad Request           | Invalid JSON, missing required fields, validation errors |
| 404  | Not Found             | Collection/vector doesn't exist, wrong URL               |
| 409  | Conflict              | Collection already exists                                |
| 500  | Internal Server Error | Server bug, system error                                 |

### Error Response Format

```json
{
  "error": {
    "type": "collection_not_found",
    "message": "Collection 'my_collection' not found",
    "status_code": 404
  }
}
```

## Debugging Tips

### Enable Debug Logging

```bash
# Set log level to debug
export VECTORIZER_LOG_LEVEL=debug

# Restart service
sudo systemctl restart vectorizer  # Linux
Restart-Service Vectorizer  # Windows
```

### Check Service Health

```bash
# Health check endpoint
curl http://localhost:15002/health

# Expected response
{"status": "healthy", "version": "1.3.0"}
```

### Monitor Real-Time Logs

```bash
# Linux
sudo journalctl -u vectorizer -f

# Windows
Get-EventLog -LogName Application -Source Vectorizer -Newest 50 -Wait
```

### Test API Endpoints

```bash
# List collections
curl http://localhost:15002/collections

# Get collection info
curl http://localhost:15002/collections/my_collection

# Health check
curl http://localhost:15002/health
```

## Getting Help

If you're still experiencing issues:

1. **Check the documentation**:

   - [Main README](../../README.md)
   - [API Reference](../api/API_REFERENCE.md)
   - [Performance Guide](../configuration/PERFORMANCE_TUNING.md)

2. **Review logs** for detailed error messages:

   ```bash
   sudo journalctl -u vectorizer -n 100 | grep -i error
   ```

3. **Verify configuration**:

   - Check server configuration
   - Verify collection settings
   - Review environment variables

4. **Open an issue on GitHub** with:
   - Exact error messages
   - Steps to reproduce
   - System information (OS, Rust version)
   - Relevant logs (with sensitive data removed)
   - Vectorizer version: `vectorizer --version`

## Related Topics

- [Service Management](./SERVICE_MANAGEMENT.md) - Service troubleshooting
- [Configuration](../configuration/CONFIGURATION.md) - Configuration issues
- [Installation Guide](../getting-started/INSTALLATION.md) - Installation problems
