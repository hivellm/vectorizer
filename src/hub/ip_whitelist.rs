//! IP Whitelisting Module
//!
//! Provides IP-based access control for HiveHub integration.
//! Supports both IPv4 and IPv6 addresses with CIDR notation.
//!
//! ## Features
//!
//! - IPv4 and IPv6 support
//! - CIDR notation (e.g., 192.168.1.0/24)
//! - Private network detection
//! - Configurable allowlist and blocklist
//! - Tenant-specific IP restrictions

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::error::{Result, VectorizerError};

/// Configuration for IP whitelisting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpWhitelistConfig {
    /// Whether IP whitelisting is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Default policy when IP doesn't match any rule
    #[serde(default)]
    pub default_policy: IpPolicy,

    /// Global allowlist (applies to all requests)
    #[serde(default)]
    pub global_allowlist: Vec<String>,

    /// Global blocklist (applies to all requests, takes precedence over allowlist)
    #[serde(default)]
    pub global_blocklist: Vec<String>,

    /// Whether to allow private/internal IPs
    #[serde(default = "default_allow_private")]
    pub allow_private: bool,

    /// Whether to allow localhost
    #[serde(default = "default_allow_localhost")]
    pub allow_localhost: bool,

    /// Trusted proxy headers for getting real client IP
    #[serde(default = "default_trusted_headers")]
    pub trusted_headers: Vec<String>,

    /// Maximum number of hops for X-Forwarded-For
    #[serde(default = "default_max_forwarded_hops")]
    pub max_forwarded_hops: usize,
}

fn default_allow_private() -> bool {
    true
}

fn default_allow_localhost() -> bool {
    true
}

fn default_trusted_headers() -> Vec<String> {
    vec![
        "X-Forwarded-For".to_string(),
        "X-Real-IP".to_string(),
        "CF-Connecting-IP".to_string(),
    ]
}

fn default_max_forwarded_hops() -> usize {
    1
}

impl Default for IpWhitelistConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            default_policy: IpPolicy::Allow,
            global_allowlist: vec![],
            global_blocklist: vec![],
            allow_private: true,
            allow_localhost: true,
            trusted_headers: default_trusted_headers(),
            max_forwarded_hops: default_max_forwarded_hops(),
        }
    }
}

/// Default policy for IP addresses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum IpPolicy {
    /// Allow by default, check blocklist
    #[default]
    Allow,
    /// Deny by default, check allowlist
    Deny,
}

/// Parsed IP range (supports CIDR notation)
#[derive(Debug, Clone)]
pub struct IpRange {
    /// Network address
    network: IpAddr,
    /// Prefix length (CIDR)
    prefix_len: u8,
}

impl IpRange {
    /// Parse IP range from string
    /// Supports: "192.168.1.1", "192.168.1.0/24", "::1", "2001:db8::/32"
    pub fn parse(s: &str) -> Result<Self> {
        if s.contains('/') {
            // CIDR notation
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() != 2 {
                return Err(VectorizerError::ConfigurationError(format!(
                    "Invalid CIDR notation: {}",
                    s
                )));
            }

            let network = IpAddr::from_str(parts[0]).map_err(|e| {
                VectorizerError::ConfigurationError(format!("Invalid IP address: {}", e))
            })?;

            let prefix_len: u8 = parts[1].parse().map_err(|e| {
                VectorizerError::ConfigurationError(format!("Invalid prefix length: {}", e))
            })?;

            let max_prefix = if network.is_ipv4() { 32 } else { 128 };
            if prefix_len > max_prefix {
                return Err(VectorizerError::ConfigurationError(format!(
                    "Prefix length {} exceeds maximum {} for {:?}",
                    prefix_len, max_prefix, network
                )));
            }

            Ok(Self {
                network,
                prefix_len,
            })
        } else {
            // Single IP address
            let network = IpAddr::from_str(s).map_err(|e| {
                VectorizerError::ConfigurationError(format!("Invalid IP address: {}", e))
            })?;

            let prefix_len = if network.is_ipv4() { 32 } else { 128 };

            Ok(Self {
                network,
                prefix_len,
            })
        }
    }

    /// Check if an IP address is within this range
    pub fn contains(&self, ip: &IpAddr) -> bool {
        match (&self.network, ip) {
            (IpAddr::V4(net), IpAddr::V4(addr)) => {
                let net_bits = u32::from(*net);
                let addr_bits = u32::from(*addr);
                let mask = if self.prefix_len == 0 {
                    0
                } else {
                    !0u32 << (32 - self.prefix_len)
                };
                (net_bits & mask) == (addr_bits & mask)
            }
            (IpAddr::V6(net), IpAddr::V6(addr)) => {
                let net_bits = u128::from(*net);
                let addr_bits = u128::from(*addr);
                let mask = if self.prefix_len == 0 {
                    0
                } else {
                    !0u128 << (128 - self.prefix_len)
                };
                (net_bits & mask) == (addr_bits & mask)
            }
            _ => false, // IPv4/IPv6 mismatch
        }
    }
}

/// Result of IP access check
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpAccessResult {
    /// Access allowed
    Allowed,
    /// Access denied (with reason)
    Denied(String),
    /// Access allowed but IP is private/internal
    AllowedPrivate,
    /// Access allowed but IP is localhost
    AllowedLocalhost,
}

impl IpAccessResult {
    pub fn is_allowed(&self) -> bool {
        matches!(
            self,
            IpAccessResult::Allowed
                | IpAccessResult::AllowedPrivate
                | IpAccessResult::AllowedLocalhost
        )
    }
}

/// IP whitelist validator
#[derive(Debug)]
pub struct IpWhitelist {
    /// Configuration
    config: IpWhitelistConfig,
    /// Parsed global allowlist
    allowlist: Vec<IpRange>,
    /// Parsed global blocklist
    blocklist: Vec<IpRange>,
    /// Tenant-specific allowlists
    tenant_allowlists: Arc<RwLock<HashMap<String, Vec<IpRange>>>>,
    /// Tenant-specific blocklists
    tenant_blocklists: Arc<RwLock<HashMap<String, Vec<IpRange>>>>,
}

impl IpWhitelist {
    /// Create a new IP whitelist validator
    pub fn new(config: IpWhitelistConfig) -> Result<Self> {
        let allowlist = config
            .global_allowlist
            .iter()
            .map(|s| IpRange::parse(s))
            .collect::<Result<Vec<_>>>()?;

        let blocklist = config
            .global_blocklist
            .iter()
            .map(|s| IpRange::parse(s))
            .collect::<Result<Vec<_>>>()?;

        info!(
            "IP whitelist initialized: {} allowed, {} blocked, default: {:?}",
            allowlist.len(),
            blocklist.len(),
            config.default_policy
        );

        Ok(Self {
            config,
            allowlist,
            blocklist,
            tenant_allowlists: Arc::new(RwLock::new(HashMap::new())),
            tenant_blocklists: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Check if IP whitelisting is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Check access for an IP address
    pub fn check_access(&self, ip: &IpAddr) -> IpAccessResult {
        if !self.config.enabled {
            return IpAccessResult::Allowed;
        }

        // Check localhost
        if self.is_localhost(ip) {
            if self.config.allow_localhost {
                return IpAccessResult::AllowedLocalhost;
            } else {
                return IpAccessResult::Denied("Localhost access denied".to_string());
            }
        }

        // Check private networks
        if self.is_private(ip) {
            if self.config.allow_private {
                return IpAccessResult::AllowedPrivate;
            } else {
                return IpAccessResult::Denied("Private network access denied".to_string());
            }
        }

        // Check global blocklist first (highest priority)
        for range in &self.blocklist {
            if range.contains(ip) {
                debug!("IP {} matched blocklist entry", ip);
                return IpAccessResult::Denied(format!("IP {} is blocked", ip));
            }
        }

        // Check global allowlist
        let in_allowlist = self.allowlist.iter().any(|range| range.contains(ip));

        match self.config.default_policy {
            IpPolicy::Allow => {
                // Allow by default, only deny if explicitly blocked
                IpAccessResult::Allowed
            }
            IpPolicy::Deny => {
                // Deny by default, only allow if explicitly whitelisted
                if in_allowlist {
                    IpAccessResult::Allowed
                } else {
                    IpAccessResult::Denied(format!("IP {} not in allowlist", ip))
                }
            }
        }
    }

    /// Check access for an IP address with tenant context
    pub fn check_access_for_tenant(&self, ip: &IpAddr, tenant_id: &str) -> IpAccessResult {
        // First check global rules
        let global_result = self.check_access(ip);
        if !global_result.is_allowed() {
            return global_result;
        }

        // Then check tenant-specific blocklist
        {
            let blocklists = self.tenant_blocklists.read();
            if let Some(tenant_blocklist) = blocklists.get(tenant_id) {
                for range in tenant_blocklist {
                    if range.contains(ip) {
                        return IpAccessResult::Denied(format!(
                            "IP {} blocked for tenant {}",
                            ip, tenant_id
                        ));
                    }
                }
            }
        }

        // Check tenant-specific allowlist if tenant has one
        {
            let allowlists = self.tenant_allowlists.read();
            if let Some(tenant_allowlist) = allowlists.get(tenant_id) {
                if !tenant_allowlist.is_empty() {
                    let in_tenant_allowlist =
                        tenant_allowlist.iter().any(|range| range.contains(ip));
                    if !in_tenant_allowlist {
                        return IpAccessResult::Denied(format!(
                            "IP {} not in tenant {} allowlist",
                            ip, tenant_id
                        ));
                    }
                }
            }
        }

        global_result
    }

    /// Set tenant-specific allowlist
    pub fn set_tenant_allowlist(&self, tenant_id: &str, ips: Vec<String>) -> Result<()> {
        let ranges = ips
            .iter()
            .map(|s| IpRange::parse(s))
            .collect::<Result<Vec<_>>>()?;

        let mut allowlists = self.tenant_allowlists.write();
        if ranges.is_empty() {
            allowlists.remove(tenant_id);
        } else {
            allowlists.insert(tenant_id.to_string(), ranges);
        }

        info!(
            "Updated allowlist for tenant {}: {} entries",
            tenant_id,
            ips.len()
        );
        Ok(())
    }

    /// Set tenant-specific blocklist
    pub fn set_tenant_blocklist(&self, tenant_id: &str, ips: Vec<String>) -> Result<()> {
        let ranges = ips
            .iter()
            .map(|s| IpRange::parse(s))
            .collect::<Result<Vec<_>>>()?;

        let mut blocklists = self.tenant_blocklists.write();
        if ranges.is_empty() {
            blocklists.remove(tenant_id);
        } else {
            blocklists.insert(tenant_id.to_string(), ranges);
        }

        info!(
            "Updated blocklist for tenant {}: {} entries",
            tenant_id,
            ips.len()
        );
        Ok(())
    }

    /// Remove tenant IP rules
    pub fn remove_tenant_rules(&self, tenant_id: &str) {
        self.tenant_allowlists.write().remove(tenant_id);
        self.tenant_blocklists.write().remove(tenant_id);
        info!("Removed IP rules for tenant {}", tenant_id);
    }

    /// Check if IP is localhost
    fn is_localhost(&self, ip: &IpAddr) -> bool {
        match ip {
            IpAddr::V4(addr) => addr.is_loopback(),
            IpAddr::V6(addr) => addr.is_loopback(),
        }
    }

    /// Check if IP is private/internal
    fn is_private(&self, ip: &IpAddr) -> bool {
        match ip {
            IpAddr::V4(addr) => {
                addr.is_private()
                    || addr.is_link_local()
                    || addr.octets()[0] == 100 && (addr.octets()[1] & 0xC0) == 64 // Carrier-grade NAT
            }
            IpAddr::V6(addr) => {
                // Check for link-local, unique local, etc.
                let segments = addr.segments();
                // Link-local (fe80::/10)
                (segments[0] & 0xffc0) == 0xfe80
                // Unique local (fc00::/7)
                || (segments[0] & 0xfe00) == 0xfc00
            }
        }
    }

    /// Extract real client IP from headers
    pub fn extract_client_ip(
        &self,
        headers: &HashMap<String, String>,
        connection_ip: &IpAddr,
    ) -> IpAddr {
        // Check trusted headers in order
        for header in &self.config.trusted_headers {
            if let Some(value) = headers.get(&header.to_lowercase()) {
                // Parse X-Forwarded-For (may have multiple IPs)
                if header.to_lowercase() == "x-forwarded-for" {
                    let ips: Vec<&str> = value.split(',').map(|s| s.trim()).collect();
                    // Take the first N IPs based on max_forwarded_hops
                    if let Some(ip_str) =
                        ips.get(ips.len().saturating_sub(self.config.max_forwarded_hops))
                    {
                        if let Ok(ip) = IpAddr::from_str(ip_str) {
                            return ip;
                        }
                    }
                } else {
                    // Single IP header
                    if let Ok(ip) = IpAddr::from_str(value.trim()) {
                        return ip;
                    }
                }
            }
        }

        // Fall back to connection IP
        *connection_ip
    }

    /// Get statistics
    pub fn stats(&self) -> IpWhitelistStats {
        let tenant_allowlists = self.tenant_allowlists.read();
        let tenant_blocklists = self.tenant_blocklists.read();

        IpWhitelistStats {
            enabled: self.config.enabled,
            global_allowlist_size: self.allowlist.len(),
            global_blocklist_size: self.blocklist.len(),
            tenant_count: tenant_allowlists
                .keys()
                .chain(tenant_blocklists.keys())
                .collect::<std::collections::HashSet<_>>()
                .len(),
        }
    }
}

/// IP whitelist statistics
#[derive(Debug, Clone, Serialize)]
pub struct IpWhitelistStats {
    pub enabled: bool,
    pub global_allowlist_size: usize,
    pub global_blocklist_size: usize,
    pub tenant_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_range_parse_single_ipv4() {
        let range = IpRange::parse("192.168.1.1").unwrap();
        assert!(range.contains(&IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))));
        assert!(!range.contains(&IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2))));
    }

    #[test]
    fn test_ip_range_parse_cidr_ipv4() {
        let range = IpRange::parse("192.168.1.0/24").unwrap();
        assert!(range.contains(&IpAddr::V4(Ipv4Addr::new(192, 168, 1, 0))));
        assert!(range.contains(&IpAddr::V4(Ipv4Addr::new(192, 168, 1, 255))));
        assert!(!range.contains(&IpAddr::V4(Ipv4Addr::new(192, 168, 2, 1))));
    }

    #[test]
    fn test_ip_range_parse_single_ipv6() {
        let range = IpRange::parse("::1").unwrap();
        assert!(range.contains(&IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1))));
        assert!(!range.contains(&IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 2))));
    }

    #[test]
    fn test_ip_range_parse_cidr_ipv6() {
        let range = IpRange::parse("2001:db8::/32").unwrap();
        assert!(range.contains(&IpAddr::V6(Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 1))));
        assert!(range.contains(&IpAddr::V6(Ipv6Addr::new(
            0x2001, 0x0db8, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff
        ))));
        assert!(!range.contains(&IpAddr::V6(Ipv6Addr::new(0x2001, 0x0db9, 0, 0, 0, 0, 0, 1))));
    }

    #[test]
    fn test_ip_whitelist_allow_localhost() {
        let config = IpWhitelistConfig {
            enabled: true,
            allow_localhost: true,
            ..Default::default()
        };
        let whitelist = IpWhitelist::new(config).unwrap();

        let result = whitelist.check_access(&IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        assert_eq!(result, IpAccessResult::AllowedLocalhost);
    }

    #[test]
    fn test_ip_whitelist_deny_localhost() {
        let config = IpWhitelistConfig {
            enabled: true,
            allow_localhost: false,
            ..Default::default()
        };
        let whitelist = IpWhitelist::new(config).unwrap();

        let result = whitelist.check_access(&IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        assert!(matches!(result, IpAccessResult::Denied(_)));
    }

    #[test]
    fn test_ip_whitelist_blocklist() {
        let config = IpWhitelistConfig {
            enabled: true,
            global_blocklist: vec!["1.2.3.4".to_string()],
            ..Default::default()
        };
        let whitelist = IpWhitelist::new(config).unwrap();

        let result = whitelist.check_access(&IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)));
        assert!(matches!(result, IpAccessResult::Denied(_)));
    }

    #[test]
    fn test_ip_whitelist_allowlist_deny_default() {
        let config = IpWhitelistConfig {
            enabled: true,
            default_policy: IpPolicy::Deny,
            global_allowlist: vec!["203.0.113.0/24".to_string()], // TEST-NET-3 (public)
            allow_private: false,
            allow_localhost: false,
            ..Default::default()
        };
        let whitelist = IpWhitelist::new(config).unwrap();

        // Allowed IP (in allowlist, public IP)
        let result = whitelist.check_access(&IpAddr::V4(Ipv4Addr::new(203, 0, 113, 50)));
        assert!(result.is_allowed());

        // Not in allowlist (public IP, should be denied)
        let result = whitelist.check_access(&IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)));
        assert!(matches!(result, IpAccessResult::Denied(_)));
    }

    #[test]
    fn test_ip_whitelist_tenant_rules() {
        let config = IpWhitelistConfig {
            enabled: true,
            ..Default::default()
        };
        let whitelist = IpWhitelist::new(config).unwrap();

        // Set tenant-specific blocklist
        whitelist
            .set_tenant_blocklist("tenant1", vec!["8.8.8.8".to_string()])
            .unwrap();

        // IP blocked for tenant1
        let result =
            whitelist.check_access_for_tenant(&IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), "tenant1");
        assert!(matches!(result, IpAccessResult::Denied(_)));

        // Same IP allowed for tenant2
        let result =
            whitelist.check_access_for_tenant(&IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), "tenant2");
        assert!(result.is_allowed());
    }

    #[test]
    fn test_extract_client_ip() {
        let config = IpWhitelistConfig::default();
        let whitelist = IpWhitelist::new(config).unwrap();

        let connection_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));

        // No headers, use connection IP
        let headers = HashMap::new();
        assert_eq!(
            whitelist.extract_client_ip(&headers, &connection_ip),
            connection_ip
        );

        // X-Forwarded-For header
        let mut headers = HashMap::new();
        headers.insert("x-forwarded-for".to_string(), "8.8.8.8".to_string());
        assert_eq!(
            whitelist.extract_client_ip(&headers, &connection_ip),
            IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))
        );

        // X-Forwarded-For with multiple IPs
        let mut headers = HashMap::new();
        headers.insert(
            "x-forwarded-for".to_string(),
            "1.1.1.1, 2.2.2.2, 3.3.3.3".to_string(),
        );
        assert_eq!(
            whitelist.extract_client_ip(&headers, &connection_ip),
            IpAddr::V4(Ipv4Addr::new(3, 3, 3, 3))
        );
    }
}
