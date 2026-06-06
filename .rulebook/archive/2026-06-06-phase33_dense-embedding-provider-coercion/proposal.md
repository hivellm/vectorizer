# Proposal: phase33_dense-embedding-provider-coercion

Source: [issue #306](https://github.com/hivellm/vectorizer/issues/306)

## Why

The running `hivehub/vectorizer:3.3.0` image silently coerces every
collection to BM25-512 regardless of the requested
`embedding_provider`. A `POST /collections` with
`embedding_provider: "fastembed"` (or `onnx`, `dense`,
`sentence-transformers`, `minilm`, `bge-small`,
`nomic-embed-text-v1.5`) returns `201 created` but reads back as
`embedding_provider: "bm25", dimension: 512`. The server-side
`POST /embed` endpoint ignores the `model` param and always
returns a 512-dim BM25 vector.

Downstream impact (hivellm/cortex): the hybrid retrieval pipeline
(vector + keyword + graph) is degraded to keyword-only. Paraphrase
and semantic queries that share no vocabulary with the corpus
return irrelevant results because the vector lane contributes
nothing. Cortex was forced to pin `CORTEX_EMBEDDER_DIM=512` only to
avoid `Invalid dimension` insert failures against this image.

Vectorizer is advertised as "designed for semantic search and
top-k nearest neighbor queries", but the shipped image cannot
actually serve semantic search. Three problems to address:

1. **Silent coercion is a contract bug.** Even if dense is
   unsupported in a given deployment, `embedding_provider` MUST be
   honoured or rejected with a clear `unsupported_provider`
   error — never silently swapped to a different provider /
   dimension.
2. **`/embed` MUST honour `model`** or reject the request — never
   ignore the param.
3. **Dense providers need to be reachable in the shipped image**
   (FastEmbed bundled or a documented enable path via env / config
   / model mount) so consumers can run semantic search out of the
   box.

## What Changes

1. **Root-cause investigation** — confirm whether the coercion is
   a code bug (provider lookup falls through to BM25), a config
   gap (Docker image disables `fastembed` feature), or a model-
   asset gap (ONNX weights not present). Findings recorded in the
   task's `design.md`.
2. **Honour `embedding_provider`** on `POST /collections` — if the
   requested provider is registered and ready, use it; otherwise
   return `400 unsupported_provider` with a list of available
   providers in the body. No silent fall-through.
3. **Honour `/embed` `model`** — same rule: serve the requested
   model or return `400 unsupported_model` with the registered
   model list. The default model when `model` is omitted MUST be
   logged and exposed via `GET /stats` so callers can discover it.
4. **Stats endpoint advertises providers** — `GET /stats` (or a
   new `GET /providers`) lists every registered embedding provider
   with its dimension so callers can discover what the deployment
   actually supports.
5. **Ship a working dense default** — the official Docker image
   bundles or auto-downloads at least one dense provider (proposal:
   FastEmbed `all-MiniLM-L6-v2`, 384-dim) so a fresh container can
   serve semantic search without extra setup. Larger models
   (`nomic-embed-text-v1.5`, `bge-small`) remain opt-in via env /
   config.
6. **Documentation** — `docs/embedding/providers.md` lists
   supported providers, default vs. opt-in, dimensions, and how to
   enable each in a Docker deployment. CHANGELOG entry under
   v3.4.0 calling out the contract change.

## Impact

- Affected specs: `specs/phase33_dense-embedding-provider-coercion/`
- Affected code:
  - `crates/vectorizer/src/embedding/` (provider registry, default
    selection, model loader)
  - `crates/vectorizer-server/src/handlers/collections.rs`
    (create-collection contract)
  - `crates/vectorizer-server/src/handlers/embed.rs` (`/embed`
    contract)
  - `crates/vectorizer-server/src/handlers/stats.rs` (advertise
    providers)
  - `Dockerfile` (bundle / pre-fetch dense model assets)
  - `docs/embedding/providers.md`
- Breaking change: **YES — contract change** for clients that
  silently relied on the coercion. Callers that posted
  `embedding_provider: "fastembed"` and expected success will now
  get `400 unsupported_provider` if the deployment cannot serve
  fastembed. The change is signalled via the CHANGELOG and v3.4.0
  release notes.
- User benefit:
  - Misconfiguration surfaces immediately with an explicit error
    instead of silently writing BM25-512 vectors to a "fastembed"
    collection.
  - Cortex (and any other downstream) can run real semantic search
    against a default Docker image.
  - `GET /stats` / `GET /providers` lets callers discover the
    available embedding surface without trial-and-error.
