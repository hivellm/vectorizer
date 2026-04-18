# Proposal: phase4_embedding-provider-trait-dispatch

## Why

`src/embedding/mod.rs:1600-1610` dispatches to concrete provider types via `as_any().downcast_ref::<T>()` inside an if-chain:

```rust
if let Some(bm25) = provider.as_any().downcast_ref::<Bm25Embedding>() { ... }
else if let Some(tfidf) = provider.as_any().downcast_ref::<TfIdfEmbedding>() { ... }
else if let Some(char_ngram) = provider.as_any().downcast_ref::<CharNGramEmbedding>() { ... }
// ... 4 more branches
```

And `src/server/mcp_handlers.rs:30-87` duplicates a similar 7-branch match on provider kind. Problems:

- Adding a new provider requires edits in multiple files.
- `downcast_ref` defeats the point of `dyn Trait` — any abstraction the trait aimed to enforce is bypassed.
- Duplicated dispatch sites drift from each other (one site gets a fix, the other doesn't).

## What Changes

1. Extend the `EmbeddingProvider` trait with the methods currently requiring downcast (e.g., `fn name(&self) -> &str`, `fn config_json(&self) -> serde_json::Value`, any capability introspection).
2. Remove every `as_any().downcast_ref::<T>()` call in `src/embedding/` and `src/server/mcp_handlers.rs`.
3. If some concrete behavior really can't be expressed as a trait method (rare), route through a sealed enum `ProviderKind` stored on the provider, not a runtime downcast.
4. Document the "how to add a new provider" flow in `docs/development/adding-embedding-provider.md` — it should be a single-file change.

## Impact

- Affected specs: embedding spec
- Affected code: `src/embedding/mod.rs`, `src/embedding/*_provider.rs`, `src/server/mcp_handlers.rs`
- Breaking change: NO (internal)
- User benefit: new providers trivially added; dispatch sites stay in sync by construction.
