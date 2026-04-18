mod quantization_model_tests {
    use vectorizer::models::qdrant::{
        QdrantBinaryQuantization, QdrantBinaryQuantizationConfig, QdrantPQCompression,
        QdrantProductQuantization, QdrantProductQuantizationConfig, QdrantQuantizationConfig,
        QdrantScalarQuantization, QdrantScalarQuantizationConfig, QdrantScalarQuantizationType,
    };

    #[test]
    fn test_scalar_quantization_serialization() {
        let config = QdrantQuantizationConfig::Scalar(QdrantScalarQuantizationConfig {
            scalar: QdrantScalarQuantization {
                r#type: QdrantScalarQuantizationType::Int8,
                quantile: Some(0.99),
                always_ram: Some(true),
            },
        });

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("scalar"));
        assert!(json.contains("int8"));
        assert!(json.contains("0.99"));
    }

    #[test]
    fn test_scalar_quantization_deserialization() {
        let json = r#"{"scalar":{"type":"int8","quantile":0.95}}"#;
        let config: QdrantQuantizationConfig = serde_json::from_str(json).unwrap();

        match config {
            QdrantQuantizationConfig::Scalar(scalar_config) => {
                assert!(matches!(
                    scalar_config.scalar.r#type,
                    QdrantScalarQuantizationType::Int8
                ));
                assert_eq!(scalar_config.scalar.quantile, Some(0.95));
            }
            _ => panic!("Expected Scalar quantization config"),
        }
    }

    #[test]
    fn test_product_quantization_serialization() {
        let config = QdrantQuantizationConfig::Product(QdrantProductQuantizationConfig {
            product: QdrantProductQuantization {
                compression: Some(QdrantPQCompression::X16),
                always_ram: Some(false),
            },
        });

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("product"));
        assert!(json.contains("x16"));
    }

    #[test]
    fn test_product_quantization_deserialization() {
        let json = r#"{"product":{"compression":"x32","always_ram":true}}"#;
        let config: QdrantQuantizationConfig = serde_json::from_str(json).unwrap();

        match config {
            QdrantQuantizationConfig::Product(product_config) => {
                assert!(matches!(
                    product_config.product.compression,
                    Some(QdrantPQCompression::X32)
                ));
                assert_eq!(product_config.product.always_ram, Some(true));
            }
            _ => panic!("Expected Product quantization config"),
        }
    }

    #[test]
    fn test_binary_quantization_serialization() {
        let config = QdrantQuantizationConfig::Binary(QdrantBinaryQuantizationConfig {
            binary: QdrantBinaryQuantization {
                always_ram: Some(true),
            },
        });

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("binary"));
        assert!(json.contains("always_ram"));
    }

    #[test]
    fn test_binary_quantization_deserialization() {
        let json = r#"{"binary":{"always_ram":false}}"#;
        let config: QdrantQuantizationConfig = serde_json::from_str(json).unwrap();

        match config {
            QdrantQuantizationConfig::Binary(binary_config) => {
                assert_eq!(binary_config.binary.always_ram, Some(false));
            }
            _ => panic!("Expected Binary quantization config"),
        }
    }

    #[test]
    fn test_all_pq_compression_levels() {
        let compression_levels = vec![
            (QdrantPQCompression::X4, "x4"),
            (QdrantPQCompression::X8, "x8"),
            (QdrantPQCompression::X16, "x16"),
            (QdrantPQCompression::X32, "x32"),
            (QdrantPQCompression::X64, "x64"),
        ];

        for (compression, expected_str) in compression_levels {
            let config = QdrantQuantizationConfig::Product(QdrantProductQuantizationConfig {
                product: QdrantProductQuantization {
                    compression: Some(compression),
                    always_ram: None,
                },
            });

            let json = serde_json::to_string(&config).unwrap();
            assert!(
                json.contains(expected_str),
                "Expected {expected_str} in JSON: {json}"
            );
        }
    }

    #[test]
    fn test_quantization_config_none_values() {
        let config = QdrantQuantizationConfig::Scalar(QdrantScalarQuantizationConfig {
            scalar: QdrantScalarQuantization {
                r#type: QdrantScalarQuantizationType::Int8,
                quantile: None,
                always_ram: None,
            },
        });

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("int8"));
        // Optional fields with None should not appear in JSON with skip_serializing_if
    }

    #[test]
    fn test_pq_default_compression() {
        let config = QdrantQuantizationConfig::Product(QdrantProductQuantizationConfig {
            product: QdrantProductQuantization {
                compression: None,
                always_ram: None,
            },
        });

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("product"));
    }

    #[test]
    fn test_binary_quantization_minimal() {
        let config = QdrantQuantizationConfig::Binary(QdrantBinaryQuantizationConfig {
            binary: QdrantBinaryQuantization { always_ram: None },
        });

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("binary"));
    }
}

// ============================================================================
// Unit Tests for Cluster Models
// ============================================================================

#[cfg(test)]
mod quantization_api_tests {
    use serde_json::json;
    use vectorizer::models::qdrant::search::QdrantQuantizationSearchParams;
    use vectorizer::models::qdrant::{
        QdrantBinaryQuantization, QdrantBinaryQuantizationConfig, QdrantPQCompression,
        QdrantProductQuantization, QdrantProductQuantizationConfig, QdrantQuantizationConfig,
        QdrantScalarQuantization, QdrantScalarQuantizationConfig, QdrantScalarQuantizationType,
    };

    #[test]
    fn test_quantization_config_in_collection_request() {
        // Simulate the request body for creating a collection with quantization
        let scalar_config = QdrantQuantizationConfig::Scalar(QdrantScalarQuantizationConfig {
            scalar: QdrantScalarQuantization {
                r#type: QdrantScalarQuantizationType::Int8,
                quantile: Some(0.99),
                always_ram: Some(true),
            },
        });

        let request_body = json!({
            "vectors": {
                "size": 128,
                "distance": "Cosine"
            },
            "quantization_config": scalar_config
        });

        let json_str = serde_json::to_string(&request_body).unwrap();
        assert!(json_str.contains("quantization_config"));
        assert!(json_str.contains("scalar"));
        assert!(json_str.contains("int8"));
    }

    #[test]
    fn test_product_quantization_in_request() {
        let pq_config = QdrantQuantizationConfig::Product(QdrantProductQuantizationConfig {
            product: QdrantProductQuantization {
                compression: Some(QdrantPQCompression::X16),
                always_ram: Some(false),
            },
        });

        let request_body = json!({
            "vectors": {
                "size": 256,
                "distance": "Euclid"
            },
            "quantization_config": pq_config
        });

        let json_str = serde_json::to_string(&request_body).unwrap();
        assert!(json_str.contains("product"));
        assert!(json_str.contains("x16"));
    }

    #[test]
    fn test_binary_quantization_in_request() {
        let binary_config = QdrantQuantizationConfig::Binary(QdrantBinaryQuantizationConfig {
            binary: QdrantBinaryQuantization {
                always_ram: Some(true),
            },
        });

        let request_body = json!({
            "vectors": {
                "size": 1024,
                "distance": "Dot"
            },
            "quantization_config": binary_config
        });

        let json_str = serde_json::to_string(&request_body).unwrap();
        assert!(json_str.contains("binary"));
        assert!(json_str.contains("always_ram"));
    }

    #[test]
    fn test_quantization_search_params() {
        let params = QdrantQuantizationSearchParams {
            ignore: Some(false),
            rescore: Some(true),
            oversampling: Some(2.0),
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"ignore\":false"));
        assert!(json.contains("\"rescore\":true"));
        assert!(json.contains("\"oversampling\":2"));
    }

    #[test]
    fn test_quantization_search_params_minimal() {
        let params = QdrantQuantizationSearchParams {
            ignore: None,
            rescore: Some(true),
            oversampling: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("rescore"));
    }

    #[test]
    fn test_all_compression_levels_round_trip() {
        let compressions = vec![
            QdrantPQCompression::X4,
            QdrantPQCompression::X8,
            QdrantPQCompression::X16,
            QdrantPQCompression::X32,
            QdrantPQCompression::X64,
        ];

        for compression in compressions {
            let config = QdrantQuantizationConfig::Product(QdrantProductQuantizationConfig {
                product: QdrantProductQuantization {
                    compression: Some(compression.clone()),
                    always_ram: None,
                },
            });

            // Serialize
            let json = serde_json::to_string(&config).unwrap();

            // Deserialize back
            let deserialized: QdrantQuantizationConfig = serde_json::from_str(&json).unwrap();

            match deserialized {
                QdrantQuantizationConfig::Product(p) => {
                    assert!(p.product.compression.is_some());
                }
                _ => panic!("Expected Product quantization"),
            }
        }
    }
}
