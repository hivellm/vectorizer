//! VectorizerRPC command dispatch.
//!
//! One arm per command name in the wire spec's v1 catalog
//! (`docs/specs/VECTORIZER_RPC.md` § 6). Every command goes through a
//! single `match`; reflection / dynamic loading is explicitly out of
//! scope per `.rulebook/specs/RPC.md` § "Forbidden".
//!
//! ## State machine
//!
//! Every connection starts in [`ConnectionAuth::default`]
//! (`authenticated = false`). The first request MUST be `HELLO`, which
//! validates credentials (when auth is enabled) and sets the connection
//! state. Subsequent commands inherit that state — no auth payload is
//! sent per request.
//!
//! When `state.auth` is `None` (`auth.enabled = false` in config),
//! every connection is treated as the implicit local admin and `HELLO`
//! credentials are accepted-but-ignored. This matches the existing
//! REST/MCP behaviour for single-user local setups.

use std::sync::Arc;

use parking_lot::RwLock;
use tracing::debug;

use super::server::RpcState;
use super::types::{Request, Response, VectorizerValue};
use crate::auth::roles::Role;

/// Per-connection auth state. Mutated only by the `HELLO` handler;
/// every other handler reads it under a shared lock.
#[derive(Debug, Clone, Default)]
pub struct ConnectionAuth {
    /// `true` once `HELLO` succeeded (or auth is globally disabled).
    pub authenticated: bool,
    /// `true` when the authenticated principal carries `Role::Admin`.
    pub admin: bool,
    /// Display name surfaced in tracing.
    pub principal: Option<String>,
}

const PROTOCOL_VERSION: i64 = 1;

/// Dispatch one [`Request`] against the live capability set. Always
/// returns a [`Response`] — a transport-level error becomes
/// `Response::err`, never a `Result::Err` from this fn (the caller
/// must always have a frame to write back to the client).
pub async fn dispatch(
    state: &Arc<RpcState>,
    auth: &Arc<RwLock<ConnectionAuth>>,
    req: Request,
) -> Response {
    let Request { id, command, args } = req;

    debug!(id, %command, "rpc dispatch");

    match command.as_str() {
        "HELLO" => handle_hello(state, auth, id, args).await,
        "PING" => handle_ping(id, &args),
        other => {
            if !auth.read().authenticated {
                return Response::err(id, "authentication required: send HELLO first");
            }
            match other {
                "collections.list" => handle_collections_list(state, id),
                "collections.get_info" => handle_collection_info(state, id, &args),
                "vectors.get" => handle_vector_get(state, id, &args),
                "search.basic" => handle_search_basic(state, id, &args),
                "search.intelligent" => {
                    Response::err(id, "search.intelligent: not yet wired in v1 dispatch")
                }
                _ => Response::err(id, format!("unknown command '{}'", other)),
            }
        }
    }
}

// ── Handshake & health ───────────────────────────────────────────────────────

async fn handle_hello(
    state: &Arc<RpcState>,
    auth: &Arc<RwLock<ConnectionAuth>>,
    id: u32,
    args: Vec<VectorizerValue>,
) -> Response {
    // Single Map argument carrying { version, token?, api_key?, client_name? }.
    let payload: Vec<(VectorizerValue, VectorizerValue)> = match args.into_iter().next() {
        Some(VectorizerValue::Map(pairs)) => pairs,
        Some(_) => return Response::err(id, "HELLO expects a Map argument"),
        None => Vec::new(),
    };

    let (token, api_key, client_name, requested_version) = parse_hello_payload(&payload);

    if requested_version != PROTOCOL_VERSION {
        // Always reply with the protocol version we speak; client decides
        // whether to downgrade or close. Don't fail the handshake — the
        // spec allows a higher requested version to negotiate down.
        debug!(
            requested_version,
            our_version = PROTOCOL_VERSION,
            "HELLO version mismatch — replying with our version"
        );
    }

    let auth_state = match &state.auth {
        // Auth disabled globally: every caller is the implicit local admin.
        None => ConnectionAuth {
            authenticated: true,
            admin: true,
            principal: client_name.clone(),
        },
        Some(handler) => {
            match validate_credentials(handler, token.as_deref(), api_key.as_deref()).await {
                Ok((authenticated_principal, is_admin)) => ConnectionAuth {
                    authenticated: true,
                    admin: is_admin,
                    principal: Some(authenticated_principal),
                },
                Err(msg) => return Response::err(id, msg),
            }
        }
    };

    *auth.write() = auth_state.clone();

    let reply = VectorizerValue::Map(vec![
        (
            VectorizerValue::Str("server_version".into()),
            VectorizerValue::Str(env!("CARGO_PKG_VERSION").to_string()),
        ),
        (
            VectorizerValue::Str("protocol_version".into()),
            VectorizerValue::Int(PROTOCOL_VERSION),
        ),
        (
            VectorizerValue::Str("authenticated".into()),
            VectorizerValue::Bool(auth_state.authenticated),
        ),
        (
            VectorizerValue::Str("admin".into()),
            VectorizerValue::Bool(auth_state.admin),
        ),
        (
            VectorizerValue::Str("capabilities".into()),
            VectorizerValue::Array(rpc_capability_names()),
        ),
    ]);
    Response::ok(id, reply)
}

fn handle_ping(id: u32, _args: &[VectorizerValue]) -> Response {
    Response::ok(id, VectorizerValue::Str("PONG".into()))
}

// ── Collections ──────────────────────────────────────────────────────────────

fn handle_collections_list(state: &Arc<RpcState>, id: u32) -> Response {
    let names = state.store.list_collections();
    let arr = names
        .into_iter()
        .map(VectorizerValue::Str)
        .collect::<Vec<_>>();
    Response::ok(id, VectorizerValue::Array(arr))
}

fn handle_collection_info(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    let name = match args.first().and_then(|v| v.as_str()) {
        Some(n) => n,
        None => {
            return Response::err(id, "collections.get_info expects [Str(name)]");
        }
    };
    match state.store.get_collection_metadata(name) {
        Ok(meta) => {
            let map = vec![
                (
                    VectorizerValue::Str("name".into()),
                    VectorizerValue::Str(meta.name),
                ),
                (
                    VectorizerValue::Str("vector_count".into()),
                    VectorizerValue::Int(meta.vector_count as i64),
                ),
                (
                    VectorizerValue::Str("document_count".into()),
                    VectorizerValue::Int(meta.document_count as i64),
                ),
                (
                    VectorizerValue::Str("dimension".into()),
                    VectorizerValue::Int(meta.config.dimension as i64),
                ),
                (
                    VectorizerValue::Str("metric".into()),
                    VectorizerValue::Str(format!("{:?}", meta.config.metric)),
                ),
                (
                    VectorizerValue::Str("created_at".into()),
                    VectorizerValue::Str(meta.created_at.to_rfc3339()),
                ),
                (
                    VectorizerValue::Str("updated_at".into()),
                    VectorizerValue::Str(meta.updated_at.to_rfc3339()),
                ),
            ];
            Response::ok(id, VectorizerValue::Map(map))
        }
        Err(e) => Response::err(id, format!("collection '{}' not found: {}", name, e)),
    }
}

// ── Vectors ──────────────────────────────────────────────────────────────────

fn handle_vector_get(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    let collection = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => {
            return Response::err(id, "vectors.get expects [Str(collection), Str(vector_id)]");
        }
    };
    let vector_id = match args.get(1).and_then(|v| v.as_str()) {
        Some(v) => v,
        None => {
            return Response::err(id, "vectors.get expects [Str(collection), Str(vector_id)]");
        }
    };
    match state.store.get_vector(collection, vector_id) {
        Ok(vector) => Response::ok(id, vector_to_value(&vector)),
        Err(e) => Response::err(id, format!("vectors.get: {}", e)),
    }
}

// ── Search ───────────────────────────────────────────────────────────────────

fn handle_search_basic(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    let collection = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => {
            return Response::err(
                id,
                "search.basic expects [Str(collection), Str(query), Int(limit)?, Float(threshold)?]",
            );
        }
    };
    let query = match args.get(1).and_then(|v| v.as_str()) {
        Some(q) => q,
        None => {
            return Response::err(
                id,
                "search.basic expects [Str(collection), Str(query), Int(limit)?, Float(threshold)?]",
            );
        }
    };
    let limit = args
        .get(2)
        .and_then(|v| v.as_int())
        .map(|n| n.max(1) as usize)
        .unwrap_or(10);

    let embedding = match state.embedding_manager.embed(query) {
        Ok(e) => e,
        Err(e) => return Response::err(id, format!("embedding failed: {}", e)),
    };

    let results = match state.store.search(collection, &embedding, limit) {
        Ok(r) => r,
        Err(e) => return Response::err(id, format!("search.basic: {}", e)),
    };

    let arr = results
        .into_iter()
        .map(|r| {
            let mut entries = vec![
                (
                    VectorizerValue::Str("id".into()),
                    VectorizerValue::Str(r.id),
                ),
                (
                    VectorizerValue::Str("score".into()),
                    VectorizerValue::Float(r.score as f64),
                ),
            ];
            if let Some(payload) = r.payload {
                let json = serde_json::to_string(&payload.data).unwrap_or_default();
                entries.push((
                    VectorizerValue::Str("payload".into()),
                    VectorizerValue::Str(json),
                ));
            }
            VectorizerValue::Map(entries)
        })
        .collect::<Vec<_>>();
    Response::ok(id, VectorizerValue::Array(arr))
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn vector_to_value(v: &crate::models::Vector) -> VectorizerValue {
    let data = v
        .data
        .iter()
        .map(|f| VectorizerValue::Float(*f as f64))
        .collect::<Vec<_>>();
    let mut map = vec![
        (
            VectorizerValue::Str("id".into()),
            VectorizerValue::Str(v.id.clone()),
        ),
        (
            VectorizerValue::Str("data".into()),
            VectorizerValue::Array(data),
        ),
    ];
    if let Some(payload) = &v.payload {
        let json = serde_json::to_string(&payload.data).unwrap_or_default();
        map.push((
            VectorizerValue::Str("payload".into()),
            VectorizerValue::Str(json),
        ));
    }
    if let Some(doc_id) = &v.document_id {
        map.push((
            VectorizerValue::Str("document_id".into()),
            VectorizerValue::Str(doc_id.clone()),
        ));
    }
    VectorizerValue::Map(map)
}

fn parse_hello_payload(
    pairs: &[(VectorizerValue, VectorizerValue)],
) -> (Option<String>, Option<String>, Option<String>, i64) {
    let mut token = None;
    let mut api_key = None;
    let mut client_name = None;
    let mut version = PROTOCOL_VERSION;
    for (k, v) in pairs {
        let Some(key) = k.as_str() else { continue };
        match key {
            "token" => token = v.as_str().map(str::to_owned),
            "api_key" => api_key = v.as_str().map(str::to_owned),
            "client_name" => client_name = v.as_str().map(str::to_owned),
            "version" => {
                if let Some(n) = v.as_int() {
                    version = n;
                }
            }
            _ => {}
        }
    }
    (token, api_key, client_name, version)
}

async fn validate_credentials(
    handler: &crate::server::AuthHandlerState,
    token: Option<&str>,
    api_key: Option<&str>,
) -> Result<(String, bool), String> {
    // The handler exposes `is_token_blacklisted` and the inner
    // `auth_manager` for JWT/API-key validation. See
    // `src/server/auth_handlers/state.rs` for the full surface.
    if let Some(token) = token {
        if handler.is_token_blacklisted(token).await {
            return Err("token blacklisted (logged out)".into());
        }
        return handler
            .auth_manager
            .validate_jwt(token)
            .map(|claims| {
                let admin = claims.roles.contains(&Role::Admin);
                (claims.username, admin)
            })
            .map_err(|e| format!("invalid JWT: {}", e));
    }
    if let Some(api_key) = api_key {
        return handler
            .auth_manager
            .validate_api_key(api_key)
            .await
            .map(|claims| {
                let admin = claims.roles.contains(&Role::Admin);
                (claims.username, admin)
            })
            .map_err(|e| format!("invalid API key: {}", e));
    }
    Err("HELLO requires either `token` or `api_key`".into())
}

/// The names of the v1 RPC capabilities the handshake reports back to
/// the client. Kept local to this module so the wire surface is
/// reviewable in one place.
fn rpc_capability_names() -> Vec<VectorizerValue> {
    [
        "PING",
        "collections.list",
        "collections.get_info",
        "vectors.get",
        "search.basic",
    ]
    .iter()
    .map(|s| VectorizerValue::Str((*s).to_string()))
    .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn fake_state() -> Arc<RpcState> {
        Arc::new(RpcState {
            store: Arc::new(crate::db::VectorStore::new()),
            embedding_manager: Arc::new(crate::embedding::EmbeddingManager::new()),
            auth: None,
        })
    }

    fn fake_auth_unauthenticated() -> Arc<RwLock<ConnectionAuth>> {
        Arc::new(RwLock::new(ConnectionAuth::default()))
    }

    #[tokio::test]
    async fn ping_replies_pong() {
        let resp = dispatch(
            &fake_state(),
            // PING is reachable even before HELLO so the auth gate
            // doesn't block health checks. Keep that policy stable.
            &fake_auth_unauthenticated(),
            Request {
                id: 7,
                command: "PING".into(),
                args: vec![],
            },
        )
        .await;
        assert_eq!(resp.id, 7);
        assert_eq!(resp.result.as_ref().unwrap().as_str(), Some("PONG"));
    }

    #[tokio::test]
    async fn unauthenticated_command_is_rejected() {
        let resp = dispatch(
            &fake_state(),
            &fake_auth_unauthenticated(),
            Request {
                id: 8,
                command: "collections.list".into(),
                args: vec![],
            },
        )
        .await;
        let err = resp.result.unwrap_err();
        assert!(err.contains("authentication required"), "got: {err}");
    }

    #[tokio::test]
    async fn hello_with_no_auth_succeeds_in_single_user_mode() {
        let auth = fake_auth_unauthenticated();
        let resp = dispatch(
            &fake_state(),
            &auth,
            Request {
                id: 9,
                command: "HELLO".into(),
                args: vec![VectorizerValue::Map(vec![(
                    VectorizerValue::Str("version".into()),
                    VectorizerValue::Int(1),
                )])],
            },
        )
        .await;
        assert!(resp.result.is_ok());
        assert!(auth.read().authenticated);
        assert!(auth.read().admin);
    }

    #[tokio::test]
    async fn unknown_command_returns_named_error() {
        let auth = fake_auth_unauthenticated();
        // First HELLO so subsequent commands aren't rejected by the
        // auth gate (this test exercises the unknown-command path
        // specifically).
        let _ = dispatch(
            &fake_state(),
            &auth,
            Request {
                id: 10,
                command: "HELLO".into(),
                args: vec![VectorizerValue::Map(vec![])],
            },
        )
        .await;
        let resp = dispatch(
            &fake_state(),
            &auth,
            Request {
                id: 11,
                command: "no.such.command".into(),
                args: vec![],
            },
        )
        .await;
        let err = resp.result.unwrap_err();
        assert!(
            err.contains("unknown command 'no.such.command'"),
            "got: {err}"
        );
    }
}
