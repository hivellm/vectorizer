## 1. Implementation

Upstream first: the counter exists before anything publishes it, and the
loader publishes before any handler reads it. Build after each item.

- [x] 1.1 Add `CollectionLoadProgress` to the `vectorizer` crate (`src/db/`):
      atomics for `expected` / `loaded`, a `complete` flag, and an `failed`
      flag with the error string. Accessors return a plain snapshot struct so
      handlers never touch atomics directly. Export from `db::`.
      **Done when:** the type compiles with unit tests covering
      `snapshot()` before start, mid-load, after completion, and after
      failure.
- [x] 1.2 Teach the loader to report: `load_all_persisted_collections_tracked(
      &self, progress: &CollectionLoadProgress)` in
      `persistence/loading.rs`, setting `expected` right after
      `extract_all_collections()` and incrementing `loaded` per inserted
      collection, on both the `.vecdb` and legacy raw-file paths. The existing
      `load_all_persisted_collections()` delegates with a throwaway progress,
      so no caller changes.
      **Done when:** existing loader tests still pass and a new test asserts
      the counters land at `expected == loaded == N` for a seeded store.
- [x] 1.3 Hold it in server state: `pub collection_load:
      Arc<CollectionLoadProgress>` on `VectorizerServer`
      (`server/mod.rs`), constructed in both the real bootstrap and the test
      harness (`new_for_tests`, which must start already-complete — it loads
      nothing).
      **Done when:** the workspace compiles, including the test harness.
- [x] 1.4 Publish from the background task (`core/bootstrap.rs`): call the
      tracked loader, and settle the flag on **every** exit — success,
      the loader `Err` arm (~line 706), and the auto-load-disabled arm
      (~line 714). A path that leaves `complete == false` is the bug this
      task is meant to prevent.
      **Done when:** each exit path sets `complete`, verified by reading the
      three arms, and the failure arm records the error string.
      **Found a fourth:** the task planned for three exits, but the
      cancel-before-start arm (`bootstrap.rs:512`) returns early too. Left
      alone it would strand the handle at `Pending` for the process's whole
      life and `/ready` would never answer. All four now settle.
- [x] 1.5 `GET /collections` (`rest_handlers/collections.rs`): add `loading`,
      `loaded_collections`, `expected_collections`. Leave `total_collections`
      as the count of returned items — SDKs read it. Also emits `load_state`,
      which carries the failure reason the other three cannot express.
      **Done when:** the response carries all four fields and existing REST
      tests are untouched.
- [x] 1.6 Readiness surface: a `readiness` block on `GET /health`
      (`rest_handlers/meta.rs`) — **without** changing `status: "healthy"`,
      which the Dockerfile `HEALTHCHECK` depends on — plus a new `GET /ready`
      returning 200 when complete and 503 + `Retry-After: 5` while loading.
      Register the route in `core/routing.rs`.
      **Done when:** both endpoints answer correctly for loading and loaded
      states.
      **Wider than planned:** `public_routes` alone was not enough. `/health`
      is exempted in three separate allow-lists — the auth middleware
      (`routing.rs:1218`), the HiveHub middleware
      (`hub/middleware.rs:155`) and `SigningConfig::exempt_paths`
      (`hub/request_signing.rs:80`) — so `/ready` would have 401'd under auth
      without all three. A fourth, different in kind: the HA middleware
      (`routing.rs:1328`) redirects to the leader, which would let a follower
      still loading answer with the leader's "ready" and take traffic it
      cannot serve. Exempted with the reason in place.

## 2. Tail (docs + tests — check or waive with tailWaiver)
- [x] 2.1 Update or create documentation covering the implementation
      (REST API reference for the new fields and `/ready`; a note in the
      upgrade docs that a partial list during warm-up is expected and how to
      tell it apart from data loss).
      `docs/api/openapi.yaml`: the `/ready` path with both responses and the
      `Retry-After` header, the four new `ListCollectionsResponse` fields,
      `CollectionLoadState` / `ReadinessResponse`, and a rewritten `/health`
      description saying plainly that it is liveness.
      **Found en route:** `docs/api/openapi.json` — the mirror the dashboard
      serves at `/api/docs/openapi.json` — had been frozen since 2025-11-30,
      a whole version behind (`1.6.0` vs `2.1.0`) and missing 3 endpoints.
      Regenerated from the YAML; it is a strict subset, so nothing
      hand-written was lost. Committed separately, since it rewrites the file.
- [x] 2.2 Write tests covering the new behavior: an integration test that
      observes a *partial* list carrying `loading: true` with
      `expected_collections > loaded_collections`, then the complete list with
      `loading: false`; `/ready` 503→200; and the settle-on-failure path.
      `rest_collection_load_readiness.rs` — 5 tests, including one pinning
      that `/health` keeps answering 200 during warm-up, and `/ready` added to
      the anonymous-route list in `rest_auth_enforcement.rs`, which is what
      actually exercises the three middleware allow-lists.
- [ ] 2.3 Run tests and confirm they pass (`cargo nextest run --workspace
      --lib --bins --tests`, plus clippy and fmt).
