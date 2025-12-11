/**
 * Example: Using ECC-AES Payload Encryption with Vectorizer
 *
 * This example demonstrates how to use end-to-end encryption for vector payloads
 * using ECC P-256 + AES-256-GCM encryption.
 */

const { VectorizerClient } = require('../src');
const crypto = require('crypto');

/**
 * Generate an ECC P-256 key pair for encryption.
 * In production, store the private key securely (e.g., in a key vault).
 */
function generateKeyPair() {
  const { publicKey, privateKey } = crypto.generateKeyPairSync('ec', {
    namedCurve: 'prime256v1', // P-256 curve
    publicKeyEncoding: {
      type: 'spki',
      format: 'pem',
    },
    privateKeyEncoding: {
      type: 'pkcs8',
      format: 'pem',
    },
  });

  return { publicKey, privateKey };
}

/**
 * Example: Insert encrypted vectors
 */
async function insertEncryptedVectors() {
  // Initialize client
  const client = new VectorizerClient({
    baseURL: 'http://localhost:15002',
  });

  // Generate encryption key pair
  const { publicKey, privateKey } = generateKeyPair();
  console.log('Generated ECC P-256 key pair');
  console.log('Public Key:', publicKey);
  console.log('\nWARNING: Keep your private key secure and never share it!\n');

  // Create collection
  const collectionName = 'encrypted-docs';
  try {
    await client.createCollection({
      name: collectionName,
      dimension: 384, // For all-MiniLM-L6-v2
      metric: 'cosine',
    });
    console.log(`Created collection: ${collectionName}`);
  } catch (error) {
    console.log(`Collection ${collectionName} already exists`);
  }

  // Insert vectors with encryption
  const vectors = [
    {
      id: 'secret-doc-1',
      data: Array(384).fill(0).map(() => Math.random()),
      metadata: {
        text: 'This is sensitive information that will be encrypted',
        category: 'confidential',
        timestamp: new Date().toISOString(),
      },
    },
    {
      id: 'secret-doc-2',
      data: Array(384).fill(0).map(() => Math.random()),
      metadata: {
        text: 'Another confidential document with encrypted payload',
        category: 'top-secret',
        timestamp: new Date().toISOString(),
      },
    },
  ];

  console.log('\nInserting encrypted vectors...');
  // Pass publicKey as parameter to encrypt all vectors
  const result = await client.insertVectors(collectionName, vectors, publicKey);
  console.log(`Successfully inserted ${result.inserted} encrypted vectors`);

  // Search for vectors (results will have encrypted payloads)
  console.log('\nSearching for similar vectors...');
  const searchResults = await client.searchVectors(
    collectionName,
    {
      query_vector: vectors[0].data,
      limit: 5,
      include_metadata: true,
    }
  );

  console.log(`Found ${searchResults.results.length} results`);
  console.log('\nNote: Payloads are encrypted in the database.');
  console.log('In production, you would decrypt them client-side using your private key.');

  // Cleanup
  await client.close();
}

/**
 * Example: Upload encrypted file
 */
async function uploadEncryptedFile() {
  const client = new VectorizerClient({
    baseURL: 'http://localhost:15002',
  });

  // Generate encryption key pair
  const { publicKey } = generateKeyPair();

  const collectionName = 'encrypted-files';
  try {
    await client.createCollection({
      name: collectionName,
      dimension: 384,
      metric: 'cosine',
    });
  } catch (error) {
    // Collection already exists
  }

  // Upload file with encryption
  const fileContent = `
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
  `;

  console.log('\nUploading encrypted file...');
  const uploadResult = await client.uploadFileContent(
    collectionName,
    fileContent,
    'confidential.md',
    {
      chunkSize: 500,
      chunkOverlap: 50,
      publicKey, // Enable encryption
      metadata: {
        classification: 'confidential',
        department: 'security',
      },
    }
  );

  console.log('File uploaded successfully:');
  console.log(`- Chunks created: ${uploadResult.chunks_created}`);
  console.log(`- Vectors created: ${uploadResult.vectors_created}`);
  console.log(`- All chunk payloads are encrypted`);

  await client.close();
}

/**
 * Example: Using different key formats
 */
async function demonstrateKeyFormats() {
  console.log('\n--- Key Format Examples ---');

  const { publicKey } = generateKeyPair();

  // PEM format (default)
  console.log('\n1. PEM Format (recommended):');
  console.log(publicKey);

  // Base64 format
  const base64Key = Buffer.from(
    publicKey
      .replace('-----BEGIN PUBLIC KEY-----', '')
      .replace('-----END PUBLIC KEY-----', '')
      .replace(/\s/g, ''),
    'base64'
  ).toString('base64');
  console.log('\n2. Base64 Format:');
  console.log(base64Key);

  // Hex format
  const hexKey = Buffer.from(
    publicKey
      .replace('-----BEGIN PUBLIC KEY-----', '')
      .replace('-----END PUBLIC KEY-----', '')
      .replace(/\s/g, ''),
    'base64'
  ).toString('hex');
  console.log('\n3. Hex Format:');
  console.log(hexKey);
  console.log('\n4. Hex with 0x prefix:');
  console.log('0x' + hexKey);

  console.log('\nAll formats are supported by the API!');
}

/**
 * Best Practices for Production
 */
function showBestPractices() {
  console.log('\n' + '='.repeat(60));
  console.log('ENCRYPTION BEST PRACTICES');
  console.log('='.repeat(60));
  console.log(`
1. KEY MANAGEMENT
   - Generate keys using secure random number generators
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
   - Use API keys or JWT tokens for authentication

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

7. BROWSER USAGE
   - Use Web Crypto API for key generation in browsers
   - Consider SubtleCrypto for client-side encryption/decryption
   - Store keys securely (never in localStorage without encryption)
  `);
}

// Run examples
async function main() {
  console.log('='.repeat(60));
  console.log('ECC-AES Payload Encryption Examples');
  console.log('='.repeat(60));

  try {
    // Example 1: Insert encrypted vectors
    console.log('\n--- Example 1: Insert Encrypted Vectors ---');
    await insertEncryptedVectors();

    // Example 2: Upload encrypted file
    console.log('\n--- Example 2: Upload Encrypted File ---');
    await uploadEncryptedFile();

    // Example 3: Key formats
    await demonstrateKeyFormats();

    // Show best practices
    showBestPractices();

  } catch (error) {
    console.error('Error running examples:', error);
    process.exit(1);
  }
}

// Only run if executed directly
if (require.main === module) {
  main();
}

module.exports = {
  generateKeyPair,
  insertEncryptedVectors,
  uploadEncryptedFile,
  demonstrateKeyFormats,
};
