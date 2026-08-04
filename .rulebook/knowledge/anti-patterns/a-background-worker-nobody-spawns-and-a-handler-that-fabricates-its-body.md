# A background worker nobody spawns, and a handler that fabricates its body

**Category**: correctness
**Tags**: correctness, ttl, rest, dead-code, anti-pattern

## Description

Two dead features found in one sweep, both shaped so that every local signal looked healthy:

**TtlReaper was never spawned.** The type was implemented, exported from `db::mod`, instrumented with three Prometheus families, and documented. `grep -rn 'TtlReaper::spawn'` outside its own module returned nothing. So `vectors.set_expiry` recorded `__expires_at` and nothing ever acted on it — and because `Payload::is_expired` had exactly one vector caller (the reaper's own sweep), expired vectors also kept being returned by search. Write path present, read path present, worker absent.

**`GET /collections/{name}/vectors/{id}` fabricated its response**: `vec![0.1; 512]` for any id, in any collection, existing or not, with a comment admitting it. Worse than a 404, because the body is indistinguishable from real data. The same handler was mounted on `POST /vector` — the route the capability registry declares for `vector.get` — where its `Path<(String, String)>` extractor cannot be satisfied on a parameterless route.

Why the tests did not catch either:
- `capability_registry_route_reachability` asserts a route *resolves* (not 404/405). A handler that resolves and then returns garbage, or fails extraction, passes.
- No test asserted the *content* of a vector fetch, only status codes.
- The reaper had zero tests, so nothing exercised the spawn.

Checks worth repeating on this codebase:
- For any background worker, grep for its spawn outside its own module. Exported + instrumented is not running.
- For a per-entity worker, ask whether entities are created at runtime. A boot-time spawn per collection misses everything created later, and there is no single choke point for collection creation here (REST, RPC, MCP, disk load) — so sweep store-wide and enumerate per tick.
- `TtlReaper::drop` signals shutdown, so the handle must be held for the server's lifetime. A spawn whose handle is dropped stops immediately and silently.
- A route-resolution test needs a companion content test, or fabricated bodies survive.

## When to Use

Auditing whether a feature that looks complete is actually reachable: background workers, and handlers whose responses nothing asserts the content of.
