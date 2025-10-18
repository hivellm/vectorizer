/**
 * Batch operation models for the Hive Vectorizer JavaScript SDK.
 * 
 * This module contains data models for batch operations including
 * batch insert, search, update, and delete operations.
 */

/**
 * Text request for batch operations
 */
export class BatchTextRequest {
  constructor(id, text, metadata = null) {
    this.id = id;
    this.text = text;
    this.metadata = metadata;
    
    this.validate();
  }
  
  validate() {
    if (!this.id || typeof this.id !== 'string') {
      throw new Error('Text ID must be a non-empty string');
    }
    if (!this.text || typeof this.text !== 'string') {
      throw new Error('Text content must be a non-empty string');
    }
  }
  
  toJSON() {
    return {
      id: this.id,
      text: this.text,
      metadata: this.metadata
    };
  }
}

/**
 * Configuration for batch operations
 */
export class BatchConfig {
  constructor(options = {}) {
    this.max_batch_size = options.max_batch_size || null;
    this.parallel_workers = options.parallel_workers || null;
    this.atomic = options.atomic || null;
  }
  
  toJSON() {
    return {
      max_batch_size: this.max_batch_size,
      parallel_workers: this.parallel_workers,
      atomic: this.atomic
    };
  }
}

/**
 * Request for batch text insertion
 */
export class BatchInsertRequest {
  constructor(texts, config = null) {
    this.texts = texts;
    this.config = config;
    
    this.validate();
  }
  
  validate() {
    if (!Array.isArray(this.texts) || this.texts.length === 0) {
      throw new Error('Texts must be a non-empty array');
    }
    this.texts.forEach((text, index) => {
      if (!(text instanceof BatchTextRequest)) {
        throw new Error(`Text at index ${index} must be a BatchTextRequest instance`);
      }
    });
  }
  
  toJSON() {
    return {
      texts: this.texts.map(text => text.toJSON()),
      config: this.config ? this.config.toJSON() : null
    };
  }
}

/**
 * Response for batch operations
 */
export class BatchResponse {
  constructor(data) {
    this.success = data.success;
    this.collection = data.collection;
    this.operation = data.operation;
    this.total_operations = data.total_operations;
    this.successful_operations = data.successful_operations;
    this.failed_operations = data.failed_operations;
    this.duration_ms = data.duration_ms;
    this.errors = data.errors || [];
  }
}

/**
 * Search query for batch operations
 */
export class BatchSearchQuery {
  constructor(query, limit = null, score_threshold = null) {
    this.query = query;
    this.limit = limit;
    this.score_threshold = score_threshold;
    
    this.validate();
  }
  
  validate() {
    if (!this.query || typeof this.query !== 'string') {
      throw new Error('Query must be a non-empty string');
    }
  }
  
  toJSON() {
    return {
      query: this.query,
      limit: this.limit,
      score_threshold: this.score_threshold
    };
  }
}

/**
 * Request for batch search operations
 */
export class BatchSearchRequest {
  constructor(queries, config = null) {
    this.queries = queries;
    this.config = config;
    
    this.validate();
  }
  
  validate() {
    if (!Array.isArray(this.queries) || this.queries.length === 0) {
      throw new Error('Queries must be a non-empty array');
    }
    this.queries.forEach((query, index) => {
      if (!(query instanceof BatchSearchQuery)) {
        throw new Error(`Query at index ${index} must be a BatchSearchQuery instance`);
      }
    });
  }
  
  toJSON() {
    return {
      queries: this.queries.map(query => query.toJSON()),
      config: this.config ? this.config.toJSON() : null
    };
  }
}

/**
 * Response for batch search operations
 */
export class BatchSearchResponse {
  constructor(data) {
    this.success = data.success;
    this.collection = data.collection;
    this.total_queries = data.total_queries;
    this.successful_queries = data.successful_queries;
    this.failed_queries = data.failed_queries;
    this.duration_ms = data.duration_ms;
    this.results = data.results || [];
    this.errors = data.errors || [];
  }
}

/**
 * Vector update for batch operations
 */
export class BatchVectorUpdate {
  constructor(id, data = null, metadata = null) {
    this.id = id;
    this.data = data;
    this.metadata = metadata;
    
    this.validate();
  }
  
  validate() {
    if (!this.id || typeof this.id !== 'string') {
      throw new Error('Vector ID must be a non-empty string');
    }
  }
  
  toJSON() {
    return {
      id: this.id,
      data: this.data,
      metadata: this.metadata
    };
  }
}

/**
 * Request for batch vector updates
 */
export class BatchUpdateRequest {
  constructor(updates, config = null) {
    this.updates = updates;
    this.config = config;
    
    this.validate();
  }
  
  validate() {
    if (!Array.isArray(this.updates) || this.updates.length === 0) {
      throw new Error('Updates must be a non-empty array');
    }
    this.updates.forEach((update, index) => {
      if (!(update instanceof BatchVectorUpdate)) {
        throw new Error(`Update at index ${index} must be a BatchVectorUpdate instance`);
      }
    });
  }
  
  toJSON() {
    return {
      updates: this.updates.map(update => update.toJSON()),
      config: this.config ? this.config.toJSON() : null
    };
  }
}

/**
 * Request for batch vector deletion
 */
export class BatchDeleteRequest {
  constructor(vector_ids, config = null) {
    this.vector_ids = vector_ids;
    this.config = config;
    
    this.validate();
  }
  
  validate() {
    if (!Array.isArray(this.vector_ids) || this.vector_ids.length === 0) {
      throw new Error('Vector IDs must be a non-empty array');
    }
    this.vector_ids.forEach((id, index) => {
      if (typeof id !== 'string') {
        throw new Error(`Vector ID at index ${index} must be a string`);
      }
    });
  }
  
  toJSON() {
    return {
      vector_ids: this.vector_ids,
      config: this.config ? this.config.toJSON() : null
    };
  }
}
