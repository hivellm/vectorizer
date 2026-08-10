# Proposal: phase5_external-benchmark-comparison

Benchmark Vectorizer against Qdrant, Weaviate and pgvector on a footing a
reader outside this repository can trust.

Branch: `bench/external-comparison`.

## Why

There is already a comparison in the tree, `docs/specs/benchmarks/
qdrant_comparison_2025-11-24_20-45-29.md`, and it is not usable. Its headline
table reads:

| Metric | Vectorizer | Qdrant | Winner |
|---|---|---|---|
| Search latency | 0.16ms | 0.84ms | **Vectorizer, 5.31x** |
| Precision@10 | **0.00%** | 100.00% | Qdrant |
| Recall@10 | **0.00%** | 100.00% | Qdrant |

It declares Vectorizer the search winner at **zero recall**. A search that
returns nothing relevant is arbitrarily fast; the latency column measures
nothing once recall is zero. Whether the harness computed recall wrongly or
the engine genuinely returned nothing, the report cannot tell you which — and
it is committed under `docs/specs/` where it reads as authoritative.

The harness that produced it, `benches/comparison/qdrant_comparison_benchmark.rs`,
is not a declared bench target. The crate wires 17 benches through explicit
`[[bench]]` entries and this file is not among them, so nothing builds or runs
it. A benchmark nobody runs, publishing a result nobody can reproduce, is
worse than no benchmark: it is a number people will quote.

Vectorizer also has no external comparison anyone else can rerun. Qdrant,
Weaviate, Redis and Milvus all publish against a shared framework; a
self-built harness scoring its own engine is not evidence, however careful.

## What Changes

**Add Vectorizer as an engine to `qdrant/vector-db-benchmark`** rather than
writing another in-house harness.

That framework already ships engine clients for `qdrant`, `weaviate` and
`pgvector` — precisely the three named — plus Milvus, Redis, Elasticsearch and
OpenSearch. It handles the parts the previous attempt got wrong: real datasets
with precomputed ground truth, recall computed against it, parallel client
load, and a configuration format the other vendors publish against. An engine
client is small: pgvector's is six files totalling about 7 KB
(`config`, `configure`, `upload`, `search`, `parser`).

The work:

1. A `vectorizer` engine client, in this repo under `benchmarks/external/`,
   plus a runner that clones the upstream framework at a **pinned commit** and
   overlays our client. Pinning matters — an unpinned upstream silently
   changes what the numbers mean between runs.
2. **Recall as a gate, not a column.** The runner refuses to emit a latency
   comparison when recall falls below a floor. This is the specific defect in
   the existing report, and a threshold is the only thing that prevents it
   recurring.
3. Retract the existing report — it is committed as a spec and is wrong — and
   either wire or delete the orphaned Rust harness. Leaving a dead file that
   produced a published number invites someone to run it again.
4. A runbook: how to bring up all four engines, which datasets, how to read
   the output, and what the numbers do **not** say.

Deliberately out of scope: tuning Vectorizer to win. The first honest run is
the deliverable, whatever it says. If we lose somewhere, that is a finding,
and the point of using a framework the competition publishes against is that
we cannot quietly grade ourselves.

## Impact

- Affected specs: retracts `docs/specs/benchmarks/qdrant_comparison_*`.
- Affected code: new `benchmarks/external/`; `benches/comparison/` removed or
  wired; possibly `docs/specs/BENCHMARKING.md` (which describes the internal
  criterion suite and should point at this for external comparison).
- Breaking change: NO — nothing ships in the server binary.
- User benefit: a number that survives being checked by someone who does not
  trust us, and an internal signal when a change costs recall rather than only
  latency.
