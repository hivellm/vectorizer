# Proposal: Add ECC and AES-256-GCM Encryption for Payloads

## Why

Vectorizer currently stores payload data in plaintext, which poses security risks for sensitive information. Organizations handling confidential data (medical records, financial information, personal data) need end-to-end encryption capabilities. This feature enables clients to encrypt payloads using ECC (Elliptic Curve Cryptography) for key exchange and AES-256-GCM for symmetric encryption, ensuring that even if the database is compromised, payload data remains protected. The zero-knowledge architecture (Vectorizer never stores or accesses decryption keys) provides maximum security and compliance with data protection regulations like GDPR and HIPAA.

## What Changes

- **ADDED**: Optional ECC public key parameter in vector insertion/update operations
- **ADDED**: Payload encryption module using ECC for key derivation and AES-256-GCM for encryption
- **ADDED**: Encrypted payload storage format with metadata (nonce, tag, encrypted key)
- **ADDED**: Configuration option to enable/disable payload encryption per collection
- **ADDED**: REST API and MCP endpoints support for encryption key parameter
- **MODIFIED**: Vector and Payload models to support encrypted payload format
- **MODIFIED**: Vector insertion/update operations to encrypt payloads when public key is provided

## Impact

- Affected specs: `docs/specs/security/spec.md`, `docs/specs/api/spec.md`
- Affected code: 
  - `src/models/mod.rs` - Payload and Vector models
  - `src/db/collection.rs` - Vector insertion/update logic
  - `src/api/rest_handlers.rs` - REST API endpoints
  - `src/mcp/server.rs` - MCP tool handlers
  - New module: `src/security/payload_encryption.rs`
- Breaking change: NO (encryption is optional, backward compatible)
- User benefit: Enhanced security for sensitive payload data, compliance with data protection regulations, zero-knowledge architecture ensures Vectorizer cannot decrypt data
