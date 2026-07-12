# Hot-Path Spec — Lock Contention, Quantization Wiring, SIMD

## ADDED Requirements

### Requirement: Bounded HNSW write-lock scope

`insert_batch` MUST NOT hold the HNSW index write lock across
payload indexing, sparse indexing, quantization, or graph discovery.
The write lock SHALL cover only index mutation calls.

#### Scenario: Search latency under concurrent batch insert

Given a collection receiving a 1000-vector `insert_batch`
When concurrent `search` requests arrive during the batch
Then each search MUST acquire the index read lock without waiting
for the full batch to complete

### Requirement: Insert path avoids redundant vector copies

The insert path MUST NOT clone a vector's data array more than once
per inserted vector.

#### Scenario: Bulk insert allocation profile

Given a batch of 768-dimension vectors
When `insert_batch` executes
Then at most one copy of each vector's f32 data is made end-to-end

### Requirement: PQ and Binary quantization usable through HNSW

The HNSW quantization integration MUST accept
`QuantizationType::Product` and `QuantizationType::Binary` in
addition to `Scalar`. Unsupported types MUST produce an explicit
error, never a silent fallback.

#### Scenario: PQ collection round-trip

Given a collection configured with Product quantization
When vectors are inserted and searched
Then the search MUST execute against PQ-quantized data and return
results (no `Unsupported` error, no silent 8-bit substitution)

#### Scenario: Unsupported type is an error

Given a quantization type not implemented by the integration
When index construction runs
Then construction MUST fail with an explicit unsupported-type error

### Requirement: SIMD quantize kernels

`quantize_f32_to_u8` and `dequantize_u8_to_f32` MUST have AVX2 and
NEON implementations, and `int8_dot_product` MUST have an AVX2
implementation. All SIMD implementations MUST match the scalar
oracle bit-exactly or within documented tolerance.

#### Scenario: Oracle agreement per backend

Given each available SIMD backend forced via `VECTORIZER_SIMD_BACKEND`
When the scalar-oracle test suite runs
Then every backend's output MUST agree with the scalar reference

### Requirement: CI-verified SIMD correctness

CI MUST run the scalar-oracle suite against every forceable SIMD
backend on at least one x86_64 and one aarch64 target.

#### Scenario: Divergent intrinsic caught

Given a SIMD kernel change that diverges from the scalar oracle
When CI runs
Then the SIMD matrix job MUST fail

### Requirement: Registered hot-path benchmarks

The bench suite MUST include compiling, runnable benchmarks covering
insert pipeline, end-to-end search, query cache, quantization, and
BM25 vocabulary build.

#### Scenario: Bench suite compiles

Given the workspace at HEAD
When `cargo bench --no-run` executes
Then all registered benches MUST compile
