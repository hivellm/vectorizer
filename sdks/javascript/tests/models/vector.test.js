/**
 * Tests for Vector model validation.
 */

import { validateVector, validateCreateVectorRequest } from '../../src/models/vector.js';
import { ValidationError } from '../../src/exceptions/index.js';

describe('Vector Model Validation', () => {
  describe('validateVector', () => {
    it('should validate a correct vector', () => {
      const vector = {
        id: 'test-id',
        data: [0.1, 0.2, 0.3, 0.4],
        metadata: { source: 'test.pdf' }
      };

      expect(() => validateVector(vector)).not.toThrow();
    });

    it('should validate vector without metadata', () => {
      const vector = {
        id: 'test-id',
        data: [0.1, 0.2, 0.3]
      };

      expect(() => validateVector(vector)).not.toThrow();
    });

    it('should throw error for missing ID', () => {
      const vector = {
        data: [0.1, 0.2, 0.3]
      };

      expect(() => validateVector(vector)).toThrow(ValidationError);
      expect(() => validateVector(vector)).toThrow('Vector ID must be a non-empty string');
    });

    it('should throw error for empty ID', () => {
      const vector = {
        id: '',
        data: [0.1, 0.2, 0.3]
      };

      expect(() => validateVector(vector)).toThrow(ValidationError);
      expect(() => validateVector(vector)).toThrow('Vector ID must be a non-empty string');
    });

    it('should throw error for non-string ID', () => {
      const vector = {
        id: 123,
        data: [0.1, 0.2, 0.3]
      };

      expect(() => validateVector(vector)).toThrow(ValidationError);
      expect(() => validateVector(vector)).toThrow('Vector ID must be a non-empty string');
    });

    it('should throw error for missing data', () => {
      const vector = {
        id: 'test-id'
      };

      expect(() => validateVector(vector)).toThrow(ValidationError);
      expect(() => validateVector(vector)).toThrow('Vector data must be a non-empty array');
    });

    it('should throw error for empty data array', () => {
      const vector = {
        id: 'test-id',
        data: []
      };

      expect(() => validateVector(vector)).toThrow(ValidationError);
      expect(() => validateVector(vector)).toThrow('Vector data must be a non-empty array');
    });

    it('should throw error for non-array data', () => {
      const vector = {
        id: 'test-id',
        data: 'not-an-array'
      };

      expect(() => validateVector(vector)).toThrow(ValidationError);
      expect(() => validateVector(vector)).toThrow('Vector data must be a non-empty array');
    });

    it('should throw error for invalid number in data', () => {
      const vector = {
        id: 'test-id',
        data: [0.1, 'invalid', 0.3]
      };

      expect(() => validateVector(vector)).toThrow(ValidationError);
      expect(() => validateVector(vector)).toThrow('Vector data must contain only valid finite numbers');
    });

    it('should throw error for NaN in data', () => {
      const vector = {
        id: 'test-id',
        data: [0.1, NaN, 0.3]
      };

      expect(() => validateVector(vector)).toThrow(ValidationError);
      expect(() => validateVector(vector)).toThrow('Vector data must contain only valid finite numbers');
    });

    it('should throw error for Infinity in data', () => {
      const vector = {
        id: 'test-id',
        data: [0.1, Infinity, 0.3]
      };

      expect(() => validateVector(vector)).toThrow(ValidationError);
      expect(() => validateVector(vector)).toThrow('Vector data must contain only valid finite numbers');
    });

    it('should validate large vector', () => {
      const vector = {
        id: 'large-vector',
        data: Array.from({ length: 1000 }, (_, i) => i * 0.001)
      };

      expect(() => validateVector(vector)).not.toThrow();
    });
  });

  describe('validateCreateVectorRequest', () => {
    it('should validate a correct create vector request', () => {
      const request = {
        data: [0.1, 0.2, 0.3, 0.4],
        metadata: { source: 'test.pdf' }
      };

      expect(() => validateCreateVectorRequest(request)).not.toThrow();
    });

    it('should validate request without metadata', () => {
      const request = {
        data: [0.1, 0.2, 0.3]
      };

      expect(() => validateCreateVectorRequest(request)).not.toThrow();
    });

    it('should throw error for missing data', () => {
      const request = {};

      expect(() => validateCreateVectorRequest(request)).toThrow(ValidationError);
      expect(() => validateCreateVectorRequest(request)).toThrow('Vector data must be a non-empty array');
    });

    it('should throw error for empty data array', () => {
      const request = {
        data: []
      };

      expect(() => validateCreateVectorRequest(request)).toThrow(ValidationError);
      expect(() => validateCreateVectorRequest(request)).toThrow('Vector data must be a non-empty array');
    });

    it('should throw error for non-array data', () => {
      const request = {
        data: 'not-an-array'
      };

      expect(() => validateCreateVectorRequest(request)).toThrow(ValidationError);
      expect(() => validateCreateVectorRequest(request)).toThrow('Vector data must be a non-empty array');
    });

    it('should throw error for invalid number in data', () => {
      const request = {
        data: [0.1, 'invalid', 0.3]
      };

      expect(() => validateCreateVectorRequest(request)).toThrow(ValidationError);
      expect(() => validateCreateVectorRequest(request)).toThrow('Vector data must contain only valid finite numbers');
    });

    it('should validate large vector request', () => {
      const request = {
        data: Array.from({ length: 1000 }, (_, i) => i * 0.001),
        metadata: { dimension: 1000 }
      };

      expect(() => validateCreateVectorRequest(request)).not.toThrow();
    });
  });
});
