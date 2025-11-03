# GPU Setup Guide - Metal Acceleration

Complete guide for setting up and troubleshooting GPU acceleration in Vectorizer using Apple Metal.

## Table of Contents

1. [System Requirements](#system-requirements)
2. [Quick Start](#quick-start)
3. [Configuration](#configuration)
4. [Verification](#verification)
5. [Troubleshooting](#troubleshooting)
6. [Performance Tuning](#performance-tuning)
7. [Monitoring](#monitoring)
8. [FAQ](#faq)

## System Requirements

### Hardware Requirements

✅ **Supported Platforms:**
- macOS 10.13+ (High Sierra or later)
- Apple Silicon (M1, M2, M3, M4 series) **[Recommended]**
- Intel Macs with Metal-capable GPU

❌ **Not Supported:**
- Linux (Metal not available)
- Windows (Metal not available)
- macOS versions older than 10.13

### Software Requirements

- **Rust:** 1.85+ (nightly toolchain)
- **Cargo:** Latest version
- **Xcode Command Line Tools:** `xcode-select --install`

## Quick Start

### 1. Enable Metal GPU Support

Build Vectorizer with Metal GPU support:

```bash
# Development build with Metal
cargo build --features hive-gpu

# Production release with Metal
cargo build --release --features hive-gpu
```

### 2. Verify Metal GPU Detection

Run the GPU detection tests:

```bash
# Test Metal detection
cargo test --features hive-gpu --lib gpu_detection -- --nocapture

# Expected output:
# ✓ Detected backend: Metal
# ✓ Metal available: true
```

### 3. Run Vectorizer

Start the server with GPU acceleration enabled:

```bash
# Using the compiled binary
./target/release/vectorizer

# Or with cargo
cargo run --release --features hive-gpu
```

You should see in the logs:

```
🚀 Detecting GPU capabilities...
✅ Metal GPU detected and enabled!
📊 GPU Info: 🍎 Metal - Apple M1 Pro
```

## Configuration

### YAML Configuration

Edit your `config.yml`:

```yaml
# GPU Configuration (Metal GPU - macOS only)
gpu:
  # Enable GPU acceleration (default: true on macOS, false on other platforms)
  enabled: true
  
  # Batch size for GPU batch operations (default: 1000)
  # Higher values = better GPU utilization, more memory usage
  batch_size: 1000
  
  # Fallback to CPU if GPU initialization fails (default: true)
  fallback_to_cpu: true
  
  # Preferred backend: auto (detect best), metal (force Metal), cpu (disable GPU)
  preferred_backend: "auto"
```

### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | bool | true (macOS) / false (other) | Enable GPU acceleration |
| `batch_size` | int | 1000 | Batch size for GPU operations |
| `fallback_to_cpu` | bool | true | Fallback to CPU on GPU errors |
| `preferred_backend` | string | "auto" | Backend selection: auto/metal/cpu |

### Environment Variables

Override configuration via environment variables:

```bash
# Force GPU backend
export VECTORIZER_GPU_BACKEND="metal"

# Set batch size
export VECTORIZER_GPU_BATCH_SIZE=2000

# Disable GPU
export VECTORIZER_GPU_ENABLED="false"
```

## Verification

### Check GPU Status

#### 1. System Information Endpoint

```bash
curl http://localhost:15002/api/v1/info
```

Response includes GPU information:

```json
{
  "version": "1.2.3",
  "gpu": {
    "backend": "Metal",
    "device": "Apple M1 Pro",
    "enabled": true
  }
}
```

#### 2. Prometheus Metrics

Check GPU metrics:

```bash
curl http://localhost:15002/prometheus/metrics | grep gpu
```

Expected metrics:

```prometheus
# HELP vectorizer_gpu_backend_type GPU backend type (0=None, 1=Metal)
vectorizer_gpu_backend_type 1

# HELP vectorizer_gpu_memory_usage_bytes GPU memory usage in bytes
vectorizer_gpu_memory_usage_bytes 0

# HELP vectorizer_gpu_search_requests_total Total GPU search requests
vectorizer_gpu_search_requests_total 0
```

#### 3. Run Integration Tests

```bash
# Run Metal GPU validation tests
cargo test --test metal_gpu_validation --features hive-gpu -- --nocapture

# Expected: All 5 tests pass
# test metal_tests::test_metal_detection_on_macos ... ok
# test metal_tests::test_metal_availability ... ok
# test metal_tests::test_gpu_info_retrieval ... ok
# test metal_tests::test_gpu_context_creation ... ok
# test metal_tests::test_vector_store_with_metal ... ok
```

## Troubleshooting

### Issue 1: Metal Not Detected

**Symptoms:**
```
💻 No Metal GPU detected or available, falling back to CPU mode.
```

**Possible Causes:**
1. Not running on macOS
2. macOS version too old (< 10.13)
3. Metal not supported by GPU
4. `hive-gpu` feature not enabled

**Solutions:**

1. **Verify platform:**
   ```bash
   uname -s  # Should show: Darwin
   sw_vers   # Check macOS version
   ```

2. **Check Metal support:**
   ```bash
   system_profiler SPDisplaysDataType | grep Metal
   # Should show: Metal: Supported
   ```

3. **Rebuild with correct features:**
   ```bash
   cargo clean
   cargo build --release --features hive-gpu
   ```

4. **Verify feature flags:**
   ```bash
   cargo tree --features hive-gpu | grep hive-gpu
   # Should show: hive-gpu with metal-native feature
   ```

### Issue 2: GPU Context Creation Failed

**Symptoms:**
```
⚠️  Failed to create GPU context: MetalContextError
✅ Falling back to CPU mode
```

**Possible Causes:**
1. GPU drivers out of date
2. Metal framework not available
3. Memory allocation failure
4. Conflicting GPU usage

**Solutions:**

1. **Update macOS:**
   ```bash
   softwareupdate --list
   softwareupdate --install --all
   ```

2. **Check GPU memory:**
   ```bash
   # Monitor GPU usage
   sudo powermetrics --samplers gpu_power -i 1000 -n 1
   ```

3. **Restart Metal service:**
   ```bash
   # Close GPU-intensive apps
   # Restart Vectorizer
   ./target/release/vectorizer
   ```

4. **Enable verbose logging:**
   ```yaml
   logging:
     level: "debug"  # In config.yml
   ```

### Issue 3: Low GPU Performance

**Symptoms:**
- Search operations slower than expected
- CPU usage high despite GPU enabled
- GPU not fully utilized

**Solutions:**

1. **Increase batch size:**
   ```yaml
   gpu:
     batch_size: 2000  # Increase from 1000
   ```

2. **Use batch operations:**
   ```rust
   // Instead of:
   for vector in vectors {
       collection.add_vector(vector);
   }
   
   // Use:
   collection.add_vectors_batch(vectors);
   ```

3. **Optimize collection config:**
   ```yaml
   collections:
     defaults:
       quantization:
         type: "sq"
         sq:
           bits: 8  # Reduce memory, increase speed
   ```

### Issue 4: GPU Memory Errors

**Symptoms:**
```
Error: GPU memory allocation failed
Error: Insufficient VRAM
```

**Solutions:**

1. **Check available memory:**
   ```bash
   # Get GPU info via test
   cargo test --features hive-gpu test_gpu_info_retrieval -- --nocapture
   ```

2. **Reduce batch size:**
   ```yaml
   gpu:
     batch_size: 500  # Reduce if low memory
   ```

3. **Enable CPU fallback:**
   ```yaml
   gpu:
     fallback_to_cpu: true  # Auto fallback on errors
   ```

4. **Close other GPU applications:**
   - Quit browsers with hardware acceleration
   - Close video editing software
   - Stop other ML/GPU workloads

## Performance Tuning

### Optimal Settings by System

#### Apple Silicon (M1/M2/M3/M4)

**Recommended Configuration:**

```yaml
gpu:
  enabled: true
  batch_size: 2000        # Higher is better for Apple Silicon
  fallback_to_cpu: true
  preferred_backend: "auto"

collections:
  defaults:
    quantization:
      type: "sq"
      sq:
        bits: 8           # Scalar quantization for memory efficiency
```

**Expected Performance:**
- Search: 5-10x faster than CPU
- Batch insert: 50-100x faster than CPU
- Batch search: 100-200x faster than CPU

#### Intel Mac with Metal

**Recommended Configuration:**

```yaml
gpu:
  enabled: true
  batch_size: 1000        # Conservative for Intel Macs
  fallback_to_cpu: true
  preferred_backend: "auto"
```

**Expected Performance:**
- Search: 2-5x faster than CPU
- Batch operations: 10-50x faster than CPU

### Batch Operation Guidelines

| Vector Count | Recommended Batch Size |
|--------------|------------------------|
| < 100 | Use single operations |
| 100 - 1,000 | 500 |
| 1,000 - 10,000 | 1000 |
| 10,000 - 100,000 | 2000 |
| > 100,000 | 5000 |

### Memory Usage Guidelines

| Collection Size | Expected VRAM Usage | Recommended System |
|-----------------|---------------------|-------------------|
| 10k vectors (512d) | ~20-30 MB | Any Metal-capable Mac |
| 100k vectors (512d) | ~200-300 MB | 8GB+ VRAM |
| 1M vectors (512d) | ~2-3 GB | 16GB+ VRAM |
| 10M vectors (512d) | ~20-30 GB | 32GB+ VRAM (Apple Silicon) |

## Monitoring

### Real-time Monitoring

#### 1. Activity Monitor

Use macOS Activity Monitor to track GPU usage:

1. Open **Activity Monitor**
2. Go to **Window > GPU History**
3. Monitor **GPU %** while Vectorizer runs

#### 2. Prometheus + Grafana

Set up monitoring dashboard:

```yaml
# config.yml
monitoring:
  prometheus:
    enabled: true
    endpoint: "/prometheus/metrics"
```

Access metrics:
```bash
curl http://localhost:15002/prometheus/metrics | grep gpu
```

#### 3. Log Monitoring

Enable detailed GPU logging:

```yaml
logging:
  level: "debug"
  log_requests: true
```

Watch logs:
```bash
tail -f vectorizer.log | grep -i gpu
```

### Key Metrics to Monitor

1. **`gpu_backend_type`**: Should be 1 (Metal) if GPU active
2. **`gpu_search_requests_total`**: Track GPU search usage
3. **`gpu_search_latency_seconds`**: Monitor search performance
4. **`gpu_batch_operations_total`**: Track batch operation usage
5. **`gpu_memory_usage_bytes`**: Monitor GPU memory consumption

## FAQ

### Q: Can I run Vectorizer without GPU?

**A:** Yes! GPU is optional. On non-macOS platforms or if GPU is disabled, Vectorizer automatically uses CPU-based collections with high performance.

```yaml
gpu:
  enabled: false  # Disable GPU entirely
```

### Q: Will GPU improve all operations?

**A:** GPU acceleration primarily benefits:
- ✅ Vector search (5-10x faster)
- ✅ Batch operations (50-200x faster)
- ✅ Large collections (>10k vectors)

CPU may be faster for:
- ❌ Small collections (<100 vectors)
- ❌ Single vector operations
- ❌ Non-vector operations (indexing, metadata)

### Q: Can I mix GPU and CPU collections?

**A:** Yes! You can have both GPU and CPU collections in the same VectorStore. Collections are created based on availability and configuration.

### Q: What happens if GPU fails during operation?

**A:** With `fallback_to_cpu: true`, Vectorizer automatically falls back to CPU for that operation. The error is logged but doesn't crash the server.

### Q: Does GPU support work with Docker?

**A:** Metal GPU is not available inside Docker containers. When running in Docker on macOS, Vectorizer will automatically use CPU mode.

### Q: How do I force CPU mode on macOS?

**A:** Set `preferred_backend: "cpu"` in config:

```yaml
gpu:
  preferred_backend: "cpu"  # Force CPU even on macOS
```

### Q: Will CUDA/ROCm be supported in the future?

**A:** Yes! CUDA (NVIDIA) and ROCm (AMD) support is planned when the `hive-gpu` library adds support for these backends. The architecture is already prepared for multi-backend support.

### Q: How much faster is GPU vs CPU?

**A:** Performance improvements (Apple Silicon M1/M2/M3):
- Single search: **5-10x faster**
- Batch insert (1000 vectors): **50-100x faster**
- Batch search (100 queries): **100-200x faster**

Actual speedup depends on:
- GPU model and memory
- Vector dimensions
- Collection size
- Batch size

## Additional Resources

- **Architecture Documentation**: `docs/ARCHITECTURE.md`
- **Implementation Details**: `docs/GPU_METAL_IMPLEMENTATION.md`
- **API Documentation**: Run `cargo doc --open --features hive-gpu`
- **GitHub Issues**: [Report GPU issues](https://github.com/your-org/vectorizer/issues)

## Support

### Community Support

- **Discord**: [Join our community](https://discord.gg/vectorizer)
- **GitHub Discussions**: [Ask questions](https://github.com/your-org/vectorizer/discussions)

### Enterprise Support

For production deployments and custom GPU optimization:
- Email: support@vectorizer.ai
- Enterprise docs: https://docs.vectorizer.ai/enterprise

---

**Last Updated:** 2025-01-07  
**Vectorizer Version:** 1.2.3+  
**Metal Support:** macOS 10.13+


