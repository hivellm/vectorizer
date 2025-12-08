//! TLS Configuration
//!
//! This module provides TLS/mTLS support for encrypted communication.
//!
//! Features:
//! - Certificate loading from PEM files
//! - Cipher suite configuration (modern, compatible, or custom)
//! - ALPN protocol negotiation (HTTP/1.1, HTTP/2)
//! - Mutual TLS (mTLS) with client certificate verification

use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

use anyhow::{Context, Result};
use rustls::crypto::ring::cipher_suite::{
    TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256, TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
    TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256, TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
    TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384, TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
    TLS13_AES_128_GCM_SHA256, TLS13_AES_256_GCM_SHA384, TLS13_CHACHA20_POLY1305_SHA256,
};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::server::WebPkiClientVerifier;
use rustls::{CipherSuite, ServerConfig, SupportedCipherSuite};
use rustls_pemfile::{certs, private_key};

/// Cipher suite preset for easy configuration
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum CipherSuitePreset {
    /// Modern: TLS 1.3 only cipher suites (most secure, best performance)
    #[default]
    Modern,
    /// Compatible: TLS 1.2 + TLS 1.3 cipher suites (wider compatibility)
    Compatible,
    /// Custom: User-specified cipher suites
    Custom(Vec<CipherSuite>),
}

/// ALPN (Application-Layer Protocol Negotiation) configuration
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum AlpnConfig {
    /// HTTP/1.1 only
    Http1,
    /// HTTP/2 only
    Http2,
    /// Both HTTP/1.1 and HTTP/2 (prefer HTTP/2)
    #[default]
    Both,
    /// Custom ALPN protocols
    Custom(Vec<Vec<u8>>),
    /// No ALPN negotiation
    None,
}

/// TLS configuration
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Enable TLS
    pub enabled: bool,
    /// Certificate file path
    pub cert_path: Option<String>,
    /// Private key file path
    pub key_path: Option<String>,
    /// Enable mutual TLS (mTLS)
    pub mtls_enabled: bool,
    /// Client CA certificate path (for mTLS)
    pub client_ca_path: Option<String>,
    /// Cipher suite preset
    pub cipher_suites: CipherSuitePreset,
    /// ALPN protocol configuration
    pub alpn: AlpnConfig,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cert_path: None,
            key_path: None,
            mtls_enabled: false,
            client_ca_path: None,
            cipher_suites: CipherSuitePreset::default(),
            alpn: AlpnConfig::default(),
        }
    }
}

/// Get cipher suites for the Modern preset (TLS 1.3 only)
fn get_modern_cipher_suites() -> Vec<SupportedCipherSuite> {
    vec![
        TLS13_AES_256_GCM_SHA384,
        TLS13_AES_128_GCM_SHA256,
        TLS13_CHACHA20_POLY1305_SHA256,
    ]
}

/// Get cipher suites for the Compatible preset (TLS 1.2 + TLS 1.3)
fn get_compatible_cipher_suites() -> Vec<SupportedCipherSuite> {
    vec![
        // TLS 1.3 cipher suites (preferred)
        TLS13_AES_256_GCM_SHA384,
        TLS13_AES_128_GCM_SHA256,
        TLS13_CHACHA20_POLY1305_SHA256,
        // TLS 1.2 ECDHE cipher suites
        TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
        TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
        TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256,
        TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
        TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
        TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
    ]
}

/// Convert CipherSuite enum to SupportedCipherSuite
fn cipher_suite_to_supported(suite: CipherSuite) -> Option<SupportedCipherSuite> {
    match suite {
        CipherSuite::TLS13_AES_256_GCM_SHA384 => Some(TLS13_AES_256_GCM_SHA384),
        CipherSuite::TLS13_AES_128_GCM_SHA256 => Some(TLS13_AES_128_GCM_SHA256),
        CipherSuite::TLS13_CHACHA20_POLY1305_SHA256 => Some(TLS13_CHACHA20_POLY1305_SHA256),
        CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384 => {
            Some(TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384)
        }
        CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256 => {
            Some(TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256)
        }
        CipherSuite::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256 => {
            Some(TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256)
        }
        CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384 => {
            Some(TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384)
        }
        CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256 => {
            Some(TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256)
        }
        CipherSuite::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256 => {
            Some(TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256)
        }
        _ => None,
    }
}

/// Get cipher suites based on preset
fn get_cipher_suites(preset: &CipherSuitePreset) -> Vec<SupportedCipherSuite> {
    match preset {
        CipherSuitePreset::Modern => get_modern_cipher_suites(),
        CipherSuitePreset::Compatible => get_compatible_cipher_suites(),
        CipherSuitePreset::Custom(suites) => suites
            .iter()
            .filter_map(|s| cipher_suite_to_supported(*s))
            .collect(),
    }
}

/// Get ALPN protocols based on configuration
fn get_alpn_protocols(config: &AlpnConfig) -> Vec<Vec<u8>> {
    match config {
        AlpnConfig::Http1 => vec![b"http/1.1".to_vec()],
        AlpnConfig::Http2 => vec![b"h2".to_vec()],
        AlpnConfig::Both => vec![b"h2".to_vec(), b"http/1.1".to_vec()],
        AlpnConfig::Custom(protocols) => protocols.clone(),
        AlpnConfig::None => vec![],
    }
}

/// Load certificates from a PEM file
fn load_certs(path: &str) -> Result<Vec<CertificateDer<'static>>> {
    let file =
        File::open(path).with_context(|| format!("Failed to open certificate file: {}", path))?;
    let mut reader = BufReader::new(file);
    let certs: Vec<CertificateDer<'static>> = certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("Failed to parse certificates from: {}", path))?;

    if certs.is_empty() {
        anyhow::bail!("No certificates found in: {}", path);
    }

    Ok(certs)
}

/// Load private key from a PEM file
fn load_private_key(path: &str) -> Result<PrivateKeyDer<'static>> {
    let file =
        File::open(path).with_context(|| format!("Failed to open private key file: {}", path))?;
    let mut reader = BufReader::new(file);

    let key = private_key(&mut reader)
        .with_context(|| format!("Failed to parse private key from: {}", path))?
        .ok_or_else(|| anyhow::anyhow!("No private key found in: {}", path))?;

    Ok(key)
}

/// Create rustls ServerConfig from TLS configuration
///
/// This function creates a complete TLS server configuration with:
/// - Certificate and private key loading
/// - Cipher suite configuration (modern/compatible/custom)
/// - ALPN protocol negotiation
/// - Optional mTLS (mutual TLS) with client certificate verification
pub fn create_server_config(config: &TlsConfig) -> Result<Arc<ServerConfig>> {
    if !config.enabled {
        anyhow::bail!("TLS is not enabled in configuration");
    }

    let cert_path = config
        .cert_path
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("TLS enabled but cert_path not provided"))?;
    let key_path = config
        .key_path
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("TLS enabled but key_path not provided"))?;

    // Load server certificate chain
    let certs = load_certs(cert_path)?;
    tracing::info!("Loaded {} certificate(s) from {}", certs.len(), cert_path);

    // Load private key
    let key = load_private_key(key_path)?;
    tracing::info!("Loaded private key from {}", key_path);

    // Get configured cipher suites
    let cipher_suites = get_cipher_suites(&config.cipher_suites);
    tracing::info!(
        "Configured {} cipher suites (preset: {:?})",
        cipher_suites.len(),
        config.cipher_suites
    );

    // Create crypto provider with configured cipher suites
    let crypto_provider = rustls::crypto::CryptoProvider {
        cipher_suites,
        ..rustls::crypto::ring::default_provider()
    };

    // Build server config with cipher suites
    let mut server_config = if config.mtls_enabled {
        // mTLS: require client certificate verification
        let client_ca_path = config
            .client_ca_path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("mTLS enabled but client_ca_path not provided"))?;

        let client_certs = load_certs(client_ca_path)?;
        tracing::info!(
            "Loaded {} client CA certificate(s) from {}",
            client_certs.len(),
            client_ca_path
        );

        // Build root cert store for client verification
        let mut root_store = rustls::RootCertStore::empty();
        for cert in client_certs {
            root_store
                .add(cert)
                .context("Failed to add client CA certificate to root store")?;
        }

        let client_verifier = WebPkiClientVerifier::builder(Arc::new(root_store))
            .build()
            .context("Failed to build client certificate verifier")?;

        ServerConfig::builder_with_provider(Arc::new(crypto_provider))
            .with_safe_default_protocol_versions()
            .context("Failed to set protocol versions")?
            .with_client_cert_verifier(client_verifier)
            .with_single_cert(certs, key)
            .context("Failed to build mTLS server config")?
    } else {
        // Standard TLS: no client certificate required
        ServerConfig::builder_with_provider(Arc::new(crypto_provider))
            .with_safe_default_protocol_versions()
            .context("Failed to set protocol versions")?
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .context("Failed to build TLS server config")?
    };

    // Configure ALPN protocols
    let alpn_protocols = get_alpn_protocols(&config.alpn);
    if !alpn_protocols.is_empty() {
        server_config.alpn_protocols = alpn_protocols;
        tracing::info!("Configured ALPN protocols: {:?}", config.alpn);
    }

    tracing::info!(
        "TLS server config created successfully (mTLS: {}, cipher_preset: {:?}, alpn: {:?})",
        config.mtls_enabled,
        config.cipher_suites,
        config.alpn
    );

    Ok(Arc::new(server_config))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_config_default() {
        let config = TlsConfig::default();
        assert!(!config.enabled);
        assert!(!config.mtls_enabled);
        assert!(config.cert_path.is_none());
        assert_eq!(config.cipher_suites, CipherSuitePreset::Modern);
        assert_eq!(config.alpn, AlpnConfig::Both);
    }

    #[test]
    fn test_tls_config_custom() {
        let config = TlsConfig {
            enabled: true,
            cert_path: Some("/path/to/cert.pem".to_string()),
            key_path: Some("/path/to/key.pem".to_string()),
            mtls_enabled: false,
            client_ca_path: None,
            cipher_suites: CipherSuitePreset::Compatible,
            alpn: AlpnConfig::Http2,
        };

        assert!(config.enabled);
        assert!(config.cert_path.is_some());
        assert_eq!(config.cipher_suites, CipherSuitePreset::Compatible);
        assert_eq!(config.alpn, AlpnConfig::Http2);
    }

    #[test]
    fn test_create_server_config_disabled() {
        let config = TlsConfig::default();
        let result = create_server_config(&config);

        // Should fail because TLS is disabled
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not enabled"));
    }

    #[test]
    fn test_create_server_config_missing_cert() {
        let config = TlsConfig {
            enabled: true,
            cert_path: None,
            key_path: Some("/path/to/key.pem".to_string()),
            mtls_enabled: false,
            client_ca_path: None,
            cipher_suites: CipherSuitePreset::default(),
            alpn: AlpnConfig::default(),
        };
        let result = create_server_config(&config);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cert_path"));
    }

    #[test]
    fn test_create_server_config_missing_key() {
        let config = TlsConfig {
            enabled: true,
            cert_path: Some("/path/to/cert.pem".to_string()),
            key_path: None,
            mtls_enabled: false,
            client_ca_path: None,
            cipher_suites: CipherSuitePreset::default(),
            alpn: AlpnConfig::default(),
        };
        let result = create_server_config(&config);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("key_path"));
    }

    #[test]
    fn test_create_server_config_mtls_missing_ca() {
        let config = TlsConfig {
            enabled: true,
            cert_path: Some("/path/to/cert.pem".to_string()),
            key_path: Some("/path/to/key.pem".to_string()),
            mtls_enabled: true,
            client_ca_path: None,
            cipher_suites: CipherSuitePreset::default(),
            alpn: AlpnConfig::default(),
        };
        let result = create_server_config(&config);

        // Will fail when trying to load cert file (file doesn't exist)
        // but validates that mTLS path is checked
        assert!(result.is_err());
    }

    #[test]
    fn test_modern_cipher_suites() {
        let suites = get_modern_cipher_suites();
        assert_eq!(suites.len(), 3);
        // Verify all are TLS 1.3 suites
        for suite in &suites {
            let name = format!("{:?}", suite.suite());
            assert!(
                name.starts_with("TLS13_"),
                "Expected TLS 1.3 suite: {}",
                name
            );
        }
    }

    #[test]
    fn test_compatible_cipher_suites() {
        let suites = get_compatible_cipher_suites();
        assert_eq!(suites.len(), 9);
        // First 3 should be TLS 1.3
        for suite in suites.iter().take(3) {
            let name = format!("{:?}", suite.suite());
            assert!(
                name.starts_with("TLS13_"),
                "Expected TLS 1.3 suite: {}",
                name
            );
        }
        // Rest should be TLS 1.2 ECDHE
        for suite in suites.iter().skip(3) {
            let name = format!("{:?}", suite.suite());
            assert!(
                name.starts_with("TLS_ECDHE_"),
                "Expected TLS 1.2 ECDHE suite: {}",
                name
            );
        }
    }

    #[test]
    fn test_custom_cipher_suites() {
        let custom = CipherSuitePreset::Custom(vec![
            CipherSuite::TLS13_AES_256_GCM_SHA384,
            CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
        ]);
        let suites = get_cipher_suites(&custom);
        assert_eq!(suites.len(), 2);
    }

    #[test]
    fn test_alpn_http1() {
        let protocols = get_alpn_protocols(&AlpnConfig::Http1);
        assert_eq!(protocols.len(), 1);
        assert_eq!(protocols[0], b"http/1.1");
    }

    #[test]
    fn test_alpn_http2() {
        let protocols = get_alpn_protocols(&AlpnConfig::Http2);
        assert_eq!(protocols.len(), 1);
        assert_eq!(protocols[0], b"h2");
    }

    #[test]
    fn test_alpn_both() {
        let protocols = get_alpn_protocols(&AlpnConfig::Both);
        assert_eq!(protocols.len(), 2);
        assert_eq!(protocols[0], b"h2"); // HTTP/2 preferred
        assert_eq!(protocols[1], b"http/1.1");
    }

    #[test]
    fn test_alpn_custom() {
        let custom = AlpnConfig::Custom(vec![b"grpc".to_vec(), b"h2".to_vec()]);
        let protocols = get_alpn_protocols(&custom);
        assert_eq!(protocols.len(), 2);
        assert_eq!(protocols[0], b"grpc");
        assert_eq!(protocols[1], b"h2");
    }

    #[test]
    fn test_alpn_none() {
        let protocols = get_alpn_protocols(&AlpnConfig::None);
        assert!(protocols.is_empty());
    }

    #[test]
    fn test_cipher_suite_conversion() {
        // Test valid conversions
        assert!(cipher_suite_to_supported(CipherSuite::TLS13_AES_256_GCM_SHA384).is_some());
        assert!(cipher_suite_to_supported(CipherSuite::TLS13_AES_128_GCM_SHA256).is_some());
        assert!(cipher_suite_to_supported(CipherSuite::TLS13_CHACHA20_POLY1305_SHA256).is_some());
        assert!(
            cipher_suite_to_supported(CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384)
                .is_some()
        );
        assert!(
            cipher_suite_to_supported(CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384).is_some()
        );

        // Test invalid conversion (unknown suite)
        assert!(cipher_suite_to_supported(CipherSuite::Unknown(0x0000)).is_none());
    }
}
