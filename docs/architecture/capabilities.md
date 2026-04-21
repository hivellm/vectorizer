# Capability Registry — REST/MCP Parity

`CLAUDE.md`'s **REST-First Architecture** rule mandates: *"REST and MCP must
have identical functionality."* This document explains how that contract is
made checkable, what the registry covers, what it intentionally does not,
and how to add a new operation without breaking either surface.

## Where the registry lives

- `src/server/capabilities.rs` — the [`Capability`] struct, the
  [`Transport`] / [`AuthBucket`] enums, the live [`inventory()`]
  function, and the [`assert_inventory_invariants()`] boot-time check.
- `src/server/mcp/tools.rs::tools_from_inventory()` — derives MCP
  [`Tool`] structs from the registry. Two unit tests
  (`registry_tools_are_a_subset_of_legacy_tools` and
  `registry_and_legacy_agree_on_overlapping_input_schemas`) enforce
  byte-for-byte schema parity with the legacy hand-written list at
  `get_mcp_tools()`.
- `src/server/core/bootstrap.rs` — calls
  `assert_inventory_invariants()` on every server boot so a registry
  typo crashes startup loudly instead of silently desyncing the MCP
  tool list and the REST router.

## What's in the registry

Every entry covers one **conceptual operation** the server exposes on
its data plane. Each `Capability` declares:

- `id` — stable identifier (e.g. `"vector.delete"`).
- `summary` — one-line description used as both the MCP tool description
  and (eventually) the OpenAPI `summary` field.
- `mcp_tool_name` + `mcp_input_schema` — present for any operation
  reachable via MCP.
- `rest` — `(method, path)` for any operation reachable via REST.
- `auth` — `Public` / `User` / `Admin` bucket the operation lives in.
- `transport` — explicit `Both` / `RestOnly` / `McpOnly` flag that
  drives the parity test's reachability matrix.

The registry currently covers **31 MCP tools + the canonical REST
counterparts for 29 of them**. Two tools (`search_extra`,
`get_collection_stats`) are tagged `McpOnly` with documented rationale
in the registry. One `RestOnly` entry (`auth.login`) pins the design
choice that authentication is REST-only because MCP clients attach
pre-issued JWTs at the transport layer.

## What's intentionally NOT in the registry

The audit at the start of this work catalogued **130+ REST routes** vs.
31 MCP tools. The non-overlapping ~100 routes are intentionally
transport-specific:

| Surface | REST routes | Why REST-only |
|---|---|---|
| Authentication (`/auth/*`) | 12 | MCP carries JWT in transport; session + key + user mgmt is HTTP-shaped. |
| Admin / Setup / Backups / Workspace | 18 | Server lifecycle, file I/O, admin gates. Not data-plane. |
| Qdrant compatibility (`/qdrant/*`) | 40 | Faithful Qdrant API replica; MCP would re-invent. Use REST `/qdrant/*` for Qdrant clients. |
| Replication (`/replication/*`) | 4 | HA control plane. Operators reach over REST. |
| Monitoring (`/metrics`, `/prometheus/metrics`, `/logs`, `/indexing/progress`) | 4 | Prometheus + ops dashboards consume HTTP scrapers. |
| Multipart upload (`/files/upload`) | 1 | HTTP multipart streaming; MCP would need base64 + chunking. |
| Alternative protocols (`/graphql`, `/graphiql`, `/umicp`, `/dashboard*`) | 7 | Different query language / static assets. |
| HiveHub multi-tenant (`/hub/backups/*`, `/hub/usage/*`) | 6 | Tenant isolation, backup multipart restore. |

These are documented in `docs/api/parity-matrix.md` as known-gaps with
rationale; they are deliberately *not* in `inventory()` so the parity
test does not try to call MCP for them.

## Adding a new operation

1. Decide `Transport`:
   - **`Both`** — the default for any new data-plane operation. Needs
     both an MCP tool name + schema and a REST `(method, path)`.
   - **`RestOnly`** — only when the operation is HTTP-shaped (auth,
     multipart upload, lifecycle). Document the *why* in the registry
     entry's `summary`.
   - **`McpOnly`** — should be rare and time-bounded. Either add a
     REST counterpart or accept the asymmetry with a documented reason.
2. Add the `Capability` entry to `inventory()` — keep it grouped with
   topical neighbours (the order is the order MCP clients see).
3. Add a `schema_<tool_name>()` helper near the bottom of
   `capabilities.rs`. Run the suite:
   - `registry_tools_are_a_subset_of_legacy_tools` will fail until you
     also add the legacy entry to `mcp/tools.rs::get_mcp_tools()`.
   - `registry_and_legacy_agree_on_overlapping_input_schemas` will fail
     on any byte-level schema drift between the two sources.
4. Wire the REST route in `src/server/core/routing.rs`. The boot-time
   invariant assertion will catch a forgotten entry on the next start.
5. Update `docs/api/parity-matrix.md` if the new operation changes the
   gap counts.

## Why a hand-written registry instead of `#[derive]` codegen?

Earlier sketches considered a procedural macro that walked handler
signatures to produce the inventory. We rejected it for three reasons:

1. **Auth bucket is not in the type system** — the macro would need
   to read attribute parsing for every handler, doubling complexity.
2. **JSON schema authoring is intentional design** — the input shape a
   client sees is API surface; auto-deriving from request types
   couples the API to internal struct evolution.
3. **The static `Vec<Capability>` is grep-able** — adding an entry is
   ~25 lines and reviewable in a diff. A macro hides that surface.

The cost of writing each entry by hand is paid once per operation; the
schema-parity unit tests catch the failure mode (silent drift between
MCP and REST schemas) that drove the work in the first place.

## Source tree

```
src/server/
├── capabilities.rs           # Capability struct + inventory()
├── core/
│   └── bootstrap.rs          # calls assert_inventory_invariants() on boot
└── mcp/
    └── tools.rs              # tools_from_inventory() + parity tests
docs/
├── architecture/
│   └── capabilities.md       # this file
└── api/
    └── parity-matrix.md      # gap catalogue with rationale
```
