## 1. Server: broadcast bus + WebSocket handler

- [x] 1.1 `DashboardEvent` enum + `broadcast::Sender<DashboardEvent>` field added to `RuntimeSampler` in `runtime_metrics.rs`. `start()` clones the snapshot once and forwards `DashboardEvent::Runtime(snap)` on every tick when a bus is wired. `dashboard_rx()` returns a fresh receiver (or a closed one when no bus is set)
- [x] 1.2 New `crates/vectorizer-server/src/server/ws/{mod,dashboard}.rs` housing the `WebSocketUpgrade` handler. Topic enum currently carries `Runtime` only (this task ships that topic; status / collections / logs land in §2)
- [x] 1.3 Wire protocol enums shipped: `ClientFrame::{Subscribe, Unsubscribe, Ping}` (`serde(tag = "op")`), `ServerFrame::{Event, Op}` with `Op::{Pong, Error}` and `ErrorCode::{StreamLag, BadFrame}`. JSON shapes pinned by 5 unit tests (`topic_round_trips_via_json`, `client_subscribe_frame_parses`, `client_unsubscribe_and_ping_parse`, `server_pong_and_error_serialize_to_op`, `server_event_frame_carries_topic_and_data`)
- [x] 1.4 Per-connection `serve_connection` does `tokio::select!` between `socket.recv()` and `rx.recv()`. Subscribe / unsubscribe mutate a per-connection `HashSet<Topic>`; outbound events are filtered against it. `RecvError::Lagged` emits `{op:"error",code:"stream_lag"}` and closes the socket. Bad client frames emit `bad_frame`. Binary frames are rejected. `cargo clippy -- -D warnings` clean

## 2. Server: status / collections / logs publishers

- [x] 2.1 `StatusSnapshot` struct + `DashboardEvent::Status(StatusSnapshot)` variant added in `runtime_metrics.rs`. `bootstrap.rs` spawns a 5 s `tokio::time::interval` task that builds `{online, version, uptime_seconds, collections_count}` (mirrors `GET /status`) and broadcasts via the shared `Sender`. `Topic::Status` and `Topic::of` mapping added in `ws/dashboard.rs`. 2 new unit tests pin the topic mapping + frame shape (8 total in the WS suite). cargo clippy clean
- [ ] 2.2 Collections snapshot publisher — every 30 s, broadcast `DashboardEvent::Collections` carrying the same shape `GET /collections` returns. Tied to the existing collection-mutation events so a create/delete also triggers an immediate publish
- [ ] 2.3 Logs publisher — tail the same log file `GET /logs` reads from; for each new line emit `DashboardEvent::Log(LogEntry)`. Cap inflight buffer at 256 entries per connection (drop oldest with a `stream_lag` error if the client falls behind)

## 3. Server: auth + routing

- [x] 3.1 `GET /ws/dashboard` registered inside `admin_router` in `routing.rs` so the existing admin auth gate validates the cookie session. CSRF only fires on `POST/PUT/PATCH/DELETE` so the upgrade `GET` already bypasses it without a special case
- [x] 3.2 `bootstrap.rs` creates the `broadcast::channel::<DashboardEvent>(1024)` and feeds the `Sender` to `RuntimeSampler::set_broadcast(...)` before `start()`. Status / collections / logs publishers (§2) reuse the same `Sender` once those topics ship

## 4. Client: provider + hook

- [x] 4.1 `dashboard/src/providers/WsDashboardProvider.tsx` opens one `WebSocket` per app instance to `${proto}//${host}/ws/dashboard` (cookie auth is automatic — browsers send `vectorizer_session` on the upgrade GET). Exponential-backoff reconnect (250 ms → 5 s cap) on `close` / `error`. Per-topic refcount drives subscribe / unsubscribe ops on the wire so the server stops publishing topics no tab is consuming. Mounted under `ProtectedRoute` in `AppRouter.tsx` so the bus only spins up for authenticated dashboard sessions
- [x] 4.2 `useWsTopic<T>(topic)` hook returns the latest typed snapshot via `useSyncExternalStore` (React renders only on actual change). `useWsStatus()` companion exposes `'connecting' | 'open' | 'closed'` for diagnostic UI without forcing every consumer to re-render on state flips. The `WsTopic` type currently carries `'runtime'`; `'status'`, `'collections'`, `'logs'` will widen as §2 publishers ship

## 5. Client: rewrite the polling hooks

- [x] 5.1 `useRuntimeMetrics` rewritten on `useWsTopic('runtime')`. The 1–2 s `setInterval` block is gone. Defensive snake↔camel mapping stays for partial-payload tolerance. A one-shot REST fetch on mount seeds the snapshot + qpsHistory before the first WS frame arrives so the UI doesn't flash "loading…" while the socket negotiates. Pages drop the obsolete `intervalMs` argument (`OverviewPage`, `MonitoringPage`); 6/6 hook unit tests green
- [x] 5.2 `useStatus` rewritten on `useWsTopic('status')` — REST one-shot seeds the snapshot, all subsequent updates flow via WS pushes (5 s cadence). `useStats` reads the cache + WAL snapshot from `/health` which is not on a WS topic yet, so it stays on REST polling. `WsTopic` widened to `'runtime' | 'status'`
- [ ] 5.3 Drop `setInterval` blocks from `OverviewPage`, `CollectionsPage`, `MonitoringPage`, `FileWatcherPage`, `LogsPage`. Each consumes `useWsTopic` for its primary data; one-shot lookups (e.g. opening a vector detail) keep using REST

## 6. Tests

- [ ] 6.1 Server: WS handshake test — open a connection, subscribe to `runtime`, assert at least one `DashboardEvent::Runtime` arrives within 2 s with the same shape `GET /metrics/runtime` returns
- [ ] 6.2 Server: subscribe / unsubscribe round-trip — subscribe to `status`, receive a frame, unsubscribe, confirm no further `status` frames arrive within 6 s (one publisher tick)
- [ ] 6.3 Server: slow-consumer handling — fill the broadcast queue past capacity, verify the laggy connection receives `error: "stream_lag"` and is closed
- [ ] 6.4 Client: `useWsTopic` unit test (vitest) — feed mock WS frames via a stub `WebSocket` and assert React renders the latest snapshot

## 7. Docs

- [ ] 7.1 `docs/specs/API_REFERENCE.md` — new "Streaming" section with the WS URL, auth contract (cookie session), client / server frame schemas, and topic catalogue
- [ ] 7.2 `dashboard/README.md` "Recent changes" entry calling out the WS migration and noting that REST endpoints stay live as fallback / for SDK callers

## 8. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 8.1 Update or create documentation covering the implementation
- [ ] 8.2 Write tests covering the new behavior
- [ ] 8.3 Run tests and confirm they pass
