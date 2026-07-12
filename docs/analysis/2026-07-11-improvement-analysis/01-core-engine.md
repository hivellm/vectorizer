# §1 — Core Engine

> Scope: `crates/vectorizer/src/{db,persistence,storage,quantization,normalization,cache}`
> + `crates/vectorizer-core`. All refs verified at analysis time
> (release/3.5.0, post-phase36).

## 1.1 Architecture debt — the umbrella-crate back-reference map

The workspace split (phase4) stalled because low-level modules reach
*up* into service modules. The nine precise couplings:

| Consumer → Provider | Location | Symbol pulled |
|---|---|---|
| `db` → `cluster` | `db/distributed_sharded_collection.rs:16` | `ClusterClientPool, ClusterManager, DistributedShardRouter, NodeId` |
| `db` → `monitoring` | `db/ttl_reaper.rs:27` | `metrics::METRICS` |
| `cache` → `monitoring` | `cache/query_cache.rs:157` | `metrics::METRICS` |
| `config` → `auth` | `config/vectorizer.rs:7` | `AuthConfig` |
| `config` → `hub` | `config/vectorizer.rs:9` | `HubConfig` |
| `config` → `cluster` | `config/vectorizer.rs:39,663` | `ClusterConfig` |
| `cluster` → `hub` | `cluster/server_client.rs:14,460` | `TenantContext` |
| `hub` → `monitoring` | `hub/quota.rs:17` | `METRICS` |
| `auth` → `monitoring` | `auth/mod.rs:298,329,337` | `ApiKeyUsageRecorder` |

**Fix strategy (inversion):**

1. `monitoring::METRICS` becomes a `MetricsSink` trait injected into
   db/cache/hub/auth — removes 4 of 9 back-refs in one move.
2. `config` holds generic serde sub-structs (or newtypes owned by
   config); auth/hub/cluster parse them — breaks the
   config→{auth,hub,cluster} triangle.
3. `db → cluster` needs a `ShardRouter` trait in `vectorizer-core` so
   `distributed_sharded_collection` depends on an abstraction.

## 1.2 Oversized files (>700 lines in scope)

| File | Lines | Mixed concerns |
|---|---|---|
| `db/vector_store/collections.rs` | 1050 | lifecycle + alias resolution + lazy disk-load + multi-tenancy + disk listing |
| `db/distributed_sharded_collection.rs` | 1028 | sharding logic + cluster RPC + routing + rebalance |
| `db/collection_tests.rs` | 925 | test file (acceptable) |
| `storage/snapshot.rs` | 884 | .vecdb archive create/restore + fs walk + metadata |
| `db/vector_store/persistence.rs` | 884 | compact + legacy `.bin` load + HNSW cache-path + snapshots |
| `db/hive_gpu_collection.rs` | 771 | GPU wrapper + data layout + fallbacks |
| `db/async_indexing.rs` | 729 | progress state + build task + index swap |
| `db/payload_index.rs` | 725 | secondary index + query filters |
| `persistence/wal.rs` | 704 | WAL append/read/recover/checkpoint + JSON codec |

Split `collections.rs` / `persistence.rs` along the doc-comment seams
already present (lifecycle vs. disk-load vs. tenancy).

## 1.3 Dead code / feature flags

- **Commented `[[bin]]` blocks** — `crates/vectorizer/Cargo.toml:261-372,450-463`:
  ~20 dead stanzas. Delete.
- **Non-existent features referenced** in those comments: `wgpu-gpu`
  (`:453`), `hive-gpu-metal` (`:357`), `hive-gpu-wgpu` (`:372`) — no
  matching `[features]` entry. Misleading.
- **Deprecated aliases** `metal-native`, `gpu-accel` (`:404-405`) both
  just `= ["hive-gpu"]`. No in-tree consumer; remove after external
  audit.
- Optional embedding deps (`candle-*`, `ort`, `arrow`, `parquet`) are
  off-by-default and correctly gated — fine.

## 1.4 Persistence layer — fragile spots

| Sev | Location | Issue | Fix |
|---|---|---|---|
| **HIGH** | `persistence/wal.rs:198,217` | WAL only `file.flush()` — no `sync_all()`/`sync_data()`. Data sits in the OS page cache; power loss drops entries the caller believes durable. | fsync after flush (or `O_DSYNC`); make configurable. |
| **HIGH** | `persistence/wal.rs:222-226` | WAL is JSON-lines with **no checksum / length prefix**. A torn final write on crash yields a partial line; `serde_json::from_str` (`:145`) errors and aborts recovery of **all later entries**. | Per-record CRC32 (crate already deps `crc32fast`) + length framing; skip trailing partial. |
| **MED** | `persistence/wal.rs:179` vs `:194,201` | `append_transaction` loads `base_sequence` (Relaxed) *before* taking the file lock; two concurrent txns compute the same base → overlapping sequence numbers. | Compute sequence under the file lock, or `fetch_add(n)` up-front. |
| **LOW** | `persistence/wal.rs:149` | After recover, `sequence.store(max_sequence)`; next append `fetch_add(1)` returns `max_sequence`, duplicating the last entry's seq. | store `max_sequence + 1`. |

WAL `unwrap()`s (32×) are all in `#[cfg(test)]` (`:470+`) — not a
concern. `snapshot.rs` `.ok()` uses (`:726-730`) are mtime metadata
reads — benign.

## 1.5 Concurrency smells

- **Lock-library mixing**: `db/auto_save.rs:7` and
  `db/wal_integration.rs` use `tokio::sync::RwLock` while the rest of
  `db` uses `parking_lot`. `auto_save` holds these across `.await`
  (11 async-lock sites) — correct for tokio locks, but the
  inconsistency invites future parking_lot-guard-across-await bugs.
  Standardize: parking_lot for short critical sections, tokio locks
  only where held across await; document which is which.
- **DashMap re-entrancy trap**: `collections.rs` `get_collection`
  returns a `Ref`; `get_collection_mut` (`:731`) takes a write
  `RefMut` on the same map. A caller holding a `Ref` that then calls
  `get_collection_mut` on the same shard **deadlocks**. Document
  "never hold a Ref across a mut call," or route mutations through
  `alter`/`entry`.

## 1.6 Quantization / HNSW

| Sev | Location | Issue | Fix |
|---|---|---|---|
| **MED** | `quantization/hnsw_integration.rs:74-83` | HNSW integration accepts **only** `QuantizationType::Scalar`; `_ => Err(Unsupported)`. `product.rs` (PQ, 512 ln) and `binary.rs` (412 ln) are fully implemented but **never wired into HNSW**. | Add PQ/Binary arms; box the trait object generically. |
| **LOW** | `quantization/hnsw_integration.rs:122-125` | On first fit, non-Scalar silently falls back to hardcoded 8-bit instead of erroring — dead-branch masking. | Unreachable/error after the above is fixed. |
| **LOW** | `quantization/scalar.rs:152-153,196-197` | 4-/2-bit values stored one-per-`u8` (no bit-packing at this layer), so the advertised "4x compression" isn't realized here. | Verify packing happens in `storage.rs`; else pack. |

No `TODO/FIXME/unimplemented!` markers exist in `db` or
`quantization` — the gaps are structural (missing wiring), not
annotated debt.

## Priorities

1. **WAL fsync + checksums** — silent data-loss risk (→ phase37).
2. **`MetricsSink` trait inversion** — unblocks the crate split,
   kills 4 of 9 back-refs (→ phase41).
3. **Wire PQ/Binary into HNSW** or remove the modules (→ phase38).
