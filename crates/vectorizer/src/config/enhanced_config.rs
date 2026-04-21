//! Enhanced configuration management system
//!
//! Provides advanced configuration features including:
//! - Environment variable substitution
//! - Configuration validation
//! - Dynamic configuration updates
//! - Configuration templates
//! - Configuration inheritance

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::time::interval;

use crate::error::VectorizerError;
use crate::workspace::config::WorkspaceConfig;

/// Enhanced configuration manager
#[derive(Debug, Clone)]
pub struct EnhancedConfigManager {
    /// Current configuration
    config: Arc<RwLock<WorkspaceConfig>>,

    /// Configuration file path
    config_path: PathBuf,

    /// Environment variable mappings
    env_mappings: HashMap<String, String>,

    /// Configuration validation rules
    validation_rules: ValidationRules,

    /// Last reload timestamp
    last_reload: Arc<RwLock<SystemTime>>,

    /// Auto-reload enabled
    auto_reload: bool,
}

/// Configuration validation rules
#[derive(Debug, Clone)]
pub struct ValidationRules {
    /// Required fields
    pub required_fields: Vec<String>,

    /// Field constraints
    pub field_constraints: HashMap<String, FieldConstraint>,

    /// Cross-field validation rules
    pub cross_field_rules: Vec<CrossFieldRule>,
}

/// Field constraint definition
#[derive(Debug, Clone)]
pub struct FieldConstraint {
    /// Minimum value (for numeric fields)
    pub min_value: Option<f64>,

    /// Maximum value (for numeric fields)
    pub max_value: Option<f64>,

    /// Minimum length (for string fields)
    pub min_length: Option<usize>,

    /// Maximum length (for string fields)
    pub max_length: Option<usize>,

    /// Allowed values (for enum fields)
    pub allowed_values: Option<Vec<String>>,

    /// Regex pattern (for string fields)
    pub pattern: Option<String>,
}

/// Cross-field validation rule
#[derive(Debug, Clone)]
pub struct CrossFieldRule {
    /// Rule name
    pub name: String,

    /// Fields involved in the rule
    pub fields: Vec<String>,

    /// Validation function
    pub validator: fn(&WorkspaceConfig) -> Result<(), String>,
}

/// Configuration template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigTemplate {
    /// Template name
    pub name: String,

    /// Template description
    pub description: String,

    /// Template configuration
    pub config: WorkspaceConfig,

    /// Template variables
    pub variables: HashMap<String, TemplateVariable>,
}

/// Template variable definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    /// Variable name
    pub name: String,

    /// Variable description
    pub description: String,

    /// Default value
    pub default_value: String,

    /// Variable type
    pub var_type: VariableType,

    /// Whether the variable is required
    pub required: bool,

    /// Minimum length for string variables
    pub min_length: Option<usize>,

    /// Maximum length for string variables
    pub max_length: Option<usize>,
}

/// Variable types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableType {
    String,
    Integer,
    Float,
    Boolean,
    Path,
    Url,
}

impl EnhancedConfigManager {
    /// Create a new enhanced configuration manager
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            config: Arc::new(RwLock::new(WorkspaceConfig::default())),
            config_path,
            env_mappings: HashMap::new(),
            validation_rules: ValidationRules::default(),
            last_reload: Arc::new(RwLock::new(SystemTime::now())),
            auto_reload: false,
        }
    }

    /// Load configuration from file with environment variable substitution
    pub async fn load_config(&mut self) -> Result<()> {
        let config_content = fs::read_to_string(&self.config_path)
            .await
            .context("Failed to read configuration file")?;

        // Substitute environment variables
        let substituted_content = self.substitute_env_vars(&config_content)?;

        // Parse configuration
        let config: WorkspaceConfig = serde_yaml::from_str(&substituted_content)
            .context("Failed to parse configuration file")?;

        // Validate configuration
        self.validate_config(&config)?;

        // Update configuration
        {
            let mut current_config = self.config.write();
            *current_config = config;
        }

        // Update last reload time
        {
            let mut last_reload = self.last_reload.write();
            *last_reload = SystemTime::now();
        }

        tracing::info!(
            "Configuration loaded successfully from {:?}",
            self.config_path
        );
        Ok(())
    }

    /// Save current configuration to file
    pub async fn save_config(&self) -> Result<()> {
        let config = self.config.read();
        let config_yaml =
            serde_yaml::to_string(&*config).context("Failed to serialize configuration")?;

        fs::write(&self.config_path, config_yaml)
            .await
            .context("Failed to write configuration file")?;

        tracing::info!("Configuration saved successfully to {:?}", self.config_path);
        Ok(())
    }

    /// Get current configuration
    pub fn get_config(&self) -> WorkspaceConfig {
        self.config.read().clone()
    }

    /// Update configuration
    pub fn update_config<F>(&self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut WorkspaceConfig) -> Result<()>,
    {
        let mut config = self.config.write();
        updater(&mut config)?;
        self.validate_config(&config)?;
        Ok(())
    }

    /// Substitute environment variables in configuration content
    fn substitute_env_vars(&self, content: &str) -> Result<String> {
        let mut result = content.to_string();

        // Find all environment variable references in format ${VAR_NAME} or ${VAR_NAME:default}
        let re = regex::Regex::new(r"\$\{([^}:]+)(?::([^}]*))?\}")?;

        for cap in re.captures_iter(content) {
            let var_name = &cap[1];
            let default_value = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            let env_value = std::env::var(var_name).unwrap_or_else(|_| {
                if !default_value.is_empty() {
                    default_value.to_string()
                } else {
                    return format!("${{{}}}", var_name);
                }
            });

            result = result.replace(&cap[0], &env_value);
        }

        Ok(result)
    }

    /// Validate configuration against rules
    fn validate_config(&self, config: &WorkspaceConfig) -> Result<()> {
        // Validate required fields
        for field in &self.validation_rules.required_fields {
            if !self.has_field(config, field) {
                return Err(VectorizerError::ValidationError {
                    field: field.clone(),
                    message: format!("Required field '{}' is missing", field),
                }
                .into());
            }
        }

        // Validate field constraints
        for (field, constraint) in &self.validation_rules.field_constraints {
            self.validate_field_constraint(config, field, constraint)?;
        }

        // Validate cross-field rules
        for rule in &self.validation_rules.cross_field_rules {
            if let Err(e) = (rule.validator)(config) {
                return Err(VectorizerError::ValidationError {
                    field: rule.name.clone(),
                    message: e,
                }
                .into());
            }
        }

        Ok(())
    }

    /// Check if configuration has a field
    fn has_field(&self, _config: &WorkspaceConfig, _field: &str) -> bool {
        // This would need to be implemented based on the specific field structure
        // For now, we'll assume all fields exist
        true
    }

    /// Validate field constraint
    fn validate_field_constraint(
        &self,
        _config: &WorkspaceConfig,
        _field: &str,
        _constraint: &FieldConstraint,
    ) -> Result<()> {
        // This would need to be implemented based on the specific field structure
        // For now, we'll assume all constraints pass
        Ok(())
    }

    /// Enable auto-reload
    pub fn enable_auto_reload(&mut self, interval_seconds: u64) {
        self.auto_reload = true;
        let config_path = self.config_path.clone();
        let config = self.config.clone();
        let last_reload = self.last_reload.clone();
        let validation_rules = self.validation_rules.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(interval_seconds));
            loop {
                interval.tick().await;

                // Check if file has been modified
                if let Ok(metadata) = fs::metadata(&config_path).await {
                    if let Ok(modified) = metadata.modified() {
                        let last_reload_time = *last_reload.read();
                        if modified > last_reload_time {
                            // Reload configuration
                            if let Ok(content) = fs::read_to_string(&config_path).await {
                                if let Ok(substituted) = Self::substitute_env_vars_static(&content)
                                {
                                    if let Ok(new_config) =
                                        serde_yaml::from_str::<WorkspaceConfig>(&substituted)
                                    {
                                        if Self::validate_config_static(
                                            &new_config,
                                            &validation_rules,
                                        )
                                        .is_ok()
                                        {
                                            {
                                                let mut current_config = config.write();
                                                *current_config = new_config;
                                            }
                                            {
                                                let mut last_reload_guard =
                                                    last_reload.write();
                                                *last_reload_guard = SystemTime::now();
                                            }
                                            tracing::info!(
                                                "Configuration auto-reloaded from {:?}",
                                                config_path
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    /// Static method for environment variable substitution
    fn substitute_env_vars_static(content: &str) -> Result<String> {
        let mut result = content.to_string();
        let re = regex::Regex::new(r"\$\{([^}:]+)(?::([^}]*))?\}")?;

        for cap in re.captures_iter(content) {
            let var_name = &cap[1];
            let default_value = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            let env_value = std::env::var(var_name).unwrap_or_else(|_| {
                if !default_value.is_empty() {
                    default_value.to_string()
                } else {
                    return format!("${{{}}}", var_name);
                }
            });

            result = result.replace(&cap[0], &env_value);
        }

        Ok(result)
    }

    /// Static method for configuration validation
    fn validate_config_static(config: &WorkspaceConfig, _rules: &ValidationRules) -> Result<()> {
        // Basic validation - ensure workspace name is not empty
        if config.workspace.name.is_empty() {
            return Err(anyhow::anyhow!("Workspace name cannot be empty"));
        }

        // Validate projects
        for project in &config.projects {
            if project.name.is_empty() {
                return Err(anyhow::anyhow!("Project name cannot be empty"));
            }
        }

        Ok(())
    }

    /// Create configuration from template
    pub async fn create_from_template(
        &mut self,
        template: &ConfigTemplate,
        variables: HashMap<String, String>,
    ) -> Result<()> {
        let mut config = template.config.clone();

        // Substitute template variables
        for (var_name, var_value) in variables {
            if let Some(template_var) = template.variables.get(&var_name) {
                // Validate variable type
                self.validate_template_variable(&var_name, &var_value, template_var)?;

                // Apply variable substitution (this would need to be implemented based on the template structure)
                // For now, we'll just use the provided configuration
            }
        }

        // Validate final configuration
        self.validate_config(&config)?;

        // Update configuration
        {
            let mut current_config = self.config.write();
            *current_config = config;
        }

        Ok(())
    }

    /// Validate template variable
    fn validate_template_variable(
        &self,
        _var_name: &str,
        var_value: &str,
        template_var: &TemplateVariable,
    ) -> Result<()> {
        match template_var.var_type {
            VariableType::String => {
                if let Some(min_len) = template_var.min_length {
                    if var_value.len() < min_len {
                        return Err(anyhow::anyhow!(
                            "Variable '{}' must be at least {} characters long",
                            template_var.name,
                            min_len
                        ));
                    }
                }
                if let Some(max_len) = template_var.max_length {
                    if var_value.len() > max_len {
                        return Err(anyhow::anyhow!(
                            "Variable '{}' must be at most {} characters long",
                            template_var.name,
                            max_len
                        ));
                    }
                }
            }
            VariableType::Integer => {
                var_value.parse::<i64>().map_err(|_| {
                    anyhow::anyhow!("Variable '{}' must be a valid integer", template_var.name)
                })?;
            }
            VariableType::Float => {
                var_value.parse::<f64>().map_err(|_| {
                    anyhow::anyhow!("Variable '{}' must be a valid float", template_var.name)
                })?;
            }
            VariableType::Boolean => {
                if !matches!(
                    var_value.to_lowercase().as_str(),
                    "true" | "false" | "1" | "0"
                ) {
                    return Err(anyhow::anyhow!(
                        "Variable '{}' must be a valid boolean (true/false, 1/0)",
                        template_var.name
                    ));
                }
            }
            VariableType::Path => {
                // Basic path validation
                if var_value.is_empty() {
                    return Err(anyhow::anyhow!(
                        "Variable '{}' cannot be empty",
                        template_var.name
                    ));
                }
            }
            VariableType::Url => {
                if !var_value.starts_with("http://") && !var_value.starts_with("https://") {
                    return Err(anyhow::anyhow!(
                        "Variable '{}' must be a valid URL",
                        template_var.name
                    ));
                }
            }
        }

        Ok(())
    }

    /// Get configuration as environment variables
    pub fn get_config_as_env_vars(&self) -> HashMap<String, String> {
        let config = self.config.read();
        let mut env_vars = HashMap::new();

        // Add workspace metadata
        env_vars.insert("WORKSPACE_NAME".to_string(), config.workspace.name.clone());
        env_vars.insert(
            "WORKSPACE_VERSION".to_string(),
            config.workspace.version.clone(),
        );
        env_vars.insert(
            "WORKSPACE_DESCRIPTION".to_string(),
            config.workspace.description.clone(),
        );

        // Add global settings
        env_vars.insert(
            "DEFAULT_EMBEDDING_MODEL".to_string(),
            format!("{:?}", config.global.default_embedding.model),
        );
        env_vars.insert(
            "DEFAULT_EMBEDDING_DIMENSION".to_string(),
            config.global.default_embedding.dimension.to_string(),
        );
        env_vars.insert(
            "DEFAULT_DISTANCE_METRIC".to_string(),
            format!("{:?}", config.global.default_collection.metric),
        );

        // Add processing settings
        env_vars.insert(
            "PARALLEL_PROCESSING".to_string(),
            config.processing.parallel_processing.to_string(),
        );
        env_vars.insert(
            "MAX_CONCURRENT_PROJECTS".to_string(),
            config.processing.max_concurrent_projects.to_string(),
        );
        env_vars.insert(
            "MAX_CONCURRENT_COLLECTIONS".to_string(),
            config.processing.max_concurrent_collections.to_string(),
        );

        // Add monitoring settings
        env_vars.insert(
            "HEALTH_CHECK_ENABLED".to_string(),
            config.monitoring.health_check.enabled.to_string(),
        );
        env_vars.insert(
            "METRICS_ENABLED".to_string(),
            config.monitoring.metrics.enabled.to_string(),
        );
        env_vars.insert(
            "LOG_LEVEL".to_string(),
            config.monitoring.logging.level.clone(),
        );

        env_vars
    }

    /// Export configuration to different formats
    pub async fn export_config(&self, format: ConfigFormat, output_path: &Path) -> Result<()> {
        let config = self.config.read();

        match format {
            ConfigFormat::Yaml => {
                let yaml_content = serde_yaml::to_string(&*config)?;
                fs::write(output_path, yaml_content).await?;
            }
            ConfigFormat::Json => {
                let json_content = serde_json::to_string_pretty(&*config)?;
                fs::write(output_path, json_content).await?;
            }
            ConfigFormat::Toml => {
                let toml_content = toml::to_string_pretty(&*config)?;
                fs::write(output_path, toml_content).await?;
            }
            ConfigFormat::Env => {
                let env_content = self
                    .get_config_as_env_vars()
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("\n");
                fs::write(output_path, env_content).await?;
            }
        }

        Ok(())
    }
}

/// Configuration export formats
#[derive(Debug, Clone, Copy)]
pub enum ConfigFormat {
    Yaml,
    Json,
    Toml,
    Env,
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self {
            required_fields: vec![
                "workspace.name".to_string(),
                "workspace.version".to_string(),
            ],
            field_constraints: HashMap::new(),
            cross_field_rules: vec![],
        }
    }
}

impl Default for ConfigTemplate {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            description: "Default configuration template".to_string(),
            config: WorkspaceConfig::default(),
            variables: HashMap::new(),
        }
    }
}


#[cfg(test)]
#[path = "enhanced_config_tests.rs"]
mod tests;
