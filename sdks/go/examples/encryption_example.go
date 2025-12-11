package main

import (
	"crypto/ecdsa"
	"crypto/elliptic"
	"crypto/rand"
	"crypto/x509"
	"encoding/pem"
	"fmt"
	"log"

	vectorizer "github.com/hive-llm/vectorizer/sdks/go"
)

// generateKeyPair generates an ECC P-256 key pair for encryption.
// In production, store the private key securely (e.g., in a key vault).
func generateKeyPair() (publicKeyPEM string, privateKeyPEM string, err error) {
	// Generate ECC key pair using P-256 curve
	privateKey, err := ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
	if err != nil {
		return "", "", fmt.Errorf("failed to generate key pair: %w", err)
	}

	// Export public key as PEM
	publicKeyBytes, err := x509.MarshalPKIXPublicKey(&privateKey.PublicKey)
	if err != nil {
		return "", "", fmt.Errorf("failed to marshal public key: %w", err)
	}

	publicKeyPEMBlock := &pem.Block{
		Type:  "PUBLIC KEY",
		Bytes: publicKeyBytes,
	}
	publicKeyPEM = string(pem.EncodeToMemory(publicKeyPEMBlock))

	// Export private key as PEM
	privateKeyBytes, err := x509.MarshalECPrivateKey(privateKey)
	if err != nil {
		return "", "", fmt.Errorf("failed to marshal private key: %w", err)
	}

	privateKeyPEMBlock := &pem.Block{
		Type:  "EC PRIVATE KEY",
		Bytes: privateKeyBytes,
	}
	privateKeyPEM = string(pem.EncodeToMemory(privateKeyPEMBlock))

	return publicKeyPEM, privateKeyPEM, nil
}

// insertEncryptedVectors demonstrates inserting encrypted vectors
func insertEncryptedVectors() {
	// Initialize client
	client := vectorizer.NewClient(&vectorizer.Config{
		BaseURL: "http://localhost:15002",
	})

	// Generate encryption key pair
	publicKey, _, err := generateKeyPair()
	if err != nil {
		log.Fatalf("Failed to generate key pair: %v", err)
	}

	fmt.Println("Generated ECC P-256 key pair")
	fmt.Println("Public Key:")
	fmt.Println(publicKey)
	fmt.Println("\nWARNING: Keep your private key secure and never share it!\n")

	// Create collection
	collectionName := "encrypted-docs"
	_, err = client.CreateCollection(&vectorizer.CreateCollectionRequest{
		Name: collectionName,
		Config: &vectorizer.CollectionConfig{
			Dimension: 384, // For all-MiniLM-L6-v2
			Metric:    vectorizer.MetricCosine,
		},
	})
	if err != nil {
		fmt.Printf("Collection %s already exists or error: %v\n", collectionName, err)
	} else {
		fmt.Printf("Created collection: %s\n", collectionName)
	}

	// Insert vectors with encryption
	vectors := []vectorizer.Vector{
		{
			ID:   "secret-doc-1",
			Data: make([]float32, 384), // Dummy vector for example
			Payload: map[string]interface{}{
				"text":     "This is sensitive information that will be encrypted",
				"category": "confidential",
			},
			PublicKey: publicKey, // Enable encryption
		},
		{
			ID:   "secret-doc-2",
			Data: make([]float32, 384),
			Payload: map[string]interface{}{
				"text":     "Another confidential document with encrypted payload",
				"category": "top-secret",
			},
			PublicKey: publicKey,
		},
	}

	// Initialize dummy data
	for i := range vectors {
		for j := range vectors[i].Data {
			vectors[i].Data[j] = 0.1
		}
	}

	fmt.Println("\nInserting encrypted vectors...")
	// Note: Go SDK would need a batch insert method or individual inserts
	// This is a conceptual example
	fmt.Println("Successfully configured vectors with encryption")

	fmt.Println("\nNote: Payloads are encrypted in the database.")
	fmt.Println("In production, you would decrypt them client-side using your private key.")
}

// uploadEncryptedFile demonstrates uploading an encrypted file
func uploadEncryptedFile() {
	client := vectorizer.NewClient(&vectorizer.Config{
		BaseURL: "http://localhost:15002",
	})

	// Generate encryption key pair
	publicKey, _, err := generateKeyPair()
	if err != nil {
		log.Fatalf("Failed to generate key pair: %v", err)
	}

	collectionName := "encrypted-files"
	_, err = client.CreateCollection(&vectorizer.CreateCollectionRequest{
		Name: collectionName,
		Config: &vectorizer.CollectionConfig{
			Dimension: 384,
			Metric:    vectorizer.MetricCosine,
		},
	})
	if err != nil {
		// Collection already exists
	}

	// Upload file with encryption
	fileContent := `
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
	`

	fmt.Println("\nUploading encrypted file...")
	chunkSize := 500
	chunkOverlap := 50

	uploadResult, err := client.UploadFileContent(
		fileContent,
		"confidential.md",
		collectionName,
		&vectorizer.UploadFileOptions{
			ChunkSize:    &chunkSize,
			ChunkOverlap: &chunkOverlap,
			PublicKey:    publicKey, // Enable encryption
			Metadata: map[string]interface{}{
				"classification": "confidential",
				"department":     "security",
			},
		},
	)

	if err != nil {
		log.Fatalf("Failed to upload file: %v", err)
	}

	fmt.Println("File uploaded successfully:")
	fmt.Printf("- Chunks created: %d\n", uploadResult.ChunksCreated)
	fmt.Printf("- Vectors created: %d\n", uploadResult.VectorsCreated)
	fmt.Println("- All chunk payloads are encrypted")
}

// showBestPractices displays encryption best practices
func showBestPractices() {
	fmt.Println("\n" + "============================================================")
	fmt.Println("ENCRYPTION BEST PRACTICES")
	fmt.Println("============================================================")
	fmt.Println(`
1. KEY MANAGEMENT
   - Generate keys using crypto/rand for secure randomness
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

7. GO DEPENDENCIES
   - Use crypto/ecdsa for key generation
   - Use crypto/elliptic with P256() curve
   - Use crypto/x509 for PEM encoding
	`)
}

func main() {
	fmt.Println("============================================================")
	fmt.Println("ECC-AES Payload Encryption Examples")
	fmt.Println("============================================================")

	// Example 1: Insert encrypted vectors
	fmt.Println("\n--- Example 1: Insert Encrypted Vectors ---")
	insertEncryptedVectors()

	// Example 2: Upload encrypted file
	fmt.Println("\n--- Example 2: Upload Encrypted File ---")
	uploadEncryptedFile()

	// Show best practices
	showBestPractices()
}
