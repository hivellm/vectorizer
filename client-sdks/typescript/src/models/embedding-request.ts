/**
 * Embedding request model for text embedding generation.
 */

export interface EmbeddingRequest {
  /** Text to embed */
  text: string;
  /** Optional model to use for embedding */
  model?: string;
  /** Optional parameters for embedding generation */
  parameters?: {
    /** Maximum sequence length */
    max_length?: number;
    /** Whether to normalize the embedding */
    normalize?: boolean;
    /** Optional prefix for the text */
    prefix?: string;
  };
}

export interface EmbeddingResponse {
  /** Generated embedding vector */
  embedding: number[];
  /** Model used for embedding */
  model: string;
  /** Text that was embedded */
  text: string;
  /** Embedding generation parameters */
  parameters?: {
    max_length?: number;
    normalize?: boolean;
    prefix?: string;
  };
}

/**
 * Validates embedding request data.
 * 
 * @param request - Embedding request to validate
 * @throws {Error} If embedding request data is invalid
 */
export function validateEmbeddingRequest(request: EmbeddingRequest): void {
  if (!request.text || typeof request.text !== 'string') {
    throw new Error('Embedding request text must be a non-empty string');
  }
  
  if (request.model && typeof request.model !== 'string') {
    throw new Error('Embedding request model must be a string');
  }
  
  if (request.parameters) {
    if (request.parameters.max_length !== undefined) {
      if (typeof request.parameters.max_length !== 'number' || request.parameters.max_length <= 0) {
        throw new Error('Max length must be a positive number');
      }
    }
    
    if (request.parameters.normalize !== undefined) {
      if (typeof request.parameters.normalize !== 'boolean') {
        throw new Error('Normalize must be a boolean');
      }
    }
    
    if (request.parameters.prefix !== undefined) {
      if (typeof request.parameters.prefix !== 'string') {
        throw new Error('Prefix must be a string');
      }
    }
  }
}

/**
 * Validates embedding response data.
 * 
 * @param response - Embedding response to validate
 * @throws {Error} If embedding response data is invalid
 */
export function validateEmbeddingResponse(response: EmbeddingResponse): void {
  if (!Array.isArray(response.embedding) || response.embedding.length === 0) {
    throw new Error('Embedding response embedding must be a non-empty array');
  }
  
  if (!response.embedding.every(x => typeof x === 'number' && !isNaN(x))) {
    throw new Error('Embedding response embedding must contain only valid numbers');
  }
  
  if (!response.model || typeof response.model !== 'string') {
    throw new Error('Embedding response model must be a non-empty string');
  }
  
  if (!response.text || typeof response.text !== 'string') {
    throw new Error('Embedding response text must be a non-empty string');
  }
}
