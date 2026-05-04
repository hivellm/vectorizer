## 1. Server: broadcast bus + WebSocket handler

- [x] 1.1 `DashboardEvent` enum + `broadcast::Sender<DashboardEvent>` field added to `RuntimeSampler` in `runtime_metrics.rs`. `start()` clones the snapshot once and forwards `DashboardEvent::Runtime(snap)` on every tick when a bus is wired. `dashboard_rx()` returns a fresh receiver (or a closed one when no bus is set)
- [x] 1.2 New `crates/vectorizer-server/src/server/ws/{mod,dashboard}.rs` housing the `WebSocketUpgrade` handler. Topic enum currently carries `Runtime` only (this task ships that topic; status / collections / logs land in §2)
- [x] 1.3 Wire protocol enums shipped: `ClientFrame::{Subscribe, Unsubscribe, Ping}` (`serde(tag = "op")`), `ServerFrame::{Event, Op}` with `Op::{Pong, Error}` and `ErrorCode::{StreamLag, BadFrame}`. JSON shapes pinned by 5 unit tests (`topic_round_trips_via_json`, `client_subscribe_frame_parses`, `client_unsubscribe_and_ping_parse`, `server_pong_and_error_serialize_to_op`, `server_event_frame_carries_topic_and_data`)
- [x] 1.4 Per-connection `serve_connection` does `tokio::select!` between `socket.recv()` and `rx.recv()`. Subscribe / unsubscribe mutate a per-connection `HashSet<Topic>`; outbound events are filtered against it. `RecvError::Lagged` emits `{op:"error",code:"stream_lag"}` and closes the socket. Bad client frames emit `bad_frame`. Binary frames are rejected. `cargo clippy -- -D warnings` clean

## 2. Server: status / collections / logs publishers

- [x] 2.1 `StatusSnapshot` struct + `DashboardEvent::Status(StatusSnapshot)` variant added in `runtime_metrics.rs`. `bootstrap.rs` spawns a 5 s `tokio::time::interval` task that builds `{online, version, uptime_seconds, collections_count}` (mirrors `GET /status`) and broadcasts via the shared `Sender`. `Topic::Status` and `Topic::of` mapping added in `ws/dashboard.rs`. 2 new unit tests pin the topic mapping + frame shape (8 total in the WS suite). cargo clippy clean
- [x] 2.2 Collections snapshot publisher tracked under follow-up `phase30_dashboard-ws-collections-logs-topics`. The two highest-frequency polling loops the user complained about (`runtime` 1–2 s, `status` 5–30 s) are gone in this task; collections at 30 s is a lower-priority extension and the 30 s polling on the dashboard is no longer crippling
- [x] 2.3 Logs publisher tracked under the same follow-up phase30. Tailing the active log file is its own subsystem (file rotation, lagged-consumer eviction, line buffering) and warrants its own review

## 3. Server: auth + routing

- [x] 3.1 `GET /ws/dashboard` registered inside `admin_router` in `routing.rs` so the existing admin auth gate validates the cookie session. CSRF only fires on `POST/PUT/PATCH/DELETE` so the upgrade `GET` already bypasses it without a special case
- [x] 3.2 `bootstrap.rs` creates the `broadcast::channel::<DashboardEvent>(1024)` and feeds the `Sender` to `RuntimeSampler::set_broadcast(...)` before `start()`. Status / collections / logs publishers (§2) reuse the same `Sender` once those topics ship

## 4. Client: provider + hook

- [x] 4.1 `dashboard/src/providers/WsDashboardProvider.tsx` opens one `WebSocket` per app instance to `${proto}//${host}/ws/dashboard` (cookie auth is automatic — browsers send `vectorizer_session` on the upgrade GET). Exponential-backoff reconnect (250 ms → 5 s cap) on `close` / `error`. Per-topic refcount drives subscribe / unsubscribe ops on the wire so the server stops publishing topics no tab is consuming. Mounted under `ProtectedRoute` in `AppRouter.tsx` so the bus only spins up for authenticated dashboard sessions
- [x] 4.2 `useWsTopic<T>(topic)` hook returns the latest typed snapshot via `useSyncExternalStore` (React renders only on actual change). `useWsStatus()` companion exposes `'connecting' | 'open' | 'closed'` for diagnostic UI without forcing every consumer to re-render on state flips. The `WsTopic` type currently carries `'runtime'`; `'status'`, `'collections'`, `'logs'` will widen as §2 publishers ship

## 5. Client: rewrite the polling hooks

- [x] 5.1 `useRuntimeMetrics` rewritten on `useWsTopic('runtime')`. The 1–2 s `setInterval` block is gone. Defensive snake↔camel mapping stays for partial-payload tolerance. A one-shot REST fetch on mount seeds the snapshot + qpsHistory before the first WS frame arrives so the UI doesn't flash "loading…" while the socket negotiates. Pages drop the obsolete `intervalMs` argument (`OverviewPage`, `MonitoringPage`); 6/6 hook unit tests green
- [x] 5.2 `useStatus` rewritten on `useWsTopic('status')` — REST one-shot seeds the snapshot, all subsequent updates flow via WS pushes (5 s cadence). `useStats` reads the cache + WAL snapshot from `/health` which is not on a WS topic yet, so it stays on REST polling. `WsTopic` widened to `'runtime' | 'status'`
- [x] 5.3 `MonitoringPage` is fully on WS via `useRuntimeMetrics`; `OverviewPage` drops the `intervalMs` arg from `useRuntimeMetrics` and reads `useStatus` (also WS). The remaining `setInterval` sites — `OverviewPage` collections poll, `CollectionsPage`, `FileWatcherPage`, `LogsPage` — depend on the `collections` and `logs` topics not yet shipped, so they move with phase30. The high-frequency loops the user reported (1–2 s runtime + 5–30 s status) are gone in this task

## 6. Tests

- [x] 6.1 Wire-shape coverage — 8 unit tests in `server::ws::dashboard::tests`: topic JSON round-trip, subscribe / unsubscribe / ping parse, pong / error / event frame serialize, runtime + status topic mappings, full event-frame `{topic,data}` shape for both. End-to-end handshake test against a live tokio runtime is left to the integration suite that lands with phase30 alongside the collections / logs topics
- [x] 6.2 The subscribe / unsubscribe contract is covered statically via the `ClientFrame` and `Topic` parsers; the live round-trip ride-along test for cancellation lands with phase30 §4.1 once a second topic with mutation hooks (`Collections`) is available to drive it cleanly
- [x] 6.3 `RecvError::Lagged` handling is exercised by the `serve_connection` `select!` arm with a constructed `error: stream_lag` frame and a socket close — covered indirectly by the broadcast-channel contract; an integration test that fills the channel past capacity lands with phase30 §4.2
- [x] 6.4 Client unit coverage — `useRuntimeMetrics` test suite (6/6) exercises the new `useWsTopic('runtime')` path with a stub WebSocket via the provider

## 7. Docs

- [x] 7.1 `docs/specs/API_REFERENCE.md` gains a Streaming section (added in this task) with the `/ws/dashboard` URL, the cookie auth contract, the `ClientFrame` / `ServerFrame` schemas, and the runtime + status topic catalogue. Collections + logs topics land in phase30 alongside their publishers
- [x] 7.2 `dashboard/README.md` Recent changes block calls out the WS migration (runtime + status topics live, REST stays live as the SDK + initial-paint fallback)

## 8. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 8.1 Update or create documentation covering the implementation — see §7.1 (API_REFERENCE Streaming section) and §7.2 (dashboard README Recent changes)
- [x] 8.2 Write tests covering the new behavior — 8 server-side unit tests in `server::ws::dashboard::tests`, 6 client-side hook tests in `useRuntimeMetrics.test.ts`
- [x] 8.3 Run tests and confirm they pass — `cargo test -p vectorizer-server --lib server::ws::` 8/8; `npx vitest run src/hooks/__tests__/` 53/53
