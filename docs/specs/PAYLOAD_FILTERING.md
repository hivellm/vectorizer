# Payload Filtering — Specification

Status: draft
Source of truth: `crates/vectorizer/src/db/payload_index.rs` (725 lines, 14 `pub fn`)
Related: [QDRANT_FILTERS.md](./QDRANT_FILTERS.md) — the Qdrant-compat filter DSL evaluated by `FilterProcessor`

## Overview

Payload filtering lets callers narrow vector search results by metadata
(arbitrary JSON stored alongside each vector in `Payload.data`) instead of
— or in addition to — vector similarity.

Two subsystems implement filtering in Vectorizer and they are currently
**decoupled**:

1. **`PayloadIndex`** (`src/db/payload_index.rs`) — an in-memory inverted /
   typed index keyed by field name. Built incrementally as vectors are
   inserted or updated. Exposes primitive lookup functions
   (`get_ids_for_keyword`, `get_ids_in_range`, `get_ids_in_geo_radius`, …)
   that return `HashSet<String>` of vector IDs.
2. **`FilterProcessor`** (`src/models/qdrant/filter_processor.rs`) — a
   post-search evaluator for Qdrant-style filter trees
   (`must` / `must_not` / `should` with `match`, `range`, `geo_*`,
   `values_count`, `nested`). Iterates candidate results and scans each
   `Payload` directly. Does **not** consult `PayloadIndex`.

The Qdrant compatibility layer documented in
[QDRANT_FILTERS.md](./QDRANT_FILTERS.md) is served by `FilterProcessor`.
`PayloadIndex` is built on every write, but its query methods are not
wired into any REST handler, MCP tool, or SDK surface today (see
[Limitations](#limitations)).

This document specifies the behaviour, storage, and API surface of the
`PayloadIndex` subsystem, and notes how/where it intersects with the
Qdrant-style filter DSL.

## Supported filter types

`PayloadIndexType` (lines 16–28) enumerates the index families:

| Variant    | Storage                                         | Query method(s)                                             | Query kind                                  |
|------------|-------------------------------------------------|-------------------------------------------------------------|---------------------------------------------|
| `Keyword`  | `HashMap<String, HashSet<VectorId>>`            | `get_ids_for_keyword(field, value)`                         | Exact string match                          |
| `Integer`  | `HashMap<VectorId, i64>`                        | `get_ids_in_range(field, min, max)`                         | Inclusive numeric range (`min..=max`)       |
| `Float`    | `HashMap<VectorId, f64>`                        | `get_ids_in_float_range(field, min, max)`                   | Inclusive numeric range (`min..=max`)       |
| `Text`     | `HashMap<Term, HashSet<VectorId>>` + id→text    | `search_text(field, query)`                                 | Tokenized AND-of-terms, case-insensitive    |
| `Geo`      | `HashMap<VectorId, (lat, lon)>`                 | `get_ids_in_geo_bounding_box(...)`, `get_ids_in_geo_radius(...)` | Rectangular or haversine-radius containment |

Notes:

- **Keyword index** coerces `i64` and `bool` payload values into their
  string representation before indexing (lines 529–535). Strings are
  indexed verbatim (case-sensitive).
- **Integer index** accepts both `i64` and `f64` JSON values; floats are
  truncated with `as i64` (lines 540–544).
- **Float index** accepts both `f64` and `i64` JSON values; integers are
  widened to `f64` (lines 549–553).
- **Text index** tokenisation is minimal: `to_lowercase()` + split on
  `!char::is_alphanumeric()` (lines 307–313). There is no stemming,
  stopword removal, synonym expansion, or ranking — matches have
  **AND semantics** (every query term must hit) and the result is an
  unranked set of vector IDs.
- **Geo index** supports two payload shapes: object `{ "lat": x, "lon": y }`
  and two-element array `[lat, lon]` (lines 566–578). Radius is in
  kilometres (see `haversine_distance`, lines 396–406). This differs from
  the Qdrant-compat DSL whose radius field is in metres — see
  [Qdrant compatibility](#qdrant-compatibility).

## Index configuration

### Programmatic configuration

A field is indexed by passing a `PayloadIndexConfig` to
`PayloadIndex::add_index_config` (lines 449–482):

```rust
use vectorizer::db::payload_index::{PayloadIndex, PayloadIndexConfig, PayloadIndexType};

let idx = PayloadIndex::new();
idx.add_index_config(PayloadIndexConfig::new(
    "category".to_string(),
    PayloadIndexType::Keyword,
));
idx.add_index_config(PayloadIndexConfig::new(
    "price".to_string(),
    PayloadIndexType::Float,
));
```

`PayloadIndexConfig` fields (lines 31–39):

- `field_name: String` — JSON key in `Payload.data`. Dot notation is
  supported for nested objects (e.g. `"user.profile.age"`, resolved by
  `get_nested_value`, lines 485–499).
- `index_type: PayloadIndexType` — one of the five variants above.
- `enabled: bool` (defaults to `true` in `new`) — when `false`,
  `index_vector` skips the field (lines 508–510) but the configured
  index remains allocated.

`add_index_config` is idempotent per `field_name` — the config map
overwrites, and the backing index is lazily created via
`entry().or_insert_with(...)` so re-adding the same config does **not**
wipe existing entries.

### Automatic configuration on collection creation

Every `Collection` auto-registers two fields at construction
(`src/db/collection/mod.rs` lines 154–164):

- `file_path` → `Keyword`
- `chunk_index` → `Integer`

These reflect the file-ingestion pipeline's dominant payload shape and
are not configurable via public API today.

### Lifecycle hooks

`Collection::data` invokes the index on every mutation
(`src/db/collection/data.rs`):

- Insert (line 105): `payload_index.index_vector(id, payload)`
- Update (lines 298–299): `remove_vector(id)` then `index_vector(id, payload)`
- Delete (line 350): `remove_vector(id)`

`index_vector` is a no-op for fields absent from the payload (line 521–524);
`remove_vector` sweeps every configured index (lines 589–611) so a caller
does not need to know which fields were populated.

### Cost model

All state is in-memory, heap-allocated, and duplicated per `VectorId`.
Each index reports an `estimate_memory` figure via
`PayloadIndexStats.memory_bytes` (see `get_stats`, lines 680–704). Rough
upper bounds per vector per indexed field:

| Index type | Memory per vector                                                    |
|------------|----------------------------------------------------------------------|
| Keyword    | ~`2 * size_of::<String>() + len(value_string)`                       |
| Integer    | ~`size_of::<String>() + 8` bytes                                     |
| Float      | ~`size_of::<String>() + 8` bytes                                     |
| Text       | proportional to `len(text) + sum(len(term)) + num_terms * size_of::<String>()` |
| Geo        | ~`size_of::<String>() + 16` bytes                                    |

Build time is O(1) amortised per inserted vector for keyword / integer /
float / geo (hashmap insertions) and O(T) for text, where T is the term
count of the field value. No batch-rebuild or snapshot APIs exist.

### Persistence

**None.** The `PayloadIndex` holds only in-memory `DashMap`s and is
rebuilt implicitly when vectors are reloaded (because every insert calls
`index_vector`). Check the collection's persistence layer — the index
itself is not serialised to `.vecdb` or mmap files.

## Filter DSL

`PayloadIndex` itself **has no query-language layer**. Its `pub fn`
methods (see [API surface](#api-surface)) are invoked directly with
primitive arguments.

The user-facing filter DSL used by REST / gRPC search endpoints is the
Qdrant-compat tree processed by `FilterProcessor::apply_filter`. Its
shape is defined in `src/models/qdrant/filter.rs` (`QdrantFilter`,
`QdrantCondition`, `QdrantRange`, `QdrantGeoPoint`, `QdrantValuesCount`)
and documented in full in [QDRANT_FILTERS.md](./QDRANT_FILTERS.md).
Skeleton:

```json
{
  "filter": {
    "must":     [ /* AND — all must match */ ],
    "must_not": [ /* NOT — none may match */ ],
    "should":   [ /* OR  — at least one must match */ ]
  }
}
```

Leaf conditions include `match` (string / int / bool / text:contains |
prefix | suffix | exact), `range` (`gt`/`gte`/`lt`/`lte` — exclusive or
inclusive), `geo_bounding_box`, `geo_radius` (**radius in metres**),
`values_count` (array/object cardinality) and `nested { filter }` for
recursion.

Because `FilterProcessor` walks each candidate payload, **it does not
require any `PayloadIndex` entry to be configured**; the index and the
DSL are currently independent. A future optimiser could intersect
`PayloadIndex` lookups with HNSW candidates before evaluating the
residual DSL tree — see [Limitations](#limitations).

## Usage examples

All examples below use `PayloadIndex` directly. For DSL examples hitting
the REST surface, see [QDRANT_FILTERS.md](./QDRANT_FILTERS.md).

### Filter by tag (keyword)

```rust
idx.add_index_config(PayloadIndexConfig::new("tag".into(), PayloadIndexType::Keyword));
idx.index_vector("v1".into(), &Payload { data: json!({ "tag": "news" }) });
idx.index_vector("v2".into(), &Payload { data: json!({ "tag": "sports" }) });

let news_ids: Option<HashSet<String>> = idx.get_ids_for_keyword("tag", "news");
// Some({"v1"})
```

Returns `None` when the field is not indexed, `Some(empty_set)` when the
field is indexed but no vector has that value.

### Range query (integer timestamp)

```rust
idx.add_index_config(PayloadIndexConfig::new("ts".into(), PayloadIndexType::Integer));
for (i, v) in vectors.iter().enumerate() {
    idx.index_vector(v.id.clone(), &Payload { data: json!({ "ts": i as i64 * 1000 }) });
}

// Closed interval [5000, 10000]
let ids = idx.get_ids_in_range("ts", Some(5000), Some(10_000));

// Open-ended lower bound (<= 10000)
let ids = idx.get_ids_in_range("ts", None, Some(10_000));
```

`get_ids_in_range(field, None, None)` returns every ID currently in the
integer index (lines 157–161).

### Range query (float price)

```rust
idx.add_index_config(PayloadIndexConfig::new("price".into(), PayloadIndexType::Float));
// …inserts…
let in_budget = idx.get_ids_in_float_range("price", Some(10.0), Some(99.99));
```

### Geo radius

```rust
idx.add_index_config(PayloadIndexConfig::new("loc".into(), PayloadIndexType::Geo));
idx.index_vector("store_1".into(),
    &Payload { data: json!({ "loc": { "lat": 40.7128, "lon": -74.0060 } }) });

// 5 km around Times Square — PayloadIndex uses kilometres
let nearby = idx.get_ids_in_geo_radius("loc", 40.758, -73.985, 5.0);
```

### Geo bounding box

```rust
let in_manhattan = idx.get_ids_in_geo_bounding_box(
    "loc",
    40.70,  // min_lat
    40.88,  // max_lat
    -74.02, // min_lon
    -73.91, // max_lon
);
```

### Text search (AND of tokens)

```rust
idx.add_index_config(PayloadIndexConfig::new("title".into(), PayloadIndexType::Text));
idx.index_vector("v1".into(), &Payload { data: json!({ "title": "Rust async runtime" }) });
idx.index_vector("v2".into(), &Payload { data: json!({ "title": "Python asyncio" }) });

let hits = idx.search_text("title", "rust async"); // {"v1"}
let none = idx.search_text("title", "rust python"); // {} (AND semantics)
```

### Combined filters (AND / OR / NOT)

`PayloadIndex` returns plain `HashSet<String>`s, so boolean composition
is just set arithmetic in caller code:

```rust
let news     = idx.get_ids_for_keyword("tag", "news").unwrap_or_default();
let priced   = idx.get_ids_in_float_range("price", Some(10.0), Some(100.0)).unwrap_or_default();
let archived = idx.get_ids_for_keyword("status", "archived").unwrap_or_default();

// must: news AND priced;  must_not: archived
let result: HashSet<&String> = news.intersection(&priced)
    .filter(|id| !archived.contains(*id))
    .collect();
```

The Qdrant-style DSL (`must`/`must_not`/`should` with `nested`) is only
available through `FilterProcessor` on `Payload` values directly, not
through `PayloadIndex` — see [Limitations](#limitations).

## Performance

### Build time

- Keyword / integer / float / geo insert: O(1) amortised hashmap writes.
- Text insert: O(T) where T = number of alphanumeric tokens in the value.
- Update = remove + reinsert. Remove is O(F) over all configured fields
  for keyword/integer/float/geo and O(T) for text.

### Query time

- `get_ids_for_keyword`: O(1) hashmap lookup; returns a *clone* of the
  underlying `HashSet<String>` (line 618 — `.cloned()`), so cost scales
  with match cardinality.
- `get_ids_in_range` / `get_ids_in_float_range`: **O(N)** linear scan
  over all indexed vectors for that field (lines 152–165, 199–212).
  There is no sorted structure; heavy range workloads on wide fields
  are expensive.
- `search_text`: O(Q · avg(|posting|)) where Q is the token count of
  the query; multi-term queries compute successive set intersections
  (lines 289–296).
- `get_ids_in_geo_bounding_box` / `get_ids_in_geo_radius`: **O(N)** over
  every coordinate in the field (lines 366–394). Haversine is computed
  per point for radius queries.

### Memory overhead

Every indexed vector adds storage in each configured index. For a
collection of N vectors with K indexed fields, total overhead is
O(N · K) with per-index costs as listed in [Cost model](#cost-model).

### Cardinality considerations

- **High-cardinality keyword fields** (e.g. unique IDs) bloat
  `value_to_ids` with one-element buckets — equivalent storage to a
  plain `HashMap<VectorId, String>` plus hash overhead.
- **Low-cardinality keyword fields** (e.g. `status ∈ {active, inactive}`)
  are the most memory-efficient and produce the largest bucket hits
  per query.
- **High-cardinality text fields** (long free-form documents) cause the
  posting-list sum to dominate memory. Tokenisation is naive (no
  stopwording) so "the", "a", "of" each build huge posting sets.
- **Geo radius scans** are purely linear — for very large collections
  consider pre-filtering with a bounding box (same caveat as
  [QDRANT_FILTERS.md Performance Tip #5](./QDRANT_FILTERS.md#5-use-geo-filters-wisely)).

## API surface

All functions live in `crates/vectorizer/src/db/payload_index.rs`.

### Public types

| Item                         | Lines   | Purpose                                                    |
|------------------------------|---------|------------------------------------------------------------|
| `enum PayloadIndexType`      | 16–28   | Keyword / Text / Integer / Float / Geo                     |
| `struct PayloadIndexConfig`  | 31–50   | Field name + index type + enabled flag                     |
| `struct PayloadIndexStats`   | 53–61   | `indexed_count`, `unique_values`, `memory_bytes`           |
| `struct PayloadIndex`        | 419–433 | Main manager (clones share state via `Arc<DashMap>`)       |

### Public functions (14 total)

| Function                                                         | Lines    | Purpose                                                      | Called from                                                                                                      |
|------------------------------------------------------------------|----------|--------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------|
| `PayloadIndexConfig::new(field_name, index_type)`                | 43       | Constructor; defaults `enabled = true`                       | `Collection::new` (`db/collection/mod.rs:157, 161`); tests                                                       |
| `PayloadIndex::new()`                                            | 437      | Empty manager                                                | `Collection::new` (`db/collection/mod.rs:154`); `Default`                                                        |
| `PayloadIndex::add_index_config(config)`                         | 449–482  | Register / allocate index for a field                        | `Collection::new` (auto-indexes `file_path`, `chunk_index`); **no REST/MCP/SDK caller**                          |
| `PayloadIndex::index_vector(vector_id, payload)`                 | 502–584  | Upsert vector across all configured field indexes            | `db/collection/data.rs:105` (insert), `:299` (update)                                                            |
| `PayloadIndex::remove_vector(vector_id)`                         | 587–612  | Delete vector from every index                               | `db/collection/data.rs:298` (update), `:350` (delete)                                                            |
| `PayloadIndex::get_ids_for_keyword(field, value)`                | 615–619  | Exact keyword lookup → `HashSet<VectorId>`                   | Tests only — not wired into REST / MCP / SDK                                                                     |
| `PayloadIndex::get_ids_in_range(field, min, max)`                | 622–631  | Integer range → `HashSet<VectorId>` (inclusive bounds)       | Tests only                                                                                                       |
| `PayloadIndex::get_ids_in_float_range(field, min, max)`          | 634–643  | Float range → `HashSet<VectorId>` (inclusive bounds)         | Tests only                                                                                                       |
| `PayloadIndex::search_text(field, query)`                        | 646–650  | Tokenised AND search → `HashSet<VectorId>`                   | Tests only                                                                                                       |
| `PayloadIndex::get_ids_in_geo_bounding_box(field, min/max lat/lon)` | 653–664 | Rectangular geo query (lat/lon in degrees)                    | Tests only                                                                                                       |
| `PayloadIndex::get_ids_in_geo_radius(field, lat, lon, radius_km)`| 667–677  | Haversine radius query (**radius in km**)                    | Tests only                                                                                                       |
| `PayloadIndex::get_stats()`                                      | 680–704  | `HashMap<FieldName, PayloadIndexStats>` over every index     | Tests only                                                                                                       |
| `PayloadIndex::get_config(field)`                                | 707–709  | Return the `PayloadIndexConfig` for a field, if any          | Tests only                                                                                                       |
| `PayloadIndex::list_indexed_fields()`                            | 712–714  | List every registered field name                             | Tests only                                                                                                       |

The original task brief referenced "30 public functions"; the file
actually exposes 14 `pub fn` items. The 30 figure likely counted
private helpers (`insert`, `remove`, `stats`, `estimate_memory`,
`tokenize`, `haversine_distance`, …) on the inner index structs, which
are implementation detail and not part of the public surface.

## Qdrant compatibility

Vectorizer ships a separate Qdrant-compat filter surface
(`src/models/qdrant/filter.rs` + `filter_processor.rs`) that REST / gRPC
clients interact with. See [QDRANT_FILTERS.md](./QDRANT_FILTERS.md) for
the full user-facing DSL.

### What matches Qdrant

- Boolean tree shape (`must` / `must_not` / `should`).
- Match conditions on strings, integers, booleans.
- Range conditions with `gt` / `gte` / `lt` / `lte`.
- Geo bounding box and geo radius (haversine, same formula Qdrant uses).
- Values-count predicates on arrays/objects.
- Nested conditions via `nested { filter }` and dot-notation keys.
- Text-match variants: `exact`, `prefix`, `suffix`, `contains`.

### Differences from Qdrant

1. **Filters are applied post-search**, not pre-search — `FilterProcessor`
   evaluates each candidate payload after HNSW has returned it. Qdrant
   can intersect an indexed field with the vector-index traversal.
   (`FilterProcessor::apply_filter` is a pure payload scan.)
2. **Geo radius units**: the Qdrant-compat DSL takes radius in **metres**
   (matching Qdrant); the underlying `PayloadIndex::get_ids_in_geo_radius`
   takes **kilometres**. If the two paths are ever unified, a conversion
   is required.
3. **`PayloadIndex` is not consulted** when evaluating a DSL filter, so
   adding a keyword index does not speed up a `match` condition on that
   field today.
4. **Text match in the DSL** (`contains`/`prefix`/`suffix`/`exact`) is
   implemented in `FilterProcessor`, not via the `Text` payload index.
   The two use different tokenisation strategies.
5. **Regex matching, full-text ranking, and aggregations** are
   unsupported (documented in [QDRANT_FILTERS.md](./QDRANT_FILTERS.md)
   § Error Handling).

## Limitations

1. **Query methods unused by any handler.** A repo-wide grep shows
   `get_ids_for_keyword`, `get_ids_in_range`, `get_ids_in_float_range`,
   `search_text`, `get_ids_in_geo_bounding_box`, and
   `get_ids_in_geo_radius` have **no callers outside tests**. REST
   search, MCP tools, and SDKs rely on `FilterProcessor::apply_filter`
   (payload scan). The `PayloadIndex` is therefore built and maintained
   on every write but never read on the query path.
2. **No public API to register custom fields.** Collections hard-code
   two auto-indexed fields (`file_path` keyword, `chunk_index` integer)
   in `Collection::new`. There is no REST endpoint, MCP tool, or
   `VectorStore` method to call `add_index_config` after the collection
   is created.
3. **No sorted structure for range queries.** Range methods do a full
   O(N) scan over the field's id→value map. There is no B-tree,
   skip-list, or roaring-bitmap representation.
4. **Text index is minimal.** No stemming, stopwording, synonyms,
   phrase queries, or ranking. AND-of-tokens only. Not suitable as a
   general-purpose FTS engine — treat it as a tokenised `contains-all`
   filter.
5. **Keyword index is case-sensitive** (values are inserted verbatim,
   lines 529–535), whereas the text index is lowercased at tokenisation
   (line 308). Match semantics therefore differ between the two when
   indexing the same field.
6. **No persistence.** The index is in-memory only and rebuilt from the
   vector data whenever a collection is reloaded (through per-vector
   `index_vector` calls on insert). There is no snapshot / restore path.
7. **Fixed units and formats.**
   - Geo radius is kilometres (not configurable).
   - Geo coordinates only accept `{lat, lon}` objects or `[lat, lon]`
     arrays (lines 566–578); longitude-first arrays (`[lon, lat]`,
     GeoJSON style) are not supported.
8. **Auto-indexing has known issues.** The integration test
   `test_payload_index_auto_indexing_on_insert`
   (`tests/integration/payload_index.rs:15`) is `#[ignore]`-d with
   the note *"Payload index auto indexing has issues - skipping until
   fixed"*, implying the write-path integration is not fully trusted.
9. **Not filterable today** (via the Qdrant DSL, without application
   code): arbitrary arrays, nested repeated fields, and any type other
   than the five listed `PayloadIndexType`s. `values_count` filtering
   works through the DSL's payload scan but has no corresponding
   `PayloadIndex` backing.
10. **No unified result-ranking.** Because `PayloadIndex` returns
    `HashSet<VectorId>`s with no score, callers combining it with vector
    similarity must re-rank externally.

## See also

- [QDRANT_FILTERS.md](./QDRANT_FILTERS.md) — user-facing Qdrant-compat
  filter DSL (served by `FilterProcessor`).
- [QDRANT_COMPATIBILITY_INDEX.md](./QDRANT_COMPATIBILITY_INDEX.md) —
  overall compatibility status.
- `crates/vectorizer/src/db/payload_index.rs` — implementation.
- `crates/vectorizer/src/models/qdrant/filter_processor.rs` — DSL
  evaluator.
- `crates/vectorizer/tests/integration/payload_index.rs` — behavioural
  fixtures.
