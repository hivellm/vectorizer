## 1. Implementation

Ordered so the sentinel cannot be shadowed before anything starts honouring
it, and so no text path can reach a provider-less collection unguarded.

- [ ] 1.1 Reserve the name. A `RAW_VECTOR_PROVIDER: &str = "none"` constant
      next to the embedding manager, and `register_provider` rejects it.
      Without this, registering a provider literally called `none` would
      shadow the sentinel and silently re-enable the coercion phase33 removed.
      **Done when:** a unit test asserts registration under that name fails,
      and the constant is the single definition every other item imports.
- [ ] 1.2 `create_collection` (`rest_handlers/collections.rs`) accepts
      `embedding_provider: "none"`: skip provider resolution and skip the
      dimension check, persisting the sentinel on `CollectionConfig`. Every
      other value keeps today's behaviour — unknown names still 400
      `unsupported_provider`, mismatched widths still 400
      `provider_dimension_mismatch`.
      **Done when:** creating a 384-, 768- and 1536-wide collection succeeds
      with the sentinel and still fails without it.
- [ ] 1.3 Close the text paths. `insert_text` / `batch_insert_texts` /
      `search/text` / `hybrid_search` against a sentinel collection return a
      typed error naming `/insert_vectors` and `POST /collections/{n}/search`
      as the operations that do work. A new `VectorizerError` variant with a
      stable `code()` — the SDKs match on `error_type`, so this is contract.
      **Done when:** each text entry point returns the typed error rather than
      embedding with BM25, verified per endpoint.
- [ ] 1.4 Same treatment on the other transports: RPC dispatch and the MCP
      tools that create collections or insert text. REST-only would leave the
      hole open on two surfaces, and the capability registry asserts parity.
      **Done when:** the RPC/MCP create paths accept the sentinel and their
      text paths reject it with the same code.
- [ ] 1.5 Make the sentinel **discoverable**, not just accepted. Verified on a
      running 3.6.1: `GET /stats` answers
      `{"providers": [{"name": "bm25", "dimension": 512, "default": true}],
      "default_provider": "bm25"}` — one provider, and the RPC doc comment on
      `handle_embedding_list_providers` tells clients to use exactly this list
      to "pick a valid `embedding_provider` (and matching dimension)". A
      client that follows that instruction can never learn `"none"` exists, so
      accepting it silently would ship a feature nobody can find.
      Advertise it in all three inventories — `GET /stats`, the RPC
      `embedding.list_providers`, the MCP `list_providers` — as an entry that
      names itself as taking any dimension and carrying no text support.
      **Done when:** each inventory lists it, and the doc comment above stops
      implying the dimension must match a registered provider.
- [ ] 1.6 Report sentinel collections as having no provider rather than
      listing them under `bm25`. The phase33 §4 block exists precisely so
      callers can see which provider a collection uses; showing `bm25` for a
      collection that has none reintroduces the confusion it was added to
      remove.
      **Done when:** a sentinel collection appears with no provider and is
      excluded from the per-provider counts.

## 2. Tail (docs + tests — check or waive with tailWaiver)
- [ ] 2.1 Document the pre-vectorized workflow end to end — create with
      `embedding_provider: "none"`, insert with `/insert_vectors`, search with
      `POST /collections/{name}/search` — in the REST reference and
      `openapi.yaml`, including what text operations do on such a collection.
      Update `benchmarks/external/overlay/engine/clients/vectorizer/configure.py`
      to use the native endpoint and delete the comment explaining the
      Qdrant-compat detour; that comment existing is the bug report.
- [ ] 2.2 Tests: creation at several widths with and without the sentinel;
      each text entry point rejecting with the typed error; the round trip
      through `.vecdb` (a restarted collection must still be provider-less, or
      the legacy `#[serde(default)]` quietly turns it back into `bm25` — the
      exact shape of the persistence bugs this repo has had before); and the
      reserved-name registration failing.
- [ ] 2.3 Full gate: `cargo nextest run --workspace --lib --bins --tests`,
      clippy, fmt.
