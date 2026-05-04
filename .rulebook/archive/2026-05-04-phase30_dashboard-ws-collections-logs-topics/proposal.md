# Proposal: phase30_dashboard-ws-collections-logs-topics

## Why

Phase29 shipped the dashboard WebSocket multiplexer and migrated the
two highest-frequency polling loops onto it (`runtime` 1–2 s,
`status` 5–30 s). Three lower-cadence sites remain on REST polling:

| Page / hook         | Cadence | Endpoint           |
|---------------------|---------|--------------------|
| `OverviewPage`      | 30 s    | `GET /collections` |
| `CollectionsPage`   | 30 s    | `GET /collections` |
| `FileWatcherPage`   | 5 s     | filewatcher status |
| `LogsPage`          | 2 s     | `GET /logs`        |

The 2 s `/logs` loop is the next-worst offender after the
already-killed runtime / status loops. Collections at 30 s is mostly
a refresh hint after a mutation, but a push topic gives instant
feedback on create / delete / rename.

This task adds the two remaining publishers + topics to the
multiplexer so the dashboard can drop the last four `setInterval`
blocks.

## What Changes

### Server side

1. **`Topic::Collections` + `DashboardEvent::Collections(CollectionsSnapshot)`**.
   `CollectionsSnapshot` carries a slim list of `{name, vector_count,
   dimension}` per collection — enough for the dashboard table
   without re-shipping the full per-collection metadata that
   `GET /collections/{name}` returns. A 30 s tick task in
   `bootstrap.rs` builds the snapshot from `state.store.list_collections()`
   and broadcasts. Collection mutations (`create_collection`,
   `delete_collection`, `rename_collection`) get an optional hook
   that triggers an immediate publish so the UI reflects the change
   without waiting for the next tick.

2. **`Topic::Logs` + `DashboardEvent::Log(LogEntry)`**. Tail the same
   log file `GET /logs` reads from. Each new line emits a single
   `Log` event. Per-connection inflight buffer cap of 256 entries —
   if a slow consumer falls behind by more than that the connection
   is dropped with `error: stream_lag` (broadcast::RecvError::Lagged
   already handles this by virtue of being on the shared bus).

3. **Auth caveat for `Logs`**: the topic is admin-only. The handler
   already runs behind the admin gate so all subscribed connections
   are admins; no extra filtering needed.

### Client side

4. **Drop `setInterval`** from `OverviewPage`, `CollectionsPage`,
   `FileWatcherPage`, `LogsPage`. Each consumes the new
   `useWsTopic('collections' | 'logs')`. One-shot REST is kept as the
   initial-paint fallback (matches the `useRuntimeMetrics` /
   `useStatus` pattern from phase29).

5. **`WsTopic` widens** to `'runtime' | 'status' | 'collections' | 'logs'`.

## Impact

- Affected code:
  - `crates/vectorizer-server/src/server/runtime_metrics.rs`
    (`CollectionsSnapshot`, `LogEntry`, two new `DashboardEvent`
    variants).
  - `crates/vectorizer-server/src/server/ws/dashboard.rs` (two new
    `Topic` variants + `Topic::of` arms).
  - `crates/vectorizer-server/src/server/core/bootstrap.rs` (two new
    publisher tasks).
  - `crates/vectorizer-server/src/server/rest_handlers/collections.rs`
    (publish hook from create / delete / rename).
  - `dashboard/src/providers/WsDashboardProvider.tsx` (widen
    `WsTopic`).
  - `dashboard/src/pages/{Overview,Collections,FileWatcher,Logs}Page.tsx`
    (drop `setInterval`).
- Affected specs: `docs/specs/API_REFERENCE.md` Streaming section
  gains the two extra topics.
- Breaking change: NO — REST endpoints stay live as fallback / SDK
  callers.
- User benefit: zero remaining polling loops in the dashboard. Every
  live data path is push-driven.
