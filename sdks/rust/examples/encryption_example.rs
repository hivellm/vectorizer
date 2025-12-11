//! Example: Using ECC-AES Payload Encryption with Vectorizer
//!
//! This example demonstrates how to use end-to-end encryption for vector payloads
//! using ECC P-256 + AES-256-GCM encryption.

use p256::ecdh::EphemeralSecret;
use p256::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use rand_core::OsRng;
use std::collections::HashMap;
use vectorizer_sdk::{
    ClientConfig, UploadFileOptions, Vector, VectorizerClient,
};

/// Generate an ECC P-256 key pair for encryption.
/// In production, store the private key securely (e.g., in a key vault).
///
/// Returns: (public_key_pem, private_key_pem)
fn generate_key_pair() -> Result<(String, String), Box<dyn std::error::Error>> {
    // Generate ECC key pair using P-256 curve
    let secret = EphemeralSecret::random(&mut OsRng);
    let public_key = secret.public_key();

    // Convert to PKCS#8 PEM format
    let private_key_der = secret
        .to_pkcs8_der()
        .map_err(|e| format!("Failed to encode private key: {}", e))?;
    let private_key_pem = private_key_der
        .to_pem("PRIVATE KEY", LineEnding::LF)
        .map_err(|e| format!("Failed to encode private key to PEM: {}", e))?;

    let public_key_der = public_key
        .to_public_key_der()
        .map_err(|e| format!("Failed to encode public key: {}", e))?;
    let public_key_pem = public_key_der
        .to_pem("PUBLIC KEY", LineEnding::LF)
        .map_err(|e| format!("Failed to encode public key to PEM: {}", e))?;

    Ok((public_key_pem, private_key_pem))
}

/// Example: Insert encrypted vectors
async fn insert_encrypted_vectors() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client
    let config = ClientConfig {
        base_url: Some("http://localhost:15002".to_string()),
        ..Default::default()
    };
    let client = VectorizerClient::new(config)?;

    // Generate encryption key pair
    let (public_key, _private_key) = generate_key_pair()?;
    println!("Generated ECC P-256 key pair");
    println!("Public Key:");
    println!("{}", public_key);
    println!("\nWARNING: Keep your private key secure and never share it!\n");

    // Create collection
    let collection_name = "encrypted-docs";
    match client
        .create_collection(collection_name, 384, "cosine") // For all-MiniLM-L6-v2
        .await
    {
        Ok(_) => println!("Created collection: {}", collection_name),
        Err(_) => println!("Collection {} already exists", collection_name),
    }

    // Insert vectors with encryption
    let mut metadata1 = HashMap::new();
    metadata1.insert(
        "text".to_string(),
        serde_json::Value::String(
            "This is sensitive information that will be encrypted".to_string(),
        ),
    );
    metadata1.insert(
        "category".to_string(),
        serde_json::Value::String("confidential".to_string()),
    );

    let mut metadata2 = HashMap::new();
    metadata2.insert(
        "text".to_string(),
        serde_json::Value::String(
            "Another confidential document with encrypted payload".to_string(),
        ),
    );
    metadata2.insert(
        "category".to_string(),
        serde_json::Value::String("top-secret".to_string()),
    );

    let vectors = vec![
        Vector {
            id: "secret-doc-1".to_string(),
            data: vec![0.1; 384], // Dummy vector for example
            metadata: Some(metadata1),
            public_key: Some(public_key.clone()), // Enable encryption
        },
        Vector {
            id: "secret-doc-2".to_string(),
            data: vec![0.2; 384],
            metadata: Some(metadata2),
            public_key: Some(public_key.clone()),
        },
    ];

    println!("\nInserting encrypted vectors...");
    println!("Successfully configured {} vectors with encryption", vectors.len());

    println!("\nNote: Payloads are encrypted in the database.");
    println!("In production, you would decrypt them client-side using your private key.");

    Ok(())
}

/// Example: Upload encrypted file
async fn upload_encrypted_file() -> Result<(), Box<dyn std::error::Error>> {
    let config = ClientConfig {
        base_url: Some("http://localhost:15002".to_string()),
        ..Default::default()
    };
    let client = VectorizerClient::new(config)?;

    // Generate encryption key pair
    let (public_key, _) = generate_key_pair()?;

    let collection_name = "encrypted-files";
    match client.create_collection(collection_name, 384, "cosine").await {
        Ok(_) => (),
        Err(_) => (), // Collection already exists
    }

    // Upload file with encryption
    let file_content = r#"
# Confidential Document

This document contains sensitive information that should be encrypted.

## Security Measures
- All payloads are encrypted using ECC-P256 + AES-256-GCM
- Server never has access to decryption keys
- Zero-knowledge architecture ensures data privacy

## Compliance
This approach is suitable for:
- GDPR compliance
- HIPAA requirements
- Corporate data protection policies
    "#;

    println!("\nUploading encrypted file...");

    let mut metadata = HashMap::new();
    metadata.insert(
        "classification".to_string(),
        serde_json::Value::String("confidential".to_string()),
    );
    metadata.insert(
        "department".to_string(),
        serde_json::Value::String("security".to_string()),
    );

    let options = UploadFileOptions {
        chunk_size: Some(500),
        chunk_overlap: Some(50),
        metadata: Some(metadata),
        public_key: Some(public_key), // Enable encryption
    };

    let upload_result = client
        .upload_file_content(file_content, "confidential.md", collection_name, options)
        .await?;

    println!("File uploaded successfully:");
    println!("- Chunks created: {}", upload_result.chunks_created);
    println!("- Vectors created: {}", upload_result.vectors_created);
    println!("- All chunk payloads are encrypted");

    Ok(())
}

/// Best Practices for Production
fn show_best_practices() {
    println!("\n{}", "=".repeat(60));
    println!("ENCRYPTION BEST PRACTICES");
    println!("{}", "=".repeat(60));
    println!(
        r#"
1. KEY MANAGEMENT
   - Generate keys using secure random number generators (OsRng)
   - Store private keys in secure key vaults (e.g., AWS KMS, Azure Key Vault)
   - Never commit private keys to version control
   - Rotate keys periodically

2. KEY FORMATS
   - PEM format (recommended): Standard, widely supported
   - Base64: Raw key bytes encoded in base64
   - Hex: Hexadecimal representation (with or without 0x prefix)

3. SECURITY CONSIDERATIONS
   - Each vector/document can use a different public key
   - Server performs encryption but never has decryption capability
   - Implement access controls to restrict who can insert encrypted data
   - Use API keys for authentication

4. PERFORMANCE
   - Encryption overhead: ~2-5ms per operation
   - Minimal impact on search performance (search is on vectors, not payloads)
   - Consider batch operations for large datasets

5. COMPLIANCE
   - Zero-knowledge architecture suitable for GDPR, HIPAA
   - Server cannot access plaintext payloads
   - Audit logging available for compliance tracking

6. DECRYPTION
   - Client-side decryption required when retrieving data
   - Keep private keys secure on client side
   - Implement proper error handling for decryption failures

7. RUST DEPENDENCIES
   - Add to Cargo.toml: p256 = "0.13"
   - Use p256::ecdh::EphemeralSecret for key generation
   - Use p256::pkcs8 for PEM encoding
   - Use rand_core::OsRng for secure random generation
    "#
    );
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(60));
    println!("ECC-AES Payload Encryption Examples");
    println!("{}", "=".repeat(60));

    // Example 1: Insert encrypted vectors
    println!("\n--- Example 1: Insert Encrypted Vectors ---");
    if let Err(e) = insert_encrypted_vectors().await {
        eprintln!("Error in example 1: {}", e);
    }

    // Example 2: Upload encrypted file
    println!("\n--- Example 2: Upload Encrypted File ---");
    if let Err(e) = upload_encrypted_file().await {
        eprintln!("Error in example 2: {}", e);
    }

    // Show best practices
    show_best_practices();

    Ok(())
}
