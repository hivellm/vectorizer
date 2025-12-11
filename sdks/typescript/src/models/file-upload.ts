/**
 * File upload models for the Hive Vectorizer SDK.
 */

/**
 * Request to upload a file for indexing.
 */
export interface FileUploadRequest {
  /** Target collection name */
  collectionName: string;

  /** Chunk size in characters (uses server default if not specified) */
  chunkSize?: number;

  /** Chunk overlap in characters (uses server default if not specified) */
  chunkOverlap?: number;

  /** Additional metadata to attach to all chunks */
  metadata?: Record<string, unknown>;
}

/**
 * Response from file upload operation.
 */
export interface FileUploadResponse {
  /** Whether the upload was successful */
  success: boolean;

  /** Original filename */
  filename: string;

  /** Target collection */
  collection_name: string;

  /** Number of chunks created from the file */
  chunks_created: number;

  /** Number of vectors created and stored */
  vectors_created: number;

  /** File size in bytes */
  file_size: number;

  /** Detected language/file type */
  language: string;

  /** Processing time in milliseconds */
  processing_time_ms: number;
}

/**
 * Configuration for file uploads.
 */
export interface FileUploadConfig {
  /** Maximum file size in bytes */
  max_file_size: number;

  /** Maximum file size in megabytes */
  max_file_size_mb: number;

  /** List of allowed file extensions */
  allowed_extensions: string[];

  /** Whether binary files are rejected */
  reject_binary: boolean;

  /** Default chunk size in characters */
  default_chunk_size: number;

  /** Default chunk overlap in characters */
  default_chunk_overlap: number;
}

/**
 * Options for uploading a file.
 */
export interface UploadFileOptions {
  /** Chunk size in characters */
  chunkSize?: number;

  /** Chunk overlap in characters */
  chunkOverlap?: number;

  /** Additional metadata to attach to all chunks */
  metadata?: Record<string, unknown>;

  /** Optional ECC public key for payload encryption (PEM/hex/base64 format) */
  publicKey?: string;
}
