# VectorizerRPC — Wire Protocol Specification

**Status**: v1 (frozen)
**Default port**: 15503/tcp
**Implementation**: [Thunder](https://github.com/hivellm/thunder)
(`thunder-rpc`), the HiveLLM family's shared binary RPC stack. Vectorizer no
longer ships a codec of its own: the server runs Thunder's server half and
every SDK runs the Thunder client for its language, so the two ends of the
wire cannot drift. Thunder is the packaged form of the SynapRPC layer this
protocol was originally ported from, so the bytes below are unchanged.

This document is the authoritative byte-level contract between Vectorizer
servers and clients. Every SDK (`phase6_sdk-{rust,go,python,javascript,
typescript,csharp}-rpc`) must conform to this spec; the server enforces
it. Anything not listed here is unspecified and may change without notice.

## Why a new transport?

The project already exposes REST (Axum), MCP (WebSocket JSON-RPC), gRPC
(tonic), and now the capability registry from
`phase4_rest-mcp-parity-tests` makes adding a fourth transport mechanical
— same handlers, new framing.

| Transport | Per-request overhead | Codegen | Best for |
|---|---|---|---|
| REST | HTTP framing + JSON parse + TLS handshake | none | browsers, scripts, ops tooling |
| gRPC | HTTP/2 + protobuf | yes (per language) | strongly-typed RPC across services |
| MCP | WebSocket + JSON-RPC | none | interactive AI tools |
| **RPC (this spec)** | **u32 length + MessagePack** | **none** | **bulk ingest, low-latency search, embedded SDK use** |

RPC is the **default transport** for first-party SDKs starting with
`phase6_make-rpc-default-transport`. REST stays as the universal
fallback.

## 1. Framing

Every frame on the wire — request and response — has identical shape:

```text
┌───────────────────┬──────────────────────────┐
│  length: u32 (LE) │  body: MessagePack bytes  │
└───────────────────┴──────────────────────────┘
    4 bytes              `length` bytes
```

- `length` is the size of the body **only**, in bytes, encoded as a
  little-endian unsigned 32-bit integer.
- `body` is a single MessagePack-encoded value. The server uses
  `rmp-serde` with the default externally-tagged enum representation;
  clients must encode/decode using a compatible MessagePack library.
- **Maximum body size: 512 MiB** (`512 * 1024 * 1024 = 536_870_912` bytes),
  the family default shared with Synap — large enough for batch inserts and
  raw little-endian f32 embedding payloads. The cap is validated against the
  length prefix *before* the body is allocated, so a hostile peer cannot drive
  an unbounded allocation from four bytes; a frame declaring more is refused.
  (Pre-Thunder servers and SDKs capped this at 64 MiB.)

A connection is a stream of frames. Frames are not interleaved at the
byte level; the server reads a complete frame, dispatches it (possibly
on its own task), and may write responses out of order.

## 2. Request / Response envelope

Every request is a single MessagePack-encoded `Request` struct; every
response is a single MessagePack-encoded `Response` struct.

```rust
pub struct Request {
    pub id: u32,            // client-chosen monotonic ID
    pub command: String,    // dotted name from the capability registry
    pub args: Vec<Value>,   // positional arguments
}

pub struct Response {
    pub id: u32,                       // echoes Request.id
    pub result: Result<Value, String>, // Ok payload OR error message
}
```

### Multiplexing

Clients **must** treat `id` as opaque and unique per in-flight request
on a connection. The server runs each `Request` on its own
`tokio::spawn` task and emits `Response` frames in completion order, not
arrival order — clients dispatch responses to the originating call by
matching `Response.id` to a pending-call table.

A `u32` gives ~4 billion distinct IDs per connection. SDKs SHOULD wrap
on overflow; collisions on a long-lived connection are vanishingly rare
because the in-flight set is bounded by application backpressure.

### Error encoding (v1)

`result` is a serde `Result<Value, String>` — on success the inner is
`Ok(value)`, on failure it is `Err(message)`. The error string is a
human-readable message; v1 does not carry a structured error code. SDKs
SHOULD parse the string only for display; **do not** branch on it.

A future v2 may upgrade `Err(String)` to `Err(Error { code: u16,
message: String, details: Option<Value> })` once the project's error
enums are unified (see `phase3_unify-error-enums`). The version
negotiated by `HELLO` (§ 5) tells the client which form to expect.

## 3. The `Value` type

The on-wire dynamically-typed value mirrors SynapRPC's `SynapValue`:

```rust
pub enum VectorizerValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Bytes(Vec<u8>),       // raw bytes — NOT base64-encoded
    Str(String),          // valid UTF-8
    Array(Vec<VectorizerValue>),
    Map(Vec<(VectorizerValue, VectorizerValue)>),
}
```

- Encoded with `rmp-serde`'s default externally-tagged representation:
  unit variants are a bare string (`"Null"`), newtype variants are a
  single-key map (`{"Int": 42}`).
- `Bytes` carries raw octets. Unlike the JSON transports there is no
  base64 wrapping — embedding vectors and document payloads on the wire
  is the principal motivation for the binary transport.
- `Map` is an `Vec<(K, V)>` of pairs (not a `HashMap`) because
  MessagePack maps preserve insertion order and keys may be any value,
  not just strings.

### Cross-language mapping

Every SDK gets these mappings from its Thunder client rather than
implementing them: Rust `thunder-rpc`, Python `hivellm-thunder` (imported as
`thunder_rpc`), TypeScript `@hivehub/thunder`, Go
`github.com/hivellm/thunder-go`, C# `HiveLLM.Thunder`.

| Vectorizer | Rust | Python | TypeScript | Go |
|---|---|---|---|---|
| `Null` | `()` | `None` | `null` | `nil` |
| `Bool` | `bool` | `bool` | `boolean` | `bool` |
| `Int` | `i64` | `int` | `bigint` / `number` | `int64` |
| `Float` | `f64` | `float` | `number` | `float64` |
| `Bytes` | `Vec<u8>` | `bytes` | `Uint8Array` | `[]byte` |
| `Str` | `String` | `str` | `string` | `string` |
| `Array` | `Vec<Value>` | `list` | `array` | `[]interface{}` |
| `Map` | `Vec<(Value, Value)>` | `dict` (string keys) / `list` of tuples | `Map` | `map[interface{}]interface{}` |

## 4. Authentication

Authentication is a **per-connection state**, resolved once and sticky for
the connection's lifetime. Subsequent requests carry no token — auth is
implicit in the connection state. This trades per-frame overhead for a
stickier connection model (connections are stateful, but on a long-lived TCP
socket the overhead is amortized to zero).

Credentials travel in the **`AUTH` command**, Thunder's `AuthCommand`
handshake: the client sends `AUTH <secret>` on connect (its client library
does this for it when credentials are configured), and the server validates
the secret as a JWT first and as an API key second. Both credential forms are
therefore accepted through the same frame:

- A bearer JWT, the same format REST `/auth/login` returns, OR
- An API key.

Until a session authenticates, only the pre-auth allowlist —
`PING`, `HELLO`, `AUTH`, `QUIT` — is answered; anything else is refused with
`Err("NOAUTH ...")`. Invalid credentials are refused with
`Err("WRONGPASS ...")`. Clients classify both by prefix (the `Resp3Prefixes`
error convention) rather than by message text.

When `auth.enabled = false` server-side (single-user local setups), the
listener is *open*: no `AUTH` is required, every command runs as the implicit
local admin, and `AUTH` is accepted-but-ignored. This matches the existing
REST/MCP behaviour.

> **Pre-Thunder clients**: credentials used to travel inside `HELLO`, and the
> gate error read `"authentication required: send HELLO first"`. `HELLO` still
> validates any credentials it carries and reports them in its reply, but the
> *session* is authenticated by `AUTH` — which is why the first-party SDKs
> re-dial when a `HELLO` payload carries a token or an API key. Server and
> SDKs ship in lockstep (all at 3.6.x), and REST remains the fallback.

### Admin role

Commands tagged `Admin` in the capability registry require the
authenticated principal to carry `Role::Admin` claims. The server
returns `Err("admin role required")` and does not advance the request
to the handler.

## 5. The `HELLO` command

`HELLO` is the protocol-version handshake and the capability advertisement.
It is Vectorizer's own command (Thunder's `HelloStyle::NotUsed`), so it
reaches the server's dispatch table like any other — it is not a
Thunder-constructed reply. It stays in the pre-auth allowlist, and it still
validates credentials it carries and reports the resulting flags, but the
session's auth state comes from `AUTH` (§ 4).

```rust
Request {
    id: <client-chosen>,
    command: "HELLO",
    args: vec![Value::Map(vec![
        (Value::Str("version".into()), Value::Int(1)),
        (Value::Str("token".into()),   Value::Str("<jwt>".into())),  // OR
        (Value::Str("api_key".into()), Value::Str("<api-key>".into())),
        (Value::Str("client_name".into()), Value::Str("vectorizer-rust/2.5.16".into())),
    ])],
}
```

Server reply on success:

```rust
Response {
    id: <echoes Request.id>,
    result: Ok(Value::Map(vec![
        (Value::Str("server_version".into()), Value::Str("2.5.16".into())),
        (Value::Str("protocol_version".into()), Value::Int(1)),
        (Value::Str("capabilities".into()), Value::Array(vec![
            Value::Str("collections.list".into()),
            Value::Str("collections.get_info".into()),
            // … one entry per capability the connection's principal can call
        ])),
        (Value::Str("authenticated".into()), Value::Bool(true)),
        (Value::Str("admin".into()),         Value::Bool(false)),
    ])),
}
```

If the client requests `version > server's max`, the server replies with
its highest supported version and the client SHOULD downgrade or close.
A `HELLO` failure (bad credentials, version unsupported) is delivered as
a normal `Err(message)` response and the connection stays open in
`Unauthenticated` state — the client may retry with corrected
credentials before the server closes for inactivity.

## 6. Command catalog

RPC is the default protocol and the **primary data source**: every
data-plane capability in the registry (`src/server/capabilities.rs`) has a
command here. `assert_inventory_invariants` checks that at boot against
`protocol::rpc::dispatch::RPC_COMMANDS`, so a registry entry with no RPC
command fails the server rather than surprising a client.

`Auth` is the bucket enforced on the connection: `any` runs pre-auth,
`User` needs an authenticated session, `Admin` needs `Role::Admin`.

### Handshake

| Command | Auth | Args | Returns |
|---|---|---|---|
| `HELLO` | none | `[Map { version, token?, api_key?, client_name? }]` | `Map { server_version, protocol_version, capabilities, authenticated, admin }` |
| `PING` | any | `[]` | `Str("PONG")` |
| `AUTH` | none | (Thunder handshake, not a catalog command) | — |
| `QUIT` | any | (Thunder handshake) | `Str("OK")` |

### Collections

| Command | Auth | Args | Returns |
|---|---|---|---|
| `collections.list` | User | `[]` | `Array<Map>` |
| `collections.get_info` | User | `[Str(name)]` | `Map { dimension, metric, vector_count, … }` |
| `collections.get_stats` | User | `[Str(name)]` | `Map { collection, vector_count, is_empty }` |
| `collections.create` | User | `[Str(name), Map { dimension?, metric?, embedding_provider?, graph? }]` | `Map { name, dimension, metric, graph, success }` |
| `collections.delete` | Admin | `[Str(name)]` | `Map { success, name }` |
| `collections.list_empty` | User | `[]` | `Array<Str>` |
| `collections.cleanup_empty` | Admin | `[Map { dry_run }]` | `Map { removed, dry_run }` |
| `collections.force_save` | User | `[Str(name)]` | `Map { success, name, scope }` |
| `collections.set_ttl` | User | `[Str(name), Int(ttl_secs)?]` | `Map { collection, ttl_secs, status }` |
| `collections.get_ttl` | User | `[Str(name)]` | `Map { collection, ttl_secs }` |

`collections.create` rejects an `embedding_provider` the server does not
have, and a `dimension` that disagrees with that provider's native size —
the same two guards REST applies. `graph: { enabled: true }` attaches a
graph immediately, which is what makes the `graph.*` family reachable
without a second transport. `dry_run` on `cleanup_empty` must be a Map
field; a bare `Bool` is not read and the default is a real deletion.

`collections.set_ttl` configures the rule "vectors inserted or updated on
this collection expire `ttl_secs` seconds after they arrive". Omit the
second argument (or pass `Null`) to clear it; `0` is rejected, since it
would expire every insert on arrival. `VectorStore::insert` stamps
`__expires_at = now + ttl_secs` on each vector before the WAL record is
written, so a replay restores the original expiry, and a replica receives
the stamp as part of the vector rather than needing the rule. A vector
that already carries its own `__expires_at` keeps it — a per-vector expiry
is more specific than the collection rule. A payload whose JSON root is
not an object cannot hold the field and is rejected rather than stored
without an expiry.

The rule itself lives in the process-scoped store metadata map (key
`ttl:<collection>`), so it must be re-applied after a restart; the stamps
it produced are durable, because they are part of the payload.

### Vectors

| Command | Auth | Args | Returns |
|---|---|---|---|
| `vectors.get` | User | `[Str(collection), Str(id)]` | `Map { id, data, payload? }` |
| `vectors.insert` | User | `[Str(collection), Str(id), Array<Float>(data), Map(payload)?]` | `Map { id, success }` |
| `vectors.insert_text` | User | `[Str(collection), Str(id)?, Str(text), Map(payload)?]` | `Map { id, success }` |
| `vectors.update` | User | `[Str(collection), Str(id), Array<Float>(data), Map(payload)?]` | `Map { id, success }` |
| `vectors.delete` | User | `[Str(collection), Str(id)]` | `Map { success }` |
| `vectors.list` | User | `[Str(collection), Int(page)?, Int(limit)?]` | `Map { vectors, total, … }` |
| `vectors.embed` | User | `[Str(text), Str(model)?]` | `Map { embedding, model, dimension }` |
| `vectors.batch_insert` | User | `[Str(collection), Array<Map { id?, data, payload? }>]` | `Map { inserted, failed, results }` |
| `vectors.batch_insert_texts` | User | `[Str(collection), Array<Map { id?, text, payload? }>]` | `Map { inserted, failed, results }` |
| `vectors.batch_search` | User | `[Array<Map { collection, query, limit? }>]` | `Array<Map>` |
| `vectors.batch_update` | User | `[Str(collection), Array<Map { id, data, payload? }>]` | `Map { updated, failed, results }` |
| `vectors.batch_delete` | User | `[Str(collection), Array<Str>(ids)]` | `Map { deleted, failed, results }` |
| `vectors.move` | User | `[Str(src), Str(dst), Array<Str>(ids)]` | `Map { src, dst, moved, failed }` |
| `vectors.copy` | User | `[Str(src), Str(dst), Array<Str>(ids)]` | `Map { src, dst, copied, failed }` |
| `vectors.delete_by_filter` | User | `[Str(collection), Map(QdrantFilter)]` | `Map { deleted }` |
| `vectors.bulk_update_metadata` | User | `[Str(collection), Map(QdrantFilter), Map(patch)]` | `Map { updated }` |
| `vectors.set_expiry` | User | `[Str(collection), Str(id), Str(expires_at)]` | `Map { id, expires_at, success }` |

The batch commands answer `Ok` with per-item `results`; a failed item
carries `status: "error"` and its reason, so the envelope being `Ok` does
not mean every item landed. `QdrantFilter` conditions are tagged:
`{ must: [{ type: "match", key, match_value }] }`. Vectors are stored
normalized under a cosine metric, so a read returns the unit vector, not the
input.

`vectors.set_expiry` accepts a Unix-ms integer string or RFC3339 and stamps
`__expires_at` into the vector's payload. It takes effect on reads immediately:
`vectors.get` reports an expired vector as not found, and the search and
`vectors.list` paths drop it. Reclaiming the memory is the TTL reaper's job — it
sweeps every collection once per interval (60 s by default) — so an expired
vector still occupies space until the next sweep, it just is not served.

### Search

| Command | Auth | Args | Returns |
|---|---|---|---|
| `search.basic` | User | `[Str(collection), Str(query), Int(limit)?, Float(threshold)?]` | `Array<Map { id, score, payload? }>` |
| `search.by_text` | User | `[Str(collection), Str(query), Int(limit)?]` | `Array<Map>` |
| `search.by_file` | User | `[Str(collection), Map(request)]` | `Array<Map>` |
| `search.hybrid` | User | `[Str(collection), Map { query, limit?, algorithm?, alpha?, dense_k?, sparse_k?, final_k? }]` | `Map` |
| `search.semantic` | User | `[Map { collection, query, max_results?, semantic_reranking?, cross_encoder_reranking?, similarity_threshold? }]` | `Map` |
| `search.contextual` | User | `[Map { collection, query, max_results?, context_reranking?, context_weight?, context_filters? }]` | `Map` |
| `search.multi_collection` | User | `[Map { collections, query, max_per_collection?, max_total_results?, cross_collection_reranking? }]` | `Map` |
| `search.intelligent` | User | `[Map { query, collections?, max_results?, domain_expansion? }]` | `Map` |
| `search.explain` | User | `[Str(collection), Map { vector, k }]` | `Map` |
| `search.extra` | User | `[Map { query, collection, strategies?, max_results?, similarity_threshold? }]` | `Map { query, collection, strategies_used, total, results }` |

`search.extra` merges `basic`, `semantic` and `intelligent` (default
`["basic", "semantic"]`), first strategy wins on a duplicate id, sorted by
score. An unknown strategy name is skipped, not an error.

### Discovery

| Command | Auth | Args | Returns |
|---|---|---|---|
| `discovery.discover` | User | `[Map { query, include_collections?, exclude_collections?, max_bullets? }]` | `Map` |
| `discovery.filter_collections` | User | `[Map { query, include?, exclude? }]` | `Map` |
| `discovery.score_collections` | User | `[Map { query }]` | `Map` |
| `discovery.expand_queries` | User | `[Map { query, max_expansions?, include_definition?, include_features?, include_architecture? }]` | `Map` |
| `discovery.broad_discovery` | User | `[Map { queries, k? }]` | `Map` |
| `discovery.semantic_focus` | User | `[Map { collection, queries, k? }]` | `Map` |
| `discovery.promote_readme` | User | `[Map { chunks }]` | `Map` |
| `discovery.compress_evidence` | User | `[Map { chunks, max_bullets?, max_per_doc? }]` | `Map` |
| `discovery.build_answer_plan` | User | `[Map { bullets }]` | `Map` |
| `discovery.render_llm_prompt` | User | `[Map { plan, sources }]` | `Map` |

### File operations

| Command | Auth | Args | Returns |
|---|---|---|---|
| `file.content` | User | `[Map { collection, file_path, max_size_kb? }]` | `Map` |
| `file.list` | User | `[Map { collection, max_results?, sort_by?, min_chunks?, filter_by_type? }]` | `Map` |
| `file.summary` | User | `[Map { collection, file_path, summary_type?, max_sentences? }]` | `Map` |
| `file.chunks` | User | `[Map { collection, file_path, start_chunk?, limit?, include_context? }]` | `Map` |
| `file.outline` | User | `[Map { collection, max_depth?, include_summaries?, highlight_key_files? }]` | `Map` |
| `file.related` | User | `[Map { collection, file_path, limit?, similarity_threshold?, include_reason? }]` | `Map` |
| `file.search_by_type` | User | `[Map { collection, query, file_types, limit?, return_full_files? }]` | `Map` |
| `files.config_get` | User | `[]` | `Map { max_file_size, max_file_size_mb, allowed_extensions, reject_binary, default_chunk_size, default_chunk_overlap }` |

File **upload** stays REST-only (`POST /files/upload`): it is multipart
streaming, and RPC v1 has no streaming (§7).

### Graph

| Command | Auth | Args | Returns |
|---|---|---|---|
| `graph.enable` | User | `[Str(collection)]` | `Map { success, collection, node_count }` |
| `graph.status` | User | `[Str(collection)]` | `Map { collection, enabled, node_count, edge_count }` |
| `graph.list_nodes` | User | `[Str(collection)]` | `Map { nodes, count }` |
| `graph.neighbors` | User | `[Str(collection), Str(node_id), Int(depth)?]` | `Map { neighbors }` |
| `graph.find_related` | User | `[Str(collection), Str(node_id), Int(max_hops)?]` | `Map { related }` |
| `graph.find_path` | User | `[Str(collection), Str(from), Str(to)]` | `Map { path, found }` |
| `graph.create_edge` | User | `[Str(collection), Map { source, target, relationship_type, weight? }]` | `Map { edge_id, success }` |
| `graph.delete_edge` | User | `[Str(collection), Str(edge_id)]` | `Map { success }` |
| `graph.list_edges` | User | `[Str(collection)]` | `Map { edges, count }` |
| `graph.discover_edges` | User | `[Str(collection), Map { similarity_threshold?, max_per_node? }]` | `Map` |
| `graph.discover_edges_for_node` | User | `[Str(collection), Str(node_id), Map { similarity_threshold?, max_per_node? }]` | `Map` |
| `graph.discovery_status` | User | `[Str(collection)]` | `Map { total_nodes, nodes_with_edges, total_edges, progress_percentage }` |

Every command except `graph.enable` and `graph.status` requires a
graph-enabled collection. `relationship_type` is one of `SIMILAR_TO`,
`REFERENCES`, `CONTAINS`, `DERIVED_FROM`; anything else is rejected.

### Stats and providers

| Command | Auth | Args | Returns |
|---|---|---|---|
| `embedding.list_providers` | User | `[]` | `Map { providers, default_provider }` |
| `stats.database` | User | `[]` | `Map { collections, total_vectors, version, providers, default_provider }` |

### Admin

| Command | Auth | Args | Returns |
|---|---|---|---|
| `admin.stats` | User | `[]` | `Map { collections_count, total_vectors, version }` |
| `admin.status` | User | `[]` | `Map` |
| `admin.logs` | User | `[]` | `Map` |
| `admin.indexing_progress` | User | `[]` | `Map` |
| `admin.config_get` | User | `[]` | `Map` |
| `admin.config_update` | Admin | `[Map(patch)]` | `Map { success }` |
| `admin.backups_list` | User | `[Map { date? }]?` | `Map` |
| `admin.backups_create` | Admin | `[Map { name, collections? }]` | `Map` |
| `admin.backups_restore` | Admin | `[Map { backup_id }]` | `Map` |
| `admin.workspaces_list` | User | `[]` | `Map` |
| `admin.workspace_get` | User | `[]` | `Map` |
| `admin.workspace_add` | Admin | `[Map { collection_name, path }]` | `Map` |
| `admin.workspace_remove` | Admin | `[Str(path)]` | `Map` |
| `admin.restart` | Admin | `[]` | `Map { success, message }` |
| `admin.slow_queries_list` | User | `[]` | `Map` |
| `admin.slow_queries_config` | User | `[Map { threshold_ms, capacity? }]` | `Map` |

`admin.config_update` writes the patch as the whole `config.yml`; it is a
replace, not a merge. `admin.workspace_remove` keys on the workspace
**path**, not the collection name. `admin.restart` signals the process
(SIGHUP on unix, exit on Windows) — under a supervisor that does not act
on SIGHUP, the process keeps running.

### Auth / RBAC

| Command | Auth | Args | Returns |
|---|---|---|---|
| `auth.me` | User | `[Str(principal)?]` | `Map` |
| `auth.logout` | User | `[Str(token)]` | `Map` |
| `auth.refresh_token` | User | `[Str(token)]` | `Map` |
| `auth.validate_password` | User | `[Str(password)]` | `Map { valid, errors }` |
| `auth.introspect` | User | `[Str(token)]` | `Map` |
| `auth.audit` | User | `[Map { limit?, actor?, action?, from?, to? }]?` | `Map` |
| `auth.api_keys_create` | User | `[Map { name, permissions?, expires_in? }]` | `Map` |
| `auth.api_keys_list` | User | `[]` | `Map` |
| `auth.api_keys_revoke` | User | `[Str(key_id)]` | `Map` |
| `auth.api_keys_rotate` | User | `[Str(key_id)]` | `Map` |
| `auth.api_keys_create_scoped` | User | `[Map { name, permissions?, scopes?, collection?, expires_in? }]` | `Map` |
| `auth.users_create` | Admin | `[Map { username, password, roles? }]` | `Map { user_id, username, roles, success }` |
| `auth.users_list` | Admin | `[]` | `Map { count, users }` |
| `auth.users_delete` | Admin | `[Str(username)]` or `[Map { username }]` | `Map { success, username }` |
| `auth.users_change_password` | User (self) / Admin (anyone) | `[Map { username, new_password, current_password? }]` | `Map { success, username }` |

`auth.users_delete` refuses to remove the calling principal or the last
remaining admin. `auth.users_change_password` requires
`current_password` unless the caller is an admin. There is no
`auth.login` command: credentials travel in Thunder's `AUTH` handshake
(§4).

### Replication and cluster

| Command | Auth | Args | Returns |
|---|---|---|---|
| `replication.status` | User | `[]` | `Map` |
| `replication.stats` | User | `[]` | `Map` |
| `replication.replicas_list` | User | `[]` | `Map` (master only) |
| `replication.configure` | User | `[Map { role, bind_address?, master_address? }]` | `Map` |
| `cluster.nodes_list` | User | `[]` | `Map { count, nodes }` |
| `cluster.node_get` | User | `[Str(node_id)]` | `Map` |
| `cluster.node_remove` | Admin | `[Str(node_id)]` | `Map { success, node_id }` |
| `cluster.leader` | User | `[]` | `Map { mode, message }` |
| `cluster.role` | User | `[]` | `Map { role, node_id, leader_id, leader_url }` |
| `cluster.peer_add` | Admin | `[Map { address, role? }]` | `Map` |
| `cluster.failover` | Admin | `[Str(replica_id)]` | `Map` (master only) |
| `cluster.replica_resync` | Admin | `[Str(replica_id)]` | `Map` (master only) |
| `cluster.rebalance` | Admin | `[]` | `Map` |
| `cluster.rebalance_status` | User | `[]` | `Map` |

Commands that need replication or cluster mode answer with an explicit
"not enabled" error when the feature is off, rather than failing opaquely.

### Durability of writes

A successful mutating command marks the auto-save manager, which is what
lets the periodic compaction loop persist it — that loop only runs when
changes are pending. `collections.force_save` drives an immediate
store-wide compaction for a client that needs the write on disk now.
Creates and vector writes also replicate to replicas when this node runs
as master.

### Command name conventions

- All-lowercase, dot-separated. Dot represents a topical group
  (`collections.*`, `vectors.*`, `search.*`, `graph.*`, `file.*`,
  `discovery.*`, `admin.*`, `auth.*`, `cluster.*`).
- Names do **not** match the registry `id` field: the registry is
  singular and verb-suffixed (`collection.list`, `file.get_content`),
  the RPC catalog is plural and verb-first (`collections.list`,
  `file.content`). `capabilities::rpc_command_for` holds the mapping, and
  the boot assertion uses it to prove every capability is reachable.
- Adding a command means adding a dispatch arm, listing it in
  `RPC_COMMANDS`, and — if it serves a registry capability — mapping it in
  `rpc_command_for`.
- Server returns `Err("unknown command 'foo'")` for any command not in
  the dispatch table — no dynamic invocation, no reflection.

## 7. Streaming (deferred to v2)

Search results that exceed a single 64 MiB frame would need chunking.
v1 does not implement streaming; instead, the server caps the result
set at the number of items that fit within the 512 MiB frame cap (§ 1).
Larger result sets must page via the existing REST scroll API or wait
for v2.

When v2 lands, streaming will use a `last: bool` field in the response
envelope: the server emits multiple `Response` frames with the same
`id`, each containing a partial array; the final frame sets `last:
true`. SDKs MUST handle a single-frame response (`last` absent or
`true`) for v1 compatibility.

## 8. Versioning

The wire spec is versioned by an integer `protocol_version` returned
from `HELLO`. v1 is the only version. Breaking changes to framing,
envelope shape, or the `Value` type bump the major version; additive
changes (new commands, new fields with `#[serde(default)]`) do not.

A server that receives a `HELLO` declaring `version > server.max`
replies with `Ok(Map { protocol_version: <highest supported>, … })` and
the client SHOULD speak the older version. Clients SHOULD NOT
preemptively probe versions; declare what you can speak, accept what
the server returns.

## 9. Comparison with SynapRPC

Both products now run Thunder, so the wire is not merely compatible — it is
the same implementation. What differs is each product's *profile*: the parts
Thunder makes configurable.

| Aspect | SynapRPC | VectorizerRPC v1 |
|---|---|---|
| Framing | `[u32 LE len][msgpack body]` | identical (Thunder's) |
| Request shape | `{id, command, args}` | identical |
| Response shape | `{id, result: Result<Value, String>}` | identical |
| Value model | `thunder::Value` | identical; the SDKs alias it `VectorizerValue` |
| Auth | `AUTH` handshake, sticky per connection | identical |
| Error convention | RESP3 prefixes (`NOAUTH`/`WRONGPASS`/`NOPERM`) | identical |
| Server push | `SUBSCRIBE` + push frames (`PushPolicy::Enabled`) | **reserved** — no push-producing command in v1 |
| Scheme / default port | `synap://`, 15501 | `vectorizer://`, **15503** (Vectorizer's port range) |
| Max frame | 512 MiB | 512 MiB |

Wire-level parity is what lets one client library serve both products: a
Synap-compatible client talks to a Vectorizer server with only command-name
changes, and the SDK matrix shrinks to the command catalog because the
framing/codec/handshake layer is shared.

## 10. Security model

- **TLS**: optional, controlled by `config.rpc.tls.cert_path` /
  `tls.key_path`. When enabled, the server wraps the TCP listener with
  `tokio-rustls`. Clients connect to the same port over TLS. There is
  no STARTTLS — TLS is decided at connection time by the server config.
- **Origin pinning**: not applicable (this is a back-end transport, no
  cross-origin requests).
- **Rate limiting**: per-connection. The server caps in-flight requests
  per connection at `config.rpc.max_in_flight` (default 256). A
  connection that exceeds the limit will pause reading until a slot
  frees; clients SHOULD respect server-side backpressure rather than
  open a second connection.
- **Replay**: there is no replay protection at the protocol layer. If
  you need it, use a JWT with a short expiry plus client-side nonces in
  payload metadata.
- **Admin commands**: the server checks `Role::Admin` on the
  authenticated principal before dispatching admin-tagged registry
  entries. Failure does not terminate the connection.

## 11. Reference test vectors

These vectors are stable across server versions. SDKs SHOULD include
them as fixtures.

### Encode `Request { id: 1, command: "PING", args: [] }`

```
08 00 00 00                    # length = 8
93                             # array(3)
01                             #   id = 1
a4 50 49 4e 47                 #   command = "PING"
90                             #   args = array(0)
```

### Encode `Response { id: 1, result: Ok(Str("PONG")) }`

Both `Result<T, E>` *and* `VectorizerValue` use rmp-serde's default
externally-tagged enum representation, so an `Ok(Str("PONG"))` round-trips
as **two** nested one-key maps (one for the `Result` variant, one for
the `Value` variant). Clients MUST decode through both layers.

```
10 00 00 00                    # length = 16
92                             # array(2)
01                             #   id = 1
81                             #   result = map(1)
a2 4f 6b                       #     key = "Ok"
81                             #     value = map(1)
a3 53 74 72                    #       key = "Str"
a4 50 4f 4e 47                 #       value = "PONG"
```

### Decode failure cases

- `length` larger than 64 MiB → server closes connection, no response.
- `body` is not a valid MessagePack value → server emits `Err("frame
  decode failed: <rmp_serde error>")` with `id: u32::MAX` and closes
  the connection.
- `command` empty string → `Err("missing command")`.
- `args` arity mismatch → `Err("command '<name>' expects N args, got M")`.

## 12. Glossary

- **Frame**: the smallest unit on the wire — `[u32 LE len][body]`.
- **Envelope**: the `Request` or `Response` struct, MessagePack-encoded
  inside a frame's body.
- **Capability**: an operation registered in `src/server/capabilities.rs`.
- **Sticky auth**: per-connection auth state established by `HELLO`,
  not re-sent per request.
- **Default port 15503**: Vectorizer's RPC listener. The 15500-range is
  reserved for binary transports across the HiveLLM family
  (Synap=15500, Vectorizer=15503, future=15504+).
