# Proposal: phase16_rpc-full-capability-parity

Source: gap audit follow-up to phase11_sdk-tier-demotion-api. Companion
to phases 12-15 (REST + SDK parity). Targets the native VectorizerRPC
transport (`vectorizer://host:15503`, length-prefixed MessagePack/TCP),
NOT gRPC.

Wire spec: `docs/specs/VECTORIZER_RPC.md`.
Server dispatch: `crates/vectorizer-server/src/protocol/rpc/dispatch.rs`.
Client wrappers: `sdks/{rust,typescript,python,go,csharp}/.../rpc/`.

## Why

The native VectorizerRPC transport is positioned as the *recommended*
client transport (`sdks/rust/README.md` and the wire spec § 0). It is
also the path Cortex's `cortex-embedder-worker` and the Synap
consumers use for hot-path traffic. Yet the v1 dispatch table in
`crates/vectorizer-server/src/protocol/rpc/dispatch.rs:57-76` wires
only **5 commands**:

- `HELLO`, `PING` (handshake/health)
- `collections.list`, `collections.get_info`
- `vectors.get`
- `search.basic`

`search.intelligent` is declared in the catalog but explicitly
**unwired** (`dispatch.rs:69-71`: `Response::err(id, "search.intelligent: not yet wired in v1 dispatch")`).
Every other REST/MCP capability — write ops, batch ops, discovery
pipeline, file ops, graph, replication, admin, auth — has **no RPC
command** at all. Consumers that prefer RPC for performance reasons
must either (a) downgrade to HTTP for any non-read operation, or (b)
implement the missing RPC command server-side themselves.

This is exactly the gap that phase11 + phases 12-15 are closing on the
REST/SDK side. Without the same parity work on RPC, every operation
phases 12-15 add becomes "REST-only" for the first release after
landing — and any consumer that uses the RPC transport ends up running
*two* transports per process.

The capability registry (`crates/vectorizer-server/src/server/capabilities.rs`,
referenced by the wire spec § 6) already enumerates every operation
with a `Transport::Both` or `Transport::McpOnly` flag. The dispatch
table just hasn't followed.

## What Changes

### 1. Server dispatch — full v1 catalog

Extend `crates/vectorizer-server/src/protocol/rpc/dispatch.rs` so every
operation flagged `Transport::Both` in the capability registry has a
dispatch arm. The set MUST include, at minimum:

**Collection management**
- `collections.create`, `collections.delete`
- `collections.list_empty`, `collections.cleanup_empty`
- `collections.force_save`
- `collections.rename`, `collections.reindex` (phase14 — see below)
- `collections.snapshot`, `collections.list_snapshots`, `collections.restore_snapshot` (phase14)
- `collections.set_ttl`, `collections.reencode` (phase13)

**Vector ops (single + batch)**
- `vectors.insert`, `vectors.insert_text`, `vectors.update`, `vectors.delete`
- `vectors.list`, `vectors.embed`
- `vectors.batch_insert`, `vectors.batch_insert_texts`, `vectors.batch_search`, `vectors.batch_update`, `vectors.batch_delete`
- `vectors.move` (phase11), `vectors.copy` (phase13)
- `vectors.delete_by_filter`, `vectors.bulk_update_metadata` (phase13)
- `vectors.set_expiry` (phase13)

**Search**
- `search.by_text`, `search.by_file`, `search.hybrid`
- `search.intelligent` (un-stub the existing arm)
- `search.semantic`, `search.contextual`, `search.multi_collection`
- `search.explain` (phase14)

**Discovery pipeline**
- `discovery.discover`, `discovery.filter_collections`, `discovery.score_collections`, `discovery.expand_queries`
- `discovery.broad_discovery`, `discovery.semantic_focus`, `discovery.promote_readme`, `discovery.compress_evidence`, `discovery.build_answer_plan`, `discovery.render_llm_prompt`

**File ops**
- `file.content`, `file.list`, `file.summary`, `file.chunks`, `file.outline`, `file.related`, `file.search_by_type`

**Graph**
- `graph.list_nodes`, `graph.neighbors`, `graph.find_related`, `graph.find_path`
- `graph.create_edge`, `graph.delete_edge`, `graph.list_edges`
- `graph.discover_edges`, `graph.discover_edges_for_node`, `graph.discovery_status`

**Admin/observability**
- `admin.stats`, `admin.status`, `admin.logs`, `admin.indexing_progress`
- `admin.config_get`, `admin.config_update`
- `admin.backups_list`, `admin.backups_create`, `admin.backups_restore`
- `admin.workspaces_list`, `admin.workspace_get`, `admin.workspace_add`, `admin.workspace_remove`
- `admin.restart`
- `admin.slow_queries_list`, `admin.slow_queries_config` (phase14)

**Auth/RBAC**
- `auth.me`, `auth.logout`, `auth.refresh_token`, `auth.validate_password`
- `auth.api_keys_create`, `auth.api_keys_list`, `auth.api_keys_revoke`
- `auth.api_keys_rotate`, `auth.api_keys_create_scoped` (phase15)
- `auth.users_create`, `auth.users_list`, `auth.users_delete`, `auth.users_change_password`
- `auth.introspect`, `auth.audit` (phase15)

**Replication / cluster**
- `replication.status`, `replication.configure`, `replication.stats`, `replication.replicas_list`
- `cluster.failover`, `cluster.replica_resync`, `cluster.peer_add`, `cluster.rebalance`, `cluster.rebalance_status` (phase15)

### 2. Capability registry alignment

Every RPC command added to dispatch MUST have a registry entry (id +
`Transport::Both`) in `crates/vectorizer-server/src/server/capabilities.rs`.
The `HELLO` reply's `capabilities` array MUST list every wired
command — clients use it for runtime feature detection.

### 3. Argument shape consistency

Each command's `args` array MUST match the catalog table format from
the wire spec (positional, MessagePack `VectorizerValue`s). Where REST
accepts a JSON body with optional fields, RPC takes a single
`Map`-typed `args[N]` carrying the same field set. No JSON-over-RPC
detour.

### 4. SDKs — typed RPC wrappers

For every dispatch arm added, expose a typed wrapper in:

- `sdks/rust/src/rpc/commands.rs` (extend the existing
  `impl RpcClient` block)
- `sdks/typescript/src/rpc/commands.ts`
- `sdks/python/rpc/commands.py`
- `sdks/go/rpc/commands.go`
- `sdks/csharp/.../Rpc/Commands.cs`

Each wrapper builds the positional `args`, calls
`RpcClient::call(name, args)`, and decodes the response into a typed
struct using the same field-by-field decoders that
`commands.rs:CollectionInfo` uses today (no `serde_json::from_value`
detour — wire is MessagePack).

### 5. Streaming

`vectors.batch_search` and `discovery.broad_discovery` results can
exceed the v1 64 MiB single-frame cap. Spec § 7 promises streaming in
v2; this task does NOT ship v2 streaming, but it MUST surface a
typed `RpcError::FrameTooLarge` so clients can fall back to REST for
oversize results without ambiguous decoding errors.

### 6. Versioning

This is an additive expansion of v1's command catalog. `protocol_version`
stays at 1; clients negotiate via the `capabilities` array returned by
`HELLO`. SDKs bump 3.6 → 3.7 (or 3.7 → 3.8 if landed after phase15).

## Impact

- Affected specs:
  - `.rulebook/tasks/phase16_rpc-full-capability-parity/specs/rpc-full-capability-parity/spec.md`
  - Update `docs/specs/VECTORIZER_RPC.md` § 6 catalog table
- Affected code:
  - `crates/vectorizer-server/src/protocol/rpc/dispatch.rs` (every new arm)
  - `crates/vectorizer-server/src/protocol/rpc/server.rs` (state plumbing for new caps)
  - `crates/vectorizer-server/src/server/capabilities.rs` (Transport::Both flags)
  - `crates/vectorizer/src/protocol/rpc/mod.rs` (shared types if any new ones)
  - `sdks/rust/src/rpc/commands.rs` (extend `impl RpcClient`)
  - `sdks/{typescript,python,go,csharp}/.../rpc/...` (mirror)
- Breaking change: NO — purely additive. Existing 5 commands keep their
  exact signatures and semantics.
- User benefit:
  - RPC consumers (Cortex embedder worker, Synap consumers, perf-sensitive
    clients) can use the entire feature surface over the recommended
    transport instead of two-transport hybrids.
  - `search.intelligent` stops returning `"not yet wired"` errors.
  - Phases 12-15 land with same-day RPC parity instead of REST-first
    rollouts.

## Constraints

- Wire format MUST stay MessagePack `VectorizerValue` per the spec —
  no JSON envelopes.
- Each new command MUST be reachable through the same dispatch
  state-machine (`HELLO`-then-data) — no parallel auth path.
- Admin-flagged commands MUST check `ConnectionAuth::admin` before
  dispatching, mirroring the REST `require_admin_middleware`.
- No reflection, no dynamic loading. One match arm per command (spec
  § 1 "Forbidden").
- Per-frame size cap stays at 64 MiB; oversized responses MUST surface
  as a typed error, not a transport panic.

## Acceptance

- Every operation flagged `Transport::Both` in the capability registry
  has a dispatch arm and a typed SDK wrapper in all 5 SDKs.
- `search.intelligent` no longer returns the "not yet wired" stub.
- The `HELLO` response's `capabilities` array lists every wired
  command and matches the dispatch table exactly (asserted by an
  integration test).
- Each new command has a unit test (request/response round-trip) and
  an integration test against a live server.
- `docs/specs/VECTORIZER_RPC.md` § 6 catalog reflects the wired set
  exactly — no entries marked "not yet wired".
- Workspace SDK versions bumped (3.6 → 3.7 minimum, dependent on
  landing order with phases 12-15).
