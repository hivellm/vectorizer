# Design: phase33_dense-embedding-provider-coercion

## Root cause (confirmed by code trace 2026-06-06)

The silent BM25-512 coercion is a **three-part code bug**, not a
config or asset gap. FastEmbed IS compiled into the published
`hivehub/vectorizer:3.3.0` image (default features enabled per
`Cargo.toml:3`), but it is **not registered** at runtime and **not
reachable** via any API endpoint.

### Part 1 — `POST /collections` ignores `embedding_provider`

`crates/vectorizer-server/src/server/rest_handlers/collections.rs:169`
(`create_collection`) extracts `name`, `dimension`, `metric` from
the body but never reads `embedding_provider`. The field is
silently dropped.

`CollectionConfig` (`crates/vectorizer/src/models/mod.rs:339`) has
no `embedding_provider` field at all — there is no on-disk record
of which provider a collection was created with. Every collection
inherits the server's single default provider.

### Part 2 — Bootstrap registers only ONE provider

`crates/vectorizer-server/src/server/core/bootstrap.rs:230-238`
constructs an `EmbeddingManager`, calls `build_default_provider()`
once, and registers exactly that provider as the default. The
manager supports a multi-provider registry
(`crates/vectorizer/src/embedding/providers/manager.rs:33,112-114`)
but only one slot is ever populated.

`build_default_provider()` (bootstrap.rs:72-110) reads
`config.embedding.model` from `config.yml`. If the value is
missing or unparseable, it defaults to `bm25` (lines 46-58). Since
the official Docker image ships no `config.yml`, **every container
boots with BM25-512 as the only registered provider**.

### Part 3 — `POST /embed` ignores `model`

`crates/vectorizer-server/src/server/rest_handlers/vectors.rs:238`
(`embed_text`) accepts the request body, ignores any `model`
field, and calls `embedding_manager.embed(text)` — the default
provider unconditionally.

## Decisions

### D1: store the provider on the collection

Add `embedding_provider: String` (not `Option`) to
`CollectionConfig` so persistence is deterministic and old
collections become unambiguous after migration (legacy collections
default to `"bm25"`).

Reason: `Option<String>` opens the same silent-coercion door
(`None` falls back to default). A string with a documented default
forces the call site to be explicit.

### D2: register every compiled-in provider at boot

Bootstrap MUST iterate every provider gated by an enabled feature
flag and register each. The default provider stays config-driven
(`config.embedding.model`); when no config is present in the
Docker image, the default is the **highest-dimensional dense
provider available** (FastEmbed if built in), with BM25 kept as a
named fallback.

Reason: makes per-request provider selection actually possible.
Without this, `embedding_provider: "fastembed"` cannot be honoured
even when fastembed is compiled in.

### D3: error shapes

- `400 unsupported_provider` — `{ error, requested, available }`
- `400 dimension_mismatch` — `{ error, provider,
  provider_dimension, requested_dimension }`
- `400 unsupported_model` — `{ error, requested, available }`

Map via `VectorizerError::UnsupportedProvider { .. }` and
`UnsupportedModel { .. }` variants. Existing handlers translate
via the central error responder.

### D4: discovery endpoint

Extend `GET /stats` with a `providers` array rather than adding
`GET /providers`:

```json
{
  "providers": [
    {"name": "fastembed", "kind": "dense",  "dimension": 384, "default": true},
    {"name": "bm25",       "kind": "sparse", "dimension": 512, "default": false}
  ],
  ...
}
```

Reason: clients already poll `/stats`; one extra field beats a
second round-trip. MCP gets a mirrored `list_providers` tool.

### D5: bundled dense model

FastEmbed `all-MiniLM-L6-v2` (384-dim, cosine). Pinned by checksum
in `Dockerfile`. Pre-fetched at image build time so first-boot
needs no network. Larger models (`nomic-embed-text-v1.5`,
`bge-small`) stay opt-in via config / env.

Reason: 384-dim is the cheapest dense model that still beats BM25
on paraphrase tasks. ~80 MB on disk fits the image budget.

### D6: Dockerfile features

Pin `--features fastembed,hive-gpu,transmutation,simd` explicitly
instead of relying on `default = [...]`. Drops the silent failure
mode where `NO_DEFAULT_FEATURES=1` without `FEATURES=fastembed`
produces a BM25-only image.

## Migration

- Existing `.vecdb` files without `embedding_provider` in
  metadata are loaded as `embedding_provider = "bm25"` — matches
  the actual behaviour they shipped with.
- v3.4.0 release notes document the contract change. Clients that
  silently relied on coercion will now see `400 unsupported_provider`
  and can either correct the request or call `GET /stats` to
  discover what is available.
