## 1. Server: broadcast bus + WebSocket handler

- [ ] 1.1 Add a `broadcast::Sender<DashboardEvent>` to `runtime_metrics.rs`. `RuntimeSampler::start()` publishes a `DashboardEvent::Runtime(snapshot.clone())` on every tick after writing to the `RwLock` snapshot
- [ ] 1.2 New `crates/vectorizer-server/src/server/ws/mod.rs` + `ws/dashboard.rs` housing the `WebSocketUpgrade` handler, the topic enum (`runtime`, `status`, `collections`, `logs`), and the per-connection subscription set
- [ ] 1.3 Define the wire protocol enums (`ClientFrame::{Subscribe, Unsubscribe, Ping}`, `ServerFrame::{Topic, Pong, Error}`) with `serde(tag = "op")` and `serde(rename_all = "snake_case")`
- [ ] 1.4 Per-connection task: `select!` between the `broadcast::Receiver` (push frames matching subscribed topics) and the WS read half (`Subscribe` / `Unsubscribe` / `Ping`). Slow consumers get `RecvError::Lagged` → drop with `error: "stream_lag"` and let the client reconnect

## 2. Server: status / collections / logs publishers

- [ ] 2.1 Status snapshot publisher — every 5 s, build the `/status` JSON and broadcast `DashboardEvent::Status`. Cheap (server uptime + collection count) so 5 s is fine
- [ ] 2.2 Collections snapshot publisher — every 30 s, broadcast `DashboardEvent::Collections` carrying the same shape `GET /collections` returns. Tied to the existing collection-mutation events so a create/delete also triggers an immediate publish
- [ ] 2.3 Logs publisher — tail the same log file `GET /logs` reads from; for each new line emit `DashboardEvent::Log(LogEntry)`. Cap inflight buffer at 256 entries per connection (drop oldest with a `stream_lag` error if the client falls behind)

## 3. Server: auth + routing

- [ ] 3.1 Register `GET /ws/dashboard` in `routing.rs` behind the same cookie / JWT auth middleware as the rest of `/auth/*`. CSRF middleware exempts WS upgrades (the upgrade is a GET; mutating ops are not on this path)
- [ ] 3.2 Boot wiring in `bootstrap.rs` — create the `broadcast::channel(1024)` once, pass the `Sender` into `RuntimeSampler::set_broadcast(...)`, the status publisher task, the collections publisher task, and the logs tail task; pass a clone into the WS handler state

## 4. Client: provider + hook

- [ ] 4.1 New `dashboard/src/providers/WsDashboardProvider.tsx` opens one `WebSocket` to `/ws/dashboard` per app instance. Reconnect with exponential backoff (250 ms → 5 s cap) on `close` / `error`. Cookie auth is automatic — no `Authorization` header needed for `WebSocket`
- [ ] 4.2 New `useWsTopic<T>(topic: 'runtime' | 'status' | 'collections' | 'logs')` hook — subscribes on mount, unsubscribes on unmount, returns the latest typed snapshot via `useSyncExternalStore` so React renders only on actual change

## 5. Client: rewrite the polling hooks

- [ ] 5.1 `useRuntimeMetrics` becomes a thin wrapper around `useWsTopic('runtime')` — defensive snake↔camel mapping stays for older servers without WS support, but the live path is push-only
- [ ] 5.2 `useStats` and `useStatus` rewrite on `useWsTopic('status')` (status payload includes the stats subset)
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
