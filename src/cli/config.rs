//! CLI configuration management
//!
//! Handles loading, saving, and validation of CLI configuration files

use super::CliConfig;
use crate::error::{Result, VectorizerError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
                jwt_secret: "vectorizer-default-secret-key-change-in-production".to_string(),
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

        // Validate authentication configuration
        if config.auth.jwt_secret.len() < 32 {
            return Err(VectorizerError::InvalidConfiguration {
                message: "JWT secret must be at least 32 characters long".to_string(),
            });
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
    use super::*;
    use tempfile::tempdir;

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

        // Valid config should pass
        assert!(ConfigManager::validate_config(&config).is_ok());

        // Invalid port should fail
        config.server.port = 0;
        assert!(ConfigManager::validate_config(&config).is_err());

        // Reset port
        config.server.port = 15002;

        // Invalid JWT secret should fail
        config.auth.jwt_secret = "short".to_string();
        assert!(ConfigManager::validate_config(&config).is_err());

        // Reset JWT secret
        config.auth.jwt_secret = "vectorizer-default-secret-key-change-in-production".to_string();

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

        // Generate default config file
        ConfigManager::generate_default_file(&config_path).unwrap();
        assert!(config_path.exists());

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
}
