//! REST `VectorizerClient` — split per API surface (phase4).
//!
//! Public-API entry point for the legacy HTTP transport. Phase4
//! split the original 1,989-line `client.rs` into one struct + 8
//! per-surface impl files; every method is reachable through the
//! same `VectorizerClient` facade for backward compat.
//!
//! - Struct, config, ctors, `with_master`, `make_request`,
//!   read/write transport selection — this file.
//! - One `impl VectorizerClient` block per surface in the matching
//!   submodule (Rust permits as many impl blocks as you like for the
//!   same struct, across files of the same module).
//!
//! ## Per-surface modules
//!
//! | Surface | Methods |
//! |---|---|
//! | [`core`] | `health_check` |
//! | [`collections`] | `list_collections`, `create_collection`, `delete_collection`, `get_collection_info` |
//! | [`vectors`] | `get_vector`, `insert_texts`, `embed_text` |
//! | [`search`] | `search_vectors`, `intelligent_search`, `semantic_search`, `contextual_search`, `multi_collection_search`, `hybrid_search` |
//! | [`discovery`] | `discover`, `filter_collections`, `score_collections`, `expand_queries` |
//! | [`files`] | `get_file_content`, `list_files_in_collection`, `get_file_summary`, `get_file_chunks_ordered`, `get_project_outline`, `get_related_files`, `search_by_file_type`, `upload_file`, `upload_file_content`, `get_upload_config` |
//! | [`graph`] | `list_graph_nodes`, `get_graph_neighbors`, `find_related_nodes`, `find_graph_path`, `create_graph_edge`, `delete_graph_edge`, `list_graph_edges`, `discover_graph_edges`, `discover_graph_edges_for_node`, `get_graph_discovery_status` |
//! | [`qdrant`] | 25 `qdrant_*` methods (Qdrant-compatible REST surface) |
//!
//! ## RPC readiness
//!
//! Every per-surface impl calls through `self.make_request` →
//! `self.transport: Arc<dyn Transport>`. The `Transport` trait
//! (declared in [`crate::transport`]) is implemented by
//! [`crate::http_transport::HttpTransport`] today; the RPC backend
//! from `phase6_sdk-rust-rpc` plugs into the same interface so the
//! per-surface modules don't need any changes when the canonical
//! `vectorizer://host:15503` transport lands as the default. See
//! [`crate::rpc`] for the RPC client built directly on `tokio::net`
//! — it lives alongside this REST facade rather than under it.

use std::sync::Arc;

use crate::error::{Result, VectorizerError};
use crate::http_transport::HttpTransport;
use crate::models::*;
use crate::transport::{Protocol, Transport};
#[cfg(feature = "umicp")]
use crate::umicp_transport::UmicpTransport;

pub mod collections;
pub mod core;
pub mod discovery;
pub mod files;
pub mod graph;
pub mod qdrant;
pub mod search;
pub mod vectors;

/// Configuration for [`VectorizerClient`].
#[derive(Clone)]
pub struct ClientConfig {
    /// Base URL for HTTP transport (single-node deployments).
    pub base_url: Option<String>,
    /// Connection string (supports `http://`, `https://`, `umicp://`).
    pub connection_string: Option<String>,
    /// Protocol to use.
    pub protocol: Option<Protocol>,
    /// API key for authentication.
    pub api_key: Option<String>,
    /// Request timeout in seconds.
    pub timeout_secs: Option<u64>,
    /// UMICP configuration.
    #[cfg(feature = "umicp")]
    pub umicp: Option<UmicpConfig>,
    /// Master/replica host configuration for read/write routing.
    pub hosts: Option<HostConfig>,
    /// Default read preference for read operations.
    pub read_preference: Option<ReadPreference>,
}

#[cfg(feature = "umicp")]
/// UMICP-specific configuration.
#[derive(Clone)]
pub struct UmicpConfig {
    /// UMICP host name or address.
    pub host: String,
    /// UMICP TCP port.
    pub port: u16,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: Some("http://localhost:15002".to_string()),
            connection_string: None,
            protocol: None,
            api_key: None,
            timeout_secs: Some(30),
            #[cfg(feature = "umicp")]
            umicp: None,
            hosts: None,
            read_preference: None,
        }
    }
}

/// Vectorizer REST client with optional master/replica topology
/// support. Public surface is identical to the pre-phase4
/// monolithic `VectorizerClient`; the methods are now organised
/// across per-surface impl blocks (see module docs).
pub struct VectorizerClient {
    pub(crate) transport: Arc<dyn Transport>,
    protocol: Protocol,
    base_url: String,
    /// Master transport for write operations (if replica mode is enabled).
    #[allow(dead_code)]
    master_transport: Option<Arc<dyn Transport>>,
    /// Replica transports for read operations (if replica mode is enabled).
    #[allow(dead_code)]
    replica_transports: Vec<Arc<dyn Transport>>,
    /// Current replica index for round-robin selection.
    #[allow(dead_code)]
    replica_index: std::sync::atomic::AtomicUsize,
    /// Default read preference.
    #[allow(dead_code)]
    read_preference: ReadPreference,
    /// Whether replica mode is enabled.
    #[allow(dead_code)]
    is_replica_mode: bool,
    /// Original config for creating child clients (e.g. `with_master`).
    pub(crate) config: ClientConfig,
}

impl VectorizerClient {
    /// Get the base URL the client is configured against.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Create a new client with the given configuration.
    pub fn new(config: ClientConfig) -> Result<Self> {
        let timeout_secs = config.timeout_secs.unwrap_or(30);

        // Determine protocol and create transport.
        let (transport, protocol, base_url): (Arc<dyn Transport>, Protocol, String) =
            if let Some(ref conn_str) = config.connection_string {
                #[allow(unused_variables)]
                let (proto, host, port) = crate::transport::parse_connection_string(conn_str)?;

                match proto {
                    Protocol::Http => {
                        let transport =
                            HttpTransport::new(&host, config.api_key.as_deref(), timeout_secs)?;
                        (Arc::new(transport), Protocol::Http, host.clone())
                    }
                    #[cfg(feature = "umicp")]
                    Protocol::Umicp => {
                        let umicp_port = port.unwrap_or(15003);
                        let transport = UmicpTransport::new(
                            &host,
                            umicp_port,
                            config.api_key.as_deref(),
                            timeout_secs,
                        )?;
                        let base_url = format!("umicp://{host}:{umicp_port}");
                        (Arc::new(transport), Protocol::Umicp, base_url)
                    }
                }
            } else {
                let proto = config.protocol.unwrap_or(Protocol::Http);

                match proto {
                    Protocol::Http => {
                        let base_url = config
                            .base_url
                            .clone()
                            .unwrap_or_else(|| "http://localhost:15002".to_string());
                        let transport =
                            HttpTransport::new(&base_url, config.api_key.as_deref(), timeout_secs)?;
                        (Arc::new(transport), Protocol::Http, base_url)
                    }
                    #[cfg(feature = "umicp")]
                    Protocol::Umicp => {
                        #[cfg(feature = "umicp")]
                        {
                            let umicp_config = config.umicp.clone().ok_or_else(|| {
                                VectorizerError::configuration(
                                    "UMICP configuration is required when using UMICP protocol",
                                )
                            })?;

                            let transport = UmicpTransport::new(
                                &umicp_config.host,
                                umicp_config.port,
                                config.api_key.as_deref(),
                                timeout_secs,
                            )?;
                            let base_url =
                                format!("umicp://{}:{}", umicp_config.host, umicp_config.port);
                            (Arc::new(transport), Protocol::Umicp, base_url)
                        }
                        #[cfg(not(feature = "umicp"))]
                        {
                            return Err(VectorizerError::configuration(
                                "UMICP feature is not enabled. Enable it with --features umicp",
                            ));
                        }
                    }
                }
            };

        // Initialise replica mode if hosts are configured.
        let (master_transport, replica_transports, is_replica_mode) =
            if let Some(ref hosts) = config.hosts {
                let master =
                    HttpTransport::new(&hosts.master, config.api_key.as_deref(), timeout_secs)?;
                let replicas: Result<Vec<Arc<dyn Transport>>> = hosts
                    .replicas
                    .iter()
                    .map(|url| {
                        let t = HttpTransport::new(url, config.api_key.as_deref(), timeout_secs)?;
                        Ok(Arc::new(t) as Arc<dyn Transport>)
                    })
                    .collect();
                (
                    Some(Arc::new(master) as Arc<dyn Transport>),
                    replicas?,
                    true,
                )
            } else {
                (None, vec![], false)
            };

        let read_preference = config.read_preference.unwrap_or(ReadPreference::Replica);

        Ok(Self {
            transport,
            protocol,
            base_url,
            master_transport,
            replica_transports,
            replica_index: std::sync::atomic::AtomicUsize::new(0),
            read_preference,
            is_replica_mode,
            config,
        })
    }

    /// Create a new client with default configuration.
    pub fn new_default() -> Result<Self> {
        Self::new(ClientConfig::default())
    }

    /// Create a client with a custom base URL.
    pub fn new_with_url(base_url: &str) -> Result<Self> {
        Self::new(ClientConfig {
            base_url: Some(base_url.to_string()),
            ..Default::default()
        })
    }

    /// Create a client with a custom base URL + API key.
    pub fn new_with_api_key(base_url: &str, api_key: &str) -> Result<Self> {
        Self::new(ClientConfig {
            base_url: Some(base_url.to_string()),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        })
    }

    /// Create a client from a full connection string
    /// (`http(s)://host[:port]` or `umicp://host[:port]`).
    pub fn from_connection_string(connection_string: &str, api_key: Option<&str>) -> Result<Self> {
        Self::new(ClientConfig {
            connection_string: Some(connection_string.to_string()),
            api_key: api_key.map(|s| s.to_string()),
            ..Default::default()
        })
    }

    /// Returns the protocol the client is currently using.
    pub fn protocol(&self) -> Protocol {
        self.protocol
    }

    /// Get transport for write operations (always master).
    #[allow(dead_code)]
    pub(crate) fn get_write_transport(&self) -> &Arc<dyn Transport> {
        if self.is_replica_mode {
            self.master_transport.as_ref().unwrap_or(&self.transport)
        } else {
            &self.transport
        }
    }

    /// Get transport for read operations based on the active read
    /// preference (or the per-call override in `options`).
    #[allow(dead_code)]
    pub(crate) fn get_read_transport(&self, options: Option<&ReadOptions>) -> &Arc<dyn Transport> {
        if !self.is_replica_mode {
            return &self.transport;
        }

        let preference = options
            .and_then(|o| o.read_preference)
            .unwrap_or(self.read_preference);

        match preference {
            ReadPreference::Master => self.master_transport.as_ref().unwrap_or(&self.transport),
            ReadPreference::Replica | ReadPreference::Nearest => {
                if self.replica_transports.is_empty() {
                    return self.master_transport.as_ref().unwrap_or(&self.transport);
                }
                let idx = self
                    .replica_index
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                    % self.replica_transports.len();
                &self.replica_transports[idx]
            }
        }
    }

    /// Execute a callback with master transport for read-your-writes
    /// scenarios. All operations within the callback are routed to
    /// master.
    pub async fn with_master<F, Fut, T>(&self, callback: F) -> Result<T>
    where
        F: FnOnce(VectorizerClient) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut master_config = self.config.clone();
        master_config.read_preference = Some(ReadPreference::Master);
        let master_client = VectorizerClient::new(master_config)?;
        callback(master_client).await
    }

    /// Construct a [`VectorizerClient`] directly from a custom
    /// [`Transport`] implementation. **Test-only / advanced use.**
    ///
    /// The dispatcher fields (`master_transport`, `replica_transports`,
    /// `is_replica_mode`) are all left empty — the client behaves as
    /// a single-transport instance. Used by mock-based tests to swap
    /// the real HTTP backend out for an in-memory one without
    /// touching the per-surface modules.
    ///
    /// This entry point is the **RPC-readiness regression guard**
    /// (phase 4 task 2.4): if any per-surface module accidentally
    /// hard-codes `HttpTransport` or `reqwest::Client`, the
    /// `MockTransport` integration test in
    /// `tests/mock_transport_regression.rs` stops compiling. The
    /// same `Transport` trait the [`crate::rpc`] backend will plug
    /// into from `phase6_sdk-rust-rpc` is what mocks ride here.
    pub fn with_transport(transport: Arc<dyn Transport>, base_url: impl Into<String>) -> Self {
        let protocol = transport.protocol();
        Self {
            transport,
            protocol,
            base_url: base_url.into(),
            master_transport: None,
            replica_transports: Vec::new(),
            replica_index: std::sync::atomic::AtomicUsize::new(0),
            read_preference: ReadPreference::Master,
            is_replica_mode: false,
            config: ClientConfig::default(),
        }
    }

    /// Internal helper: dispatch one HTTP-method-name call through
    /// the active transport. Per-surface modules call this instead
    /// of poking the `Transport` directly so future routing changes
    /// (e.g. write-vs-read selection) land in one place.
    pub(crate) async fn make_request(
        &self,
        method: &str,
        endpoint: &str,
        payload: Option<serde_json::Value>,
    ) -> Result<String> {
        match method {
            "GET" => self.transport.get(endpoint).await,
            "POST" => self.transport.post(endpoint, payload.as_ref()).await,
            "PUT" => self.transport.put(endpoint, payload.as_ref()).await,
            "DELETE" => self.transport.delete(endpoint).await,
            _ => Err(VectorizerError::configuration(format!(
                "Unsupported method: {method}"
            ))),
        }
    }
}
