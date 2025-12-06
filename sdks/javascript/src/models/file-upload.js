/**
 * File upload models for the Hive Vectorizer SDK.
 */

/**
 * @typedef {Object} FileUploadRequest
 * @property {string} collectionName - Target collection name
 * @property {number} [chunkSize] - Chunk size in characters
 * @property {number} [chunkOverlap] - Chunk overlap in characters
 * @property {Object} [metadata] - Additional metadata to attach to all chunks
 */

/**
 * @typedef {Object} FileUploadResponse
 * @property {boolean} success - Whether the upload was successful
 * @property {string} filename - Original filename
 * @property {string} collection_name - Target collection
 * @property {number} chunks_created - Number of chunks created from the file
 * @property {number} vectors_created - Number of vectors created and stored
 * @property {number} file_size - File size in bytes
 * @property {string} language - Detected language/file type
 * @property {number} processing_time_ms - Processing time in milliseconds
 */

/**
 * @typedef {Object} FileUploadConfig
 * @property {number} max_file_size - Maximum file size in bytes
 * @property {number} max_file_size_mb - Maximum file size in megabytes
 * @property {string[]} allowed_extensions - List of allowed file extensions
 * @property {boolean} reject_binary - Whether binary files are rejected
 * @property {number} default_chunk_size - Default chunk size in characters
 * @property {number} default_chunk_overlap - Default chunk overlap in characters
 */

/**
 * @typedef {Object} UploadFileOptions
 * @property {number} [chunkSize] - Chunk size in characters
 * @property {number} [chunkOverlap] - Chunk overlap in characters
 * @property {Object} [metadata] - Additional metadata to attach to all chunks
 */

/**
 * Validate a FileUploadResponse object.
 * @param {Object} response - Response to validate
 * @throws {Error} If validation fails
 */
export function validateFileUploadResponse(response) {
  if (!response || typeof response !== 'object') {
    throw new Error('Invalid file upload response');
  }
  if (typeof response.success !== 'boolean') {
    throw new Error('File upload response must have a boolean success field');
  }
  if (typeof response.filename !== 'string') {
    throw new Error('File upload response must have a string filename field');
  }
  if (typeof response.collection_name !== 'string') {
    throw new Error('File upload response must have a string collection_name field');
  }
}

/**
 * Validate a FileUploadConfig object.
 * @param {Object} config - Config to validate
 * @throws {Error} If validation fails
 */
export function validateFileUploadConfig(config) {
  if (!config || typeof config !== 'object') {
    throw new Error('Invalid file upload config');
  }
  if (typeof config.max_file_size !== 'number') {
    throw new Error('File upload config must have a number max_file_size field');
  }
  if (!Array.isArray(config.allowed_extensions)) {
    throw new Error('File upload config must have an array allowed_extensions field');
  }
}
