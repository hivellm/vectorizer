/**
 * Collection info model for detailed collection information.
 */

export interface CollectionInfo {
  /** Collection name */
  name: string;
  /** Vector dimension */
  dimension: number;
  /** Similarity metric used for search */
  similarity_metric: string;
  /** Optional description */
  description?: string;
  /** Creation timestamp */
  created_at?: Date;
  /** Last update timestamp */
  updated_at?: Date;
  /** Number of vectors in the collection */
  vector_count: number;
  /** Collection size in bytes */
  size_bytes?: number;
  /** Collection statistics */
  statistics?: {
    /** Average vector length */
    avg_vector_length?: number;
    /** Maximum vector length */
    max_vector_length?: number;
    /** Minimum vector length */
    min_vector_length?: number;
    /** Index build time in milliseconds */
    index_build_time_ms?: number;
    /** Last indexing timestamp */
    last_indexed_at?: Date;
  };
}

export interface DatabaseStats {
  /** Total number of collections */
  total_collections: number;
  /** Total number of vectors */
  total_vectors: number;
  /** Total database size in bytes */
  total_size_bytes: number;
  /** Database uptime in seconds */
  uptime_seconds: number;
  /** Memory usage in bytes */
  memory_usage_bytes?: number;
  /** CPU usage percentage */
  cpu_usage_percent?: number;
  /** Collections information */
  collections: CollectionInfo[];
}

/**
 * Validates collection info data.
 * 
 * @param info - Collection info to validate
 * @throws {Error} If collection info data is invalid
 */
export function validateCollectionInfo(info: CollectionInfo): void {
  if (!info.name || typeof info.name !== 'string') {
    throw new Error('Collection info name must be a non-empty string');
  }
  
  if (typeof info.dimension !== 'number' || info.dimension <= 0) {
    throw new Error('Collection info dimension must be a positive number');
  }
  
  if (!info.similarity_metric || typeof info.similarity_metric !== 'string') {
    throw new Error('Collection info similarity_metric must be a non-empty string');
  }
  
  if (typeof info.vector_count !== 'number' || info.vector_count < 0) {
    throw new Error('Collection info vector_count must be a non-negative number');
  }
  
  if (info.size_bytes !== undefined) {
    if (typeof info.size_bytes !== 'number' || info.size_bytes < 0) {
      throw new Error('Collection info size_bytes must be a non-negative number');
    }
  }
  
  if (info.statistics) {
    if (info.statistics.avg_vector_length !== undefined) {
      if (typeof info.statistics.avg_vector_length !== 'number' || info.statistics.avg_vector_length < 0) {
        throw new Error('Collection info statistics avg_vector_length must be a non-negative number');
      }
    }
    
    if (info.statistics.max_vector_length !== undefined) {
      if (typeof info.statistics.max_vector_length !== 'number' || info.statistics.max_vector_length < 0) {
        throw new Error('Collection info statistics max_vector_length must be a non-negative number');
      }
    }
    
    if (info.statistics.min_vector_length !== undefined) {
      if (typeof info.statistics.min_vector_length !== 'number' || info.statistics.min_vector_length < 0) {
        throw new Error('Collection info statistics min_vector_length must be a non-negative number');
      }
    }
    
    if (info.statistics.index_build_time_ms !== undefined) {
      if (typeof info.statistics.index_build_time_ms !== 'number' || info.statistics.index_build_time_ms < 0) {
        throw new Error('Collection info statistics index_build_time_ms must be a non-negative number');
      }
    }
  }
}

/**
 * Validates database stats data.
 * 
 * @param stats - Database stats to validate
 * @throws {Error} If database stats data is invalid
 */
export function validateDatabaseStats(stats: DatabaseStats): void {
  if (typeof stats.total_collections !== 'number' || stats.total_collections < 0) {
    throw new Error('Database stats total_collections must be a non-negative number');
  }
  
  if (typeof stats.total_vectors !== 'number' || stats.total_vectors < 0) {
    throw new Error('Database stats total_vectors must be a non-negative number');
  }
  
  if (typeof stats.total_size_bytes !== 'number' || stats.total_size_bytes < 0) {
    throw new Error('Database stats total_size_bytes must be a non-negative number');
  }
  
  if (typeof stats.uptime_seconds !== 'number' || stats.uptime_seconds < 0) {
    throw new Error('Database stats uptime_seconds must be a non-negative number');
  }
  
  if (stats.memory_usage_bytes !== undefined) {
    if (typeof stats.memory_usage_bytes !== 'number' || stats.memory_usage_bytes < 0) {
      throw new Error('Database stats memory_usage_bytes must be a non-negative number');
    }
  }
  
  if (stats.cpu_usage_percent !== undefined) {
    if (typeof stats.cpu_usage_percent !== 'number' || stats.cpu_usage_percent < 0 || stats.cpu_usage_percent > 100) {
      throw new Error('Database stats cpu_usage_percent must be a number between 0 and 100');
    }
  }
  
  if (!Array.isArray(stats.collections)) {
    throw new Error('Database stats collections must be an array');
  }
  
  stats.collections.forEach((collection, index) => {
    try {
      validateCollectionInfo(collection);
    } catch (error) {
      throw new Error(`Invalid collection info at index ${index}: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  });
}
