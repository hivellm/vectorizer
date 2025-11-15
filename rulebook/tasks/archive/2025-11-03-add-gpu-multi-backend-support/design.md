# Design: GPU Multi-Backend Support

## Context

Currently, the Vectorizer only supports GPU acceleration on macOS via Metal backend from `hive-gpu`. The codebase has conditional compilation that restricts GPU usage:

```rust
#[cfg(all(feature = "hive-gpu", target_os = "macos"))]
```

This limits GPU acceleration to <5% of deployments. The `hive-gpu` crate already supports multiple backends (Metal, CUDA, WebGPU), but the integration layer doesn't detect or utilize them.

**Key constraints:**
- Must maintain backward compatibility (CPU fallback)
- Cannot require GPU for functionality
- Should auto-detect best available backend
- Must handle graceful degradation

**Stakeholders:**
- End users on Linux/Windows with NVIDIA/AMD GPUs
- DevOps teams deploying on cloud GPU instances
- Developers working on search performance

## Goals / Non-Goals

### Goals
- ✅ Auto-detect GPU backend (CUDA/Metal/WebGPU) on all platforms
- ✅ Use GPU for vector search when available
- ✅ Provide 10-50x search performance improvement on GPU
- ✅ Enable GPU batch operations (100-500x faster)
- ✅ Maintain CPU fallback for systems without GPU
- ✅ Zero configuration for basic usage
- ✅ Expose GPU metrics for monitoring

### Non-Goals
- ❌ Require GPU for any functionality
- ❌ Support all GPU vendors equally (prioritize NVIDIA/Apple)
- ❌ GPU for embedding generation (out of scope)
- ❌ Multi-GPU support (future work)
- ❌ GPU memory management/pooling (rely on hive-gpu)
- ❌ Custom GPU kernels (use hive-gpu abstractions)

## Technical Decisions

### Decision 1: Priority-Based Backend Detection

**Choice:** Detect backends in order of performance: CUDA > Metal > WebGPU > CPU

**Rationale:**
- CUDA: Best performance on NVIDIA GPUs (most common in servers)
- Metal: Best performance on Apple Silicon (macOS only)
- WebGPU: Cross-platform but slower than native backends
- CPU: Universal fallback

**Alternatives considered:**
1. **User-configured backend:** Requires manual configuration, error-prone
2. **First-available backend:** May choose slow option when faster available
3. **Benchmark-based selection:** Too slow at startup

**Implementation:**
```rust
pub enum GpuBackendType {
    Cuda,    // NVIDIA GPUs (Linux/Windows)
    Metal,   // Apple GPUs (macOS only)
    WebGpu,  // Cross-platform fallback
    None,    // CPU-only mode
}

impl GpuDetector {
    pub fn detect_best_backend() -> GpuBackendType {
        if Self::is_cuda_available() { return GpuBackendType::Cuda; }
        if Self::is_metal_available() { return GpuBackendType::Metal; }
        if Self::is_webgpu_available() { return GpuBackendType::WebGpu; }
        GpuBackendType::None
    }
}
```

### Decision 2: Transparent GPU Usage in Collections

**Choice:** Collections auto-select GPU backend, transparent to API users

**Rationale:**
- Users shouldn't need to know about GPU backends
- API remains simple: `store.create_collection(name, config)`
- Backend selection happens once at collection creation
- No API changes required

**Alternatives considered:**
1. **Explicit GPU parameter:** `create_collection(..., use_gpu: bool)` - Too manual
2. **Separate GPU collection type:** Breaks API consistency
3. **Runtime GPU switching:** Complex, error-prone

**Implementation:**
```rust
impl VectorStore {
    pub fn create_collection(&self, name: &str, config: CollectionConfig) -> Result<()> {
        let backend = GpuDetector::detect_best_backend();
        
        match backend {
            GpuBackendType::None => {
                // Create CPU collection
                let collection = Collection::new(name, config);
                self.collections.insert(name, CollectionType::Cpu(collection));
            }
            _ => {
                // Create GPU collection
                let context = GpuAdapter::create_context(backend)?;
                let collection = HiveGpuCollection::new(name, config, context)?;
                self.collections.insert(name, CollectionType::HiveGpu(collection));
            }
        }
        Ok(())
    }
}
```

### Decision 3: Unified Collection Interface (CollectionType Enum)

**Choice:** Keep existing `CollectionType::Cpu` and `CollectionType::HiveGpu` enum

**Rationale:**
- Already implemented and working
- Clean separation between CPU and GPU code paths
- No runtime overhead for dispatch
- Easy to extend with new backend types if needed

**Alternatives considered:**
1. **Trait-based abstraction:** More flexible but adds complexity
2. **HybridCollection struct:** Complicates codebase, harder to maintain
3. **Dynamic dispatch:** Runtime overhead, harder to optimize

**Trade-offs:**
- ✅ Simple, clear code paths
- ✅ Zero-cost abstraction
- ✅ Easy to debug
- ❌ Requires match statements in VectorStore
- ❌ Less extensible than traits

### Decision 4: Feature Flags for Backend Support

**Choice:** Use Cargo features for opt-in backend support

**Rationale:**
- Not all users need CUDA (requires NVIDIA GPU)
- WebGPU has additional dependencies
- Allows minimal builds for specific platforms

**Implementation:**
```toml
[features]
default = ["hive-gpu", "fastembed"]
hive-gpu = ["dep:hive-gpu"]
hive-gpu-metal = ["hive-gpu", "hive-gpu/metal-native"]
hive-gpu-cuda = ["hive-gpu", "hive-gpu/cuda"]
hive-gpu-wgpu = ["hive-gpu", "hive-gpu/wgpu"]
```

**Build configurations:**
- macOS: `cargo build --features hive-gpu-metal`
- Linux+NVIDIA: `cargo build --features hive-gpu-cuda`
- Universal: `cargo build --features hive-gpu-cuda,hive-gpu-wgpu`
- Minimal: `cargo build --no-default-features`

### Decision 5: GPU Metrics via Collection Metadata

**Choice:** Extend existing metadata system to include GPU info

**Rationale:**
- Consistent with current API patterns
- No new endpoints required
- Easy to consume by monitoring systems

**Implementation:**
```rust
pub struct CollectionMetadata {
    pub name: String,
    pub vector_count: usize,
    pub config: CollectionConfig,
    // NEW FIELDS:
    pub gpu_backend: Option<String>,     // "cuda", "metal", "webgpu"
    pub gpu_device: Option<String>,      // "NVIDIA RTX 4090", "Apple M1 Max"
    pub gpu_memory_used: Option<usize>,  // Bytes
    pub gpu_memory_total: Option<usize>, // Bytes
}
```

## Architecture

### Component Diagram

```
VectorStore
    ↓
GpuDetector ← detects best backend
    ↓
GpuAdapter::create_context(backend)
    ↓ 
    ├─→ CudaContext (if CUDA available)
    ├─→ MetalNativeContext (if macOS)
    └─→ WgpuContext (if WebGPU available)
    ↓
HiveGpuCollection (wraps GpuVectorStorage)
    ↓
GPU-accelerated operations:
    - search()
    - add_vectors()
    - batch operations
```

### Data Flow

**Collection Creation:**
```
User calls create_collection()
    ↓
GpuDetector.detect_best_backend()
    ├─ Try CUDA → Success → Use CUDA
    ├─ Try Metal → Success → Use Metal
    ├─ Try WebGPU → Success → Use WebGPU
    └─ All fail → Use CPU
    ↓
Create appropriate Collection type
    ↓
Store in DashMap<String, CollectionType>
```

**Search Operation:**
```
User calls search()
    ↓
Match on CollectionType:
    ├─ Cpu → HNSW CPU search
    └─ HiveGpu → GPU-accelerated search
    ↓
Return SearchResults
```

### Error Handling Strategy

```rust
// Graceful degradation at each level
1. GPU Context Creation Fails
   → Log warning
   → Fall back to CPU
   → Continue operation

2. GPU Search Fails
   → Log error with backend info
   → Return error to user
   → Don't fallback mid-operation (collection is GPU or CPU)

3. GPU Out of Memory
   → Clear error message
   → Suggest reducing batch size
   → Don't crash process
```

## Risks / Trade-offs

### Risk 1: CUDA Installation Complexity

**Risk:** CUDA requires driver installation, library paths, version matching

**Mitigation:**
- Comprehensive installation guide per platform
- Auto-detect CUDA availability with clear error messages
- Fallback to WebGPU if CUDA unavailable
- Docker images with CUDA pre-installed

**Trade-off:** Better performance vs setup complexity

### Risk 2: WebGPU Compatibility

**Risk:** WebGPU is newer, may have driver issues or missing features

**Mitigation:**
- Use WebGPU as last resort (after CUDA/Metal)
- Test on common GPU vendors (NVIDIA, AMD, Intel)
- Document known limitations
- Keep CPU fallback always available

**Trade-off:** Universal compatibility vs bleeding-edge issues

### Risk 3: GPU Memory Management

**Risk:** Large collections may exceed GPU VRAM, causing OOM errors

**Mitigation:**
- hive-gpu handles memory management internally
- Add configuration for batch sizes
- Monitor VRAM usage via metrics
- Clear error messages when VRAM exhausted

**Trade-off:** Unlimited scale vs hardware constraints

### Risk 4: Mixed Collection Types

**Risk:** Some collections on GPU, others on CPU may confuse users

**Mitigation:**
- Add `gpu_backend` field to collection metadata
- Log backend selection during creation
- Dashboard shows GPU usage per collection
- Allow manual backend selection via config (future)

**Trade-off:** Flexibility vs simplicity

### Risk 5: Testing Across Backends

**Risk:** Hard to test all backends (need physical GPU hardware)

**Mitigation:**
- Mock GPU context for unit tests
- CI tests on CPU with fallback
- Integration tests on GPU runners (GitHub Actions)
- Community testing for exotic GPUs

**Trade-off:** Test coverage vs infrastructure cost

## Migration Plan

### Phase 1: Add Multi-Backend Support (Week 1)
1. Implement `GpuDetector` module
2. Extend `GpuAdapter` for CUDA/WebGPU
3. Update `VectorStore::create_collection()`
4. **Validation:** Existing collections still work, new ones detect GPU

### Phase 2: Optimize GPU Operations (Week 2)
1. Add batch operations
2. Add GPU metrics
3. Performance benchmarks
4. **Validation:** >10x speedup on GPU vs CPU

### Phase 3: Documentation & Polish (Week 3)
1. User documentation
2. Setup guides per platform
3. Troubleshooting guide
4. **Validation:** Users can self-serve GPU setup

### Rollback Plan

If GPU multi-backend causes issues:

1. **Immediate rollback:**
   ```rust
   // Revert to macOS-only Metal
   #[cfg(all(feature = "hive-gpu", target_os = "macos"))]
   ```

2. **Disable via config:**
   ```yaml
   gpu:
     enabled: false  # Force CPU-only mode
   ```

3. **Feature flag:**
   ```bash
   cargo build --no-default-features  # Disable all GPU
   ```

**Impact:** Return to current state (macOS-only GPU)

## Performance Expectations

### Search Latency

| Configuration | Current | Target | Improvement |
|---------------|---------|--------|-------------|
| macOS + Metal | 2-5ms | 2-5ms | ✅ Same |
| Linux + CPU | 20-50ms | 2-5ms | **10x faster** |
| Linux + CUDA | N/A | 0.5-2ms | **40x faster** |
| Windows + CPU | 20-50ms | 2-5ms | **10x faster** |
| Windows + CUDA | N/A | 0.5-2ms | **40x faster** |

### Batch Operations

| Operation | CPU | GPU (CUDA) | Speedup |
|-----------|-----|------------|---------|
| Insert 10K vectors | 5s | 50ms | **100x** |
| Search 100 queries | 2s | 5ms | **400x** |
| Index 1M vectors | 10min | 30s | **20x** |

### Memory Usage

| Backend | VRAM Overhead | Note |
|---------|---------------|------|
| CPU | 0 MB | Baseline |
| Metal | +20% | Metal framework |
| CUDA | +15% | CUDA runtime |
| WebGPU | +10% | Smallest overhead |

## Open Questions

1. **Q:** Should we support manual backend selection?
   **A:** Yes, add config option in Phase 7 (low priority)

2. **Q:** What about ROCm for AMD GPUs on Linux?
   **A:** Future work, WebGPU covers AMD for now

3. **Q:** How to handle GPU driver updates mid-operation?
   **A:** Treat as crash, require restart (acceptable)

4. **Q:** Should batch size be auto-tuned per GPU?
   **A:** Start with fixed default (1000), add auto-tuning later

5. **Q:** GPU for quantization operations?
   **A:** Out of scope, focus on search first

## References

- hive-gpu documentation: https://github.com/hivellm/hive-gpu
- CUDA best practices: https://docs.nvidia.com/cuda/cuda-c-best-practices-guide/
- Metal Performance Shaders: https://developer.apple.com/metal/
- WebGPU spec: https://www.w3.org/TR/webgpu/
- Previous analysis: `docs/GPU_INTEGRATION_ANALYSIS.md`

## Success Metrics

After implementation:

- [ ] GPU backend detected on Linux/Windows (not just macOS)
- [ ] Search performance >10x faster with GPU vs CPU
- [ ] Batch operations >100x faster with GPU
- [ ] Zero API breaking changes
- [ ] All existing tests pass
- [ ] 95%+ test coverage for new code
- [ ] Documentation complete with setup guides
- [ ] Performance benchmarks documented
- [ ] Prometheus metrics for GPU usage
- [ ] Graceful CPU fallback when GPU unavailable


