## 1. Implementation

Ordered so the sentinel cannot be shadowed before anything starts honouring
it, and so no text path can reach a provider-less collection unguarded.

- [x] 1.1 Reserve the name. A single `RAW_VECTOR_PROVIDER: &str = "none"`
      constant, defined beside the `CollectionConfig::embedding_provider`
      field it is a value of, plus an `is_raw_vector()` accessor so call sites
      compare through one place instead of spreading a string literal.
      **Design correction.** The plan said `register_provider` should reject
      the name. It returns `()` and has 80 call sites across 36 files, so
      making it fallible is a large diff for a small guard. The property that
      actually matters — the sentinel cannot be shadowed — is obtained by
      **checking it before consulting the registry** at every use site: the
      sentinel branch wins, so a provider registered under that name can never
      capture a collection. Cheaper, and it is the mechanism rather than a
      second line of defence around one.
      **Done when:** a unit test registers a provider literally named `none`
      and asserts collection creation still takes the raw-vector path.
- [x] 1.2 `create_collection` (`rest_handlers/collections.rs`) accepts
      `embedding_provider: "none"`: skip provider resolution and skip the
      dimension check, persisting the sentinel on `CollectionConfig`. Every
      other value keeps today's behaviour — unknown names still 400
      `unsupported_provider`, mismatched widths still 400
      `provider_dimension_mismatch`.
      **Done when:** creating a 384-, 768- and 1536-wide collection succeeds
      with the sentinel and still fails without it.
- [x] 1.3 Close the text paths. `insert_text` / `batch_insert_texts` /
      `search/text` / `hybrid_search` against a sentinel collection return a
      typed error naming `/insert_vectors` and `POST /collections/{n}/search`
      as the operations that do work. A new `VectorizerError` variant with a
      stable `code()` — the SDKs match on `error_type`, so this is contract.
      **Done when:** each text entry point returns the typed error rather than
      embedding with BM25, verified per endpoint.
- [x] 1.4 Same treatment on the other transports: RPC dispatch and the MCP
      tools that create collections or insert text. REST-only would leave the
      hole open on two surfaces, and the capability registry asserts parity.
      **Done when:** the RPC/MCP create paths accept the sentinel and their
      text paths reject it with the same code.
- [x] 1.5 Make the sentinel **discoverable**, not just accepted. Verified on a
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
- [x] 1.6 Report sentinel collections as having no provider rather than
      listing them under `bm25`. The phase33 §4 block exists precisely so
      callers can see which provider a collection uses; showing `bm25` for a
      collection that has none reintroduces the confusion it was added to
      remove.
      **Done when:** a sentinel collection appears with no provider and is
      excluded from the per-provider counts.
      **Correction on the second half.** There are no per-provider counts to
      exclude it from — nothing in the codebase groups collections by
      provider, so that clause described a report that does not exist. What
      the audit did find is worse and is what got fixed: both
      `GET /collections` and `GET /collections/{name}` reported the *server
      default* provider for every collection, reading nothing from the
      collection's own config. A sentinel collection read as `bm25`, and so
      did any collection created with a non-default provider. Both routes now
      report `config.embedding_provider`, `null` for the sentinel.

## 2. Tail (docs + tests — check or waive with tailWaiver)
- [x] 2.1 Document the pre-vectorized workflow end to end — create with
      `embedding_provider: "none"`, insert with `/insert_vectors`, search with
      `POST /collections/{name}/search` — in the REST reference and
      `openapi.yaml`, including what text operations do on such a collection.
      Update `benchmarks/external/overlay/engine/clients/vectorizer/configure.py`
      to use the native endpoint and delete the comment explaining the
      Qdrant-compat detour; that comment existing is the bug report.
      **Done.** `openapi.yaml` and `openapi.json` both document
      `embedding_provider` on the create request — it was undocumented on the
      request schema in *both*, so the field phase33 made load-bearing was
      invisible to anyone reading the spec — and both mark it nullable on
      `CollectionInfo`, which the sentinel now makes reachable. The REST
      reference gained a "Pre-computed Vectors" walkthrough (create → insert →
      search, with the refusal contract and the discovery fields), and
      `/insert` and `/insert_vectors` were added to its endpoint index; both
      were missing. `configure.py` now creates through `POST /collections`
      with the sentinel, and the comment explaining the Qdrant-compat detour
      is gone — which was the point: that comment was the bug report.
- [x] 2.2 Tests: creation at several widths with and without the sentinel;
      each text entry point rejecting with the typed error; the round trip
      through `.vecdb` (a restarted collection must still be provider-less, or
      the legacy `#[serde(default)]` quietly turns it back into `bm25` — the
      exact shape of the persistence bugs this repo has had before); and the
      reserved-name registration failing.
      **Correction on the last clause.** Reserved-name registration does not
      fail — 1.1 replaced that design with match-before-lookup, so there is no
      rejection to test. The property that took its place is tested instead:
      `a_provider_named_none_cannot_capture_a_raw_vector_collection` registers
      a provider literally named `none` at 512 and creates a 384-wide
      collection through it. The two branch orderings give opposite answers,
      so the test is sharp — verified by sabotage: consulting the registry
      first makes it fail with `provider_dimension_mismatch`.
      The `.vecdb` round trip found a trap worth recording: `get_collection`
      returns a DashMap `Ref` whose read lock lives to the end of the *block*,
      not the last use. Holding one across `restore_native_snapshot` — which
      deletes the collection and so wants the shard's write lock — hangs the
      test forever. Same re-entrancy as the phase39 production deadlock.
- [x] 2.3 Full gate: `cargo nextest run --workspace --lib --bins --tests`,
      clippy, fmt.
      **Green:** 2057 tests run, 2057 passed, 9 skipped; clippy clean across
      the workspace; fmt applied. The pre-push hook caught one regression this
      work introduced that the pre-commit hook does not run:
      `stats_advertises_providers_block` asserted every `providers` row has a
      numeric `dimension`, which was exactly right while every row was a
      registered provider. Both copies now branch on `supports_text` and pin
      both shapes rather than relaxing the original invariant.
