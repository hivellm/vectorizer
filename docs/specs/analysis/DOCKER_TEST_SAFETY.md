# BSOD Analysis and Docker Test Solution

## Problem: Blue Screen of Death (BSOD) During Tests

### Root Causes Identified

1. **GPU Driver Instability**
   - GPU acceleration features (`hive-gpu`, `cuda`, `metal-native`) can trigger unstable drivers
   - Windows GPU drivers are particularly sensitive to intensive workloads
   - CUDA/compute operations can cause kernel-mode crashes

2. **Parallel Build Stress**
   - Default `cargo test` uses all CPU cores
   - High parallel compilation can stress system resources
   - Memory-intensive link operations during tests

3. **Memory Pressure**
   - Vector operations allocate large memory chunks
   - Test fixtures create multiple large data structures
   - Stack overflows in deeply nested operations

4. **ONNX Runtime Issues**
   - `fastembed` with ONNX can trigger driver crashes
   - ML model loading stresses GPU memory
   - Interaction between ONNX and Windows drivers

### BSOD Symptoms

```
STOP: 0x000000D1 (DRIVER_IRQL_NOT_LESS_OR_EQUAL)
STOP: 0x0000003B (SYSTEM_SERVICE_EXCEPTION)
STOP: 0x000000C5 (DRIVER_CORRUPTED_EXPOOL)
```

Common in:
- GPU-intensive tests
- Parallel test execution
- Memory-intensive benchmarks

## Solution: Docker-Isolated Testing

### Why Docker Prevents BSODs

1. **Kernel Isolation**
   - Container runs in isolated Linux kernel
   - GPU crashes contained within container
   - Host system protected from kernel panics

2. **Resource Control**
   - CPU/memory limits prevent resource exhaustion
   - Controlled parallel execution
   - Predictable resource allocation

3. **Safe Feature Set**
   - Tests run with minimal features (no GPU)
   - Single-threaded execution (`test-safe` profile)
   - Conservative optimization levels

### Docker Test Configuration

#### Safe Profile (`Cargo.toml`)

```toml
[profile.test-safe]
inherits = "test"
codegen-units = 1        # Single-threaded codegen
opt-level = 0            # No optimization
debug = "line-tables-only"
```

#### Environment Variables

```bash
CARGO_BUILD_JOBS=1       # Single-threaded build
CARGO_TEST_THREADS=1     # Single-threaded tests
RUST_TEST_THREADS=1      # Rust runtime single-threaded
RUST_MIN_STACK=8388608   # 8MB stack (prevent overflows)
```

#### Resource Limits

```yaml
deploy:
  resources:
    limits:
      cpus: '2.0'        # Limit CPU usage
      memory: 4G         # Cap memory
    reservations:
      cpus: '1.0'
      memory: 2G
```

## Usage

### PowerShell (Windows - Recommended)

```powershell
# Run all tests safely
.\scripts\test-docker.ps1

# Run without rebuilding
.\scripts\test-docker.ps1 -NoBuild

# Run with coverage
.\scripts\test-docker.ps1 -Coverage

# Run specific tests
.\scripts\test-docker.ps1 -Filter integration

# Verbose output
.\scripts\test-docker.ps1 -Verbose
```

### Bash (WSL/Linux)

```bash
# Run all tests safely
./scripts/test-docker.sh

# Run without rebuilding
./scripts/test-docker.sh --no-build

# Run with coverage
./scripts/test-docker.sh --coverage

# Run specific tests
./scripts/test-docker.sh --filter integration

# Verbose output
./scripts/test-docker.sh --verbose
```

### Direct Docker Compose

```bash
# Build and run tests
docker-compose -f docker-compose.test.yml up --build

# Run tests without building
docker-compose -f docker-compose.test.yml run --rm vectorizer-test

# Interactive shell
docker-compose -f docker-compose.test.yml run --rm vectorizer-test bash
```

## Test Output

### Logs Location

- `target/test-results/build.log` - Build output
- `target/test-results/test.log` - Test execution log
- `coverage/lcov.info` - Coverage report (with --coverage)
- `logs/` - Runtime logs

### Reading Results

```bash
# View test results
cat target/test-results/test.log

# View build errors
cat target/test-results/build.log

# Coverage summary (if generated)
cat coverage/lcov.info
```

## Advanced Testing

### Custom Test Commands

```bash
# Run only unit tests
docker-compose -f docker-compose.test.yml run --rm vectorizer-test \
  cargo test --lib --profile test-safe -- --test-threads=1

# Run only integration tests
docker-compose -f docker-compose.test.yml run --rm vectorizer-test \
  cargo test --test '*' --profile test-safe -- --test-threads=1

# Run specific test
docker-compose -f docker-compose.test.yml run --rm vectorizer-test \
  cargo test test_search --profile test-safe -- --test-threads=1 --nocapture
```

### Debug Failed Tests

```bash
# Interactive mode
docker-compose -f docker-compose.test.yml run --rm vectorizer-test bash

# Inside container:
cargo test --profile test-safe -- --test-threads=1 --nocapture
```

### Generate Coverage

```bash
# With helper script
.\scripts\test-docker.ps1 -Coverage

# Or manually
docker-compose -f docker-compose.test.yml run --rm vectorizer-test \
  bash -c "cargo llvm-cov test --profile test-safe -- --test-threads=1 && \
           cargo llvm-cov report --lcov --output-path /vectorizer/coverage/lcov.info"
```

## Comparison: Native vs Docker Testing

| Aspect | Native Windows | Docker Container |
|--------|----------------|------------------|
| **Safety** | ⚠️ Can cause BSOD | ✅ Host protected |
| **Speed** | ⚡ Fast | 🐢 Slower (isolation overhead) |
| **GPU Tests** | ❌ Dangerous | ✅ Safe (no GPU access) |
| **Parallelism** | ⚡ All cores | 🔒 Limited (1-2 threads) |
| **Stability** | ⚠️ System stress | ✅ Isolated |
| **Debug** | ✅ Native tools | ⚠️ Container tools |

## When to Use Each Approach

### Use Docker Testing When:

✅ Running GPU-related tests
✅ Testing memory-intensive operations
✅ Running full test suite
✅ CI/CD pipeline
✅ Experienced BSODs before
✅ Testing on Windows
✅ Need isolation and safety

### Use Native Testing When:

✅ Quick iteration on single test
✅ Debugging with IDE
✅ Performance benchmarking (need native speed)
✅ Testing Windows-specific features
✅ Stable tests (no GPU/memory stress)

## Best Practices

1. **Always use Docker for full test runs**
2. **Native testing only for rapid iteration**
3. **Never enable GPU features on Windows without Docker**
4. **Monitor resource usage during tests**
5. **Keep Docker Desktop updated**
6. **Use WSL2 backend for better performance**

## Troubleshooting

### Docker Build Fails

```bash
# Clean and rebuild
docker-compose -f docker-compose.test.yml down
docker system prune -a
docker-compose -f docker-compose.test.yml build --no-cache
```

### Tests Timeout

Increase timeout in `docker-compose.test.yml`:

```yaml
deploy:
  resources:
    limits:
      cpus: '4.0'  # More CPU
      memory: 8G   # More memory
```

### Out of Memory

```yaml
environment:
  - CARGO_BUILD_JOBS=1
  - CARGO_TEST_THREADS=1
  - RUST_MIN_STACK=4194304  # Reduce stack (4MB instead of 8MB)
```

## Safety Checklist

Before running tests locally (outside Docker):

- [ ] Latest GPU drivers installed
- [ ] No GPU features enabled (`--no-default-features`)
- [ ] Single-threaded execution (`CARGO_TEST_THREADS=1`)
- [ ] Memory monitoring active
- [ ] Backup important data
- [ ] System stable and not under load
- [ ] No other intensive applications running

**If any of the above is uncertain, USE DOCKER.**

## Conclusion

Docker-based testing provides:

✅ **Safety**: Prevents BSODs and system crashes
✅ **Isolation**: Container crashes don't affect host
✅ **Reproducibility**: Consistent test environment
✅ **Resource Control**: Predictable resource usage
✅ **Peace of Mind**: Test without fear of crashes

The small performance overhead is worth the safety and stability gains.

