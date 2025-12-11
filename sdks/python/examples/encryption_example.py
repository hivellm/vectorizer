"""
Example: Using ECC-AES Payload Encryption with Vectorizer

This example demonstrates how to use end-to-end encryption for vector payloads
using ECC P-256 + AES-256-GCM encryption.
"""

import asyncio
from cryptography.hazmat.primitives.asymmetric import ec
from cryptography.hazmat.primitives import serialization
from typing import Tuple
import sys
import os

# Add parent directory to path to import client
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from client import VectorizerClient
from models import Vector


def generate_key_pair() -> Tuple[str, str]:
    """
    Generate an ECC P-256 key pair for encryption.
    In production, store the private key securely (e.g., in a key vault).

    Returns:
        Tuple of (public_key_pem, private_key_pem)
    """
    # Generate ECC key pair using P-256 curve
    private_key = ec.generate_private_key(ec.SECP256R1())

    # Export public key as PEM
    public_key_pem = private_key.public_key().public_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PublicFormat.SubjectPublicKeyInfo
    ).decode('utf-8')

    # Export private key as PEM
    private_key_pem = private_key.private_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PrivateFormat.PKCS8,
        encryption_algorithm=serialization.NoEncryption()
    ).decode('utf-8')

    return public_key_pem, private_key_pem


async def insert_encrypted_vectors():
    """Example: Insert encrypted vectors"""
    # Initialize client
    client = VectorizerClient(base_url='http://localhost:15002')

    # Generate encryption key pair
    public_key, private_key = generate_key_pair()
    print('Generated ECC P-256 key pair')
    print('Public Key:')
    print(public_key)
    print('\nWARNING: Keep your private key secure and never share it!\n')

    # Create collection
    collection_name = 'encrypted-docs'
    try:
        await client.create_collection(
            name=collection_name,
            dimension=384,  # For all-MiniLM-L6-v2
            metric='cosine'
        )
        print(f'Created collection: {collection_name}')
    except Exception as e:
        print(f'Collection {collection_name} already exists or error: {e}')

    # Insert vectors with encryption
    vectors = [
        Vector(
            id='secret-doc-1',
            data=[0.1] * 384,  # Dummy vector for example
            metadata={
                'text': 'This is sensitive information that will be encrypted',
                'category': 'confidential',
            },
            public_key=public_key  # Enable encryption
        ),
        Vector(
            id='secret-doc-2',
            data=[0.2] * 384,
            metadata={
                'text': 'Another confidential document with encrypted payload',
                'category': 'top-secret',
            },
            public_key=public_key
        ),
    ]

    print('\nInserting encrypted vectors...')
    result = await client.insert_texts(collection_name, vectors)
    print(f'Successfully inserted vectors: {result}')

    # Search for vectors (results will have encrypted payloads)
    print('\nSearching for similar vectors...')
    search_results = await client.search_vectors(
        collection_name,
        query='sensitive information',
        limit=5
    )

    print(f'Found {len(search_results)} results')
    print('\nNote: Payloads are encrypted in the database.')
    print('In production, you would decrypt them client-side using your private key.')

    await client.close()


async def upload_encrypted_file():
    """Example: Upload encrypted file"""
    client = VectorizerClient(base_url='http://localhost:15002')

    # Generate encryption key pair
    public_key, _ = generate_key_pair()

    collection_name = 'encrypted-files'
    try:
        await client.create_collection(
            name=collection_name,
            dimension=384,
            metric='cosine'
        )
    except Exception:
        pass  # Collection already exists

    # Upload file with encryption
    file_content = """
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
    """

    print('\nUploading encrypted file...')
    upload_result = await client.upload_file_content(
        content=file_content,
        filename='confidential.md',
        collection_name=collection_name,
        chunk_size=500,
        chunk_overlap=50,
        public_key=public_key,  # Enable encryption
        metadata={
            'classification': 'confidential',
            'department': 'security',
        }
    )

    print('File uploaded successfully:')
    print(f'- Chunks created: {upload_result.chunks_created}')
    print(f'- Vectors created: {upload_result.vectors_created}')
    print('- All chunk payloads are encrypted')

    await client.close()


async def qdrant_encrypted_upsert():
    """Example: Using Qdrant-compatible API with encryption"""
    client = VectorizerClient(base_url='http://localhost:15002')

    public_key, _ = generate_key_pair()

    collection_name = 'qdrant-encrypted'
    try:
        await client.qdrant_create_collection(
            collection_name,
            {
                'vectors': {
                    'size': 384,
                    'distance': 'Cosine',
                }
            }
        )
    except Exception:
        pass  # Collection exists

    # Upsert points with encryption
    points = [
        {
            'id': 'point-1',
            'vector': [0.1] * 384,
            'payload': {
                'text': 'Encrypted payload via Qdrant API',
                'sensitive': True,
            },
        },
        {
            'id': 'point-2',
            'vector': [0.2] * 384,
            'payload': {
                'text': 'Another encrypted document',
                'classification': 'restricted',
            },
        },
    ]

    print('\nUpserting encrypted points via Qdrant API...')
    await client.qdrant_upsert_points(collection_name, points, public_key=public_key)
    print('Points upserted with encryption enabled')

    await client.close()


def show_best_practices():
    """Best Practices for Production"""
    print('\n' + '=' * 60)
    print('ENCRYPTION BEST PRACTICES')
    print('=' * 60)
    print("""
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

7. PYTHON DEPENDENCIES
   - Install: pip install cryptography
   - Use cryptography.hazmat for key generation
   - ECDH with P-256 curve for key agreement
    """)


async def main():
    """Run all examples"""
    print('=' * 60)
    print('ECC-AES Payload Encryption Examples')
    print('=' * 60)

    try:
        # Example 1: Insert encrypted vectors
        print('\n--- Example 1: Insert Encrypted Vectors ---')
        await insert_encrypted_vectors()

        # Example 2: Upload encrypted file
        print('\n--- Example 2: Upload Encrypted File ---')
        await upload_encrypted_file()

        # Example 3: Qdrant API with encryption
        print('\n--- Example 3: Qdrant API with Encryption ---')
        await qdrant_encrypted_upsert()

        # Show best practices
        show_best_practices()

    except Exception as error:
        print(f'Error running examples: {error}')
        import traceback
        traceback.print_exc()


if __name__ == '__main__':
    asyncio.run(main())
