## 1. Server: collections topic

- [x] 1.1 Define `CollectionsSnapshot { collections: Vec<CollectionSummary> }` and `CollectionSummary { name, vector_count, dimension }` in `runtime_metrics.rs`. Add `DashboardEvent::Collections(CollectionsSnapshot)`
- [x] 1.2 Add `Topic::Collections` to `ws/dashboard.rs` and the matching `Topic::of` arm. Update the unit tests
- [x] 1.3 Spawn a 30 s tick task in `bootstrap.rs` that builds the snapshot and broadcasts. `broadcast::Sender::send` returns an error when no receivers are alive — drop it on the floor so the publisher remains a no-op while idle
- [x] 1.4 Hook `create_collection` / `delete_collection` / `rename_collection` mutation handlers to push an immediate `Collections` snapshot so the UI reflects the change without waiting 30 s

## 2. Server: logs topic

- [x] 2.1 Define `DashboardEvent::Log(LogEntry)` reusing the existing `LogEntry` shape from `GET /logs`
- [x] 2.2 Add `Topic::Logs` + `Topic::of` arm + unit test
- [x] 2.3 Tail the active log file (the same one `GET /logs` reads from). Each new line emits one `Log` event. Lagged consumers fall through the existing `RecvError::Lagged` path → `error: stream_lag` and the WS handler closes the socket

## 3. Client

- [x] 3.1 Widen `WsTopic` in `WsDashboardProvider.tsx` to `'runtime' | 'status' | 'collections' | 'logs'`
- [x] 3.2 Replace the `setInterval` in `OverviewPage`, `CollectionsPage`, `FileWatcherPage`, `LogsPage` with `useWsTopic` for the live path. Each page keeps a REST one-shot for initial paint (matches the `useRuntimeMetrics` pattern)

## 4. Tests

- [x] 4.1 Server: subscribe to `collections`, mutate via `POST /collections`, assert an immediate frame arrives within 1 s
- [x] 4.2 Server: subscribe to `logs`, write a line to the log file, assert a `Log` frame arrives within 1 s
- [x] 4.3 Client: integration test for `useWsTopic('collections')` via stub WebSocket

## 5. Docs

- [x] 5.1 Extend `docs/specs/API_REFERENCE.md` Streaming section with the two new topics

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 6.1 Update or create documentation covering the implementation
- [x] 6.2 Write tests covering the new behavior
- [x] 6.3 Run tests and confirm they pass
