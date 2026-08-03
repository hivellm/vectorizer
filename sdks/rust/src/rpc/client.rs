//! `RpcClient`: connect, hello, call, ping, close.
//!
//! The transport is Thunder's ([`thunder::Client`]): one TCP connection per
//! `RpcClient`, a background reader that demultiplexes responses by frame id
//! so concurrent in-flight calls don't block each other, lazy reconnect, and
//! typed errors. What lives here is Vectorizer's shape on top of it — the
//! `vectorizer://` protocol config, the HELLO payload/response types, and the
//! error mapping the typed wrappers in [`super::commands`] consume.
//!
//! Auth is **per-connection sticky** per wire spec § 4, and Thunder carries
//! credentials in the connection handshake (`AUTH`) rather than in a command.
//! [`RpcClient::hello`] therefore re-dials when its payload carries a token or
//! an API key, so the credentials reach the session that later commands run
//! under; the HELLO command itself still runs, because the server answers it
//! with the capability list and auth flags this client surfaces.

use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;

use super::types::VectorizerValue;

/// Vectorizer's slot in the 15500-range binary-transport convention shared
/// with Synap; the default when a `vectorizer://host` URL omits the port
/// (wire spec § 12).
pub const DEFAULT_RPC_PORT: u16 = 15503;

/// Frame-body cap, matching the server's listener so neither end rejects a
/// frame the other is willing to send.
const MAX_FRAME_BYTES: usize = 512 * 1024 * 1024;

/// How Vectorizer uses the Thunder wire — the client half of the server's
/// `vectorizer_config()`: `vectorizer` scheme, `AUTH`-command handshake, no
/// HELLO negotiation (the `HELLO` *command* is Vectorizer's own), RESP3-style
/// error prefixes.
///
/// Declared here rather than imported from the server so the SDK depends only
/// on registry crates — `cargo publish` rejects path dependencies.
pub fn protocol_config() -> thunder::Config {
    use thunder::wire::config::{ErrorConvention, Handshake, HelloStyle, PushPolicy};
    thunder::Config::standard()
        .scheme("vectorizer")
        .port(DEFAULT_RPC_PORT)
        .handshake(Handshake::AuthCommand)
        .hello_style(HelloStyle::NotUsed)
        .push(PushPolicy::Reserved)
        .error_codes(ErrorConvention::Resp3Prefixes)
        .max_frame_bytes(MAX_FRAME_BYTES)
}

/// Errors the [`RpcClient`] can return.
#[derive(Debug, thiserror::Error)]
pub enum RpcClientError {
    /// Transport-level failure: dial, write, or the connection dying while
    /// the call was pending.
    #[error("connection error: {0}")]
    Connection(String),

    /// Server returned `Result::Err(message)` for the call.
    #[error("server error: {0}")]
    Server(String),

    /// The server refused the session's credentials — `NOAUTH` (no `AUTH`
    /// sent, or HELLO issued without credentials against an auth-enabled
    /// server), `WRONGPASS`, or `NOPERM` for an admin-only command.
    #[error("not authenticated: {0}")]
    NotAuthenticated(String),

    /// The connect or per-call timeout elapsed.
    #[error("timed out")]
    Timeout,

    /// The peer sent a malformed or oversized frame; the connection is
    /// poisoned and the next call re-dials.
    #[error("protocol error: {0}")]
    Protocol(String),
}

impl From<thunder::ClientError> for RpcClientError {
    fn from(err: thunder::ClientError) -> Self {
        use thunder::ClientError;
        match err {
            ClientError::Auth { message } => Self::NotAuthenticated(message),
            // The raw server string, verbatim — including any `[code]`
            // prefix the server put in front of it.
            ClientError::Server { message, .. } => Self::Server(message),
            ClientError::Connection { message } => Self::Connection(message),
            ClientError::Timeout => Self::Timeout,
            ClientError::FrameTooLarge { message } | ClientError::Decode { message } => {
                Self::Protocol(message)
            }
        }
    }
}

/// Result type alias.
pub type Result<T> = std::result::Result<T, RpcClientError>;

/// HELLO request payload.
///
/// At least one of `token` / `api_key` should be populated when the server has
/// auth enabled: those credentials travel in the connection handshake, so
/// passing them to [`RpcClient::hello`] is what authenticates the session.
/// When the server runs in single-user mode (`auth.enabled: false`) the
/// listener is open, credentials are accepted-but-ignored, and the connection
/// runs as the implicit local admin.
#[derive(Debug, Clone, Default)]
pub struct HelloPayload {
    /// Bearer JWT (same shape REST `/auth/login` returns).
    pub token: Option<String>,
    /// API key.
    pub api_key: Option<String>,
    /// User-Agent-style identifier surfaced in server-side tracing.
    pub client_name: Option<String>,
    /// Wire spec protocol version; defaults to 1.
    pub version: i64,
}

impl HelloPayload {
    /// Build a minimal HELLO payload identifying the client by name.
    /// No credentials — works against a server running in single-user
    /// mode (`auth.enabled: false`).
    pub fn new(client_name: impl Into<String>) -> Self {
        Self {
            client_name: Some(client_name.into()),
            version: 1,
            ..Default::default()
        }
    }

    /// Attach a JWT bearer token. Replaces any previously set
    /// token/api_key.
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self.api_key = None;
        self
    }

    /// Attach an API key. Replaces any previously set token/api_key.
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self.token = None;
        self
    }

    /// The credentials this payload carries, if any.
    fn credentials(&self) -> Option<thunder::client::Credentials> {
        if let Some(token) = &self.token {
            return Some(thunder::client::Credentials::Token(token.clone()));
        }
        self.api_key
            .as_ref()
            .map(|key| thunder::client::Credentials::ApiKey(key.clone()))
    }

    fn into_value(self) -> VectorizerValue {
        let mut pairs = vec![(
            VectorizerValue::Str("version".into()),
            VectorizerValue::Int(self.version),
        )];
        if let Some(token) = self.token {
            pairs.push((
                VectorizerValue::Str("token".into()),
                VectorizerValue::Str(token),
            ));
        }
        if let Some(api_key) = self.api_key {
            pairs.push((
                VectorizerValue::Str("api_key".into()),
                VectorizerValue::Str(api_key),
            ));
        }
        if let Some(name) = self.client_name {
            pairs.push((
                VectorizerValue::Str("client_name".into()),
                VectorizerValue::Str(name),
            ));
        }
        VectorizerValue::Map(pairs)
    }
}

/// What the server returns for a successful `HELLO`.
#[derive(Debug, Clone)]
pub struct HelloResponse {
    /// Server crate version, e.g. `"3.6.0"`.
    pub server_version: String,
    /// Wire spec protocol version, currently always `1`.
    pub protocol_version: i64,
    /// `true` when the server accepted the supplied credentials (or
    /// when auth is globally disabled).
    pub authenticated: bool,
    /// `true` when the authenticated principal carries `Role::Admin`.
    pub admin: bool,
    /// Capability names this connection can call.
    pub capabilities: Vec<String>,
}

impl HelloResponse {
    fn parse(value: &VectorizerValue) -> Self {
        let server_version = value
            .map_get("server_version")
            .and_then(|v| v.as_str())
            .map(str::to_owned)
            .unwrap_or_default();
        let protocol_version = value
            .map_get("protocol_version")
            .and_then(|v| v.as_int())
            .unwrap_or(0);
        let authenticated = value
            .map_get("authenticated")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let admin = value
            .map_get("admin")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let capabilities = value
            .map_get("capabilities")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default();
        Self {
            server_version,
            protocol_version,
            authenticated,
            admin,
            capabilities,
        }
    }
}

/// One connection to a Vectorizer RPC server.
pub struct RpcClient {
    /// `vectorizer://host:port`, kept for re-dialing with credentials.
    endpoint: String,
    /// Credentials + timeouts the current connection was dialed with.
    client_config: Mutex<thunder::ClientConfig>,
    /// The live multiplexed connection.
    client: Mutex<Arc<thunder::Client>>,
    /// Serializes re-dials so two concurrent HELLOs can't race a swap.
    redial: tokio::sync::Mutex<()>,
}

impl RpcClient {
    /// Convenience: parse a `vectorizer://host[:port]` URL and dial.
    ///
    /// Accepts every form documented at
    /// [`crate::rpc::endpoint::parse_endpoint`]:
    ///
    /// - `vectorizer://host:port` → RPC on the given port.
    /// - `vectorizer://host` → RPC on the default port 15503.
    /// - `host:port` (no scheme) → RPC.
    /// - `http(s)://...` → returns [`RpcClientError::Connection`] with a
    ///   clear message asking the caller to use the HTTP client
    ///   instead. The SDK ships the `http` Cargo feature for that
    ///   path; an `http://` URL is not a transport an RPC client can
    ///   speak.
    pub async fn connect_url(url: &str) -> Result<Self> {
        use super::endpoint::{Endpoint, parse_endpoint};
        match parse_endpoint(url).map_err(|e| RpcClientError::Connection(e.to_string()))? {
            Endpoint::Rpc { host, port } => Self::connect(format!("{host}:{port}")).await,
            Endpoint::Rest { url } => Err(RpcClientError::Connection(format!(
                "RpcClient cannot dial REST URL '{url}'; \
                 use the HTTP client (`vectorizer_sdk::VectorizerClient`) instead, \
                 or pass a `vectorizer://` URL"
            ))),
        }
    }

    /// Dial `addr` — `host:port`, or any form [`thunder::parse_endpoint`]
    /// accepts. Does NOT authenticate: pass credentials to [`Self::hello`],
    /// which re-dials with them in the handshake.
    pub async fn connect(addr: impl AsRef<str>) -> Result<Self> {
        let endpoint = addr.as_ref().to_owned();
        let client_config = thunder::ClientConfig::new()
            .client_name(concat!("vectorizer-sdk-rust/", env!("CARGO_PKG_VERSION")));
        let client = Self::dial(&endpoint, client_config.clone()).await?;
        Ok(Self {
            endpoint,
            client_config: Mutex::new(client_config),
            client: Mutex::new(client),
            redial: tokio::sync::Mutex::new(()),
        })
    }

    /// Per-call and connect timeout for this connection. Re-dials so the
    /// new timeouts apply to the live connection as well as later ones.
    pub async fn with_timeout(&self, timeout: Duration) -> Result<()> {
        let config = {
            let current = self.client_config.lock().clone();
            current.connect_timeout(timeout).call_timeout(timeout)
        };
        self.replace_connection(config).await
    }

    async fn dial(endpoint: &str, config: thunder::ClientConfig) -> Result<Arc<thunder::Client>> {
        thunder::Client::connect_with(endpoint, protocol_config(), config)
            .await
            .map(Arc::new)
            .map_err(RpcClientError::from)
    }

    /// Dial a fresh connection with `config` and swap it in, dropping the
    /// previous one. Serialized by `redial` so concurrent callers can't
    /// interleave swaps.
    async fn replace_connection(&self, config: thunder::ClientConfig) -> Result<()> {
        let _guard = self.redial.lock().await;
        let fresh = Self::dial(&self.endpoint, config.clone()).await?;
        *self.client_config.lock() = config;
        *self.client.lock() = fresh;
        Ok(())
    }

    fn client(&self) -> Arc<thunder::Client> {
        Arc::clone(&self.client.lock())
    }

    /// Issue the `HELLO` handshake and return the server's capability list
    /// and auth flags.
    ///
    /// When `payload` carries a token or an API key, the connection is
    /// re-dialed so those credentials travel in Thunder's `AUTH` handshake —
    /// that is what authenticates the session every later command runs under.
    /// A credential-free payload reuses the existing connection.
    pub async fn hello(&self, payload: HelloPayload) -> Result<HelloResponse> {
        if let Some(credentials) = payload.credentials() {
            let mut config = self.client_config.lock().clone();
            config.credentials = Some(credentials);
            if let Some(name) = &payload.client_name {
                config = config.client_name(name.clone());
            }
            self.replace_connection(config).await?;
        }
        let result = self.call("HELLO", vec![payload.into_value()]).await?;
        Ok(HelloResponse::parse(&result))
    }

    /// Health check. `PING` is auth-exempt, so this works before HELLO; the
    /// typed wrapper still validates the response shape.
    pub async fn ping(&self) -> Result<String> {
        let result = self.call("PING", vec![]).await?;
        result
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| RpcClientError::Server("PING returned non-string payload".into()))
    }

    /// Generic call dispatcher. Most callers should use a typed
    /// wrapper from [`crate::rpc::commands`] instead.
    ///
    /// Concurrent calls multiplex over the one connection; the server gates
    /// un-authenticated sessions, surfacing
    /// [`RpcClientError::NotAuthenticated`].
    pub async fn call(
        &self,
        command: impl Into<String>,
        args: Vec<VectorizerValue>,
    ) -> Result<VectorizerValue> {
        self.client()
            .call(command.into(), args)
            .await
            .map_err(RpcClientError::from)
    }

    /// Returns `true` once the connection's handshake authenticated. Always
    /// `false` against an open (single-user) server, which authenticates
    /// nobody because it gates nothing.
    pub fn is_authenticated(&self) -> bool {
        self.client().is_authenticated()
    }

    /// Close the connection. In-flight calls receive
    /// [`RpcClientError::Connection`].
    pub async fn close(self) {
        self.client().close().await;
    }
}
