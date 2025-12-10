## 1. Planning & Design
- [ ] 1.1 Research ECC and AES-256-GCM implementation patterns in Rust
- [ ] 1.2 Design encrypted payload data structure (nonce, tag, encrypted key, encrypted data)
- [ ] 1.3 Design API changes for optional public key parameter
- [ ] 1.4 Define configuration options for encryption

## 2. Core Implementation
- [ ] 2.1 Create payload encryption module (`src/security/payload_encryption.rs`)
- [ ] 2.2 Implement ECC key derivation using provided public key
- [ ] 2.3 Implement AES-256-GCM encryption for payload data
- [ ] 2.4 Create encrypted payload data structure with metadata
- [ ] 2.5 Add encryption configuration to collection config

## 3. Model Updates
- [ ] 3.1 Update Payload model to support encrypted format
- [ ] 3.2 Add encryption metadata fields (nonce, tag, encrypted key)
- [ ] 3.3 Update Vector model serialization for encrypted payloads
- [ ] 3.4 Ensure backward compatibility with unencrypted payloads

## 4. Database Integration
- [ ] 4.1 Update vector insertion to encrypt payloads when public key provided
- [ ] 4.2 Update vector update operations to support encryption
- [ ] 4.3 Ensure encrypted payloads are stored correctly in all storage backends
- [ ] 4.4 Update batch insertion operations for encryption support

## 5. API Integration
- [ ] 5.1 Add optional public key parameter to REST insert/update endpoints
- [ ] 5.2 Add optional public key parameter to MCP insert/update tools
- [ ] 5.3 Update request/response models for encryption support
- [ ] 5.4 Add validation for public key format (PEM/DER)

## 6. Testing
- [ ] 6.1 Write unit tests for ECC key derivation
- [ ] 6.2 Write unit tests for AES-256-GCM encryption
- [ ] 6.3 Write integration tests for encrypted payload insertion
- [ ] 6.4 Write integration tests for encrypted payload updates
- [ ] 6.5 Test backward compatibility with unencrypted payloads
- [ ] 6.6 Test error handling for invalid public keys
- [ ] 6.7 Verify zero-knowledge property (no decryption capability)

## 7. Documentation
- [ ] 7.1 Update API documentation with encryption parameters
- [ ] 7.2 Add encryption usage examples to README
- [ ] 7.3 Update CHANGELOG with new encryption feature
- [ ] 7.4 Document security considerations and best practices
