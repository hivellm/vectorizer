# Graph Relationships — Specification

**Status**: Beta — graph layer is implemented for CPU collections only. GPU,
sharded, and distributed-sharded collections are explicitly rejected (see
[Limitations](#limitations)).
**Last Updated**: 2026-04-24

---

## Table of Contents

1. [Overview](#overview)
2. [Concepts](#concepts)
3. [REST API](#rest-api)
4. [MCP Tools](#mcp-tools)
5. [Relationship Types](#relationship-types)
6. [Discovery Pipeline](#discovery-pipeline)
7. [Usage Examples](#usage-examples)
8. [Limitations](#limitations)
9. [See Also](#see-also)

---

## Overview

The graph layer is an in-memory, per-collection relationship store that sits
alongside the HNSW vector index. Each vector in a collection can be projected
onto a graph node, and edges (typed relationships with a weight) can be added
between nodes either explicitly (via REST/MCP) or auto-discovered from
similarity scores and payload metadata.

The graph is intended for workflows where pure semantic similarity is not
enough: e.g. navigating `references` / `contains` / `derived_from` links
between source files in a code-search collection, walking N-hop neighborhoods,
or finding the shortest path between two documents. The graph is a **lazy,
opt-in** feature — collections have no graph until `enable_graph_for_collection`
is invoked (either through `POST /graph/enable/{collection}` or automatically
at load time when a `{collection}_graph.json` file exists under the data
directory).

Use it when you need traversal semantics that HNSW alone cannot answer
(multi-hop paths, neighborhood expansion filtered by edge type, connected
components). Use plain vector search when a single ranked similarity list is
sufficient.

Implementation lives in:
- `crates/vectorizer/src/db/graph.rs` — core `Graph`, `Node`, `Edge`,
  `RelationshipType`.
- `crates/vectorizer/src/db/graph_relationship_discovery.rs` — auto-discovery
  heuristics and the `GraphRelationshipHelper` trait.
- `crates/vectorizer/src/db/collection/graph.rs` — `Collection`-level wiring
  (`enable_graph`, `populate_graph_if_empty`, `get_graph`, `set_graph`).
- `crates/vectorizer/src/db/vector_store/collections.rs` —
  `VectorStore::enable_graph_for_collection` (alias resolution + disk load).
- `crates/vectorizer-server/src/api/graph.rs` — REST router factory
  (`create_graph_router`) and `GraphApiState`.
- `crates/vectorizer-server/src/server/graph_handlers.rs` — MCP tool handlers.
- `crates/vectorizer-server/src/server/mcp/tools.rs` — MCP tool schemas.

---

## Concepts

### Node

A `Node` represents a document, chunk, or file projected into the graph. It
is not a copy of the vector — it carries the vector's ID and a snapshot of its
payload metadata. Defined in `crates/vectorizer/src/db/graph.rs:43`.

Fields:
- `id: String` — unique identifier. Usually the vector ID or a `file_path`.
- `node_type: String` — free-form type label (default `"document"` when
  constructed via `Node::from_vector`).
- `metadata: HashMap<String, serde_json::Value>` — payload data copied from
  the source vector. `file_path` is promoted into a first-class key when
  present.
- `created_at: DateTime<Utc>` — construction timestamp.

Nodes are constructed either via `Node::new(id, node_type)` or
`Node::from_vector(vector_id, payload)`. The latter is what the `Collection`
uses when bulk-populating the graph from existing vectors.

### Edge

A directed, typed, weighted connection between two nodes. Defined in
`crates/vectorizer/src/db/graph.rs:92`.

Fields:
- `id: String` — unique edge ID. Auto-discovery and REST `create_edge` both
  use the canonical shape `"{source}:{target}:{RELATIONSHIP_TYPE}"` (the REST
  path uses `Debug` formatting of the enum — `SimilarTo`, `References`, etc.
  — while discovery writes the `SCREAMING_SNAKE_CASE` name; callers should
  treat the ID as opaque and not parse it).
- `source: String`, `target: String` — node IDs. Both nodes must already
  exist; `add_edge` returns `VectorizerError::NotFound` otherwise.
- `relationship_type: RelationshipType` — one of four values (see
  [Relationship Types](#relationship-types)).
- `weight: f32` — similarity score (for `SIMILAR_TO`) or `1.0` for the three
  metadata-derived relationship types. The BFS in `find_related` multiplies
  edge weights along a path to rank results.
- `metadata: HashMap<String, serde_json::Value>` — arbitrary per-edge
  metadata. Currently unused by the core but preserved across save/load.
- `created_at: DateTime<Utc>`.

### Discovery

Auto-edge creation from heuristics. Two inputs drive it:
1. Cosine / distance scores returned by `Collection::search_similar_vectors`
   → `SIMILAR_TO` edges above a configurable threshold.
2. Payload keys (`references`, `contains`, `derived_from`) →
   `REFERENCES`, `CONTAINS`, `DERIVED_FROM` edges at weight `1.0`.

Auto-discovery is gated by an `AutoRelationshipConfig`
(`crates/vectorizer/src/models/mod.rs:499`) with three knobs:
- `similarity_threshold: f32` (default `0.7`)
- `max_per_node: usize` (default `10`)
- `enabled_types: Vec<String>` (default `["SIMILAR_TO", "REFERENCES",
  "CONTAINS"]`; `DERIVED_FROM` is not in the default set — it is only
  discovered if explicitly listed).

### Enablement

Graphs are **opt-in** per collection. Three entry points flip the bit:

1. `VectorStore::enable_graph_for_collection(name)` — the authoritative path.
   Resolves aliases, checks for an on-disk `{name}_graph.json`, and either
   loads it or creates a fresh graph and populates nodes from the collection's
   current vectors. Bails out with `VectorizerError::Storage` on GPU / sharded
   / distributed collections.
2. `Collection::enable_graph()` — lower-level variant used internally; creates
   nodes for every vector in `vector_order`, then auto-discovers `SIMILAR_TO`
   edges for the first 100 nodes to keep the call non-blocking.
3. `Collection::populate_graph_if_empty()` — invoked on cache/persistence load
   to rebuild node set from vectors when the on-disk graph has nodes missing.

The default config returned when no `graph` block is present on the collection
config is `{ similarity_threshold: 0.7, max_per_node: 10, enabled_types:
["SIMILAR_TO"] }` (see `collection/graph.rs:97-110`, `177-184`, `271-283`).

---

## REST API

All endpoints are mounted under the server's base path by `create_graph_router`
(`crates/vectorizer-server/src/api/graph.rs:57`). State type: `GraphApiState`
(wraps the `VectorStore` and a `DashMap<edge_id, collection_name>` cache that
accelerates `DELETE /graph/edges/{id}`).

Every endpoint returns JSON. Errors are returned as
`{ "error": "<message>" }` with a non-2xx status code.

### Endpoint summary

| Method | Path | Description |
| --- | --- | --- |
| POST | `/graph/enable/{collection}` | Enable graph for a collection. |
| GET | `/graph/status/{collection}` | Return `{enabled, node_count, edge_count}`. |
| GET | `/graph/nodes/{collection}` | List all nodes. |
| GET | `/graph/nodes/{collection}/{node_id}/neighbors` | Direct outgoing neighbors. |
| POST | `/graph/nodes/{collection}/{node_id}/related` | BFS up to `max_hops`. |
| POST | `/graph/path` | Shortest path between two nodes. |
| POST | `/graph/edges` | Create an explicit edge. |
| DELETE | `/graph/edges/{edge_id}` | Delete an edge. |
| GET | `/graph/collections/{collection}/edges` | List edges (with filter + limit). |
| POST | `/graph/discover/{collection}` | Auto-discover `SIMILAR_TO` edges for every node. |
| POST | `/graph/discover/{collection}/{node_id}` | Auto-discover for one node. |
| GET | `/graph/discover/{collection}/status` | Progress view (nodes with ≥1 outgoing edge). |

### POST /graph/enable/{collection}

Enable graph for a collection. Creates nodes for all existing vectors, then
auto-discovers `SIMILAR_TO` edges for up to the first 100 nodes (blocking
avoidance — full discovery requires a separate
`POST /graph/discover/{collection}` call).

- Request body: none.
- Success (`200 OK`):
  ```json
  {
    "success": true,
    "collection": "<name>",
    "message": "Graph enabled successfully",
    "node_count": 1234
  }
  ```
- `400 Bad Request` if the underlying collection is GPU / sharded /
  distributed, or on any other enablement failure.

### GET /graph/status/{collection}

- `200 OK` body: `{ "collection", "enabled": bool, "node_count": usize,
  "edge_count": usize }`. When `enabled` is `false`, counts are `0`.
- `404 Not Found` if the collection does not exist.

### GET /graph/nodes/{collection}

List every node currently in the graph.

- `200 OK`: `{ "nodes": [Node, ...], "count": usize }`.
- `404` if collection missing, `400` if graph not enabled.

### GET /graph/nodes/{collection}/{node_id}/neighbors

Returns the direct outgoing edges and their target nodes. No relationship
filtering at this endpoint (the underlying `Graph::get_neighbors` accepts an
`Option<RelationshipType>`, but this route always passes `None`).

- `200 OK`: `{ "neighbors": [{ "node": Node, "edge": Edge }, ...] }`.
- `404` / `400` / `500` as above.

### POST /graph/nodes/{collection}/{node_id}/related

BFS traversal ranked by cumulative edge-weight product, descending.

- Request body (`FindRelatedRequest`):
  ```json
  {
    "max_hops": 2,                    // optional, default 2
    "relationship_type": "SIMILAR_TO" // optional; one of the 4 types (see Relationship Types)
  }
  ```
- `200 OK`:
  ```json
  {
    "related": [
      { "node": Node, "distance": 1, "weight": 0.83 },
      ...
    ]
  }
  ```
  `distance` is hop count from the starting node; `weight` is the product of
  edge weights along the BFS path.

### POST /graph/path

Shortest path (BFS, unweighted) between two nodes.

- Request body (`FindPathRequest`):
  ```json
  {
    "collection": "<name>",
    "source": "<node_id>",
    "target": "<node_id>"
  }
  ```
- `200 OK` when both endpoints exist and are reachable:
  ```json
  { "path": [Node, Node, ...], "found": true }
  ```
- `200 OK` with `{ "path": [], "found": false }` when no path exists (the
  core returns `VectorizerError::NotFound`; the REST handler treats that as a
  successful "no path" response rather than a 404 — see `api/graph.rs:480-486`).
- `500` for unexpected errors.

### POST /graph/edges

Create an explicit edge. Both nodes must already exist in the graph.

- Request body (`CreateEdgeRequest`):
  ```json
  {
    "collection": "<name>",
    "source": "<node_id>",
    "target": "<node_id>",
    "relationship_type": "SIMILAR_TO",
    "weight": 0.92                // optional; default 1.0
  }
  ```
  `relationship_type` is case-insensitive and accepts both snake and non-snake
  forms: `SIMILAR_TO` / `SIMILARTO`, `REFERENCES`, `CONTAINS`, `DERIVED_FROM` /
  `DERIVEDFROM` (see `parse_relationship_type` in
  `api/graph.rs:978`).
- `200 OK`: `{ "edge_id": "<id>", "success": true, "message": "..." }`.
- The new `edge_id → collection` mapping is cached in `GraphApiState.edge_index`
  to speed up subsequent deletes.
- `400` for unknown relationship type, missing graph, or if `Graph::add_edge`
  rejects the edge (source or target node not found).

### DELETE /graph/edges/{edge_id}

Delete an edge by ID. Looks up the owning collection in the `edge_index`
cache first; on cache miss falls back to scanning every collection's graph
(see `api/graph.rs:703-737`).

- `200 OK`: `{ "success": true, "message": "Edge deleted successfully" }`.
- `404` if no collection's graph contained that edge ID.

### GET /graph/collections/{collection}/edges

List edges in the graph, optionally filtered and limited.

- Query params:
  - `relationship_type` — one of `SIMILAR_TO`, `REFERENCES`, `CONTAINS`,
    `DERIVED_FROM`. Anything else is silently ignored.
  - `limit` — integer truncating the returned `edges` array. When omitted,
    **all** edges are returned.
- `200 OK`:
  ```json
  {
    "count": <after-filter-and-limit>,
    "edges": [
      {
        "id": "<edge_id>",
        "source": "<node_id>",
        "target": "<node_id>",
        "relationship_type": "SIMILAR_TO",
        "weight": 0.83,
        "metadata": { ... },
        "created_at": "<rfc3339>"
      }
    ]
  }
  ```
  Note: the response serializes `relationship_type` as SCREAMING_SNAKE_CASE,
  whereas `Edge` inside `Node`/`Edge` payloads relies on the `serde` rename
  applied to `RelationshipType` (also SCREAMING_SNAKE_CASE —
  `graph.rs:18`).

### POST /graph/discover/{collection}

Run `SIMILAR_TO` discovery for every node in the graph. CPU-only; returns
`400 Bad Request` for GPU / sharded / distributed collections.

- Request body (`DiscoverEdgesRequest`):
  ```json
  {
    "similarity_threshold": 0.7, // optional, default 0.7
    "max_per_node": 10            // optional, default 10
  }
  ```
  `enabled_types` is forced to `["SIMILAR_TO"]` by this endpoint (metadata
  discovery is not offered here — it runs implicitly on insert when enabled).
- `200 OK`:
  ```json
  {
    "success": true,
    "edges_created": 412,
    "message": "Created 412 edges for 87 nodes"
  }
  ```

### POST /graph/discover/{collection}/{node_id}

Same as above but for a single node. Same request / response shape; the
`message` field reads `"Created N edges for node '<id>'"`.

### GET /graph/discover/{collection}/status

Discovery progress for the UI. Walks every node and counts those with at
least one outgoing edge.

- `200 OK`:
  ```json
  {
    "total_nodes": 1200,
    "nodes_with_edges": 847,
    "total_edges": 8312,
    "progress_percentage": 70.58
  }
  ```

---

## MCP Tools

All eight `graph_*` tools are registered in
`crates/vectorizer-server/src/server/mcp/tools.rs:712-903` and dispatched in
`handlers.rs:131-138`. Implementations live in `graph_handlers.rs`. Every
MCP handler returns a `CallToolResult` whose single `Content::text` item is
a JSON string with the shape described below.

### graph_list_nodes — List Graph Nodes

Read-only, idempotent.

- Parameters: `{ "collection": string }` (required).
- Returns: `{ "nodes": [Node, ...], "count": usize }`.

### graph_get_neighbors — Get Graph Node Neighbors

Read-only, idempotent.

- Parameters: `{ "collection": string, "node_id": string }` (both required).
- Returns: `{ "neighbors": [{ "node": Node, "edge": Edge }, ...] }`.

### graph_find_related — Find Related Nodes

Read-only, idempotent.

- Parameters:
  ```json
  {
    "collection": "<name>",
    "node_id": "<id>",
    "max_hops": 2,                                             // optional, default 2
    "relationship_type": "SIMILAR_TO" | "REFERENCES" | "CONTAINS" | "DERIVED_FROM"  // optional
  }
  ```
- Returns: `{ "related": [{ "node": Node, "distance": usize, "weight": f32 }, ...] }`.
- Usage example (JSON-RPC parameters):
  ```json
  {
    "name": "graph_find_related",
    "arguments": {
      "collection": "hivellm_docs",
      "node_id": "docs/AGENTS.md",
      "max_hops": 3,
      "relationship_type": "REFERENCES"
    }
  }
  ```

### graph_find_path — Find Path Between Nodes

Read-only, idempotent.

- Parameters: `{ "collection", "source", "target" }` (all required).
- Returns: `{ "path": [Node, ...], "found": true }` on success or
  `{ "path": [], "found": false, "message": "<reason>" }` when no path is
  found. Unlike the REST route, the MCP handler preserves the "not found"
  message string from the core (`graph_handlers.rs:255-263`).

### graph_create_edge — Create Graph Edge

Not read-only.

- Parameters:
  ```json
  {
    "collection": "<name>",
    "source": "<node_id>",
    "target": "<node_id>",
    "relationship_type": "SIMILAR_TO" | "REFERENCES" | "CONTAINS" | "DERIVED_FROM",
    "weight": 1.0   // optional, default 1.0
  }
  ```
- Returns: `{ "edge_id": "<id>", "success": true, "message": "Edge created successfully" }`.
- Note: the MCP variant does **not** populate the REST `edge_index` cache —
  subsequent REST deletes of an MCP-created edge fall through to the
  full-scan path.

### graph_delete_edge — Delete Graph Edge

Not read-only.

- Parameters: `{ "edge_id": string }` (required).
- Behavior: scans every collection's graph until one successfully removes the
  edge. No cache lookup.
- Returns: `{ "success": true, "message": "Edge deleted successfully" }` or
  an `invalid_params` error with `"Edge '<id>' not found"` when the scan
  finds nothing.

### graph_discover_edges — Discover Graph Edges

Not read-only.

- Parameters:
  ```json
  {
    "collection": "<name>",
    "node_id": "<optional node id>",    // when present: discover for one node
    "similarity_threshold": 0.7,         // optional, default 0.7
    "max_per_node": 10                   // optional, default 10
  }
  ```
- Returns (collection-wide):
  ```json
  {
    "success": true,
    "total_nodes": usize,
    "nodes_processed": usize,
    "nodes_with_edges": usize,
    "total_edges_created": usize,
    "message": "Created N edges for M nodes"
  }
  ```
- Returns (single-node): `{ "success": true, "edges_created": usize, "node_id": "<id>", "message": "..." }`.

### graph_discover_status — Get Graph Discovery Status

Read-only.

- Parameters: `{ "collection": string }` (required).
- Returns: `{ "total_nodes", "nodes_with_edges", "total_edges", "progress_percentage" }`.

**Missing from MCP**: there is no `graph_enable` or `graph_status` MCP tool.
Enablement must go through the REST endpoint `POST /graph/enable/{collection}`
or occur implicitly at load time via `VectorStore::enable_graph_for_collection`.

---

## Relationship Types

Source: `crates/vectorizer/src/db/graph.rs:19-40`. Serialized as
SCREAMING_SNAKE_CASE.

| Variant | Wire name | Weight | Heuristic | Typical use |
| --- | --- | --- | --- | --- |
| `SimilarTo` | `SIMILAR_TO` | similarity score (`0.0..=1.0`) | Top-K vector search with score ≥ `similarity_threshold`, capped at `max_per_node`. | "Find docs semantically close to X." |
| `References` | `REFERENCES` | `1.0` | Payload key `references: [file_path, ...]`. Each string becomes an edge; the target node is looked up by `metadata.file_path` and created if absent. | Source code imports, hyperlinks, citations. |
| `Contains` | `CONTAINS` | `1.0` | Payload key `contains: [file_path, ...]`. Same lookup/creation rule as `REFERENCES`. | Directory → file, document → chunk. |
| `DerivedFrom` | `DERIVED_FROM` | `1.0` | Payload key `derived_from: "file_path"` (single string, not array). | Summary → source, generated → template. |

The three payload-driven heuristics are implemented in
`graph_relationship_discovery.rs:184-274`. Similarity discovery is in the
same file, lines `49-102`. The `find_or_create_node_by_file_path` helper
(`lines 277-298`) walks every existing node's metadata looking for a
`file_path` match, which is O(N) per lookup — worth noting when discovering
over large collections.

**Relationship gating.** Auto-discovery only runs for types listed in
`AutoRelationshipConfig.enabled_types`. The default config enables
`SIMILAR_TO`, `REFERENCES`, and `CONTAINS` but **not** `DERIVED_FROM`; the
empty-config fallback used when a collection has no `graph` block enables
only `SIMILAR_TO` (`collection/graph.rs:97-110`).

---

## Discovery Pipeline

End-to-end, the lifecycle for a brand-new collection looks like this:

1. **Create collection** via REST (`POST /collections`) or MCP. No graph yet.
2. **Ingest vectors** as usual. If the collection's config has `graph.enabled
   = true` with enabled types, per-insert metadata discovery runs inline via
   `discover_relationships` (`graph_relationship_discovery.rs:14-43`). This
   path creates the node for the inserted vector and, for enabled metadata
   types, walks the payload to create `REFERENCES` / `CONTAINS` /
   `DERIVED_FROM` edges. `SIMILAR_TO` discovery is intentionally **not** run
   inline during insert — the module-level doc comment and
   `discover_similarity_relationships` note explicitly mention this is to
   avoid timeouts.
3. **Enable graph** explicitly: `POST /graph/enable/{collection}`. Resolves
   to `VectorStore::enable_graph_for_collection` which:
   - Resolves aliases (`resolve_alias_target`).
   - Loads the collection.
   - Checks for `{data_dir}/{collection}_graph.json`. If present and
     non-empty, loads it; if it has nodes but zero edges, runs
     `discover_edges_for_node` against the first 100 nodes.
   - Otherwise calls `Collection::enable_graph()` which creates nodes for
     every vector and auto-discovers `SIMILAR_TO` edges on the first 100
     nodes.
4. **(Optional) Full discovery**: `POST /graph/discover/{collection}` to run
   `SIMILAR_TO` discovery across **every** node — the server bootstrap only
   touches the first 100 to keep enablement fast.
5. **Query**: neighbors, related (BFS with optional type filter), path
   (shortest-path BFS).
6. **Inspect progress**: `GET /graph/discover/{collection}/status` (or the
   MCP equivalent) to surface a coverage percentage in the dashboard.
7. **Persist**: `Graph::save_to_file` writes `{collection}_graph.json`
   atomically (temp file + rename). `Graph::load_from_file` restores on
   startup. Persistence is invoked from several points in
   `vector_store/persistence.rs`.

The key asymmetry worth highlighting: **similarity discovery is a separate
phase from vector insertion**, whereas metadata-driven discovery runs inline.
This means a freshly-ingested collection will have `REFERENCES` /
`CONTAINS` / `DERIVED_FROM` edges but **zero** `SIMILAR_TO` edges until
enablement or an explicit discover call.

---

## Usage Examples

### 1. Manual graph construction via REST

Build a small citation graph on top of a `docs` collection. Assumes the
collection already exists and has vectors for the referenced node IDs
(vectors must be inserted before nodes can be referenced — `add_edge`
rejects missing endpoints).

```bash
# Turn on the graph layer; creates one node per existing vector.
curl -sX POST http://localhost:15002/graph/enable/docs

# Inspect state.
curl -s http://localhost:15002/graph/status/docs
# -> { "collection": "docs", "enabled": true, "node_count": 42, "edge_count": 0 }

# Explicitly link two docs via REFERENCES.
curl -sX POST http://localhost:15002/graph/edges \
  -H 'Content-Type: application/json' \
  -d '{
    "collection": "docs",
    "source": "docs/intro.md",
    "target": "docs/architecture.md",
    "relationship_type": "REFERENCES"
  }'
# -> { "edge_id": "docs/intro.md:docs/architecture.md:References", "success": true, ... }

# Pull neighbors of intro.md.
curl -s http://localhost:15002/graph/nodes/docs/docs%2Fintro.md/neighbors

# Shortest path.
curl -sX POST http://localhost:15002/graph/path \
  -H 'Content-Type: application/json' \
  -d '{ "collection": "docs", "source": "docs/intro.md", "target": "docs/mcp.md" }'
```

### 2. Auto-discovery over a code-search collection via MCP

Given a collection `vectorizer_source` already populated with chunked source
files, ask an MCP client to bootstrap the graph and then query it.

1. Enable the graph (REST — no MCP equivalent):
   ```bash
   curl -sX POST http://localhost:15002/graph/enable/vectorizer_source
   ```
2. Run full `SIMILAR_TO` discovery through MCP:
   ```json
   {
     "name": "graph_discover_edges",
     "arguments": {
       "collection": "vectorizer_source",
       "similarity_threshold": 0.8,
       "max_per_node": 5
     }
   }
   ```
   Response (shape):
   ```json
   {
     "success": true,
     "total_nodes": 1280,
     "nodes_processed": 1280,
     "nodes_with_edges": 942,
     "total_edges_created": 4037,
     "message": "Created 4037 edges for 942 nodes"
   }
   ```
3. Poll progress:
   ```json
   { "name": "graph_discover_status", "arguments": { "collection": "vectorizer_source" } }
   ```
4. Expand from a node of interest:
   ```json
   {
     "name": "graph_find_related",
     "arguments": {
       "collection": "vectorizer_source",
       "node_id": "crates/vectorizer/src/db/graph.rs::chunk-3",
       "max_hops": 2,
       "relationship_type": "SIMILAR_TO"
     }
   }
   ```

---

## Limitations

- **CPU collections only.** `get_collection_graph_from_type`
  (`api/graph.rs:757` and `graph_handlers.rs:11`) matches only
  `CollectionType::Cpu`. `enable_graph_for_collection` explicitly returns
  `VectorizerError::Storage("Graph not yet supported for GPU/sharded/distributed
  collections")` for every other variant (`collections.rs:688-697`). Any
  non-CPU collection will respond with `400 Bad Request` on every graph
  route.
- **In-memory + JSON persistence.** The graph is an `Arc<RwLock<HashMap>>`
  triple (nodes, edges, forward+reverse adjacency) backed by a pretty-printed
  JSON file (`{collection}_graph.json`) via atomic temp-file rename. There is
  no incremental write-ahead log — every save serializes the full graph.
  Corrupted files trigger a `warn!` and degrade to an empty graph
  (`graph.rs:627-638`).
- **Bootstrap is capped at 100 nodes.** Both `Collection::enable_graph` and
  `populate_graph_if_empty` only auto-run `SIMILAR_TO` discovery on the
  first 100 nodes. Full discovery requires an explicit
  `POST /graph/discover/{collection}` call. The 100-node cap is a hard-coded
  `.take(100)` in `collection/graph.rs:115, 188, 298` and
  `collections.rs:658`.
- **`SIMILAR_TO` discovery is not inline with insert.** Metadata-derived
  edges (`REFERENCES` / `CONTAINS` / `DERIVED_FROM`) are created on vector
  insert; `SIMILAR_TO` requires a separate discovery phase. Downstream
  clients must not assume similarity edges exist for freshly-inserted
  vectors.
- **`find_or_create_node_by_file_path` is O(N).** Metadata discovery walks
  every existing node searching for a `file_path` match
  (`graph_relationship_discovery.rs:277-298`). On large graphs this becomes
  a per-edge O(N) scan.
- **No `graph_enable` / `graph_status` MCP tools.** Enablement and status
  inspection are REST-only; MCP clients need an out-of-band REST call to turn
  the graph on.
- **Edge IDs are not stable across code paths.** Discovery writes
  SCREAMING_SNAKE_CASE into edge IDs; REST `create_edge` and MCP
  `graph_create_edge` use the `Debug` impl (`SimilarTo`, `References`, ...).
  Clients should treat edge IDs as opaque.
- **`graph_delete_edge` (MCP) scans every collection.** Unlike the REST
  route, there is no cached `edge_id → collection` lookup.
- **No benchmark suite published for the graph layer.** `docs/specs/BENCHMARKING.md`
  and `docs/specs/PERFORMANCE.md` cover vector search and storage but do not
  include graph traversal numbers. TBD — no measurements to cite here.
- **Enablement is not a Beta feature flag.** There is no cargo feature
  guarding the graph layer; "Beta" here refers to the limited surface (CPU
  only, no benchmarks, one-phase discovery, no MCP enablement).

---

## See Also

- [API_REFERENCE.md](./API_REFERENCE.md) — full REST surface including
  collection/vector endpoints the graph builds on.
- [MCP.md](./MCP.md) — MCP server protocol, tool invocation, and transport
  notes.
- [DASHBOARD.md](./DASHBOARD.md) — the GraphPage dashboard view that drives
  most of these endpoints interactively.
- [PERSISTENCE.md](./PERSISTENCE.md) — collection persistence, including the
  `{collection}_graph.json` load path invoked by
  `enable_graph_for_collection`.
