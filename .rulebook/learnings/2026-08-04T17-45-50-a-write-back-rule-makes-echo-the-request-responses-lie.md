# A write-back rule makes echo-the-request responses lie
**Source**: manual
**Date**: 2026-08-04
**Related Task**: phase1_collection-ttl-is-never-applied
**Tags**: ttl, api-honesty, rest
Adding a store-level rule that rewrites what callers submit turns every "echo the request back" response into a potential lie, and the handlers doing it look correct in isolation.

Concrete case: `PATCH /collections/{n}/vectors/{id}/expiry` with `{"expires_at": null}` answered `{"expires_at": null}` by echoing the parsed request. Once a collection TTL re-stamps a cleared expiry inside `VectorStore::update`, the stored value is `now + ttl` while the response still claims null. Fixed by reading the vector back after the write and reporting the stored value.

The generalisation, worth checking on the next task of this shape: after introducing normalisation / defaulting / clamping in a lower layer, grep the handlers that write through it for responses built from the *request* rather than from the *result*. Same class as the `set_collection_ttl` bug being fixed here — the endpoint reported what the caller asked for, not what the system did.

Also worth noting: `file_size_budget` is its own test binary in `crates/vectorizer/tests/`, so a verification pass built from `--lib` plus named integration tests skips it. An earlier commit in the same session pushed `rest_handlers/vectors.rs` 25 LOC over budget and it stayed invisible until a `cargo test --workspace` run. Use `--workspace` before claiming suites are green.