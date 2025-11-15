/**
 * Hybrid search models for combining dense and sparse vectors.
 */

export interface SparseVector {
    /** Non-zero indices */
    indices: number[];
    /** Values at corresponding indices */
    values: number[];
}

export interface HybridSearchRequest {
    /** Collection name */
    collection: string;
    /** Text query for dense vector search */
    query: string;
    /** Optional sparse vector query */
    query_sparse?: SparseVector;
    /** Alpha parameter for blending (0.0-1.0) */
    alpha?: number;
    /** Scoring algorithm: 'rrf', 'weighted', or 'alpha' */
    algorithm?: 'rrf' | 'weighted' | 'alpha';
    /** Number of dense results to retrieve */
    dense_k?: number;
    /** Number of sparse results to retrieve */
    sparse_k?: number;
    /** Final number of results to return */
    final_k?: number;
}

export interface HybridSearchResult {
    /** Result ID */
    id: string;
    /** Similarity score */
    score: number;
    /** Optional vector data */
    vector?: number[];
    /** Optional payload data */
    payload?: Record<string, unknown>;
}

export interface HybridSearchResponse {
    /** Search results */
    results: HybridSearchResult[];
    /** Query text */
    query: string;
    /** Optional sparse query */
    query_sparse?: {
        indices: number[];
        values: number[];
    };
    /** Alpha parameter used */
    alpha: number;
    /** Algorithm used */
    algorithm: string;
    /** Duration in milliseconds */
    duration_ms?: number;
}

/**
 * Validates sparse vector data.
 */
export function validateSparseVector(sparse: SparseVector): void {
    if (!Array.isArray(sparse.indices) || sparse.indices.length === 0) {
        throw new Error('Sparse vector indices must be a non-empty array');
    }
    if (!Array.isArray(sparse.values) || sparse.values.length === 0) {
        throw new Error('Sparse vector values must be a non-empty array');
    }
    if (sparse.indices.length !== sparse.values.length) {
        throw new Error('Sparse vector indices and values must have the same length');
    }
    if (!sparse.indices.every(idx => typeof idx === 'number' && idx >= 0)) {
        throw new Error('Sparse vector indices must be non-negative numbers');
    }
    if (!sparse.values.every(val => typeof val === 'number' && !isNaN(val) && isFinite(val))) {
        throw new Error('Sparse vector values must be valid finite numbers');
    }
}

/**
 * Validates hybrid search request.
 */
export function validateHybridSearchRequest(request: HybridSearchRequest): void {
    if (!request.collection || typeof request.collection !== 'string') {
        throw new Error('Collection name must be a non-empty string');
    }
    if (!request.query || typeof request.query !== 'string') {
        throw new Error('Query must be a non-empty string');
    }
    if (request.query_sparse) {
        validateSparseVector(request.query_sparse);
    }
    if (request.alpha !== undefined) {
        if (typeof request.alpha !== 'number' || request.alpha < 0 || request.alpha > 1) {
            throw new Error('Alpha must be a number between 0.0 and 1.0');
        }
    }
    if (request.algorithm !== undefined) {
        if (!['rrf', 'weighted', 'alpha'].includes(request.algorithm)) {
            throw new Error("Algorithm must be 'rrf', 'weighted', or 'alpha'");
        }
    }
    if (request.dense_k !== undefined && (typeof request.dense_k !== 'number' || request.dense_k <= 0)) {
        throw new Error('dense_k must be a positive number');
    }
    if (request.sparse_k !== undefined && (typeof request.sparse_k !== 'number' || request.sparse_k <= 0)) {
        throw new Error('sparse_k must be a positive number');
    }
    if (request.final_k !== undefined && (typeof request.final_k !== 'number' || request.final_k <= 0)) {
        throw new Error('final_k must be a positive number');
    }
}

