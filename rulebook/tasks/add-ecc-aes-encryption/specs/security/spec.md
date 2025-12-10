# Security Specification (Vectorizer)

## ADDED Requirements

### Requirement: Payload Encryption with ECC and AES-256-GCM
The system SHALL support optional encryption of vector payloads using ECC (Elliptic Curve Cryptography) for key derivation and AES-256-GCM for symmetric encryption. The system MUST NOT store or have access to decryption keys, ensuring zero-knowledge architecture.

#### Scenario: Encrypt payload with public key
Given a vector insertion request with a payload and an optional ECC public key
When the public key is provided
Then the system SHALL derive an AES-256-GCM key using ECC key exchange
And the system SHALL encrypt the payload data using AES-256-GCM
And the system SHALL store the encrypted payload with metadata (nonce, authentication tag, encrypted key)
And the system SHALL NOT store the decryption key or plaintext payload

#### Scenario: Insert vector with encrypted payload
Given a REST API or MCP request to insert a vector with payload and public key
When the request includes a valid ECC public key in PEM or DER format
Then the system SHALL encrypt the payload before storage
And the system SHALL return success with the vector ID
And the encrypted payload SHALL be stored in the database

#### Scenario: Update vector with encrypted payload
Given a request to update an existing vector's payload with a public key
When the update request includes a valid ECC public key
Then the system SHALL encrypt the new payload data
And the system SHALL replace the existing payload with the encrypted version
And the system SHALL preserve the vector ID and vector data

#### Scenario: Backward compatibility with unencrypted payloads
Given a vector insertion request without a public key
When the request does not include an encryption key
Then the system SHALL store the payload in plaintext format
And the system SHALL maintain full backward compatibility
And existing unencrypted payloads SHALL continue to work

#### Scenario: Invalid public key handling
Given a vector insertion request with an invalid public key format
When the public key cannot be parsed or is malformed
Then the system SHALL return an error indicating invalid key format
And the system SHALL NOT store the vector
And the error message SHALL describe the expected key format

#### Scenario: Zero-knowledge architecture enforcement
Given the system has stored encrypted payloads
When any operation attempts to decrypt payloads
Then the system SHALL NOT have access to decryption keys
And the system SHALL NOT provide decryption functionality
And the system SHALL only return encrypted payload data to clients

### Requirement: Encryption Configuration
The system SHALL support configuration options for payload encryption at the collection level.

#### Scenario: Enable encryption per collection
Given a collection configuration
When encryption is enabled for the collection
Then the system SHALL require a public key for all payload insertions
And the system SHALL reject insertions without public keys
And the system SHALL encrypt all payloads in that collection

#### Scenario: Optional encryption per request
Given encryption is not enforced at collection level
When a vector insertion includes an optional public key
Then the system SHALL encrypt the payload if key is provided
And the system SHALL store plaintext if no key is provided
And the system SHALL support mixed encrypted and unencrypted payloads in the same collection

### Requirement: Encrypted Payload Format
The system SHALL store encrypted payloads in a structured format that includes all necessary metadata for decryption by authorized clients.

#### Scenario: Encrypted payload structure
Given a payload is encrypted using AES-256-GCM
When the encrypted payload is stored
Then the system SHALL include the ECC-encrypted AES key
And the system SHALL include the nonce used for AES-256-GCM
And the system SHALL include the authentication tag from AES-256-GCM
And the system SHALL include the encrypted payload data
And the system SHALL use a standard format (JSON or binary) for metadata

#### Scenario: Payload metadata preservation
Given an encrypted payload is stored
When the payload is retrieved
Then the system SHALL return all encryption metadata
And the system SHALL preserve the original payload structure indication
And the system SHALL allow clients to identify encrypted vs unencrypted payloads

## MODIFIED Requirements

### Requirement: Vector Payload Storage
The system SHALL support both encrypted and unencrypted payload storage formats, maintaining backward compatibility while enabling optional encryption.

#### Scenario: Payload format detection
Given a stored vector payload
When the payload is retrieved
Then the system SHALL detect whether the payload is encrypted or plaintext
And the system SHALL return the payload in its stored format
And the system SHALL include format metadata in the response

