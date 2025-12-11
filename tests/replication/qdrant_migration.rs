//! Qdrant migration integration tests

use vectorizer::db::VectorStore;
use vectorizer::migration::qdrant::{
    ConfigFormat, MigrationValidator, QdrantConfigParser, QdrantDataExporter, QdrantDataImporter,
};
use vectorizer::models::DistanceMetric;

#[tokio::test]
async fn test_config_parser_yaml() {
    let yaml = r"
collections:
  test_collection:
    vectors:
      size: 128
      distance: Cosine
    hnsw_config:
      m: 16
      ef_construct: 100
";

    let config = QdrantConfigParser::parse_str(yaml, ConfigFormat::Yaml).unwrap();
    assert!(config.collections.is_some());

    let collections = config.collections.as_ref().unwrap();
    assert!(collections.contains_key("test_collection"));

    // Validate config
    let validation = QdrantConfigParser::validate(&config).unwrap();
    assert!(validation.is_valid);
    assert!(validation.errors.is_empty());

    // Convert to Vectorizer format
    let vectorizer_configs = QdrantConfigParser::convert_to_vectorizer(&config).unwrap();
    assert_eq!(vectorizer_configs.len(), 1);

    let (name, config) = &vectorizer_configs[0];
    assert_eq!(name, "test_collection");
    assert_eq!(config.dimension, 128);
    assert_eq!(config.metric, DistanceMetric::Cosine);
}

#[tokio::test]
async fn test_config_parser_json() {
    let json = r#"
{
  "collections": {
    "my_collection": {
      "vectors": {
        "size": 384,
        "distance": "Euclidean"
      },
      "hnsw_config": {
        "m": 16,
        "ef_construct": 100
      }
    }
  }
}
"#;

    let config = QdrantConfigParser::parse_str(json, ConfigFormat::Json).unwrap();
    assert!(config.collections.is_some());

    let vectorizer_configs = QdrantConfigParser::convert_to_vectorizer(&config).unwrap();
    assert_eq!(vectorizer_configs.len(), 1);

    let (name, config) = &vectorizer_configs[0];
    assert_eq!(name, "my_collection");
    assert_eq!(config.dimension, 384);
    assert_eq!(config.metric, DistanceMetric::Euclidean);
}

#[tokio::test]
async fn test_config_validation_errors() {
    let yaml = r"
collections:
  invalid_collection:
    vectors:
      size: 0
      distance: Cosine
";

    let config = QdrantConfigParser::parse_str(yaml, ConfigFormat::Yaml).unwrap();
    let validation = QdrantConfigParser::validate(&config).unwrap();

    assert!(!validation.is_valid);
    assert!(!validation.errors.is_empty());
    assert!(
        validation
            .errors
            .iter()
            .any(|e| e.contains("vector size must be > 0"))
    );
}

#[tokio::test]
async fn test_config_validation_warnings() {
    let yaml = r"
collections:
  large_collection:
    vectors:
      size: 100000
      distance: Cosine
";

    let config = QdrantConfigParser::parse_str(yaml, ConfigFormat::Yaml).unwrap();
    let validation = QdrantConfigParser::validate(&config).unwrap();

    assert!(validation.is_valid);
    assert!(!validation.warnings.is_empty());
    assert!(
        validation
            .warnings
            .iter()
            .any(|w| w.contains("very large vector dimension"))
    );
}

#[tokio::test]
async fn test_migration_validator_compatibility() {
    use vectorizer::migration::qdrant::data_migration::{
        ExportedCollection, QdrantCollectionConfig, QdrantCollectionParams, QdrantPoint,
        QdrantVector, QdrantVectorsConfigResponse,
    };

    // Create a simple exported collection
    let exported = ExportedCollection {
        name: "test_collection".to_string(),
        config: QdrantCollectionConfig {
            params: QdrantCollectionParams {
                vectors: QdrantVectorsConfigResponse::Vector {
                    size: 128,
                    distance: "Cosine".to_string(),
                },
                hnsw_config: None,
                quantization_config: None,
            },
        },
        points: vec![QdrantPoint {
            id: "1".to_string(),
            vector: QdrantVector::Dense(vec![0.1; 128]),
            payload: Some(serde_json::json!({"text": "test"})),
        }],
    };

    // Validate export
    let validation = MigrationValidator::validate_export(&exported).unwrap();
    assert!(validation.is_valid);
    assert_eq!(validation.statistics.total_points, 1);
    assert_eq!(validation.statistics.points_with_payload, 1);

    // Validate compatibility
    let compatibility = MigrationValidator::validate_compatibility(&exported);
    assert!(compatibility.is_compatible);
    assert!(compatibility.incompatible_features.is_empty());
}

#[tokio::test]
async fn test_migration_validator_integrity() {
    use vectorizer::migration::qdrant::data_migration::{
        ExportedCollection, QdrantCollectionConfig, QdrantCollectionParams, QdrantPoint,
        QdrantVector, QdrantVectorsConfigResponse,
    };

    let exported = ExportedCollection {
        name: "test_collection".to_string(),
        config: QdrantCollectionConfig {
            params: QdrantCollectionParams {
                vectors: QdrantVectorsConfigResponse::Vector {
                    size: 128,
                    distance: "Cosine".to_string(),
                },
                hnsw_config: None,
                quantization_config: None,
            },
        },
        points: vec![
            QdrantPoint {
                id: "1".to_string(),
                vector: QdrantVector::Dense(vec![0.1; 128]),
                payload: None,
            },
            QdrantPoint {
                id: "2".to_string(),
                vector: QdrantVector::Dense(vec![0.2; 128]),
                payload: None,
            },
        ],
    };

    // Test complete import
    let integrity = MigrationValidator::validate_integrity(&exported, 2).unwrap();
    assert!(integrity.is_complete);
    assert_eq!(integrity.integrity_percentage, 100.0);
    assert_eq!(integrity.missing_count, 0);

    // Test partial import
    let integrity = MigrationValidator::validate_integrity(&exported, 1).unwrap();
    assert!(!integrity.is_complete);
    assert_eq!(integrity.integrity_percentage, 50.0);
    assert_eq!(integrity.missing_count, 1);
}

#[tokio::test]
async fn test_migration_validator_sparse_vectors() {
    use vectorizer::migration::qdrant::data_migration::{
        ExportedCollection, QdrantCollectionConfig, QdrantCollectionParams, QdrantPoint,
        QdrantSparseVector, QdrantVector, QdrantVectorsConfigResponse,
    };

    let exported = ExportedCollection {
        name: "test_collection".to_string(),
        config: QdrantCollectionConfig {
            params: QdrantCollectionParams {
                vectors: QdrantVectorsConfigResponse::Vector {
                    size: 128,
                    distance: "Cosine".to_string(),
                },
                hnsw_config: None,
                quantization_config: None,
            },
        },
        points: vec![QdrantPoint {
            id: "1".to_string(),
            vector: QdrantVector::Sparse(QdrantSparseVector {
                indices: vec![0, 1, 2],
                values: vec![0.1, 0.2, 0.3],
            }),
            payload: None,
        }],
    };

    // Validate export should fail for sparse vectors
    let validation = MigrationValidator::validate_export(&exported).unwrap();
    assert!(!validation.is_valid);
    assert!(!validation.errors.is_empty());
    assert!(
        validation
            .errors
            .iter()
            .any(|e| e.contains("Sparse vectors not supported"))
    );

    // Compatibility check should detect sparse vectors
    let compatibility = MigrationValidator::validate_compatibility(&exported);
    assert!(!compatibility.is_compatible);
    assert!(
        compatibility
            .incompatible_features
            .iter()
            .any(|f| f.contains("Sparse vectors"))
    );
}

#[tokio::test]
async fn test_data_importer_from_file() {
    use vectorizer::migration::qdrant::data_migration::{
        ExportedCollection, QdrantCollectionConfig, QdrantCollectionParams, QdrantPoint,
        QdrantVector, QdrantVectorsConfigResponse,
    };

    // Create temporary file path
    let export_file = std::env::temp_dir().join(format!("test_export_{}.json", std::process::id()));

    // Create test export data
    let exported = ExportedCollection {
        name: "test_collection".to_string(),
        config: QdrantCollectionConfig {
            params: QdrantCollectionParams {
                vectors: QdrantVectorsConfigResponse::Vector {
                    size: 128,
                    distance: "Cosine".to_string(),
                },
                hnsw_config: None,
                quantization_config: None,
            },
        },
        points: vec![QdrantPoint {
            id: "1".to_string(),
            vector: QdrantVector::Dense(vec![0.1; 128]),
            payload: Some(serde_json::json!({"text": "test"})),
        }],
    };

    // Export to file
    QdrantDataExporter::export_to_file(&exported, &export_file).unwrap();

    // Verify file exists
    assert!(export_file.exists());

    // Import from file
    let imported = QdrantDataImporter::import_from_file(&export_file).unwrap();
    assert_eq!(imported.name, exported.name);
    assert_eq!(imported.points.len(), exported.points.len());
}

#[tokio::test]
async fn test_data_importer_into_vectorizer() {
    use vectorizer::migration::qdrant::data_migration::{
        ExportedCollection, QdrantCollectionConfig, QdrantCollectionParams, QdrantPoint,
        QdrantVector, QdrantVectorsConfigResponse,
    };

    // Create VectorStore
    let store = VectorStore::new();

    // Create test export data
    let exported = ExportedCollection {
        name: "test_migration_collection".to_string(),
        config: QdrantCollectionConfig {
            params: QdrantCollectionParams {
                vectors: QdrantVectorsConfigResponse::Vector {
                    size: 128,
                    distance: "Cosine".to_string(),
                },
                hnsw_config: None,
                quantization_config: None,
            },
        },
        points: vec![
            QdrantPoint {
                id: "1".to_string(),
                vector: QdrantVector::Dense(vec![0.1; 128]),
                payload: Some(serde_json::json!({"text": "test1"})),
            },
            QdrantPoint {
                id: "2".to_string(),
                vector: QdrantVector::Dense(vec![0.2; 128]),
                payload: Some(serde_json::json!({"text": "test2"})),
            },
        ],
    };

    // Import into Vectorizer
    let result = QdrantDataImporter::import_collection(&store, &exported)
        .await
        .unwrap();

    assert_eq!(result.collection_name, "test_migration_collection");
    assert_eq!(result.imported_count, 2);
    assert_eq!(result.error_count, 0);

    // Verify collection exists
    let collections = store.list_collections();
    assert!(collections.contains(&"test_migration_collection".to_string()));

    // Verify vectors were imported
    let collection = store.get_collection("test_migration_collection").unwrap();
    let vector_count = collection.vector_count();
    assert_eq!(vector_count, 2);
}

#[tokio::test]
async fn test_config_conversion_all_metrics() {
    let metrics = vec!["Cosine", "Euclidean", "Dot"];

    for metric_str in metrics {
        let yaml = format!(
            r"
collections:
  test_collection:
    vectors:
      size: 128
      distance: {metric_str}
"
        );

        let config = QdrantConfigParser::parse_str(&yaml, ConfigFormat::Yaml).unwrap();
        let vectorizer_configs = QdrantConfigParser::convert_to_vectorizer(&config).unwrap();

        let (_, config) = &vectorizer_configs[0];
        assert_eq!(config.dimension, 128);

        match metric_str {
            "Cosine" => assert_eq!(config.metric, DistanceMetric::Cosine),
            "Euclidean" => assert_eq!(config.metric, DistanceMetric::Euclidean),
            "Dot" => assert_eq!(config.metric, DistanceMetric::DotProduct),
            _ => panic!("Unknown metric"),
        }
    }
}

#[tokio::test]
async fn test_config_conversion_hnsw() {
    let yaml = r"
collections:
  test_collection:
    vectors:
      size: 128
      distance: Cosine
    hnsw_config:
      m: 32
      ef_construct: 200
      ef: 150
";

    let config = QdrantConfigParser::parse_str(yaml, ConfigFormat::Yaml).unwrap();
    let vectorizer_configs = QdrantConfigParser::convert_to_vectorizer(&config).unwrap();

    let (_, config) = &vectorizer_configs[0];
    assert_eq!(config.hnsw_config.m, 32);
    assert_eq!(config.hnsw_config.ef_construction, 200);
    assert_eq!(config.hnsw_config.ef_search, 150);
}

#[tokio::test]
async fn test_config_conversion_quantization() {
    let yaml = r"
collections:
  test_collection:
    vectors:
      size: 128
      distance: Cosine
    quantization_config:
      quantization: int8
";

    let config = QdrantConfigParser::parse_str(yaml, ConfigFormat::Yaml).unwrap();
    let vectorizer_configs = QdrantConfigParser::convert_to_vectorizer(&config).unwrap();

    let (_, config) = &vectorizer_configs[0];
    match config.quantization {
        vectorizer::models::QuantizationConfig::SQ { bits } => assert_eq!(bits, 8),
        _ => panic!("Expected SQ8 quantization"),
    }
}
