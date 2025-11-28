/**
 * Hybrid search models for combining dense and sparse vectors.
 */

/**
 * Validates sparse vector data.
 * @param {Object} sparse - Sparse vector object
 * @param {number[]} sparse.indices - Non-zero indices
 * @param {number[]} sparse.values - Values at corresponding indices
 * @throws {Error} If sparse vector is invalid
 */
export function validateSparseVector(sparse) {
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
 * @param {Object} request - Hybrid search request
 * @param {string} request.collection - Collection name
 * @param {string} request.query - Text query for dense vector search
 * @param {Object} [request.query_sparse] - Optional sparse vector query
 * @param {number} [request.alpha=0.7] - Alpha parameter for blending (0.0-1.0)
 * @param {string} [request.algorithm='rrf'] - Scoring algorithm: 'rrf', 'weighted', or 'alpha'
 * @param {number} [request.dense_k=20] - Number of dense results to retrieve
 * @param {number} [request.sparse_k=20] - Number of sparse results to retrieve
 * @param {number} [request.final_k=10] - Final number of results to return
 * @throws {Error} If request is invalid
 */
export function validateHybridSearchRequest(request) {
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

