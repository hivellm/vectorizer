# Proposal: phase6_raw-vector-collections

Let a collection hold pre-computed vectors of any width, through the native
API.

Found while building the external benchmark
([phase5](../phase5_external-benchmark-comparison/proposal.md)), but it is a
product defect and not a benchmark inconvenience.

## Why

`POST /insert_vectors` exists for callers who bring their own embeddings — its
own doc comment says so: *"Useful when the client owns its own embedder"*.
There is no way to create a collection to put them in.

`create_collection` always resolves an embedding provider, defaulting to
`bm25`, and rejects any dimension that differs from that provider's:

```
POST /collections {"name": "x", "dimension": 384, "metric": "cosine"}
400 provider_dimension_mismatch
   Provider 'bm25' has dimension 512, request asked for 384
```

512 is BM25's width. Every real embedding model is some other width — 384 for
`all-MiniLM-L6-v2`, 768 for most BERT-family models, 1536 for OpenAI
`text-embedding-3-small`, 100 for glove. So a user who computed embeddings
elsewhere, which is the entire premise of `/insert_vectors`, cannot create a
collection for them unless a provider of exactly that dimension happens to be
registered.

The validation itself is right. Phase33 (issue #306) added it because the
server used to silently coerce mismatched dimensions to BM25, which produced
collections that searched badly for reasons nobody could see. The defect is
that closing the silent-coercion path also closed the legitimate one: there is
no way to say *"this collection has no embedding provider, I will supply the
vectors"*.

The workaround that exists today is worse than the bug. The Qdrant-compatible
`PUT /qdrant/collections/{name}` goes straight to `store.create_collection`
with no provider resolution, so it accepts any width — meaning the
documented-as-compatibility surface is the only way to reach a first-class
native feature. phase5's benchmark client uses it and says why in a comment;
that comment should become obsolete.

## What Changes

Reserve `"none"` as an embedding-provider name meaning *this collection stores
raw vectors*.

- `create_collection` accepts `embedding_provider: "none"` and skips the
  dimension check entirely — there is no provider to disagree with.
- Text-facing operations on such a collection fail with a typed error naming
  `/insert_vectors`, rather than coercing to BM25. That is the whole point:
  the silent path phase33 closed must stay closed.
- `"none"` is rejected as a name when registering a real provider, or the
  sentinel could be shadowed.

A sentinel string rather than making `embedding_provider` an `Option<String>`:
the field lives on `CollectionConfig`, which has hundreds of struct literals
across the workspace, and the same reasoning already decided
`phase1_persist-collection-ttl-config` to put `ttl_secs` on
`PersistedCollection` instead. A reserved value costs one constant; a type
change costs every construction site and buys nothing here.

Persistence needs no new field — `embedding_provider` is already part of
`CollectionConfig` and already round-trips through `.vecdb`.

## Impact

- Affected specs: none existing; behaviour recorded in this task.
- Affected code: `crates/vectorizer-server/src/server/rest_handlers/collections.rs`
  (create validation), the text insert/search handlers, the RPC/MCP equivalents,
  `crates/vectorizer/src/embedding/providers/manager.rs` (reserve the name),
  and `GET /stats` provider discovery.
- Breaking change: NO — purely additive. Collections that omit
  `embedding_provider` behave exactly as today.
- User benefit: the pre-vectorized workflow the server already advertises
  becomes reachable without the Qdrant-compat detour, and phase5 can benchmark
  the native path end to end.
