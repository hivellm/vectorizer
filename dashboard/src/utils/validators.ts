/**
 * Utility functions for validation
 */

/**
 * Validate collection name
 */
export function validateCollectionName(name: string): { valid: boolean; error?: string } {
  if (!name || name.trim().length === 0) {
    return { valid: false, error: 'Collection name is required' };
  }

  if (name.length > 100) {
    return { valid: false, error: 'Collection name must be less than 100 characters' };
  }

  // Allow alphanumeric, hyphens, underscores, and dots
  const nameRegex = /^[a-zA-Z0-9._-]+$/;
  if (!nameRegex.test(name)) {
    return {
      valid: false,
      error: 'Collection name can only contain letters, numbers, dots, hyphens, and underscores',
    };
  }

  return { valid: true };
}

/**
 * Validate dimension
 */
export function validateDimension(dimension: number | string): { valid: boolean; error?: string } {
  const dim = typeof dimension === 'string' ? parseInt(dimension, 10) : dimension;

  if (isNaN(dim) || dim <= 0) {
    return { valid: false, error: 'Dimension must be a positive number' };
  }

  if (dim > 10000) {
    return { valid: false, error: 'Dimension must be less than 10000' };
  }

  return { valid: true };
}

/**
 * Validate vector ID
 */
export function validateVectorId(id: string): { valid: boolean; error?: string } {
  if (!id || id.trim().length === 0) {
    return { valid: false, error: 'Vector ID is required' };
  }

  if (id.length > 500) {
    return { valid: false, error: 'Vector ID must be less than 500 characters' };
  }

  return { valid: true };
}

/**
 * Validate email format
 */
export function validateEmail(email: string): { valid: boolean; error?: string } {
  if (!email || email.trim().length === 0) {
    return { valid: false, error: 'Email is required' };
  }

  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  if (!emailRegex.test(email)) {
    return { valid: false, error: 'Invalid email format' };
  }

  return { valid: true };
}

/**
 * Validate URL format
 */
export function validateUrl(url: string): { valid: boolean; error?: string } {
  if (!url || url.trim().length === 0) {
    return { valid: false, error: 'URL is required' };
  }

  try {
    new URL(url);
    return { valid: true };
  } catch {
    return { valid: false, error: 'Invalid URL format' };
  }
}

/**
 * Validate similarity threshold (0-1)
 */
export function validateSimilarityThreshold(threshold: number | string): { valid: boolean; error?: string } {
  const thresh = typeof threshold === 'string' ? parseFloat(threshold) : threshold;

  if (isNaN(thresh)) {
    return { valid: false, error: 'Threshold must be a number' };
  }

  if (thresh < 0 || thresh > 1) {
    return { valid: false, error: 'Threshold must be between 0 and 1' };
  }

  return { valid: true };
}

