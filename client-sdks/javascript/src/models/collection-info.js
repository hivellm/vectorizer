/**
 * Collection info model for detailed collection information.
 */

import { ValidationError } from '../exceptions/index.js';

/**
 * Validates collection info data.
 * 
 * @param {Object} info - Collection info to validate
 * @throws {ValidationError} If collection info data is invalid
 */
export function validateCollectionInfo(info) {
  if (!info.name || typeof info.name !== 'string') {
    throw new ValidationError('Collection info name must be a non-empty string');
  }
  
  if (typeof info.dimension !== 'number' || info.dimension <= 0) {
    throw new ValidationError('Collection info dimension must be a positive number');
  }
  
  if (!info.similarity_metric || typeof info.similarity_metric !== 'string') {
    throw new ValidationError('Collection info similarity_metric must be a non-empty string');
  }
  
  if (typeof info.vector_count !== 'number' || info.vector_count < 0) {
    throw new ValidationError('Collection info vector_count must be a non-negative number');
  }
  
  if (info.size_bytes !== undefined) {
    if (typeof info.size_bytes !== 'number' || info.size_bytes < 0) {
      throw new ValidationError('Collection info size_bytes must be a non-negative number');
    }
  }
  
  if (info.statistics) {
    if (info.statistics.avg_vector_length !== undefined) {
      if (typeof info.statistics.avg_vector_length !== 'number' || info.statistics.avg_vector_length < 0) {
        throw new ValidationError('Collection info statistics avg_vector_length must be a non-negative number');
      }
    }
    
    if (info.statistics.max_vector_length !== undefined) {
      if (typeof info.statistics.max_vector_length !== 'number' || info.statistics.max_vector_length < 0) {
        throw new ValidationError('Collection info statistics max_vector_length must be a non-negative number');
      }
    }
    
    if (info.statistics.min_vector_length !== undefined) {
      if (typeof info.statistics.min_vector_length !== 'number' || info.statistics.min_vector_length < 0) {
        throw new ValidationError('Collection info statistics min_vector_length must be a non-negative number');
      }
    }
    
    if (info.statistics.index_build_time_ms !== undefined) {
      if (typeof info.statistics.index_build_time_ms !== 'number' || info.statistics.index_build_time_ms < 0) {
        throw new ValidationError('Collection info statistics index_build_time_ms must be a non-negative number');
      }
    }
  }
}

/**
 * Validates database stats data.
 * 
 * @param {Object} stats - Database stats to validate
 * @throws {ValidationError} If database stats data is invalid
 */
export function validateDatabaseStats(stats) {
  if (typeof stats.total_collections !== 'number' || stats.total_collections < 0) {
    throw new ValidationError('Database stats total_collections must be a non-negative number');
  }
  
  if (typeof stats.total_vectors !== 'number' || stats.total_vectors < 0) {
    throw new ValidationError('Database stats total_vectors must be a non-negative number');
  }
  
  if (typeof stats.total_size_bytes !== 'number' || stats.total_size_bytes < 0) {
    throw new ValidationError('Database stats total_size_bytes must be a non-negative number');
  }
  
  if (typeof stats.uptime_seconds !== 'number' || stats.uptime_seconds < 0) {
    throw new ValidationError('Database stats uptime_seconds must be a non-negative number');
  }
  
  if (stats.memory_usage_bytes !== undefined) {
    if (typeof stats.memory_usage_bytes !== 'number' || stats.memory_usage_bytes < 0) {
      throw new ValidationError('Database stats memory_usage_bytes must be a non-negative number');
    }
  }
  
  if (stats.cpu_usage_percent !== undefined) {
    if (typeof stats.cpu_usage_percent !== 'number' || stats.cpu_usage_percent < 0 || stats.cpu_usage_percent > 100) {
      throw new ValidationError('Database stats cpu_usage_percent must be a number between 0 and 100');
    }
  }
  
  if (!Array.isArray(stats.collections)) {
    throw new ValidationError('Database stats collections must be an array');
  }
  
  stats.collections.forEach((collection, index) => {
    try {
      validateCollectionInfo(collection);
    } catch (error) {
      throw new ValidationError(`Invalid collection info at index ${index}: ${error.message}`);
    }
  });
}
