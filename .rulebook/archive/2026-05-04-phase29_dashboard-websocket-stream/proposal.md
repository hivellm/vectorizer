# Proposal: phase29_dashboard-websocket-stream

## Why

The redesigned dashboard polls 8 separate REST endpoints on intervals
(per the survey done while debugging the runaway-CPU container):

| Hook / page | Cadence | Endpoint |
|---|---|---|
| `useRuntimeMetrics` | 1–2 s | `GET /metrics/runtime` |
| `useStats` | 5–30 s | `GET /stats` |
| `useStatus` | 5–30 s | `GET /status` |
| `useMetrics` (legacy) | varies | `GET /metrics` (Prometheus) |
| `useEvents` | varies | events |
| `OverviewPage` | 30 s | `GET /collections` |
| `CollectionsPage` | 30 s | `GET /collections` |
| `FileWatcherPage` | 5 s | filewatcher status |
| `LogsPage` | 2 s | `GET /logs` |

With N tabs open, the server fields N × 8 requests/min of mostly
identical JSON. Browsers stack pending requests when one of the routes
spikes (the `LogsPage` 2 s loop is particularly unkind under load), and
each request goes through the full auth + CSRF middleware stack. The
user ran into pending-request build-up and asked us to switch to a
streaming transport.

## What Changes

Add a single multiplexed WebSocket endpoint at `GET /ws/dashboard` that
the dashboard subscribes to once per session. Topics carry the same
payloads the existing REST routes return, refreshed by the server's
own samplers instead of by a client poll.

### Server side

- New `axum::extract::ws::WebSocketUpgrade` handler in
  `crates/vectorizer-server/src/server/ws/dashboard.rs`. Auth runs
  through the existing cookie / JWT session middleware (browsers send
  `vectorizer_session` automatically on upgrade).
- A `broadcast::Sender<DashboardEvent>` lives next to `RuntimeSampler`
  in the server state. The 1 Hz sampler tick publishes
  `RuntimeMetricsTick` to the channel; status / collections / logs
  publishers piggyback on the same bus. The client hot-loop is
  decoupled from the sampler — N subscribers each receive their own
  `broadcast::Receiver` clone.
- Client → server protocol (text frames, JSON):

  ```jsonc
  // c→s
  {"op": "subscribe", "topics": ["runtime", "status", "collections", "logs"]}
  {"op": "unsubscribe", "topics": ["logs"]}
  {"op": "ping"}
  // s→c
  {"topic": "runtime", "data": {...RuntimeMetrics}}
  {"topic": "status", "data": {...ServerStatus}}
  {"topic": "logs", "data": {...LogEntry}}
  {"op": "pong"}
  {"op": "error", "code": "unauthorized" | "unknown_topic" | ...}
  ```

- Topics shipped in this task: `runtime`, `status`, `collections`,
  `logs`. `events` and `metrics` (Prometheus) stay on REST — they
  don't have a server-side push source today.
- 30 s server-side ping; client must respond within 30 s or the
  connection is dropped. Client reconnects with exponential backoff.

### Client side

- New `WsDashboardProvider` (React context) opens a single WS to
  `/ws/dashboard` per app instance, manages reconnect, and exposes a
  `useWsTopic<T>(topic)` hook returning the latest snapshot per topic.
- `useRuntimeMetrics`, `useStats`, `useStatus` rewritten on top of
  `useWsTopic`. Their `setInterval` blocks go away.
- `OverviewPage`, `CollectionsPage`, `MonitoringPage`,
  `FileWatcherPage`, `LogsPage` drop their per-page `setInterval`
  blocks and consume the same hook.
- REST endpoints stay live as a fallback so the SDKs and direct
  curl callers continue to work; the WS path is purely additive.

## Impact

- Affected code:
  - `crates/vectorizer-server/src/server/ws/dashboard.rs` (new)
  - `crates/vectorizer-server/src/server/runtime_metrics.rs` (broadcast::Sender)
  - `crates/vectorizer-server/src/server/core/routing.rs` (route registration)
  - `crates/vectorizer-server/src/server/core/bootstrap.rs` (wire the broadcast bus)
  - `dashboard/src/providers/WsDashboardProvider.tsx` (new)
  - `dashboard/src/hooks/{useRuntimeMetrics,useStats,useStatus}.ts` (rewrite)
  - `dashboard/src/pages/{Overview,Collections,Monitoring,FileWatcher,Logs}Page.tsx` (drop setInterval)
- Affected specs: `docs/specs/API_REFERENCE.md` gains a WS section.
- Breaking change: NO — REST endpoints stay; WS is additive.
- User benefit: the dashboard stops issuing 8 polling loops per tab.
  One WS connection per session carries all live data; pending-request
  build-up disappears; the server stops serializing the same JSON
  every 1–30 s for each open tab.
