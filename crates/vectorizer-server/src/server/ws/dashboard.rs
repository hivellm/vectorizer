//! `GET /ws/dashboard` handler — multiplexed live data for the React
//! dashboard (phase29).
//!
//! ## Wire protocol
//!
//! Frames are JSON text. The server pushes any subscribed topic; the
//! client may subscribe / unsubscribe / ping at any time.
//!
//! ```jsonc
//! // Client → server
//! {"op": "subscribe",   "topics": ["runtime", "status"]}
//! {"op": "unsubscribe", "topics": ["status"]}
//! {"op": "ping"}
//!
//! // Server → client
//! {"topic": "runtime", "data": {...RuntimeSnapshot}}
//! {"op": "pong"}
//! {"op": "error", "code": "stream_lag" | "unknown_topic" | "bad_frame"}
//! ```
//!
//! ## Authentication
//!
//! The route is registered behind the same auth middleware as the rest
//! of `/auth/*` — the browser sends the `vectorizer_session` cookie on
//! the upgrade GET, so no `Authorization` header is needed. CSRF is
//! exempted because the upgrade is a `GET` and the WS frames carry no
//! mutating ops.
//!
//! ## Slow consumers
//!
//! The broadcast channel has a fixed capacity (1024). If a connection
//! falls behind by more than that and `broadcast::Receiver::recv`
//! returns `RecvError::Lagged(n)`, the server emits a single
//! `{op: "error", code: "stream_lag"}` frame and closes — the client
//! reconnects with backoff and resumes.

use std::collections::HashSet;

use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing::{debug, warn};

use crate::server::VectorizerServer;
use crate::server::runtime_metrics::DashboardEvent;

/// Topics the WS multiplexer can route. Kept in lock-step with
/// `DashboardEvent` so the `From<&DashboardEvent>` mapping is total.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Topic {
    /// 1 Hz runtime snapshot (CPU / memory / connections / WAL …).
    Runtime,
    /// 5 s server status snapshot (online / version / uptime / collections_count).
    Status,
}

impl Topic {
    fn of(event: &DashboardEvent) -> Self {
        match event {
            DashboardEvent::Runtime(_) => Self::Runtime,
            DashboardEvent::Status(_) => Self::Status,
        }
    }
}

/// Frames the client sends. `op` is the discriminator.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum ClientFrame {
    Subscribe { topics: Vec<Topic> },
    Unsubscribe { topics: Vec<Topic> },
    Ping,
}

/// Frames the server sends. Either a typed topic payload, a `pong`, or
/// an `error`.
#[derive(Debug, Serialize)]
#[serde(untagged)]
enum ServerFrame<'a> {
    /// Forwards a `DashboardEvent` (already serializes to
    /// `{topic, data}` via its own `serde(tag = "topic")`).
    Event(&'a DashboardEvent),
    Op(ServerOp),
}

#[derive(Debug, Serialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum ServerOp {
    Pong,
    Error { code: ErrorCode },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum ErrorCode {
    /// Client fell behind the broadcast channel; reconnect to resume.
    StreamLag,
    /// Client sent a frame that did not parse against `ClientFrame`.
    BadFrame,
}

/// `GET /ws/dashboard` upgrade handler.
pub async fn dashboard_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<VectorizerServer>,
) -> impl IntoResponse {
    let rx = state.runtime_sampler.dashboard_rx();
    ws.on_upgrade(move |socket| serve_connection(socket, rx))
}

async fn serve_connection(mut socket: WebSocket, mut rx: broadcast::Receiver<DashboardEvent>) {
    let mut subscribed: HashSet<Topic> = HashSet::new();

    loop {
        tokio::select! {
            biased;

            // ── inbound: client → server frames ────────────────────────
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Text(txt))) => {
                        match serde_json::from_str::<ClientFrame>(&txt) {
                            Ok(ClientFrame::Subscribe { topics }) => {
                                for t in topics {
                                    subscribed.insert(t);
                                }
                            }
                            Ok(ClientFrame::Unsubscribe { topics }) => {
                                for t in topics {
                                    subscribed.remove(&t);
                                }
                            }
                            Ok(ClientFrame::Ping) => {
                                if send_op(&mut socket, ServerOp::Pong).await.is_err() {
                                    return;
                                }
                            }
                            Err(e) => {
                                debug!(error = %e, "ws: client frame did not parse");
                                if send_op(
                                    &mut socket,
                                    ServerOp::Error { code: ErrorCode::BadFrame },
                                )
                                .await
                                .is_err()
                                {
                                    return;
                                }
                            }
                        }
                    }
                    // Pongs / binary frames are no-ops for this surface.
                    Some(Ok(Message::Pong(_))) | Some(Ok(Message::Ping(_))) => {}
                    Some(Ok(Message::Binary(_))) => {
                        if send_op(
                            &mut socket,
                            ServerOp::Error { code: ErrorCode::BadFrame },
                        )
                        .await
                        .is_err()
                        {
                            return;
                        }
                    }
                    // Close / read error / channel closed: drop the
                    // connection and let the client reconnect.
                    Some(Ok(Message::Close(_))) | None => return,
                    Some(Err(e)) => {
                        warn!(error = %e, "ws: read error, closing connection");
                        return;
                    }
                }
            }

            // ── outbound: broadcast bus → client ───────────────────────
            ev = rx.recv() => {
                match ev {
                    Ok(event) => {
                        if !subscribed.contains(&Topic::of(&event)) {
                            continue;
                        }
                        let frame = match serde_json::to_string(&ServerFrame::Event(&event)) {
                            Ok(s) => s,
                            Err(e) => {
                                warn!(error = %e, "ws: serialize event failed");
                                continue;
                            }
                        };
                        if socket.send(Message::Text(frame.into())).await.is_err() {
                            return;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        debug!(lagged = n, "ws: client lagged, dropping");
                        let _ = send_op(
                            &mut socket,
                            ServerOp::Error { code: ErrorCode::StreamLag },
                        )
                        .await;
                        return;
                    }
                    Err(broadcast::error::RecvError::Closed) => return,
                }
            }
        }
    }
}

async fn send_op(socket: &mut WebSocket, op: ServerOp) -> Result<(), axum::Error> {
    let frame = serde_json::to_string(&ServerFrame::Op(op)).map_err(axum::Error::new)?;
    socket.send(Message::Text(frame.into())).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topic_round_trips_via_json() {
        let json = serde_json::to_string(&Topic::Runtime).unwrap();
        assert_eq!(json, "\"runtime\"");
        let back: Topic = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Topic::Runtime);
    }

    #[test]
    fn client_subscribe_frame_parses() {
        let raw = r#"{"op":"subscribe","topics":["runtime"]}"#;
        let f: ClientFrame = serde_json::from_str(raw).unwrap();
        match f {
            ClientFrame::Subscribe { topics } => {
                assert_eq!(topics, vec![Topic::Runtime]);
            }
            other => panic!("expected Subscribe, got {other:?}"),
        }
    }

    #[test]
    fn client_unsubscribe_and_ping_parse() {
        let unsub: ClientFrame =
            serde_json::from_str(r#"{"op":"unsubscribe","topics":["runtime"]}"#).unwrap();
        assert!(matches!(unsub, ClientFrame::Unsubscribe { .. }));

        let ping: ClientFrame = serde_json::from_str(r#"{"op":"ping"}"#).unwrap();
        assert!(matches!(ping, ClientFrame::Ping));
    }

    #[test]
    fn server_pong_and_error_serialize_to_op() {
        let pong = serde_json::to_string(&ServerFrame::Op(ServerOp::Pong)).unwrap();
        assert_eq!(pong, r#"{"op":"pong"}"#);

        let err = serde_json::to_string(&ServerFrame::Op(ServerOp::Error {
            code: ErrorCode::StreamLag,
        }))
        .unwrap();
        assert_eq!(err, r#"{"op":"error","code":"stream_lag"}"#);
    }

    #[test]
    fn topic_of_event_maps_runtime() {
        let snap = crate::server::runtime_metrics::RuntimeSnapshot::default();
        let ev = DashboardEvent::Runtime(snap);
        assert_eq!(Topic::of(&ev), Topic::Runtime);
    }

    #[test]
    fn topic_of_event_maps_status() {
        let snap = crate::server::runtime_metrics::StatusSnapshot::default();
        let ev = DashboardEvent::Status(snap);
        assert_eq!(Topic::of(&ev), Topic::Status);
    }

    #[test]
    fn status_event_frame_carries_topic_and_data() {
        let snap = crate::server::runtime_metrics::StatusSnapshot {
            online: true,
            version: "3.3.0".to_string(),
            uptime_seconds: 42,
            collections_count: 7,
        };
        let ev = DashboardEvent::Status(snap);
        let json = serde_json::to_string(&ServerFrame::Event(&ev)).unwrap();
        assert!(json.contains("\"topic\":\"status\""));
        assert!(json.contains("\"online\":true"));
        assert!(json.contains("\"collections_count\":7"));
    }

    #[test]
    fn server_event_frame_carries_topic_and_data() {
        let snap = crate::server::runtime_metrics::RuntimeSnapshot {
            cpu_percent: 12.5,
            ..Default::default()
        };
        let ev = DashboardEvent::Runtime(snap);
        let json = serde_json::to_string(&ServerFrame::Event(&ev)).unwrap();
        assert!(json.contains("\"topic\":\"runtime\""));
        assert!(json.contains("\"cpu_percent\":12.5"));
    }
}
