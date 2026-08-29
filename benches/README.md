# benches/

Performance benchmarks for Vectorizer. Two flavours coexist here — both use the same `benches/` root so there's a single home for performance-related code.

## Running

```bash
# Cargo Criterion microbenchmarks (filter, simd, multi_tenant_overhead, ...)
cargo bench
cargo bench --bench filter_benchmark
cargo bench --bench simd_dot_product

# Standalone perf-tool binaries (registered as [[bin]] with required-features = ["benchmarks"])
cargo run --release --features benchmarks --bin benchmark_grpc_vs_rest
```

Cross-engine comparisons do **not** live here. They run the upstream
[qdrant/vector-db-benchmark](https://github.com/qdrant/vector-db-benchmark) from
`benchmarks/external/` — see
[external-benchmarks.md](../docs/development/external-benchmarks.md). A
home-grown comparison harness is what produced the retracted
`docs/specs/benchmarks/qdrant_comparison_2025-11-24_*` report, which declared a
5.31x search win at 0.00% recall.

The list of currently-active `[[bench]]` and `[[bin]]` entries lives in [`Cargo.toml`](../Cargo.toml). A handful of additional `[[bin]]` entries are present but commented out — they need refactoring against the current API and stay disabled until that work happens.

## Layout

| Path | Style | What it does |
|---|---|---|
| `multi_tenant_overhead.rs` | Criterion `[[bench]]` | Multi-tenant request-pipeline overhead |
| `filter/filter_benchmark.rs` | Criterion `[[bench]]` | Vector-filter primitive |
| `simd/{dot_product,euclidean,cosine,l2_norm,manhattan,quantize}.rs` | Criterion `[[bench]]` | SIMD per-op benchmarks. Baselines committed under `simd/baselines/` and compared by [`scripts/simd/check-regression.sh`](../scripts/simd/check-regression.sh). |
| `grpc/benchmark_grpc_vs_rest.rs` | `[[bin]]` perf tool | End-to-end gRPC vs REST comparison |
| `core/`, `embeddings/`, `gpu/`, `performance/`, `quantization/`, `replication/`, `scripts/`, `search/`, `storage/`, `tests/` | `[[bin]]` perf tools (currently disabled) | Topic-grouped perf programs awaiting an API refresh |
| `reports/` | Markdown + JSON snapshots | Curated runs of past benchmarks (`combined_optimization_<timestamp>.md`, etc.) |

## When to add what

- **Criterion `[[bench]]`** for per-function statistical timing. Add a new `.rs` under the matching topic subdir, register a `[[bench]]` entry in `Cargo.toml`, run `cargo bench --bench <name>`. SIMD work also commits a baseline JSON.
- **`[[bin]]` perf tool** when you need to spin up the full server, hit a real workload, and emit a markdown report. Add the `.rs` under the matching topic, register a `[[bin]]` entry with `required-features = ["benchmarks"]`, run via `cargo run --release --features benchmarks --bin <name>`. Drop the report into `reports/`.
