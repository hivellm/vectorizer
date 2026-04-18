//! CLI configuration management
//!
//! Handles loading, saving, and validation of CLI configuration files

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::CliConfig;
use crate::error::{Result, VectorizerError};

/// Configuration file format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    /// Server configuration
    pub server: ServerConfigFile,
    /// Authentication configuration
    pub auth: AuthConfigFile,
    /// Database configuration
    pub database: DatabaseConfigFile,
    /// Logging configuration
    pub logging: LoggingConfigFile,
}

/// Server configuration in file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfigFile {
    /// Host address
    pub host: String,
    /// Port number
    pub port: u16,
    /// Data directory
    pub data_dir: String,
    /// Enable authentication
    pub auth_enabled: bool,
}

/// Authentication configuration in file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfigFile {
    /// JWT secret key
    pub jwt_secret: String,
    /// JWT expiration time in seconds
    pub jwt_expiration: u64,
    /// API key length
    pub api_key_length: usize,
    /// Rate limit per minute
    pub rate_limit_per_minute: u32,
    /// Rate limit per hour
    pub rate_limit_per_hour: u32,
    /// Enable authentication
    pub enabled: bool,
}

/// Database configuration in file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfigFile {
    /// Persistence path
    pub persistence_path: String,
    /// Compression enabled
    pub compression_enabled: bool,
    /// Compression threshold in bytes
    pub compression_threshold: usize,
}

/// Logging configuration in file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfigFile {
    /// Log level
    pub level: String,
    /// Log to file
    pub log_to_file: bool,
    /// Log file path (optional)
    pub log_file: Option<String>,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            server: ServerConfigFile {
                host: "127.0.0.1".to_string(),
                port: 15002,
                data_dir: "./data".to_string(),
                auth_enabled: true,
            },
            auth: AuthConfigFile {
                // Intentionally empty — callers must populate with a real
                // secret before the config is accepted by `validate_config`.
                // See `crate::auth::LEGACY_INSECURE_DEFAULT_SECRET` for the
                // historical value rejected by validation.
                jwt_secret: String::new(),
                jwt_expiration: 3600,
                api_key_length: 32,
                rate_limit_per_minute: 100,
                rate_limit_per_hour: 1000,
                enabled: true,
            },
            database: DatabaseConfigFile {
                persistence_path: "./data".to_string(),
                compression_enabled: true,
                compression_threshold: 1024,
            },
            logging: LoggingConfigFile {
                level: "info".to_string(),
                log_to_file: false,
                log_file: None,
            },
        }
    }
}

/// Configuration manager
pub struct ConfigManager;

impl ConfigManager {
    /// Load configuration from file
    pub fn load_from_file(path: &PathBuf) -> Result<CliConfig> {
        if !path.exists() {
            return Err(VectorizerError::NotFound(format!(
                "Configuration file not found: {:?}",
                path
            )));
        }

        let content = std::fs::read_to_string(path).map_err(|e| VectorizerError::IoError(e))?;

        let config_file: ConfigFile =
            serde_yaml::from_str(&content).map_err(|e| VectorizerError::YamlError(e))?;

        Self::validate_config(&config_file)?;
        Ok(Self::convert_to_cli_config(config_file))
    }

    /// Save configuration to file
    pub fn save_to_file(config: &CliConfig, path: &PathBuf) -> Result<()> {
        let config_file = Self::convert_to_config_file(config);
        let yaml =
            serde_yaml::to_string(&config_file).map_err(|e| VectorizerError::YamlError(e))?;

        std::fs::write(path, yaml).map_err(|e| VectorizerError::IoError(e))?;

        Ok(())
    }

    /// Generate default configuration file
    pub fn generate_default_file(path: &PathBuf) -> Result<()> {
        let default_config = ConfigFile::default();
        let yaml =
            serde_yaml::to_string(&default_config).map_err(|e| VectorizerError::YamlError(e))?;

        std::fs::write(path, yaml).map_err(|e| VectorizerError::IoError(e))?;

        Ok(())
    }

    /// Validate configuration
    pub fn validate_config(config: &ConfigFile) -> Result<()> {
        // Validate server configuration
        if config.server.port == 0 {
            return Err(VectorizerError::InvalidConfiguration {
                message: "Server port cannot be 0".to_string(),
            });
        }

        if config.server.host.is_empty() {
            return Err(VectorizerError::InvalidConfiguration {
                message: "Server host cannot be empty".to_string(),
            });
        }

        // Validate authentication configuration.
        // Only when auth is enabled — a dev-mode config may ship with no
        // secret and that is acceptable because no JWT is ever issued.
        if config.auth.enabled {
            if config.auth.jwt_secret.is_empty() {
                return Err(VectorizerError::InvalidConfiguration {
                    message: "auth.jwt_secret is empty. Set it in the config file \
                              or via the VECTORIZER_JWT_SECRET env var. Generate \
                              with: openssl rand -hex 64"
                        .to_string(),
                });
            }
            if config.auth.jwt_secret == crate::auth::LEGACY_INSECURE_DEFAULT_SECRET {
                return Err(VectorizerError::InvalidConfiguration {
                    message: "auth.jwt_secret is the legacy insecure default and \
                              will be rejected at boot. Generate a new secret: \
                              openssl rand -hex 64"
                        .to_string(),
                });
            }
            if config.auth.jwt_secret.len() < crate::auth::MIN_JWT_SECRET_LEN {
                return Err(VectorizerError::InvalidConfiguration {
                    message: format!(
                        "JWT secret must be at least {} characters long",
                        crate::auth::MIN_JWT_SECRET_LEN
                    ),
                });
            }
        }

        if config.auth.api_key_length < 16 {
            return Err(VectorizerError::InvalidConfiguration {
                message: "API key length must be at least 16 characters".to_string(),
            });
        }

        if config.auth.jwt_expiration == 0 {
            return Err(VectorizerError::InvalidConfiguration {
                message: "JWT expiration cannot be 0".to_string(),
            });
        }

        if config.auth.rate_limit_per_minute == 0 {
            return Err(VectorizerError::InvalidConfiguration {
                message: "Rate limit per minute cannot be 0".to_string(),
            });
        }

        if config.auth.rate_limit_per_hour == 0 {
            return Err(VectorizerError::InvalidConfiguration {
                message: "Rate limit per hour cannot be 0".to_string(),
            });
        }

        // Validate database configuration
        if config.database.compression_threshold == 0 {
            return Err(VectorizerError::InvalidConfiguration {
                message: "Compression threshold cannot be 0".to_string(),
            });
        }

        // Validate logging configuration
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&config.logging.level.to_lowercase().as_str()) {
            return Err(VectorizerError::InvalidConfiguration {
                message: format!(
                    "Invalid log level: {}. Valid levels: {:?}",
                    config.logging.level, valid_levels
                ),
            });
        }

        Ok(())
    }

    /// Convert ConfigFile to CliConfig
    fn convert_to_cli_config(config_file: ConfigFile) -> CliConfig {
        CliConfig {
            server: super::ServerConfig {
                host: config_file.server.host,
                port: config_file.server.port,
                data_dir: PathBuf::from(config_file.server.data_dir),
                auth_enabled: config_file.server.auth_enabled,
            },
            auth: crate::auth::AuthConfig {
                jwt_secret: config_file.auth.jwt_secret,
                jwt_expiration: config_file.auth.jwt_expiration,
                api_key_length: config_file.auth.api_key_length,
                rate_limit_per_minute: config_file.auth.rate_limit_per_minute,
                rate_limit_per_hour: config_file.auth.rate_limit_per_hour,
                enabled: config_file.auth.enabled,
            },
            database: super::DatabaseConfig {
                persistence_path: PathBuf::from(config_file.database.persistence_path),
                compression_enabled: config_file.database.compression_enabled,
                compression_threshold: config_file.database.compression_threshold,
            },
            logging: super::LoggingConfig {
                level: config_file.logging.level,
                log_to_file: config_file.logging.log_to_file,
                log_file: config_file.logging.log_file.map(PathBuf::from),
            },
            storage: crate::storage::StorageConfig::default(),
        }
    }

    /// Convert CliConfig to ConfigFile
    fn convert_to_config_file(config: &CliConfig) -> ConfigFile {
        ConfigFile {
            server: ServerConfigFile {
                host: config.server.host.clone(),
                port: config.server.port,
                data_dir: config.server.data_dir.to_string_lossy().to_string(),
                auth_enabled: config.server.auth_enabled,
            },
            auth: AuthConfigFile {
                jwt_secret: config.auth.jwt_secret.clone(),
                jwt_expiration: config.auth.jwt_expiration,
                api_key_length: config.auth.api_key_length,
                rate_limit_per_minute: config.auth.rate_limit_per_minute,
                rate_limit_per_hour: config.auth.rate_limit_per_hour,
                enabled: config.auth.enabled,
            },
            database: DatabaseConfigFile {
                persistence_path: config
                    .database
                    .persistence_path
                    .to_string_lossy()
                    .to_string(),
                compression_enabled: config.database.compression_enabled,
                compression_threshold: config.database.compression_threshold,
            },
            logging: LoggingConfigFile {
                level: config.logging.level.clone(),
                log_to_file: config.logging.log_to_file,
                log_file: config
                    .logging
                    .log_file
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_config_file_default() {
        let config = ConfigFile::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 15002);
        assert!(config.auth.enabled);
        assert!(config.database.compression_enabled);
    }

    #[test]
    fn test_config_validation() {
        let mut config = ConfigFile::default();

        // Default config has auth.enabled=true with empty secret — must FAIL.
        assert!(ConfigManager::validate_config(&config).is_err());

        // Injecting a valid 64-char secret makes it pass.
        config.auth.jwt_secret = "v".repeat(64);
        assert!(ConfigManager::validate_config(&config).is_ok());

        // Invalid port should fail
        config.server.port = 0;
        assert!(ConfigManager::validate_config(&config).is_err());
        config.server.port = 15002;

        // Short JWT secret should fail
        config.auth.jwt_secret = "short".to_string();
        assert!(ConfigManager::validate_config(&config).is_err());

        // Legacy insecure default must be explicitly rejected (not merely
        // because it happens to satisfy the length check).
        config.auth.jwt_secret = crate::auth::LEGACY_INSECURE_DEFAULT_SECRET.to_string();
        assert!(ConfigManager::validate_config(&config).is_err());

        // Reset to a valid secret to isolate the log-level assertion below.
        config.auth.jwt_secret = "v".repeat(64);

        // Invalid log level should fail
        config.logging.level = "invalid".to_string();
        assert!(ConfigManager::validate_config(&config).is_err());
    }

    #[test]
    fn test_config_conversion() {
        let config_file = ConfigFile::default();
        let cli_config = ConfigManager::convert_to_cli_config(config_file.clone());

        assert_eq!(cli_config.server.host, config_file.server.host);
        assert_eq!(cli_config.server.port, config_file.server.port);
        assert_eq!(cli_config.auth.jwt_secret, config_file.auth.jwt_secret);
        assert_eq!(
            cli_config.database.compression_enabled,
            config_file.database.compression_enabled
        );
    }

    #[test]
    fn test_config_file_operations() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.yml");

        // Generate default config file (ships with empty jwt_secret — intentional,
        // forces explicit configuration). We inject a valid secret on-disk before
        // load_from_file because the post-v3.0.0 validator rejects empty secrets
        // with auth.enabled=true.
        ConfigManager::generate_default_file(&config_path).unwrap();
        assert!(config_path.exists());

        let mut default_file: ConfigFile =
            serde_yaml::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
        default_file.auth.jwt_secret = "o".repeat(64);
        std::fs::write(&config_path, serde_yaml::to_string(&default_file).unwrap()).unwrap();

        // Load config file
        let config = ConfigManager::load_from_file(&config_path).unwrap();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 15002);

        // Modify and save config
        let mut modified_config = config;
        modified_config.server.port = 8080;
        ConfigManager::save_to_file(&modified_config, &config_path).unwrap();

        // Reload and verify
        let reloaded_config = ConfigManager::load_from_file(&config_path).unwrap();
        assert_eq!(reloaded_config.server.port, 8080);
    }

    #[test]
    fn test_default_config_file_fails_validate_until_secret_injected() {
        // Regression guard for the phase1_fix-jwt-default-secret posture:
        // `generate_default_file` produces a TEMPLATE, not a ready-to-boot
        // config. Loading it unchanged must fail validation so operators
        // are forced to set a real secret.
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.yml");

        ConfigManager::generate_default_file(&config_path).unwrap();
        let load = ConfigManager::load_from_file(&config_path);
        assert!(
            load.is_err(),
            "expected freshly-generated default config to fail validation"
        );
    }
}
