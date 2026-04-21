//! Minimal RPC connection pool.
//!
//! A bounded pool of [`RpcClient`]s. `acquire()` returns an idle
//! client (or builds a new one if none are available and the pool
//! isn't at capacity); the returned guard returns the client to the
//! pool on `Drop`.
//!
//! This is intentionally NOT `bb8` / `deadpool` — those bring async
//! traits and heavyweight reconnect logic that the v1 SDK doesn't
//! need. If a future workload requires fancier pooling (e.g.
//! per-connection health checks, idle eviction), swap to a real pool
//! crate at that point.

use std::sync::Arc;

use parking_lot::Mutex;
use tokio::sync::Semaphore;

use super::client::{HelloPayload, RpcClient, RpcClientError};

/// Configuration for [`RpcPool`].
#[derive(Debug, Clone)]
pub struct RpcPoolConfig {
    /// Server address (`host:port`) every connection in the pool
    /// dials.
    pub address: String,
    /// Maximum number of concurrent open connections. Calls block on
    /// `acquire()` once this many are checked out.
    pub max_connections: usize,
    /// HELLO payload sent on every newly-built connection.
    pub hello: HelloPayload,
}

/// A minimal connection pool.
pub struct RpcPool {
    config: RpcPoolConfig,
    /// Semaphore limits the total number of live + checked-out
    /// connections to `max_connections`.
    permits: Arc<Semaphore>,
    /// Idle clients available for reuse. `None` is also a valid pool
    /// state (the slot is just empty); callers build a fresh client
    /// in that case.
    idle: Arc<Mutex<Vec<RpcClient>>>,
}

impl RpcPool {
    /// Build a new pool. Does NOT open any connections eagerly; the
    /// first `acquire()` call dials the first connection.
    pub fn new(config: RpcPoolConfig) -> Self {
        let max = config.max_connections.max(1);
        Self {
            permits: Arc::new(Semaphore::new(max)),
            idle: Arc::new(Mutex::new(Vec::with_capacity(max))),
            config,
        }
    }

    /// Acquire a client from the pool. Blocks (asynchronously) when
    /// the pool is at capacity until a slot frees. The returned
    /// [`PooledClient`] returns the client to the pool on `Drop`.
    pub async fn acquire(&self) -> Result<PooledClient, RpcClientError> {
        let permit = self
            .permits
            .clone()
            .acquire_owned()
            .await
            .expect("semaphore not closed");

        // Try idle list first; on miss, dial + handshake.
        let client = {
            let mut idle = self.idle.lock();
            idle.pop()
        };
        let client = match client {
            Some(c) => c,
            None => {
                let c = RpcClient::connect(&self.config.address).await?;
                let _ = c.hello(self.config.hello.clone()).await?;
                c
            }
        };

        Ok(PooledClient {
            inner: Some(client),
            idle: Arc::clone(&self.idle),
            _permit: permit,
        })
    }

    /// Number of idle clients currently sitting in the pool. Useful
    /// for diagnostics and testing — production code should not
    /// branch on this.
    pub fn idle_count(&self) -> usize {
        self.idle.lock().len()
    }
}

/// RAII guard returned by [`RpcPool::acquire`]. Returns the client to
/// the pool on `Drop` so subsequent acquires reuse the connection.
pub struct PooledClient {
    inner: Option<RpcClient>,
    idle: Arc<Mutex<Vec<RpcClient>>>,
    _permit: tokio::sync::OwnedSemaphorePermit,
}

impl PooledClient {
    /// Borrow the underlying client.
    pub fn client(&self) -> &RpcClient {
        self.inner
            .as_ref()
            .expect("PooledClient inner is only None during Drop")
    }
}

impl Drop for PooledClient {
    fn drop(&mut self) {
        // Move the client back into the idle list. If the inner
        // RpcClient was already torn (reader task dead) the next
        // acquire will see a stale client; for the v1 pool we rely on
        // the call-time `ConnectionClosed` error to surface the
        // problem rather than pre-validating on return. A future
        // version can add a health check here.
        if let Some(client) = self.inner.take() {
            let mut idle = self.idle.lock();
            idle.push(client);
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn config_round_trip() {
        let cfg = RpcPoolConfig {
            address: "localhost:15503".into(),
            max_connections: 4,
            hello: HelloPayload::new("test"),
        };
        // Constructing the pool with a non-empty config doesn't dial
        // — we can verify shape without a real server. The first
        // dial happens inside acquire(); that path is exercised by
        // the integration suite.
        let pool = RpcPool::new(cfg);
        assert_eq!(pool.idle_count(), 0);
    }
}
