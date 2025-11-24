/**
 * Unit tests for validator utility functions
 */

import { describe, it, expect } from 'vitest';
import {
  validateCollectionName,
  validateDimension,
  validateVectorId,
  validateEmail,
  validateUrl,
  validateSimilarityThreshold,
} from '../validators';

describe('validateCollectionName', () => {
  it('should validate correct collection names', () => {
    expect(validateCollectionName('test-collection')).toEqual({ valid: true });
    expect(validateCollectionName('collection_123')).toEqual({ valid: true });
    expect(validateCollectionName('myCollection')).toEqual({ valid: true });
  });

  it('should reject empty names', () => {
    const result = validateCollectionName('');
    expect(result.valid).toBe(false);
    expect(result.error).toBeDefined();
  });

  it('should reject names with spaces', () => {
    const result = validateCollectionName('test collection');
    expect(result.valid).toBe(false);
    expect(result.error).toBeDefined();
  });

  it('should reject names that are too long', () => {
    const longName = 'a'.repeat(256);
    const result = validateCollectionName(longName);
    expect(result.valid).toBe(false);
    expect(result.error).toBeDefined();
  });
});

describe('validateDimension', () => {
  it('should validate correct dimensions', () => {
    expect(validateDimension(128)).toEqual({ valid: true });
    expect(validateDimension(512)).toEqual({ valid: true });
    expect(validateDimension(1024)).toEqual({ valid: true });
  });

  it('should validate dimension as string', () => {
    expect(validateDimension('128')).toEqual({ valid: true });
    expect(validateDimension('512')).toEqual({ valid: true });
  });

  it('should reject invalid dimensions', () => {
    expect(validateDimension(0).valid).toBe(false);
    expect(validateDimension(-1).valid).toBe(false);
    expect(validateDimension('invalid').valid).toBe(false);
  });
});

describe('validateVectorId', () => {
  it('should validate correct vector IDs', () => {
    expect(validateVectorId('vector-123')).toEqual({ valid: true });
    expect(validateVectorId('id_456')).toEqual({ valid: true });
  });

  it('should reject empty IDs', () => {
    const result = validateVectorId('');
    expect(result.valid).toBe(false);
    expect(result.error).toBeDefined();
  });
});

describe('validateEmail', () => {
  it('should validate correct emails', () => {
    expect(validateEmail('test@example.com')).toEqual({ valid: true });
    expect(validateEmail('user.name@domain.co.uk')).toEqual({ valid: true });
  });

  it('should reject invalid emails', () => {
    expect(validateEmail('invalid').valid).toBe(false);
    expect(validateEmail('@example.com').valid).toBe(false);
    expect(validateEmail('test@').valid).toBe(false);
  });
});

describe('validateUrl', () => {
  it('should validate correct URLs', () => {
    expect(validateUrl('https://example.com')).toEqual({ valid: true });
    expect(validateUrl('http://localhost:8080')).toEqual({ valid: true });
  });

  it('should reject invalid URLs', () => {
    expect(validateUrl('not-a-url').valid).toBe(false);
    expect(validateUrl('just text').valid).toBe(false);
    expect(validateUrl('://invalid').valid).toBe(false);
  });
});

describe('validateSimilarityThreshold', () => {
  it('should validate correct thresholds', () => {
    expect(validateSimilarityThreshold(0.5)).toEqual({ valid: true });
    expect(validateSimilarityThreshold(0.0)).toEqual({ valid: true });
    expect(validateSimilarityThreshold(1.0)).toEqual({ valid: true });
  });

  it('should validate threshold as string', () => {
    expect(validateSimilarityThreshold('0.5')).toEqual({ valid: true });
    expect(validateSimilarityThreshold('0.75')).toEqual({ valid: true });
  });

  it('should reject invalid thresholds', () => {
    expect(validateSimilarityThreshold(-0.1).valid).toBe(false);
    expect(validateSimilarityThreshold(1.1).valid).toBe(false);
    expect(validateSimilarityThreshold('invalid').valid).toBe(false);
  });
});

