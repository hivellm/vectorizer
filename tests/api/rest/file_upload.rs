//! File Upload API Integration Tests

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use reqwest::multipart::{Form, Part};

    const BASE_URL: &str = "http://localhost:15002";

    /// Helper to check if server is running
    async fn server_is_running() -> bool {
        reqwest::Client::new()
            .get(format!("{BASE_URL}/health"))
            .timeout(Duration::from_secs(2))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_get_upload_config() {
        if !server_is_running().await {
            eprintln!("Skipping test: server not running");
            return;
        }

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{BASE_URL}/files/config"))
            .send()
            .await
            .expect("Failed to send request");

        assert!(response.status().is_success(), "Expected 200 OK");

        let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");

        // Verify config structure
        assert!(body.get("max_file_size").is_some());
        assert!(body.get("max_file_size_mb").is_some());
        assert!(body.get("allowed_extensions").is_some());
        assert!(body.get("reject_binary").is_some());
        assert!(body.get("default_chunk_size").is_some());
        assert!(body.get("default_chunk_overlap").is_some());

        println!(
            "Upload config: {}",
            serde_json::to_string_pretty(&body).unwrap()
        );
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_upload_text_file() {
        if !server_is_running().await {
            eprintln!("Skipping test: server not running");
            return;
        }

        let client = reqwest::Client::new();

        // Create a test text file
        let file_content = b"# Test Document\n\nThis is a test document for the file upload API.\n\n## Section 1\n\nSome content here.\n\n## Section 2\n\nMore content here.";

        let form = Form::new()
            .part(
                "file",
                Part::bytes(file_content.to_vec())
                    .file_name("test_document.md")
                    .mime_str("text/markdown")
                    .unwrap(),
            )
            .text("collection_name", "test-upload-collection");

        let response = client
            .post(format!("{BASE_URL}/files/upload"))
            .multipart(form)
            .send()
            .await
            .expect("Failed to send request");

        assert!(
            response.status().is_success(),
            "Expected 200 OK, got {}",
            response.status()
        );

        let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");

        println!(
            "Upload response: {}",
            serde_json::to_string_pretty(&body).unwrap()
        );

        // Verify response structure
        assert_eq!(body.get("success").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(
            body.get("filename").and_then(|v| v.as_str()),
            Some("test_document.md")
        );
        assert!(
            body.get("chunks_created")
                .and_then(|v| v.as_i64())
                .unwrap_or(0)
                > 0
        );
        assert!(
            body.get("vectors_created")
                .and_then(|v| v.as_i64())
                .unwrap_or(0)
                > 0
        );
        assert_eq!(
            body.get("language").and_then(|v| v.as_str()),
            Some("markdown")
        );
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_upload_code_file() {
        if !server_is_running().await {
            eprintln!("Skipping test: server not running");
            return;
        }

        let client = reqwest::Client::new();

        // Create a test Rust file
        let file_content = br"
//! A test module

/// Calculate the factorial of a number
pub fn factorial(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        _ => n * factorial(n - 1),
    }
}

/// Check if a number is prime
pub fn is_prime(n: u64) -> bool {
    if n <= 1 {
        return false;
    }
    for i in 2..=((n as f64).sqrt() as u64) {
        if n % i == 0 {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factorial() {
        assert_eq!(factorial(0), 1);
        assert_eq!(factorial(5), 120);
    }

    #[test]
    fn test_is_prime() {
        assert!(!is_prime(0));
        assert!(is_prime(2));
        assert!(is_prime(7));
        assert!(!is_prime(9));
    }
}
";

        let form = Form::new()
            .part(
                "file",
                Part::bytes(file_content.to_vec())
                    .file_name("test_math.rs")
                    .mime_str("text/x-rust")
                    .unwrap(),
            )
            .text("collection_name", "test-code-collection")
            .text("chunk_size", "512")
            .text("chunk_overlap", "64");

        let response = client
            .post(format!("{BASE_URL}/files/upload"))
            .multipart(form)
            .send()
            .await
            .expect("Failed to send request");

        assert!(
            response.status().is_success(),
            "Expected 200 OK, got {}",
            response.status()
        );

        let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");

        println!(
            "Upload response: {}",
            serde_json::to_string_pretty(&body).unwrap()
        );

        assert_eq!(body.get("success").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(body.get("language").and_then(|v| v.as_str()), Some("rust"));
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_upload_invalid_extension() {
        if !server_is_running().await {
            eprintln!("Skipping test: server not running");
            return;
        }

        let client = reqwest::Client::new();

        // Try to upload a binary file
        let file_content = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR";

        let form = Form::new()
            .part(
                "file",
                Part::bytes(file_content.to_vec())
                    .file_name("image.png")
                    .mime_str("image/png")
                    .unwrap(),
            )
            .text("collection_name", "test-collection");

        let response = client
            .post(format!("{BASE_URL}/files/upload"))
            .multipart(form)
            .send()
            .await
            .expect("Failed to send request");

        // Should fail with 400 Bad Request
        assert_eq!(
            response.status().as_u16(),
            400,
            "Expected 400 for invalid extension"
        );

        let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
        println!(
            "Error response: {}",
            serde_json::to_string_pretty(&body).unwrap()
        );

        assert!(body.get("error_type").is_some());
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_upload_missing_collection_name() {
        if !server_is_running().await {
            eprintln!("Skipping test: server not running");
            return;
        }

        let client = reqwest::Client::new();

        let file_content = b"Hello, World!";

        let form = Form::new().part(
            "file",
            Part::bytes(file_content.to_vec())
                .file_name("test.txt")
                .mime_str("text/plain")
                .unwrap(),
        );

        let response = client
            .post(format!("{BASE_URL}/files/upload"))
            .multipart(form)
            .send()
            .await
            .expect("Failed to send request");

        // Should fail with 400 Bad Request for missing collection_name
        assert_eq!(
            response.status().as_u16(),
            400,
            "Expected 400 for missing collection_name"
        );
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_upload_with_metadata() {
        if !server_is_running().await {
            eprintln!("Skipping test: server not running");
            return;
        }

        let client = reqwest::Client::new();

        let file_content = b"# README\n\nThis is a project readme file.";

        let metadata = serde_json::json!({
            "project": "test-project",
            "version": "1.0.0",
            "tags": ["documentation", "readme"]
        });

        let form = Form::new()
            .part(
                "file",
                Part::bytes(file_content.to_vec())
                    .file_name("README.md")
                    .mime_str("text/markdown")
                    .unwrap(),
            )
            .text("collection_name", "test-metadata-collection")
            .text("metadata", metadata.to_string());

        let response = client
            .post(format!("{BASE_URL}/files/upload"))
            .multipart(form)
            .send()
            .await
            .expect("Failed to send request");

        assert!(
            response.status().is_success(),
            "Expected 200 OK, got {}",
            response.status()
        );

        let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");

        println!(
            "Upload response: {}",
            serde_json::to_string_pretty(&body).unwrap()
        );

        assert_eq!(body.get("success").and_then(|v| v.as_bool()), Some(true));
    }
}
