## 1. Analysis

- [ ] 1.1 Enumerate every `as_any().downcast_ref::<T>()` call site in `src/`; record concrete type + what method is being called after downcast in `design.md`
- [ ] 1.2 Group per-behavior; design the trait method(s) that would replace each downcast

## 2. Trait extension

- [ ] 2.1 Add the required methods to `EmbeddingProvider` trait with sensible default impls where possible
- [ ] 2.2 Implement the new methods on each provider: `Bm25Embedding`, `TfIdfEmbedding`, `CharNGramEmbedding`, `SvdEmbedding`, `BertEmbedding`, `FastEmbed`, `MiniLM`

## 3. Dispatch replacement

- [ ] 3.1 Replace the if-chain at `src/embedding/mod.rs:1600-1610` with a single trait-method call
- [ ] 3.2 Replace the 7-branch match at `src/server/mcp_handlers.rs:30-87` with a single trait-method call
- [ ] 3.3 Delete any now-unused `as_any` methods (keep only if genuinely needed for debug tooling)

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 4.1 Write `docs/development/adding-embedding-provider.md` documenting the new single-file flow for adding a provider
- [ ] 4.2 Add a test asserting every currently-registered provider implements the new trait methods without panicking
- [ ] 4.3 Run `cargo test --all-features` and confirm all tests pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
