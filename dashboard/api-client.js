// Vectorizer API Client
// Conecta o dashboard com as rotas da API do backend

class VectorizerAPIClient {
    constructor(baseUrl = '') {
        this.baseUrl = baseUrl;
        this.defaultHeaders = {
            'Content-Type': 'application/json',
        };
    }

    // Helper method for making HTTP requests
    async request(endpoint, options = {}) {
        const url = `${this.baseUrl}${endpoint}`;
        const config = {
            headers: { ...this.defaultHeaders, ...options.headers },
            ...options
        };

        try {
            const response = await fetch(url, config);
            
            if (!response.ok) {
                throw new Error(`HTTP ${response.status}: ${response.statusText}`);
            }

            // Handle different content types
            const contentType = response.headers.get('content-type');
            if (contentType && contentType.includes('application/json')) {
                return await response.json();
            } else {
                return await response.text();
            }
        } catch (error) {
            console.error(`API request failed for ${endpoint}:`, error);
            throw error;
        }
    }

    // Health Check
    async healthCheck() {
        try {
            return await this.request('/health');
        } catch (error) {
            console.error('Health check failed:', error);
            return { status: 'error', message: error.message };
        }
    }

    // Test API connectivity
    async testConnection() {
        try {
            const health = await this.healthCheck();
            return { connected: true, health };
        } catch (error) {
            return { connected: false, error: error.message };
        }
    }

    // Collection Management
    async listCollections() {
        return await this.request('/collections');
    }

    async getCollection(collectionName) {
        return await this.request(`/collections/${encodeURIComponent(collectionName)}`);
    }

    async createCollection(collectionData) {
        return await this.request('/collections', {
            method: 'POST',
            body: JSON.stringify(collectionData)
        });
    }

    async deleteCollection(collectionName) {
        return await this.request(`/collections/${encodeURIComponent(collectionName)}`, {
            method: 'DELETE'
        });
    }

    // Vector Operations
    async listVectors(collectionName, limit = 100, offset = 0, minScore = 0.0) {
        const params = new URLSearchParams({
            limit: limit.toString(),
            offset: offset.toString(),
            min_score: minScore.toString()
        });
        const response = await this.request(`/collections/${encodeURIComponent(collectionName)}/vectors?${params}`);
        
        // Normalize response format
        if (response.vectors) {
            return response;
        } else if (Array.isArray(response)) {
            return { vectors: response, total: response.length };
        } else {
            return { vectors: [], total: 0 };
        }
    }

    async getVector(collectionName, vectorId) {
        return await this.request(`/collections/${encodeURIComponent(collectionName)}/vectors/${encodeURIComponent(vectorId)}`);
    }

    async insertVectors(collectionName, vectors) {
        return await this.request(`/collections/${encodeURIComponent(collectionName)}/vectors`, {
            method: 'POST',
            body: JSON.stringify(vectors)
        });
    }

    async deleteVector(collectionName, vectorId) {
        return await this.request(`/collections/${encodeURIComponent(collectionName)}/vectors/${encodeURIComponent(vectorId)}`, {
            method: 'DELETE'
        });
    }

    // Search Operations
    async searchVectors(collectionName, searchData) {
        const response = await this.request(`/collections/${encodeURIComponent(collectionName)}/search`, {
            method: 'POST',
            body: JSON.stringify(searchData)
        });
        
        // Normalize response format
        if (response.results) {
            return response;
        } else if (Array.isArray(response)) {
            return { results: response };
        } else {
            return { results: [] };
        }
    }

    async searchVectorsByText(collectionName, textSearchData) {
        const response = await this.request(`/collections/${encodeURIComponent(collectionName)}/search/text`, {
            method: 'POST',
            body: JSON.stringify(textSearchData)
        });
        
        // Normalize response format
        if (response.results) {
            return response;
        } else if (Array.isArray(response)) {
            return { results: response };
        } else {
            return { results: [] };
        }
    }

    async searchByFile(collectionName, fileSearchData) {
        return await this.request(`/collections/${encodeURIComponent(collectionName)}/search/file`, {
            method: 'POST',
            body: JSON.stringify(fileSearchData)
        });
    }

    // File Management
    async listFiles(collectionName, fileListData) {
        return await this.request(`/collections/${encodeURIComponent(collectionName)}/files`, {
            method: 'POST',
            body: JSON.stringify(fileListData)
        });
    }

    // Indexing Progress
    async getIndexingProgress() {
        return await this.request('/indexing/progress');
    }

    async updateIndexingProgress(progressData) {
        return await this.request('/indexing/progress', {
            method: 'POST',
            body: JSON.stringify(progressData)
        });
    }

    // Embedding Provider Management
    async listEmbeddingProviders() {
        return await this.request('/embedding/providers');
    }

    async setEmbeddingProvider(providerData) {
        return await this.request('/embedding/providers/set', {
            method: 'POST',
            body: JSON.stringify(providerData)
        });
    }

    // Utility methods for common operations
    async searchDocuments(collectionName, query, limit = 10) {
        const searchData = {
            query: query,
            limit: limit
        };
        return await this.searchVectorsByText(collectionName, searchData);
    }

    async getCollectionStats(collectionName) {
        const collection = await this.getCollection(collectionName);
        const vectors = await this.listVectors(collectionName, 1, 0); // Just get count

        return {
            ...collection,
            vector_count: vectors.total || 0,
            document_count: vectors.documents || 0
        };
    }


    // Batch operations
    async batchInsertVectors(collectionName, vectors) {
        const batchSize = 100;
        const results = [];
        
        for (let i = 0; i < vectors.length; i += batchSize) {
            const batch = vectors.slice(i, i + batchSize);
            const result = await this.insertVectors(collectionName, { vectors: batch });
            results.push(result);
        }
        
        return results;
    }

    // Error handling helpers
    handleError(error, context = '') {
        console.error(`API Error${context ? ` in ${context}` : ''}:`, error);
        
        if (error.message.includes('404')) {
            return { error: 'Resource not found', code: 404 };
        } else if (error.message.includes('400')) {
            return { error: 'Bad request', code: 400 };
        } else if (error.message.includes('500')) {
            return { error: 'Server error', code: 500 };
        } else {
            return { error: error.message, code: 'UNKNOWN' };
        }
    }
}

// Create a global instance
window.apiClient = new VectorizerAPIClient();

// Export for module systems
if (typeof module !== 'undefined' && module.exports) {
    module.exports = VectorizerAPIClient;
}
