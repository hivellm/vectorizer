//! Middleware tests for HiveHub integration

use vectorizer::hub::middleware::AuthErrorResponse;

#[test]
fn test_auth_error_response_new() {
    let error = AuthErrorResponse::new("Test error", "TEST_CODE");

    assert_eq!(error.error, "Test error");
    assert_eq!(error.code, "TEST_CODE");
    assert!(error.details.is_none());
}

#[test]
fn test_auth_error_response_with_details() {
    let error =
        AuthErrorResponse::new("Test error", "TEST_CODE").with_details("Additional information");

    assert_eq!(error.error, "Test error");
    assert_eq!(error.code, "TEST_CODE");
    assert_eq!(error.details, Some("Additional information".to_string()));
}

#[test]
fn test_auth_error_response_serialization() {
    let error = AuthErrorResponse::new("Invalid key", "AUTH_INVALID_KEY");

    let json = serde_json::to_string(&error).unwrap();
    assert!(json.contains("error"));
    assert!(json.contains("Invalid key"));
    assert!(json.contains("code"));
    assert!(json.contains("AUTH_INVALID_KEY"));

    // Details should be omitted when None
    assert!(!json.contains("details"));
}

#[test]
fn test_auth_error_response_with_details_serialization() {
    let error = AuthErrorResponse::new("Rate limit exceeded", "RATE_LIMIT")
        .with_details("Try again in 60 seconds");

    let json = serde_json::to_string(&error).unwrap();
    assert!(json.contains("details"));
    assert!(json.contains("Try again in 60 seconds"));
}

#[test]
fn test_auth_error_response_deserialization() {
    let json = r#"{
        "error": "Unauthorized",
        "code": "AUTH_REQUIRED"
    }"#;

    let error: AuthErrorResponse = serde_json::from_str(json).unwrap();
    assert_eq!(error.error, "Unauthorized");
    assert_eq!(error.code, "AUTH_REQUIRED");
    assert!(error.details.is_none());
}

#[test]
fn test_auth_error_response_with_details_deserialization() {
    let json = r#"{
        "error": "Quota exceeded",
        "code": "QUOTA_EXCEEDED",
        "details": "Storage limit reached"
    }"#;

    let error: AuthErrorResponse = serde_json::from_str(json).unwrap();
    assert_eq!(error.error, "Quota exceeded");
    assert_eq!(error.code, "QUOTA_EXCEEDED");
    assert_eq!(error.details, Some("Storage limit reached".to_string()));
}
