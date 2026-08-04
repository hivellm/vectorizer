# Apply per-collection policy at the store write choke point, not per handler

**Category**: architecture
**Tags**: none

## Description

Every server-side write in this repo funnels through `VectorStore::insert` / `VectorStore::update`: REST (insert_one_text, insert_vectors, copy/move), RPC dispatch, MCP handlers, gRPC, GraphQL mutations, file upload, hub tenant clone. Verified with `grep -rn 'store\.insert(' crates/vectorizer-server/src`. So a collection-scoped rule needs exactly two hooks, not ~15.

Three details that make the difference between working and subtly broken:

1. **Stamp before the WAL record.** `log_wal_insert` runs early in `insert`; mutating the vectors after it would make a replay recompute the value from replay time. Stamping first also means a replica receives the derived value as data and needs no copy of the rule.
2. **Hook `update` too, not just `insert`.** An update replaces the payload wholesale, so a caller-supplied payload silently drops anything the insert hook added. TTL-stamped vectors became immortal on first update until `update` got the same treatment.
3. **Read the rule from a source that is not the collections map.** `insert` takes `get_collection_mut` per 1000-vector chunk; reading the rule from the same DashMap risks the shard-lock re-entrancy trap documented on `get_collection`/`get_collection_mut` (a real production deadlock in phase39). The store metadata `DashMap` is a separate map, so `self.collection_ttl(name)` is safe to call before the loop.

Reject inputs the rule cannot be applied to instead of storing them unruled: a payload whose JSON root is not an object cannot hold `__expires_at`, so the insert fails loudly. Accepting it would have recreated the silent no-op the task existed to remove.

## When to Use

A collection-scoped rule (TTL, default payload fields, quota tagging, provenance stamps) must apply to every vector regardless of which transport wrote it.

## When NOT to Use

Rules that depend on request context the store cannot see (caller identity, tenant headers, auth scope) — those belong in middleware, and the store cannot reconstruct them.
