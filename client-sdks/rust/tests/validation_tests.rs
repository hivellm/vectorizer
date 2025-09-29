//! Validation utility tests for the Rust SDK

use vectorizer_rust_sdk::*;

#[test]
fn test_validate_non_empty_string() {
    // Valid cases
    assert!(utils::validation::validate_non_empty_string("valid_string", "Test").is_ok());
    assert!(utils::validation::validate_non_empty_string(" valid_string ", "Test").is_ok());
    assert!(utils::validation::validate_non_empty_string("a", "Test").is_ok());
    
    // Invalid cases
    assert!(utils::validation::validate_non_empty_string("", "Test").is_err());
    assert!(utils::validation::validate_non_empty_string("   ", "Test").is_err());
    assert!(utils::validation::validate_non_empty_string("\t\n", "Test").is_err());
    
    // Check error messages
    let result = utils::validation::validate_non_empty_string("", "Field");
    assert!(matches!(result, Err(VectorizerError::Validation { message }) 
        if message.contains("Field cannot be empty")));
}

#[test]
fn test_validate_positive_number() {
    // Valid cases
    assert!(utils::validation::validate_positive_number(1.0, "Test").is_ok());
    assert!(utils::validation::validate_positive_number(0.1, "Test").is_ok());
    assert!(utils::validation::validate_positive_number(100.0, "Test").is_ok());
    
    // Invalid cases
    assert!(utils::validation::validate_positive_number(0.0, "Test").is_err());
    assert!(utils::validation::validate_positive_number(-1.0, "Test").is_err());
    assert!(utils::validation::validate_positive_number(-0.1, "Test").is_err());
    
    // Check error messages
    let result = utils::validation::validate_positive_number(0.0, "Field");
    assert!(matches!(result, Err(VectorizerError::Validation { message }) 
        if message.contains("Field must be positive")));
}

#[test]
fn test_validate_non_negative_number() {
    // Valid cases
    assert!(utils::validation::validate_non_negative_number(0.0, "Test").is_ok());
    assert!(utils::validation::validate_non_negative_number(1.0, "Test").is_ok());
    assert!(utils::validation::validate_non_negative_number(0.1, "Test").is_ok());
    
    // Invalid cases
    assert!(utils::validation::validate_non_negative_number(-1.0, "Test").is_err());
    assert!(utils::validation::validate_non_negative_number(-0.1, "Test").is_err());
    
    // Check error messages
    let result = utils::validation::validate_non_negative_number(-1.0, "Field");
    assert!(matches!(result, Err(VectorizerError::Validation { message }) 
        if message.contains("Field must be non-negative")));
}

#[test]
fn test_validate_collection_name() {
    // Valid cases
    assert!(utils::validation::validate_collection_name("valid_collection").is_ok());
    assert!(utils::validation::validate_collection_name("valid-collection").is_ok());
    assert!(utils::validation::validate_collection_name("ValidCollection123").is_ok());
    assert!(utils::validation::validate_collection_name("collection123").is_ok());
    
    // Invalid cases
    assert!(utils::validation::validate_collection_name("").is_err());
    assert!(utils::validation::validate_collection_name("   ").is_err());
    assert!(utils::validation::validate_collection_name("invalid collection").is_err());
    assert!(utils::validation::validate_collection_name("invalid/collection").is_err());
    assert!(utils::validation::validate_collection_name("invalid\\collection").is_err());
    assert!(utils::validation::validate_collection_name("invalid@collection").is_err());
    
    // Check error messages
    let result = utils::validation::validate_collection_name("invalid collection");
    assert!(matches!(result, Err(VectorizerError::Validation { message }) 
        if message.contains("Collection name cannot contain spaces")));
}

#[test]
fn test_validate_vector_id() {
    // Valid cases
    assert!(utils::validation::validate_vector_id("valid_id").is_ok());
    assert!(utils::validation::validate_vector_id("valid-id").is_ok());
    assert!(utils::validation::validate_vector_id("ValidID123").is_ok());
    assert!(utils::validation::validate_vector_id("id123").is_ok());
    assert!(utils::validation::validate_vector_id("id with spaces").is_ok()); // Vector IDs can have spaces
    
    // Invalid cases
    assert!(utils::validation::validate_vector_id("").is_err());
    assert!(utils::validation::validate_vector_id("   ").is_err());
    
    // Check error messages
    let result = utils::validation::validate_vector_id("");
    assert!(matches!(result, Err(VectorizerError::Validation { message }) 
        if message.contains("vector ID cannot be empty")));
}

#[test]
fn test_validate_similarity_metric() {
    // Valid cases
    assert!(utils::validation::validate_similarity_metric("cosine").is_ok());
    assert!(utils::validation::validate_similarity_metric("euclidean").is_ok());
    assert!(utils::validation::validate_similarity_metric("dot_product").is_ok());
    
    // Invalid cases
    assert!(utils::validation::validate_similarity_metric("invalid").is_err());
    assert!(utils::validation::validate_similarity_metric("").is_err());
    assert!(utils::validation::validate_similarity_metric("Cosine").is_err()); // Case sensitive
    assert!(utils::validation::validate_similarity_metric("manhattan").is_err());
    
    // Check error messages
    let result = utils::validation::validate_similarity_metric("invalid");
    assert!(matches!(result, Err(VectorizerError::Validation { message }) 
        if message.contains("Invalid similarity metric")));
}

#[test]
fn test_serialization_utilities() {
    // Test to_json
    let data = vec!["test1", "test2"];
    let json_result = utils::serialization::to_json(&data);
    assert!(json_result.is_ok());
    
    let json = json_result.unwrap();
    assert_eq!(json, r#"["test1","test2"]"#);
    
    // Test from_json
    let json = r#"["test1","test2"]"#;
    let deserialized: Vec<String> = utils::serialization::from_json(json).unwrap();
    assert_eq!(deserialized, vec!["test1", "test2"]);
    
    // Test invalid JSON
    let invalid_json = r#"{"invalid": json}"#;
    let result: Result<Vec<String>> = utils::serialization::from_json(invalid_json);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), VectorizerError::Serialization(_)));
}

#[test]
fn test_validation_error_handling() {
    // Test that validation errors are properly wrapped
    let result = utils::validation::validate_non_empty_string("", "Test");
    assert!(matches!(result, Err(VectorizerError::Validation { message }) 
        if message.contains("Test cannot be empty")));
    
    let result = utils::validation::validate_positive_number(-1.0, "Test");
    assert!(matches!(result, Err(VectorizerError::Validation { message }) 
        if message.contains("Test must be positive")));
    
    let result = utils::validation::validate_non_negative_number(-0.1, "Test");
    assert!(matches!(result, Err(VectorizerError::Validation { message }) 
        if message.contains("Test must be non-negative")));
}

#[test]
fn test_validation_edge_cases() {
    // Test very long strings (should still be valid if not empty)
    let long_string = "a".repeat(1000);
    assert!(utils::validation::validate_non_empty_string(&long_string, "Test").is_ok());
    
    // Test very small positive numbers
    assert!(utils::validation::validate_positive_number(f32::MIN_POSITIVE, "Test").is_ok());
    
    // Test very large numbers
    assert!(utils::validation::validate_positive_number(f32::MAX, "Test").is_ok());
    
    // Test zero for non-negative validation
    assert!(utils::validation::validate_non_negative_number(0.0, "Test").is_ok());
    
    // Test special floating point values
    assert!(utils::validation::validate_positive_number(f32::NAN, "Test").is_err());
    assert!(utils::validation::validate_positive_number(f32::INFINITY, "Test").is_err());
    assert!(utils::validation::validate_positive_number(f32::NEG_INFINITY, "Test").is_err());
}

#[test]
fn test_validation_consistency() {
    // Test that validation is consistent across multiple calls
    for i in 0..10 {
        let test_string = format!("test_string_{}", i);
        assert!(utils::validation::validate_non_empty_string(&test_string, "Test").is_ok());
        
        let test_number = i as f32 + 0.1;
        assert!(utils::validation::validate_positive_number(test_number, "Test").is_ok());
        assert!(utils::validation::validate_non_negative_number(test_number, "Test").is_ok());
    }
}

#[test]
fn test_collection_name_special_characters() {
    // Test various special characters that should be invalid
    let invalid_names = vec![
        "collection with spaces",
        "collection/with/slashes",
        "collection\\with\\backslashes",
        "collection@with@at",
        "collection#with#hash",
        "collection$with$dollar",
        "collection%with%percent",
        "collection^with^caret",
        "collection&with&ampersand",
        "collection*with*asterisk",
        "collection(with)parens",
        "collection[with]brackets",
        "collection{with}braces",
        "collection|with|pipe",
        "collection+with+plus",
        "collection=with=equals",
        "collection?with?question",
        "collection<with>angles",
        "collection,with,comma",
        "collection.with.dot",
        "collection;with;semicolon",
        "collection:with:colon",
        "collection'with'apostrophe",
        "collection\"with\"quotes",
        "collection~with~tilde",
        "collection`with`backtick",
    ];
    
    for invalid_name in invalid_names {
        let result = utils::validation::validate_collection_name(invalid_name);
        assert!(result.is_err(), "Collection name '{}' should be invalid", invalid_name);
    }
}

#[test]
fn test_vector_id_edge_cases() {
    // Vector IDs are more permissive than collection names
    let valid_ids = vec![
        "id with spaces",
        "id-with-dashes",
        "id_with_underscores",
        "id.with.dots",
        "id123with456numbers",
        "ID_with_MIXED_case",
    ];
    
    for valid_id in valid_ids {
        assert!(utils::validation::validate_vector_id(valid_id).is_ok(), 
               "Vector ID '{}' should be valid", valid_id);
    }
}

#[test]
fn test_similarity_metric_case_sensitivity() {
    // Test that similarity metrics are case-sensitive
    let case_variations = vec![
        "Cosine",
        "COSINE",
        "Euclidean",
        "EUCLIDEAN",
        "Dot_Product",
        "DOT_PRODUCT",
    ];
    
    for variation in case_variations {
        assert!(utils::validation::validate_similarity_metric(variation).is_err(),
               "Similarity metric '{}' should be invalid due to case", variation);
    }
}
