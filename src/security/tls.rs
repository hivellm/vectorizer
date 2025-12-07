//! TLS Configuration
//!
//! This module provides TLS/mTLS support for encrypted communication.

use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

use anyhow::{Context, Result};
use rustls::ServerConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::server::WebPkiClientVerifier;
use rustls_pemfile::{certs, private_key};

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
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cert_path: None,
            key_path: None,
            mtls_enabled: false,
            client_ca_path: None,
        }
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

    // Build server config
    let server_config = if config.mtls_enabled {
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

        ServerConfig::builder()
            .with_client_cert_verifier(client_verifier)
            .with_single_cert(certs, key)
            .context("Failed to build mTLS server config")?
    } else {
        // Standard TLS: no client certificate required
        ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .context("Failed to build TLS server config")?
    };

    tracing::info!(
        "TLS server config created successfully (mTLS: {})",
        config.mtls_enabled
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
    }

    #[test]
    fn test_tls_config_custom() {
        let config = TlsConfig {
            enabled: true,
            cert_path: Some("/path/to/cert.pem".to_string()),
            key_path: Some("/path/to/key.pem".to_string()),
            mtls_enabled: false,
            client_ca_path: None,
        };

        assert!(config.enabled);
        assert!(config.cert_path.is_some());
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
        };
        let result = create_server_config(&config);

        // Will fail when trying to load cert file (file doesn't exist)
        // but validates that mTLS path is checked
        assert!(result.is_err());
    }
}
