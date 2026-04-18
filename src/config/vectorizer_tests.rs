//! Extracted unit tests (phase3 test-extraction).
//!
//! Wired from `src/config/vectorizer.rs` via the `#[path]` attribute.

use super::*;

#[test]
fn test_transmutation_config_default() {
    let config = TransmutationConfig::default();

    #[cfg(feature = "transmutation")]
    {
        assert!(config.enabled, "Should be enabled when feature is compiled");
    }

    #[cfg(not(feature = "transmutation"))]
    {
        assert!(
            !config.enabled,
            "Should be disabled when feature is not compiled"
        );
    }

    assert_eq!(config.max_file_size_mb, 50);
    assert_eq!(config.conversion_timeout_secs, 300);
    assert!(!config.preserve_images);
}

#[test]
fn test_transmutation_config_custom() {
    let config = TransmutationConfig {
        enabled: true,
        max_file_size_mb: 100,
        conversion_timeout_secs: 600,
        preserve_images: true,
    };

    assert!(config.enabled);
    assert_eq!(config.max_file_size_mb, 100);
    assert_eq!(config.conversion_timeout_secs, 600);
    assert!(config.preserve_images);
}

#[test]
fn test_transmutation_config_serialization() {
    let config = TransmutationConfig {
        enabled: true,
        max_file_size_mb: 75,
        conversion_timeout_secs: 450,
        preserve_images: false,
    };

    // Test that config can be serialized
    let serialized = serde_json::to_string(&config).unwrap();
    assert!(serialized.contains("enabled"));
    assert!(serialized.contains("max_file_size_mb"));
    assert!(serialized.contains("conversion_timeout_secs"));
    assert!(serialized.contains("preserve_images"));
}

#[test]
fn test_transmutation_config_deserialization() {
    let json = r#"{
        "enabled": true,
        "max_file_size_mb": 80,
        "conversion_timeout_secs": 500,
        "preserve_images": true
    }"#;

    let config: TransmutationConfig = serde_json::from_str(json).unwrap();
    assert!(config.enabled);
    assert_eq!(config.max_file_size_mb, 80);
    assert_eq!(config.conversion_timeout_secs, 500);
    assert!(config.preserve_images);
}

#[test]
fn test_transmutation_config_in_vectorizer_config() {
    let config = VectorizerConfig::default();

    // Verify transmutation config is present
    #[cfg(feature = "transmutation")]
    {
        assert!(config.transmutation.enabled);
    }

    #[cfg(not(feature = "transmutation"))]
    {
        assert!(!config.transmutation.enabled);
    }
}

#[test]
fn test_transmutation_config_boundaries() {
    // Test minimum values
    let config_min = TransmutationConfig {
        enabled: false,
        max_file_size_mb: 1,
        conversion_timeout_secs: 1,
        preserve_images: false,
    };
    assert_eq!(config_min.max_file_size_mb, 1);
    assert_eq!(config_min.conversion_timeout_secs, 1);

    // Test maximum reasonable values
    let config_max = TransmutationConfig {
        enabled: true,
        max_file_size_mb: 1000,        // 1GB
        conversion_timeout_secs: 3600, // 1 hour
        preserve_images: true,
    };
    assert_eq!(config_max.max_file_size_mb, 1000);
    assert_eq!(config_max.conversion_timeout_secs, 3600);
}

#[test]
fn test_transmutation_config_yaml_format() {
    let config = TransmutationConfig {
        enabled: true,
        max_file_size_mb: 50,
        conversion_timeout_secs: 300,
        preserve_images: false,
    };

    // Test YAML serialization
    let yaml = serde_yaml::to_string(&config).unwrap();
    assert!(yaml.contains("enabled"));
    assert!(yaml.contains("max_file_size_mb"));

    // Test YAML deserialization
    let config_from_yaml: TransmutationConfig = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(config.enabled, config_from_yaml.enabled);
    assert_eq!(config.max_file_size_mb, config_from_yaml.max_file_size_mb);
}

// =========================================================================
// FileUploadConfig Tests
// =========================================================================

#[test]
fn test_file_upload_config_default() {
    let config = FileUploadConfig::default();

    assert_eq!(config.max_file_size, 10 * 1024 * 1024); // 10MB
    assert!(config.reject_binary);
    assert_eq!(config.default_chunk_size, 2048);
    assert_eq!(config.default_chunk_overlap, 256);
    assert!(!config.allowed_extensions.is_empty());
}

#[test]
fn test_file_upload_config_default_extensions_count() {
    let config = FileUploadConfig::default();

    // Should have a reasonable number of extensions
    assert!(config.allowed_extensions.len() > 40);
    assert!(config.allowed_extensions.len() < 100);
}

#[test]
fn test_file_upload_config_serialization() {
    let config = FileUploadConfig {
        max_file_size: 5 * 1024 * 1024,
        allowed_extensions: vec!["rs".to_string(), "py".to_string()],
        reject_binary: true,
        default_chunk_size: 1024,
        default_chunk_overlap: 128,
    };

    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("max_file_size"));
    assert!(json.contains("5242880")); // 5MB in bytes
    assert!(json.contains("rs"));
    assert!(json.contains("py"));
    assert!(json.contains("reject_binary"));
    assert!(json.contains("default_chunk_size"));
    assert!(json.contains("default_chunk_overlap"));
}

#[test]
fn test_file_upload_config_deserialization() {
    let json = r#"{
        "max_file_size": 20971520,
        "allowed_extensions": ["md", "txt", "json"],
        "reject_binary": false,
        "default_chunk_size": 4096,
        "default_chunk_overlap": 512
    }"#;

    let config: FileUploadConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.max_file_size, 20 * 1024 * 1024); // 20MB
    assert_eq!(config.allowed_extensions.len(), 3);
    assert!(config.allowed_extensions.contains(&"md".to_string()));
    assert!(!config.reject_binary);
    assert_eq!(config.default_chunk_size, 4096);
    assert_eq!(config.default_chunk_overlap, 512);
}

#[test]
fn test_file_upload_config_yaml_format() {
    let config = FileUploadConfig {
        max_file_size: 10485760,
        allowed_extensions: vec!["rs".to_string(), "py".to_string(), "js".to_string()],
        reject_binary: true,
        default_chunk_size: 2048,
        default_chunk_overlap: 256,
    };

    // Test YAML serialization
    let yaml = serde_yaml::to_string(&config).unwrap();
    assert!(yaml.contains("max_file_size"));
    assert!(yaml.contains("allowed_extensions"));
    assert!(yaml.contains("reject_binary"));

    // Test YAML deserialization
    let config_from_yaml: FileUploadConfig = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(config.max_file_size, config_from_yaml.max_file_size);
    assert_eq!(config.reject_binary, config_from_yaml.reject_binary);
    assert_eq!(
        config.allowed_extensions.len(),
        config_from_yaml.allowed_extensions.len()
    );
}

#[test]
fn test_file_upload_config_partial_deserialization() {
    // Test that missing fields use defaults
    let json = r#"{
        "max_file_size": 5242880
    }"#;

    let config: FileUploadConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.max_file_size, 5 * 1024 * 1024);
    // Other fields should use defaults
    assert!(config.reject_binary); // default is true
    assert_eq!(config.default_chunk_size, 2048);
    assert_eq!(config.default_chunk_overlap, 256);
    assert!(!config.allowed_extensions.is_empty());
}

#[test]
fn test_file_upload_config_empty_extensions() {
    let config = FileUploadConfig {
        max_file_size: 1024,
        allowed_extensions: vec![],
        reject_binary: true,
        default_chunk_size: 100,
        default_chunk_overlap: 10,
    };

    assert!(config.allowed_extensions.is_empty());
}

#[test]
fn test_file_upload_config_in_vectorizer_config() {
    let config = VectorizerConfig::default();

    // Verify file_upload config is present and has correct defaults
    assert_eq!(config.file_upload.max_file_size, 10 * 1024 * 1024);
    assert!(config.file_upload.reject_binary);
    assert!(!config.file_upload.allowed_extensions.is_empty());
}

#[test]
fn test_file_upload_config_clone() {
    let config = FileUploadConfig::default();
    let cloned = config.clone();

    assert_eq!(config.max_file_size, cloned.max_file_size);
    assert_eq!(config.reject_binary, cloned.reject_binary);
    assert_eq!(config.default_chunk_size, cloned.default_chunk_size);
    assert_eq!(
        config.allowed_extensions.len(),
        cloned.allowed_extensions.len()
    );
}

#[test]
fn test_file_upload_config_debug() {
    let config = FileUploadConfig::default();
    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("FileUploadConfig"));
    assert!(debug_str.contains("max_file_size"));
    assert!(debug_str.contains("reject_binary"));
}

#[test]
fn test_file_upload_config_extension_categories() {
    let config = FileUploadConfig::default();

    // Text files
    assert!(config.allowed_extensions.contains(&"txt".to_string()));
    assert!(config.allowed_extensions.contains(&"md".to_string()));

    // Programming languages
    assert!(config.allowed_extensions.contains(&"rs".to_string()));
    assert!(config.allowed_extensions.contains(&"py".to_string()));
    assert!(config.allowed_extensions.contains(&"js".to_string()));
    assert!(config.allowed_extensions.contains(&"ts".to_string()));
    assert!(config.allowed_extensions.contains(&"go".to_string()));
    assert!(config.allowed_extensions.contains(&"java".to_string()));
    assert!(config.allowed_extensions.contains(&"c".to_string()));
    assert!(config.allowed_extensions.contains(&"cpp".to_string()));

    // Config files
    assert!(config.allowed_extensions.contains(&"json".to_string()));
    assert!(config.allowed_extensions.contains(&"yaml".to_string()));
    assert!(config.allowed_extensions.contains(&"yml".to_string()));
    assert!(config.allowed_extensions.contains(&"toml".to_string()));
    assert!(config.allowed_extensions.contains(&"xml".to_string()));

    // Web files
    assert!(config.allowed_extensions.contains(&"html".to_string()));
    assert!(config.allowed_extensions.contains(&"css".to_string()));
}
