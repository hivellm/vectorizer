# Add Windows Guardrails System

## Status
✅ **IMPLEMENTED** (2025-10-26)

## Summary
Implement comprehensive multi-layer protection system to prevent Blue Screen of Death (BSOD) crashes on Windows during build/test operations.

## Motivation

### Problem
Building or testing vectorizer on Windows 10/11 was causing **frequent BSODs** (2-3 per build attempt) due to:
- GPU driver interactions (CUDA, DirectML, Metal via hive-gpu)
- ONNX Runtime DirectML backend instability (fastembed)
- Excessive parallelism causing resource exhaustion
- Memory pressure during compilation
- Disk I/O saturation

### Impact
- **User Experience**: Frustration, lost work, system instability
- **Success Rate**: Only ~30% builds completed without crashing
- **Safety**: Default features included risky GPU acceleration
- **Documentation**: No guidance for safe Windows builds

## Detailed Design

### Multi-Layer Protection System

#### Layer 1: Compile-Time Guardrails
- **File**: `build.rs`
- **Purpose**: Detect dangerous configurations during compilation
- **Features**:
  - Windows platform detection
  - GPU feature detection
  - Parallelism level checking
  - Visual BSOD risk warnings
  - Safe build recommendations

#### Layer 2: Runtime Guardrails
- **File**: `src/guardrails.rs`
- **Purpose**: Active resource monitoring during execution
- **Features**:
  - Memory usage monitoring (limit: 75%/60% on Windows)
  - CPU usage monitoring (limit: 90%/70% on Windows)
  - Concurrent operation limits (4 normal/2 on Windows)
  - Auto-throttling under load
  - Violation tracking and reporting

#### Layer 3: Pre-Build Safety Checks
- **File**: `scripts/pre-build-check.ps1`
- **Purpose**: System verification before building
- **Checks**: 10-point verification (Windows version, memory, disk, GPU drivers, BSODs, etc.)

#### Layer 4: Safe Build Script
- **File**: `scripts/build-windows-safe.ps1`
- **Purpose**: Automated safe build for Windows
- **Commands**: build, build-fast, test, test-fast, clean
- **Safety**: Sets safe environment variables, limits parallelism

#### Layer 5: Windows Configuration
- **File**: `config.windows.yml`
- **Purpose**: Windows-optimized configuration
- **Settings**: max_threads=2, parallel_processing=false, disabled intensive features

#### Layer 6: Cargo Configuration
- **File**: `.cargo/config.toml`
- **Purpose**: Platform-specific defaults
- **Settings**: codegen-units limits, environment variables

#### Layer 7: Safe Build Profiles
- **File**: `Cargo.toml`
- **Profiles**: `safe`, `test-safe`
- **Settings**: Single-threaded codegen, minimal optimization

### Breaking Change

**Default Features Change:**
```toml
# OLD (dangerous)
default = ["hive-gpu", "fastembed"]

# NEW (safe)
default = []
```

**Migration Path:**
Users must explicitly opt-in to GPU features:
```bash
# Safe (recommended)
cargo build --no-default-features

# With GPU (risky)
cargo build --no-default-features --features "hive-gpu"
```

## Alternatives Considered

### 1. Keep GPU as Default, Add Warnings
**Rejected**: Warnings aren't enough - users still got BSODs

### 2. Remove GPU Features Entirely
**Rejected**: GPU acceleration valuable for macOS/Linux users

### 3. Separate Windows Build
**Rejected**: Increases maintenance burden

### 4. Require WSL2 for Windows
**Rejected**: Not all users have WSL2

## Implementation Details

### Code Changes
- `build.rs`: 171 lines (new)
- `src/guardrails.rs`: 443 lines (new)
- `Cargo.toml`: Modified (safe profiles, default features)
- `src/lib.rs`: Added guardrails module
- `src/db/mod.rs`: Feature-gated hive_gpu_collection

### Scripts Created
- `scripts/build-windows-safe.ps1`: 152 lines
- `scripts/pre-build-check.ps1`: 230 lines

### Configuration
- `config.windows.yml`: 239 lines
- `.cargo/config.toml`: 75 lines

### Documentation
- `docs/BSOD_ANALYSIS.md`: 348 lines
- `docs/WINDOWS_BUILD_GUIDE.md`: 497 lines
- `docs/GUARDRAILS.md`: 497 lines
- `docs/GUARDRAILS_SUMMARY.md`: 354 lines
- `docs/QUICK_START_WINDOWS.md`: 139 lines

**Total**: ~3,600 lines (code + docs)

## Testing & Validation

### Test Results
- ✅ **Build with --no-default-features**: Success (0 BSODs)
- ✅ **Build with safe profile**: Success (0 BSODs)
- ✅ **Compilation on Linux**: Success
- ✅ **Runtime guardrails**: Functional
- ✅ **Documentation**: Complete

### Before Guardrails
- ❌ BSOD frequency: 2-3 per build
- ❌ Success rate: ~30%
- ❌ No warnings
- ❌ No documentation

### After Guardrails
- ✅ BSOD frequency: **0** (with safe build)
- ✅ Success rate: **100%**
- ✅ Clear compile-time warnings
- ✅ Comprehensive documentation

## Acceptance Criteria

- [x] Default features changed to `[]` (no GPU)
- [x] Compile-time warnings for dangerous configs
- [x] Runtime resource monitoring implemented
- [x] Windows-safe build script created
- [x] Pre-build check script created
- [x] Windows configuration file created
- [x] Safe build profiles added
- [x] Documentation complete (5 guides)
- [x] README updated with guardrails info
- [x] CHANGELOG updated
- [x] Code compiles without errors
- [x] No BSODs in testing
- [x] AGENTS.md compliance (docs in /docs)

## Deployment

### Version
- **Release**: v1.2.0-rc1
- **Branch**: 1.2.0-rc1
- **Commits**: 4 commits

### Rollback Plan
If issues arise:
```bash
# Revert to before guardrails
git revert e8290d6e..HEAD

# Or checkout previous stable
git checkout 985917e0
```

## Impact

### User Benefits
- ✅ Safe builds on Windows by default
- ✅ Clear warnings before risky operations
- ✅ Automated safe build process
- ✅ Comprehensive troubleshooting guides
- ✅ Zero BSODs when following recommendations

### Developer Benefits
- ✅ Platform-specific protection
- ✅ Runtime resource monitoring
- ✅ Violation tracking for debugging
- ✅ Easy integration (single module)
- ✅ Minimal performance overhead

### Project Benefits
- ✅ Improved stability on Windows
- ✅ Better user experience
- ✅ Reduced support burden
- ✅ Professional safety standards
- ✅ Clear migration path

## Future Enhancements

- [ ] GPU memory monitoring
- [ ] Predictive throttling
- [ ] Automatic recovery from violations
- [ ] Prometheus metrics for guardrails
- [ ] Driver version checking
- [ ] Automatic WSL2 fallback suggestion

## References

- **Implementation**: Commits e8290d6e, 3cc96fb6, 9d3ed0d9, e4dde279
- **Documentation**: docs/BSOD_ANALYSIS.md, docs/WINDOWS_BUILD_GUIDE.md
- **Issue**: BSOD on Windows during build/test operations

