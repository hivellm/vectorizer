//! DNS-based node discovery for Kubernetes headless services

use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use parking_lot::RwLock;
use tokio::net::lookup_host;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use super::manager::ClusterManager;
use super::node::{ClusterNode, NodeId};

/// DNS-based node discovery for Kubernetes headless services.
///
/// Periodically resolves a DNS name (typically a K8s headless service) and
/// reconciles the result against current cluster membership by adding newly
/// discovered nodes and marking removed nodes as unavailable.
pub struct DnsDiscovery {
    manager: Arc<ClusterManager>,
    dns_name: String,
    grpc_port: u16,
    resolve_interval: Duration,
    /// Previously known IPs (to detect additions/removals)
    known_ips: Arc<RwLock<HashSet<IpAddr>>>,
    running: Arc<RwLock<bool>>,
}

impl DnsDiscovery {
    /// Create a new DNS discovery instance.
    pub fn new(
        manager: Arc<ClusterManager>,
        dns_name: String,
        grpc_port: u16,
        resolve_interval: Duration,
    ) -> Self {
        Self {
            manager,
            dns_name,
            grpc_port,
            resolve_interval,
            known_ips: Arc::new(RwLock::new(HashSet::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start periodic DNS resolution.
    ///
    /// Performs an initial resolution immediately, then spawns a background
    /// task that re-resolves at `resolve_interval`. Calling `start` when
    /// already running is a no-op (a warning is logged).
    pub async fn start(&self) {
        {
            let mut running = self.running.write();
            if *running {
                warn!("DNS discovery already running");
                return;
            }
            *running = true;
        }

        info!(
            "Starting DNS discovery for '{}' every {:?}",
            self.dns_name, self.resolve_interval
        );

        // Perform initial resolution before handing off to the background task.
        if let Err(e) = self.resolve_and_update().await {
            error!("Initial DNS resolution failed: {}", e);
        }

        // Clone fields needed by the spawned task.
        let task = DnsDiscovery {
            manager: self.manager.clone(),
            dns_name: self.dns_name.clone(),
            grpc_port: self.grpc_port,
            resolve_interval: self.resolve_interval,
            known_ips: self.known_ips.clone(),
            running: self.running.clone(),
        };

        tokio::spawn(async move {
            let mut tick = interval(task.resolve_interval);
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tick.tick().await;

                {
                    let is_running = task.running.read();
                    if !*is_running {
                        break;
                    }
                }

                if let Err(e) = task.resolve_and_update().await {
                    warn!("DNS resolution failed: {}", e);
                }
            }
            info!("DNS discovery stopped");
        });
    }

    /// Stop periodic DNS resolution.
    pub fn stop(&self) {
        *self.running.write() = false;
    }

    /// Resolve the configured DNS name and reconcile cluster membership.
    ///
    /// DNS resolution failures are treated as transient (common during K8s
    /// pod startup) and propagated so the caller can log at the appropriate
    /// severity.
    async fn resolve_and_update(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // `lookup_host` requires a `host:port` string.
        let lookup_addr = format!("{}:{}", self.dns_name, self.grpc_port);

        let mut resolved_ips: HashSet<IpAddr> = HashSet::new();

        match lookup_host(&lookup_addr).await {
            Ok(addrs) => {
                for addr in addrs {
                    resolved_ips.insert(addr.ip());
                }
            }
            Err(e) => {
                debug!(
                    "DNS lookup for '{}' failed: {} (may be transient)",
                    self.dns_name, e
                );
                return Err(e.into());
            }
        }

        let previous_ips = self.known_ips.read().clone();

        // Add newly discovered nodes.
        let new_ips: Vec<IpAddr> = resolved_ips.difference(&previous_ips).copied().collect();
        for ip in &new_ips {
            let node_id = NodeId::new(format!("dns-{}", ip));
            let address = ip.to_string();

            info!("DNS discovery: new node detected at {}", ip);

            let mut node = ClusterNode::new(node_id, address, self.grpc_port);
            node.mark_active();
            self.manager.add_node(node);
        }

        // Mark removed nodes as unavailable.
        let removed_ips: Vec<IpAddr> = previous_ips.difference(&resolved_ips).copied().collect();
        for ip in &removed_ips {
            let node_id = NodeId::new(format!("dns-{}", ip));

            info!("DNS discovery: node removed at {}", ip);
            self.manager.mark_node_unavailable(&node_id);
        }

        *self.known_ips.write() = resolved_ips;

        if !new_ips.is_empty() || !removed_ips.is_empty() {
            info!(
                "DNS discovery update: {} new, {} removed, {} total",
                new_ips.len(),
                removed_ips.len(),
                self.known_ips.read().len()
            );
        }

        Ok(())
    }
}
