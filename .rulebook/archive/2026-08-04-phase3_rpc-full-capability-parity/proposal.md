# Proposal: phase3_rpc-full-capability-parity

## Why

VectorizerRPC is the default protocol and is meant to be the primary data
source for every client. Today it is not: a live sweep of all 102 routed
commands (Docker container, auth enabled) showed the transport is healthy
but the command surface is incomplete. Clients that speak RPC only cannot
reach operations REST and MCP expose, which forces SDK consumers back onto
HTTP for a subset of work and breaks the promise that one protocol suffices.

Measured against `crates/vectorizer-server/src/server/capabilities.rs`
(the declared single source of truth for REST/MCP parity) and the live
route table:

- `graph.*` is **unreachable** for any collection created over RPC.
  `collections.create` hardcodes `graph: None`, disk load only enables a
  graph when `config.graph.enabled` is already true, and there is no RPC
  counterpart to `POST /graph/enable/{collection}`. The 10 graph commands
  themselves work — verified by enabling the graph over REST and then
  driving all of them over RPC — but RPC alone cannot reach that state.
- Four `auth.users_*` commands are routed only to answer
  "REST-only in v1 (RpcState does not carry AuthHandlerState)", so user
  management is impossible over RPC.
- Six registry capabilities have no RPC command at all:
  `embedding.list_providers`, `search.extra_combined`,
  `collection.get_stats`, `stats.get_database_stats`, `graph.enable`,
  `graph.status`.
- Five cluster/node routes have no RPC command: node list, node get,
  node remove, leader, role.
- `GET /files/config` has no RPC command.
- `collections.create` silently drops most of the config handed to it. It
  honours only `dimension`, `metric` and `embedding_provider`; `hnsw`,
  `quantization`, `normalization`, `sharding`, `graph` and `encryption`
  are pinned to defaults with no error, so RPC cannot create the same
  collection REST can.
- `docs/specs/VECTORIZER_RPC.md` catalogs 7 commands while dispatch routes
  102, and its claim that command names "match the registry `id` field
  exactly" is false (registry is singular `collection.list`, RPC plural
  `collections.list`).

Two further gaps found while sizing the work turn out to matter more than
any missing command, because they attack the "primary data source" claim
directly:

- **No RPC write marks auto-save.** The REST handlers call
  `AutoSaveManager::mark_changed()` in 15 places; the RPC dispatch table
  calls it zero times. The periodic loop only compacts when
  `changes_detected` is set (`crates/vectorizer/src/db/auto_save.rs`), so
  data written exclusively over RPC is never persisted by it. A graceful
  shutdown force-saves unconditionally and hides the problem; a hard kill
  (SIGKILL, container OOM, crash) loses every RPC-only write since boot.
- **`collections.force_save` is inert.** It checks that the collection
  exists and answers `success: true` without saving anything, so a client
  has no way to force durability over RPC either.

RPC mutations also never reach replicas: the REST create/insert paths call
`master.replicate(...)`, the RPC handlers do not.

## What Changes

1. `RpcState` carries what the missing handlers need: auth handler state
   for user management, and embedding-manager access for provider listing.
2. New commands close every data-plane gap listed above.
3. `collections.create` honours the full `CollectionConfig` it is given,
   including `graph`, so a graph-enabled collection is creatable over RPC.
4. `rpc_capability_names()` advertises every new command so capability
   negotiation stays truthful.
5. The capability registry gains an explicit RPC column, and the
   boot-time invariant assertion covers it, so parity becomes
   machine-checked instead of rediscovered by hand.
6. `docs/specs/VECTORIZER_RPC.md` catalogs the real command set and
   corrects the naming claim.

## Out of scope (rationale, not deferral)

These REST routes are deliberately not mirrored onto RPC because they are
not data-plane operations:

- `/dashboard`, `/graphql`, `/graphiql`, `/metrics`, `/umicp*` —
  presentation and foreign-protocol surfaces with their own wire formats.
- `/qdrant/*` — a compatibility surface whose purpose is to speak
  Qdrant's protocol; mirroring it onto RPC would defeat that purpose.
- `/files/upload` — multipart streaming upload. RPC v1 has no streaming
  (`VECTORIZER_RPC.md` §7 defers it to v2) and a single-frame upload
  would be bounded by `max_frame_bytes`.
- `/setup/*` — first-run bootstrap wizard driven by the dashboard.
- `/hub/*` — HiveHub SaaS integration, an outbound HTTP client surface.

## Impact

- Affected specs: `.rulebook/tasks/phase3_rpc-full-capability-parity/specs/rpc-parity/spec.md`,
  `docs/specs/VECTORIZER_RPC.md`
- Affected code: `crates/vectorizer-server/src/protocol/rpc/dispatch.rs`,
  `crates/vectorizer-server/src/protocol/rpc/server.rs`,
  `crates/vectorizer-server/src/server/core/bootstrap.rs`,
  `crates/vectorizer-server/src/server/capabilities.rs`
- Breaking change: NO — every change is additive. Existing commands keep
  their names, argument shapes and result shapes.
- User benefit: an RPC-only client reaches the entire data plane, so the
  SDKs stop needing an HTTP fallback.

Source: live sweep of 102/102 routed commands plus effect verification of
every mutating command, against `vectorizer:thunder-test` in Docker.
