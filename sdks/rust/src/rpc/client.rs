//! `RpcClient`: connect, hello, call, ping, close.
//!
//! The client owns one TCP connection to the server. It runs a single
//! background reader task that demultiplexes responses by `Request.id`
//! into per-call `oneshot` channels, so concurrent in-flight calls
//! on the same connection don't block each other.
//!
//! Auth is **per-connection sticky** per wire spec § 4: the first
//! frame on a connection MUST be `HELLO`; every subsequent call
//! inherits the auth state. The client tracks the authenticated /
//! admin flags from the HELLO response so callers can introspect
//! after the handshake.

use std::collections::HashMap;
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use parking_lot::Mutex;
use tokio::io::BufReader;
use tokio::net::TcpStream;
use tokio::sync::{Notify, oneshot};
use tokio::task::JoinHandle;
use tracing::{debug, warn};

use super::codec::{read_response, write_request};
use super::types::{Request, Response, VectorizerValue};

/// Errors the [`RpcClient`] can return.
#[derive(Debug, thiserror::Error)]
pub enum RpcClientError {
    /// Network-level I/O failure.
    #[error("network I/O error: {0}")]
    Io(#[from] io::Error),

    /// MessagePack encode failure (should be unreachable for the v1
    /// shapes — every type derives `Serialize`).
    #[error("encode failed: {0}")]
    Encode(#[from] rmp_serde::encode::Error),

    /// Server returned `Result::Err(message)` for the call.
    #[error("server error: {0}")]
    Server(String),

    /// The connection's reader task died before the response arrived.
    #[error("connection closed before response (reader task ended)")]
    ConnectionClosed,

    /// Caller invoked a data-plane command before HELLO succeeded.
    /// The server would reject this; the client surfaces it locally
    /// so the offending caller sees a clear panic-free error.
    #[error("HELLO must succeed before any data-plane command can be issued")]
    NotAuthenticated,
}

/// Result type alias.
pub type Result<T> = std::result::Result<T, RpcClientError>;

/// HELLO request payload — sent as the FIRST frame on a connection.
///
/// At least one of `token` / `api_key` should be populated when the
/// server has auth enabled. When the server runs in single-user mode
/// (`auth.enabled: false`), credentials are accepted-but-ignored and
/// the connection runs as the implicit local admin.
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
    /// Server crate version, e.g. `"3.0.0"`.
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
    /// Owned write half of the TCP socket. Wrapped in a mutex because
    /// every `call` writes serially; the writer is the only one that
    /// touches this half.
    writer: Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>,
    /// Map from request id → oneshot sender for the matching response.
    pending: Arc<Mutex<HashMap<u32, oneshot::Sender<Response>>>>,
    /// Monotonic id allocator.
    next_id: AtomicU32,
    /// Notified when the reader task exits, so pending calls fail
    /// fast instead of hanging forever.
    reader_done: Arc<Notify>,
    /// Handle to the spawned reader task; aborted on `Drop`.
    reader_task: Option<JoinHandle<()>>,
    /// `true` once HELLO succeeded.
    authenticated: Arc<Mutex<bool>>,
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
    /// - `http(s)://...` → returns [`RpcClientError::Server`] with a
    ///   clear message asking the caller to use the HTTP client
    ///   instead. The SDK ships the `http` Cargo feature for that
    ///   path; an `http://` URL is not a transport an RPC client can
    ///   speak.
    pub async fn connect_url(url: &str) -> Result<Self> {
        use super::endpoint::{Endpoint, parse_endpoint};
        match parse_endpoint(url).map_err(|e| RpcClientError::Server(e.to_string()))? {
            Endpoint::Rpc { host, port } => Self::connect(format!("{host}:{port}")).await,
            Endpoint::Rest { url } => Err(RpcClientError::Server(format!(
                "RpcClient cannot dial REST URL '{url}'; \
                 use the HTTP client (`vectorizer_sdk::VectorizerClient`) instead, \
                 or pass a `vectorizer://` URL"
            ))),
        }
    }

    /// Open a TCP connection to `addr` (which must be `host:port`)
    /// and start the background reader task. Does NOT send HELLO —
    /// callers MUST call [`Self::hello`] before any data-plane
    /// command, or the server will reject it.
    pub async fn connect(addr: impl tokio::net::ToSocketAddrs) -> Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        let (read_half, write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half);

        let pending: Arc<Mutex<HashMap<u32, oneshot::Sender<Response>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let reader_done = Arc::new(Notify::new());

        // Spawn the reader: read frames forever, dispatch to pending
        // by id, close down on EOF.
        let pending_for_reader = Arc::clone(&pending);
        let done_for_reader = Arc::clone(&reader_done);
        let reader_task = tokio::spawn(async move {
            loop {
                match read_response(&mut reader).await {
                    Ok(resp) => {
                        let sender = {
                            let mut p = pending_for_reader.lock();
                            p.remove(&resp.id)
                        };
                        match sender {
                            Some(tx) => {
                                let _ = tx.send(resp);
                            }
                            None => {
                                warn!(
                                    id = resp.id,
                                    "RpcClient received response with no pending caller — dropping"
                                );
                            }
                        }
                    }
                    Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                        debug!("RpcClient reader: clean EOF");
                        break;
                    }
                    Err(e) => {
                        warn!(error = %e, "RpcClient reader error — connection closed");
                        break;
                    }
                }
            }
            // Drain pending — every waiting call gets ConnectionClosed.
            let mut p = pending_for_reader.lock();
            p.clear();
            done_for_reader.notify_waiters();
        });

        Ok(Self {
            writer: Arc::new(tokio::sync::Mutex::new(write_half)),
            pending,
            next_id: AtomicU32::new(1),
            reader_done,
            reader_task: Some(reader_task),
            authenticated: Arc::new(Mutex::new(false)),
        })
    }

    /// Issue the `HELLO` handshake. Must be the first call on a fresh
    /// connection. Returns the server's capability list and auth flags.
    pub async fn hello(&self, payload: HelloPayload) -> Result<HelloResponse> {
        let value = payload.into_value();
        let result = self.raw_call("HELLO", vec![value]).await?;
        let parsed = HelloResponse::parse(&result);
        if parsed.authenticated {
            *self.authenticated.lock() = true;
        }
        Ok(parsed)
    }

    /// Health check. The server treats `PING` as auth-exempt so this
    /// works even before HELLO; the typed wrapper still validates the
    /// response shape.
    pub async fn ping(&self) -> Result<String> {
        let result = self.raw_call("PING", vec![]).await?;
        result
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| RpcClientError::Server("PING returned non-string payload".into()))
    }

    /// Generic call dispatcher. Most callers should use a typed
    /// wrapper from [`crate::rpc::commands`] instead.
    pub async fn call(
        &self,
        command: impl Into<String>,
        args: Vec<VectorizerValue>,
    ) -> Result<VectorizerValue> {
        let cmd = command.into();
        // Auth-exempt commands per wire spec § 4.
        let exempt = matches!(cmd.as_str(), "HELLO" | "PING");
        if !exempt && !*self.authenticated.lock() {
            return Err(RpcClientError::NotAuthenticated);
        }
        self.raw_call(cmd, args).await
    }

    /// Skip the local auth check — used by the HELLO + PING paths so
    /// the auth gate doesn't block the auth handshake itself.
    async fn raw_call(
        &self,
        command: impl Into<String>,
        args: Vec<VectorizerValue>,
    ) -> Result<VectorizerValue> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel::<Response>();
        {
            let mut pending = self.pending.lock();
            pending.insert(id, tx);
        }

        let req = Request {
            id,
            command: command.into(),
            args,
        };

        // Write the frame under the writer mutex so concurrent calls
        // don't interleave bytes.
        {
            let mut writer = self.writer.lock().await;
            if let Err(e) = write_request(&mut *writer, &req).await {
                self.pending.lock().remove(&id);
                return Err(RpcClientError::from(e));
            }
        }

        // Race the response against the reader-task-exited notifier so
        // a torn connection fails fast instead of hanging.
        let resp = tokio::select! {
            recv = rx => match recv {
                Ok(resp) => resp,
                Err(_) => return Err(RpcClientError::ConnectionClosed),
            },
            _ = self.reader_done.notified() => {
                self.pending.lock().remove(&id);
                return Err(RpcClientError::ConnectionClosed);
            }
        };

        match resp.result {
            Ok(value) => Ok(value),
            Err(message) => Err(RpcClientError::Server(message)),
        }
    }

    /// Returns `true` once HELLO has succeeded on this connection.
    pub fn is_authenticated(&self) -> bool {
        *self.authenticated.lock()
    }

    /// Close the connection. Aborts the reader task; in-flight calls
    /// receive `ConnectionClosed`.
    pub fn close(mut self) {
        if let Some(handle) = self.reader_task.take() {
            handle.abort();
        }
    }
}

impl Drop for RpcClient {
    fn drop(&mut self) {
        if let Some(handle) = self.reader_task.take() {
            handle.abort();
        }
    }
}
