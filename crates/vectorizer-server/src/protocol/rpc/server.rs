//! VectorizerRPC TCP listener + connection loop.
//!
//! Wire spec § 1, 4, 5: `docs/specs/VECTORIZER_RPC.md`. Ported from
//! `../Synap/synap-server/src/protocol/synap_rpc/server.rs`. Compared to
//! the Synap reference this version omits the `SUBSCRIBE` / pub-sub
//! plumbing (Vectorizer has no pub-sub use case in v1) and adapts the
//! state type from Synap's `AppState` to a minimal [`RpcState`]
//! carrying only what the dispatch table needs.
//!
//! Each accepted connection runs a reader loop that decodes
//! length-prefixed MessagePack [`Request`] frames, spawns a
//! `tokio::task` per request for dispatch concurrency, and forwards
//! [`Response`]s through an `mpsc::channel` to a single writer task.
//! The writer-task pattern keeps frames serialised on the wire even
//! though dispatch is concurrent — multiplexed responses come back
//! via the `Request.id` echo.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use tokio::io::BufReader;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tracing::{debug, error, info, info_span, warn};
use vectorizer::db::VectorStore;
use vectorizer::embedding::EmbeddingManager;
use vectorizer_protocol::rpc_wire::codec::{read_request, write_response};
use vectorizer_protocol::rpc_wire::types::{Request, Response};

use super::dispatch::{ConnectionAuth, dispatch};
use crate::server::AuthHandlerState;

/// Shared state passed into every RPC connection handler.
#[derive(Clone)]
pub struct RpcState {
    /// The live vector store the dispatch handlers query.
    pub store: Arc<VectorStore>,
    /// Embedding manager used by `search.basic` etc. to convert text
    /// queries into dense vectors.
    pub embedding_manager: Arc<EmbeddingManager>,
    /// Auth handler state. `None` when auth is globally disabled
    /// (single-user mode); the dispatch table treats every caller as
    /// the implicit local admin in that case.
    pub auth: Option<AuthHandlerState>,
}

/// Spawn the RPC TCP listener on `addr`. Returns immediately; the
/// listener and its per-connection workers run as detached background
/// tasks. The returned handle keeps the bind alive for the caller's
/// lifetime — drop it to stop accepting new connections.
pub async fn spawn_rpc_listener(state: RpcState, addr: SocketAddr) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    info!("VectorizerRPC server listening on {addr}");

    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, peer)) => {
                    debug!(peer = %peer, "VectorizerRPC connection accepted");
                    let state = state.clone();
                    tokio::spawn(async move {
                        let span = info_span!("rpc.conn", peer = %peer);
                        let _guard = span.enter();
                        if let Err(e) = handle_connection(stream, state).await {
                            debug!(peer = %peer, error = %e, "VectorizerRPC connection error");
                        }
                        debug!(peer = %peer, "VectorizerRPC connection closed");
                    });
                }
                Err(e) => {
                    error!(error = %e, "VectorizerRPC accept error");
                }
            }
        }
    });

    Ok(())
}

async fn handle_connection(stream: TcpStream, state: RpcState) -> std::io::Result<()> {
    let peer = stream.peer_addr()?;
    let (read_half, write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);

    // Writer channel: dispatch tasks send responses here; a single
    // writer task drains them to the socket so frames stay coherent.
    let (tx, mut rx) = mpsc::channel::<(Response, String, f64)>(64);
    let mut writer = write_half;

    let write_task = tokio::spawn(async move {
        while let Some((response, command, elapsed)) = rx.recv().await {
            if let Err(e) = write_response(&mut writer, &response).await {
                debug!(error = %e, "VectorizerRPC write error");
                break;
            }
            if elapsed > 0.001 {
                warn!(
                    cmd = %command,
                    elapsed_ms = elapsed * 1_000.0,
                    "VectorizerRPC slow command"
                );
            } else {
                debug!(
                    cmd = %command,
                    elapsed_us = elapsed * 1_000_000.0,
                    ok = response.result.is_ok(),
                    "VectorizerRPC command"
                );
            }
        }
    });

    // Per-connection auth state. `HELLO` flips `authenticated` once
    // valid credentials arrive; subsequent requests inherit it. Wrapped
    // in `Arc<RwLock<>>` so concurrent dispatch tasks can read it
    // without serialising on a `Mutex` — the only writer is the HELLO
    // handler which runs at most once per connection.
    let auth = Arc::new(parking_lot::RwLock::new(ConnectionAuth::default()));

    let state = Arc::new(state);

    loop {
        let req: Request = match read_request(&mut reader).await {
            Ok(r) => r,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => {
                debug!(peer = %peer, error = %e, "VectorizerRPC read error");
                break;
            }
        };

        let command = req.command.clone();
        let state = Arc::clone(&state);
        let tx = tx.clone();
        let auth = Arc::clone(&auth);

        tokio::spawn(async move {
            let start = Instant::now();
            let span = tracing::debug_span!("rpc.req", id = req.id, cmd = %req.command);
            let response = {
                let _g = span.enter();
                dispatch(&state, &auth, req).await
            };
            let elapsed = start.elapsed().as_secs_f64();
            let _ = tx.send((response, command, elapsed)).await;
        });
    }

    drop(tx);
    let _ = write_task.await;

    Ok(())
}
