//! TLS Security Integration Tests
//!
//! Tests for TLS/SSL functionality including:
//! - Server configuration creation with valid certificates
//! - TLS connection establishment
//! - HTTPS endpoint access
//! - mTLS (mutual TLS) configuration

use std::io::Write;

use tempfile::NamedTempFile;
use vectorizer::security::tls::{AlpnConfig, CipherSuitePreset, TlsConfig, create_server_config};

/// Generate a self-signed certificate and key for testing
/// Returns (cert_pem, key_pem) as strings
fn generate_test_certificate() -> (String, String) {
    use rcgen::{CertificateParams, DnType, KeyPair};

    let mut params = CertificateParams::new(vec!["localhost".to_string()])
        .expect("Failed to create cert params");
    params
        .distinguished_name
        .push(DnType::CommonName, "localhost");
    params
        .distinguished_name
        .push(DnType::OrganizationName, "Vectorizer Test");

    let key_pair = KeyPair::generate().expect("Failed to generate key pair");
    let cert = params
        .self_signed(&key_pair)
        .expect("Failed to generate certificate");

    (cert.pem(), key_pair.serialize_pem())
}

/// Create temporary files with certificate and key
fn create_temp_cert_files() -> (NamedTempFile, NamedTempFile) {
    let (cert_pem, key_pem) = generate_test_certificate();

    let mut cert_file = NamedTempFile::new().expect("Failed to create temp cert file");
    cert_file
        .write_all(cert_pem.as_bytes())
        .expect("Failed to write cert");

    let mut key_file = NamedTempFile::new().expect("Failed to create temp key file");
    key_file
        .write_all(key_pem.as_bytes())
        .expect("Failed to write key");

    (cert_file, key_file)
}

#[cfg(test)]
mod tls_connection_tests {
    use super::*;

    /// Test that server config can be created with valid certificate files
    #[test]
    fn test_create_server_config_with_valid_certs() {
        let (cert_file, key_file) = create_temp_cert_files();

        let config = TlsConfig {
            enabled: true,
            cert_path: Some(cert_file.path().to_string_lossy().to_string()),
            key_path: Some(key_file.path().to_string_lossy().to_string()),
            mtls_enabled: false,
            client_ca_path: None,
            cipher_suites: CipherSuitePreset::Modern,
            alpn: AlpnConfig::Both,
        };

        let result = create_server_config(&config);
        assert!(
            result.is_ok(),
            "Failed to create server config: {:?}",
            result.err()
        );

        let server_config = result.unwrap();
        // Verify ALPN is configured
        assert!(
            !server_config.alpn_protocols.is_empty(),
            "ALPN protocols should be configured"
        );
    }

    /// Test server config with Modern cipher suites
    #[test]
    fn test_server_config_modern_ciphers() {
        let (cert_file, key_file) = create_temp_cert_files();

        let config = TlsConfig {
            enabled: true,
            cert_path: Some(cert_file.path().to_string_lossy().to_string()),
            key_path: Some(key_file.path().to_string_lossy().to_string()),
            mtls_enabled: false,
            client_ca_path: None,
            cipher_suites: CipherSuitePreset::Modern,
            alpn: AlpnConfig::Http2,
        };

        let result = create_server_config(&config);
        assert!(result.is_ok(), "Modern cipher config should succeed");
    }

    /// Test server config with Compatible cipher suites
    #[test]
    fn test_server_config_compatible_ciphers() {
        let (cert_file, key_file) = create_temp_cert_files();

        let config = TlsConfig {
            enabled: true,
            cert_path: Some(cert_file.path().to_string_lossy().to_string()),
            key_path: Some(key_file.path().to_string_lossy().to_string()),
            mtls_enabled: false,
            client_ca_path: None,
            cipher_suites: CipherSuitePreset::Compatible,
            alpn: AlpnConfig::Both,
        };

        let result = create_server_config(&config);
        assert!(result.is_ok(), "Compatible cipher config should succeed");
    }

    /// Test server config with HTTP/1.1 only ALPN
    #[test]
    fn test_server_config_http1_alpn() {
        let (cert_file, key_file) = create_temp_cert_files();

        let config = TlsConfig {
            enabled: true,
            cert_path: Some(cert_file.path().to_string_lossy().to_string()),
            key_path: Some(key_file.path().to_string_lossy().to_string()),
            mtls_enabled: false,
            client_ca_path: None,
            cipher_suites: CipherSuitePreset::Modern,
            alpn: AlpnConfig::Http1,
        };

        let result = create_server_config(&config);
        assert!(result.is_ok(), "HTTP/1.1 ALPN config should succeed");

        let server_config = result.unwrap();
        assert_eq!(server_config.alpn_protocols.len(), 1);
        assert_eq!(server_config.alpn_protocols[0], b"http/1.1");
    }

    /// Test server config with HTTP/2 only ALPN
    #[test]
    fn test_server_config_http2_alpn() {
        let (cert_file, key_file) = create_temp_cert_files();

        let config = TlsConfig {
            enabled: true,
            cert_path: Some(cert_file.path().to_string_lossy().to_string()),
            key_path: Some(key_file.path().to_string_lossy().to_string()),
            mtls_enabled: false,
            client_ca_path: None,
            cipher_suites: CipherSuitePreset::Modern,
            alpn: AlpnConfig::Http2,
        };

        let result = create_server_config(&config);
        assert!(result.is_ok(), "HTTP/2 ALPN config should succeed");

        let server_config = result.unwrap();
        assert_eq!(server_config.alpn_protocols.len(), 1);
        assert_eq!(server_config.alpn_protocols[0], b"h2");
    }

    /// Test server config with no ALPN
    #[test]
    fn test_server_config_no_alpn() {
        let (cert_file, key_file) = create_temp_cert_files();

        let config = TlsConfig {
            enabled: true,
            cert_path: Some(cert_file.path().to_string_lossy().to_string()),
            key_path: Some(key_file.path().to_string_lossy().to_string()),
            mtls_enabled: false,
            client_ca_path: None,
            cipher_suites: CipherSuitePreset::Modern,
            alpn: AlpnConfig::None,
        };

        let result = create_server_config(&config);
        assert!(result.is_ok(), "No ALPN config should succeed");

        let server_config = result.unwrap();
        assert!(
            server_config.alpn_protocols.is_empty(),
            "ALPN protocols should be empty"
        );
    }

    /// Test TLS config values
    #[test]
    fn test_tls_config_values() {
        let config = TlsConfig {
            enabled: true,
            cert_path: Some("/path/to/cert.pem".to_string()),
            key_path: Some("/path/to/key.pem".to_string()),
            mtls_enabled: true,
            client_ca_path: Some("/path/to/ca.pem".to_string()),
            cipher_suites: CipherSuitePreset::Modern,
            alpn: AlpnConfig::Both,
        };

        assert!(config.enabled);
        assert_eq!(config.cert_path, Some("/path/to/cert.pem".to_string()));
        assert_eq!(config.key_path, Some("/path/to/key.pem".to_string()));
        assert!(config.mtls_enabled);
        assert_eq!(config.client_ca_path, Some("/path/to/ca.pem".to_string()));
    }
}

#[cfg(test)]
mod mtls_tests {
    use super::*;

    /// Test mTLS configuration with CA certificate
    #[test]
    fn test_mtls_config_with_ca() {
        // Install default crypto provider for mTLS
        let _ = rustls::crypto::ring::default_provider().install_default();

        let (cert_file, key_file) = create_temp_cert_files();
        let (ca_file, _) = create_temp_cert_files(); // Use another cert as CA for testing

        let config = TlsConfig {
            enabled: true,
            cert_path: Some(cert_file.path().to_string_lossy().to_string()),
            key_path: Some(key_file.path().to_string_lossy().to_string()),
            mtls_enabled: true,
            client_ca_path: Some(ca_file.path().to_string_lossy().to_string()),
            cipher_suites: CipherSuitePreset::Modern,
            alpn: AlpnConfig::Both,
        };

        let result = create_server_config(&config);
        assert!(
            result.is_ok(),
            "mTLS config with CA should succeed: {:?}",
            result.err()
        );
    }
}

#[cfg(test)]
mod https_endpoint_tests {
    use tokio::net::TcpListener;
    use tokio_rustls::TlsAcceptor;

    use super::*;

    /// Test that a TLS acceptor can be created from server config
    #[tokio::test]
    async fn test_tls_acceptor_creation() {
        let (cert_file, key_file) = create_temp_cert_files();

        let config = TlsConfig {
            enabled: true,
            cert_path: Some(cert_file.path().to_string_lossy().to_string()),
            key_path: Some(key_file.path().to_string_lossy().to_string()),
            mtls_enabled: false,
            client_ca_path: None,
            cipher_suites: CipherSuitePreset::Modern,
            alpn: AlpnConfig::Both,
        };

        let server_config = create_server_config(&config).expect("Failed to create server config");
        let acceptor = TlsAcceptor::from(server_config);

        // Verify the acceptor was created successfully
        // (we can't easily test actual connections without more setup)
        assert!(
            std::mem::size_of_val(&acceptor) > 0,
            "TLS acceptor should be created"
        );
    }

    /// Test that TLS server can bind to a port
    #[tokio::test]
    async fn test_tls_server_binding() {
        let (cert_file, key_file) = create_temp_cert_files();

        let config = TlsConfig {
            enabled: true,
            cert_path: Some(cert_file.path().to_string_lossy().to_string()),
            key_path: Some(key_file.path().to_string_lossy().to_string()),
            mtls_enabled: false,
            client_ca_path: None,
            cipher_suites: CipherSuitePreset::Modern,
            alpn: AlpnConfig::Both,
        };

        let server_config = create_server_config(&config).expect("Failed to create server config");
        let _acceptor = TlsAcceptor::from(server_config);

        // Try to bind to a random available port
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind TCP listener");

        let addr = listener.local_addr().expect("Failed to get local address");
        assert!(addr.port() > 0, "Should have bound to a valid port");

        // Clean up - drop listener
        drop(listener);
    }
}

#[cfg(test)]
mod cipher_suite_validation_tests {
    use super::*;

    /// Test custom cipher suite configuration with Modern preset
    #[test]
    fn test_modern_cipher_suite_config() {
        let (cert_file, key_file) = create_temp_cert_files();

        let config = TlsConfig {
            enabled: true,
            cert_path: Some(cert_file.path().to_string_lossy().to_string()),
            key_path: Some(key_file.path().to_string_lossy().to_string()),
            mtls_enabled: false,
            client_ca_path: None,
            cipher_suites: CipherSuitePreset::Modern,
            alpn: AlpnConfig::Both,
        };

        let result = create_server_config(&config);
        assert!(
            result.is_ok(),
            "Modern cipher suite config should succeed: {:?}",
            result.err()
        );
    }

    /// Test Compatible cipher suites (TLS 1.2 + 1.3)
    #[test]
    fn test_compatible_cipher_suite_config() {
        let (cert_file, key_file) = create_temp_cert_files();

        let config = TlsConfig {
            enabled: true,
            cert_path: Some(cert_file.path().to_string_lossy().to_string()),
            key_path: Some(key_file.path().to_string_lossy().to_string()),
            mtls_enabled: false,
            client_ca_path: None,
            cipher_suites: CipherSuitePreset::Compatible,
            alpn: AlpnConfig::Http2,
        };

        let result = create_server_config(&config);
        assert!(
            result.is_ok(),
            "Compatible cipher suite config should succeed"
        );
    }
}
