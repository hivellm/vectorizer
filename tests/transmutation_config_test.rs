//! Integration tests for transmutation configuration

#[cfg(test)]
mod config_tests {
    use vectorizer::config::TransmutationConfig;

    #[test]
    fn test_transmutation_config_default() {
        let config = TransmutationConfig::default();

        #[cfg(feature = "transmutation")]
        {
            assert!(config.enabled, "Should be enabled when feature is compiled");
        }

        #[cfg(not(feature = "transmutation"))]
        {
            assert!(!config.enabled, "Should be disabled when feature is not compiled");
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
        use vectorizer::config::VectorizerConfig;

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
            max_file_size_mb: 1000, // 1GB
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
}

