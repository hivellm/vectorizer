//! VectorizerRPC on Thunder's transport.
//!
//! Vectorizer's command catalog runs on `thunder-rpc` (lib `thunder`), the
//! HiveLLM family's shared binary RPC stack — the same wire (length-prefixed
//! MessagePack) Vectorizer already spoke, now maintained once for every
//! language. The accept loop, per-connection writer task, frame codec, session
//! state machine, handshake gating and graceful drain all belong to
//! [`thunder::server`]. What lives here is the [`Dispatch`] implementation that
//! binds Vectorizer's engine to it: command routing (delegated to
//! [`super::dispatch::dispatch`]) and credential validation.
//!
//! Auth: Thunder's `AuthCommand` handshake carries credentials in `AUTH`, which
//! Thunder routes to [`Dispatch::authenticate`]; it flips the session's auth
//! flag on success, so the per-command handlers never re-check "authenticated".
//! When the deployment disables auth (`RpcState.auth == None`) the listener is
//! opened with [`ListenerConfig::open`] and every caller is the implicit local
//! admin. The per-command admin ACL (`require_admin`) is preserved: the admin
//! bit resolved at `AUTH` rides the session and is rebuilt into the
//! [`ConnectionAuth`] snapshot each dispatch reads.

use std::net::SocketAddr;
use std::sync::Arc;

use parking_lot::RwLock;
use thunder::server::{
    AuthError, Credentials, Dispatch, ListenerConfig, Principal, ServerInfo, Session,
    spawn_listener,
};
use thunder::wire::config::{ErrorConvention, Handshake, HelloStyle, PushPolicy};
use thunder::{Config, Request, Value};
use tracing::info;
use vectorizer::cache::SlowQueryRing;
use vectorizer::db::VectorStore;
use vectorizer::embedding::EmbeddingManager;

use super::dispatch::{ConnectionAuth, dispatch, validate_credentials};
use crate::server::AuthHandlerState;

/// Vectorizer's slot in the 15500-range binary-transport convention shared with
/// Synap; the default the SDKs assume when a `vectorizer://host` URL omits the
/// port (`docs/specs/VECTORIZER_RPC.md` § 12).
const DEFAULT_RPC_PORT: u16 = 15503;

/// Frame-body cap. Matches the family default (Synap's 512 MiB) so large batch
/// inserts and raw-LE-f32 embedding payloads are not rejected; well above the
/// pre-Thunder codec's 64 MiB body cap.
const MAX_FRAME_BYTES: usize = 512 * 1024 * 1024;

/// Shared state passed into every RPC connection handler.
#[derive(Clone)]
pub struct RpcState {
    /// The live vector store the dispatch handlers query.
    pub store: Arc<VectorStore>,
    /// Embedding manager used by `search.basic` etc. to convert text
    /// queries into dense vectors.
    pub embedding_manager: Arc<EmbeddingManager>,
    /// Auth handler state. `None` when auth is globally disabled
    /// (single-user mode); the listener is opened and the dispatch table
    /// treats every caller as the implicit local admin in that case.
    pub auth: Option<AuthHandlerState>,
    /// Master replication node, present only when this instance runs as master.
    pub master_node: Option<Arc<vectorizer::replication::MasterNode>>,
    /// Replica replication node, present only when this instance runs as a replica.
    pub replica_node: Option<Arc<vectorizer::replication::ReplicaNode>>,
    /// Cluster manager, present only when cluster mode is enabled.
    pub cluster_manager: Option<Arc<vectorizer::cluster::ClusterManager>>,
    /// Slow-query ring buffer for `admin.slow_queries_*`.
    pub slow_query_ring: SlowQueryRing,
}

/// Product identity carried on the Thunder session (SRV-012): resolved once at
/// `AUTH`, stable for the connection. Vectorizer only needs the admin bit for
/// the per-command ACL; the principal's display name lives on
/// [`Principal::name`].
#[derive(Debug, Clone)]
pub struct RpcIdentity {
    /// `true` when the authenticated principal carries `Role::Admin`.
    pub admin: bool,
}

/// How Vectorizer uses the Thunder wire. Mirrors Synap's `synap_config` with
/// two divergences: the `vectorizer` scheme, and `PushPolicy::Reserved`
/// (Vectorizer ships no push-producing command). Both halves of the product —
/// this listener and the SDK clients — read the same shape, so the wire cannot
/// drift between them.
pub fn vectorizer_config() -> Config {
    Config::standard()
        .scheme("vectorizer")
        .port(DEFAULT_RPC_PORT)
        .handshake(Handshake::AuthCommand)
        .hello_style(HelloStyle::NotUsed)
        .push(PushPolicy::Reserved)
        .error_codes(ErrorConvention::Resp3Prefixes)
        .max_frame_bytes(MAX_FRAME_BYTES)
}

/// Vectorizer's integration with Thunder: command dispatch + credential
/// validation over the shared [`RpcState`].
struct VectorizerDispatch {
    state: Arc<RpcState>,
}

impl Dispatch for VectorizerDispatch {
    type Identity = RpcIdentity;

    async fn dispatch(
        &self,
        session: &Session<RpcIdentity>,
        command: &str,
        args: Vec<Value>,
    ) -> Result<Value, String> {
        // Rebuild the per-request auth snapshot the command handlers expect
        // from the identity captured at `AUTH`. Thunder has already gated
        // un-authenticated sessions (or the listener is `.open()` and there is
        // no principal — the implicit local admin), so `authenticated` is
        // always true by the time a command reaches here.
        let (admin, principal) = session.with_principal(|p| match p {
            Some(p) => (p.identity.admin, Some(p.name.clone())),
            None => (true, None),
        });
        let auth = Arc::new(RwLock::new(ConnectionAuth {
            authenticated: true,
            admin,
            principal,
        }));

        // Thunder owns the request id (it echoes it on the wire); the handlers
        // still take a `Request`, so pass a fixed id and return only the
        // result payload.
        let req = Request {
            id: 0,
            command: command.to_string(),
            args,
        };
        dispatch(&self.state, &auth, req).await.result
    }

    async fn authenticate(&self, creds: Credentials) -> Result<Principal<RpcIdentity>, AuthError> {
        let Some(handler) = self.state.auth.as_ref() else {
            // Auth disabled globally. With `.open()` this hook is not reached,
            // but if a client sends `AUTH` anyway, accept it as the local admin.
            return Ok(Principal::with_identity(
                "local",
                RpcIdentity { admin: true },
            ));
        };

        // Vectorizer authenticates with a single secret — a JWT or an API key.
        // Thunder tags a single-arg `AUTH <secret>` as `ApiKey`, so the secret
        // is validated as a JWT first and as an API key second, regardless of
        // which credential variant carried it.
        let secret = match creds {
            Credentials::Token(t) | Credentials::ApiKey(t) => t,
            Credentials::UserPass(_user, pass) => pass,
            Credentials::None => return Err(AuthError::InvalidCredentials),
        };

        match validate_credentials(handler, Some(&secret), None).await {
            Ok((name, admin)) => Ok(Principal::with_identity(name, RpcIdentity { admin })),
            Err(_) => match validate_credentials(handler, None, Some(&secret)).await {
                Ok((name, admin)) => Ok(Principal::with_identity(name, RpcIdentity { admin })),
                Err(msg) => Err(AuthError::Message(msg)),
            },
        }
    }
}

/// Spawn the VectorizerRPC listener on `addr`. Returns once the listener is
/// bound; the accept loop, per-connection tasks and graceful drain run inside
/// Thunder. The listener lives for the process: its [`ListenerHandle`] is
/// intentionally retained via [`std::mem::forget`] rather than threaded up to
/// the bootstrap caller — the pre-Thunder listener was likewise a detached,
/// process-lifetime accept loop with no stop handle.
pub async fn spawn_rpc_listener(state: RpcState, addr: SocketAddr) -> std::io::Result<()> {
    let auth_enabled = state.auth.is_some();
    let dispatch = Arc::new(VectorizerDispatch {
        state: Arc::new(state),
    });

    let mut listener_config = ListenerConfig::new(addr);
    if !auth_enabled {
        listener_config = listener_config.open();
    }

    let info = ServerInfo {
        name: "vectorizer".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    let handle = spawn_listener(dispatch, vectorizer_config(), info, listener_config).await?;
    info!("VectorizerRPC server listening on {}", handle.local_addr());

    // The listener runs for the process lifetime; keep it alive by leaking the
    // handle (dropping it would stop accepting connections).
    std::mem::forget(handle);

    Ok(())
}
