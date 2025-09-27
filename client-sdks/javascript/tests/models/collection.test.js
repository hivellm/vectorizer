/**
 * Tests for Collection model validation.
 */

import { validateCollection, validateCreateCollectionRequest } from '../../src/models/collection.js';
import { ValidationError } from '../../src/exceptions/index.js';

describe('Collection Model Validation', () => {
  describe('validateCollection', () => {
    it('should validate a correct collection', () => {
      const collection = {
        name: 'test-collection',
        dimension: 384,
        similarity_metric: 'cosine',
        description: 'Test collection'
      };

      expect(() => validateCollection(collection)).not.toThrow();
    });

    it('should validate collection without description', () => {
      const collection = {
        name: 'test-collection',
        dimension: 384,
        similarity_metric: 'cosine'
      };

      expect(() => validateCollection(collection)).not.toThrow();
    });

    it('should throw error for missing name', () => {
      const collection = {
        dimension: 384,
        similarity_metric: 'cosine'
      };

      expect(() => validateCollection(collection)).toThrow(ValidationError);
      expect(() => validateCollection(collection)).toThrow('Collection name must be a non-empty string');
    });

    it('should throw error for empty name', () => {
      const collection = {
        name: '',
        dimension: 384,
        similarity_metric: 'cosine'
      };

      expect(() => validateCollection(collection)).toThrow(ValidationError);
      expect(() => validateCollection(collection)).toThrow('Collection name must be a non-empty string');
    });

    it('should throw error for non-string name', () => {
      const collection = {
        name: 123,
        dimension: 384,
        similarity_metric: 'cosine'
      };

      expect(() => validateCollection(collection)).toThrow(ValidationError);
      expect(() => validateCollection(collection)).toThrow('Collection name must be a non-empty string');
    });

    it('should throw error for missing dimension', () => {
      const collection = {
        name: 'test-collection',
        similarity_metric: 'cosine'
      };

      expect(() => validateCollection(collection)).toThrow(ValidationError);
      expect(() => validateCollection(collection)).toThrow('Dimension must be a positive number');
    });

    it('should throw error for zero dimension', () => {
      const collection = {
        name: 'test-collection',
        dimension: 0,
        similarity_metric: 'cosine'
      };

      expect(() => validateCollection(collection)).toThrow(ValidationError);
      expect(() => validateCollection(collection)).toThrow('Dimension must be a positive number');
    });

    it('should throw error for negative dimension', () => {
      const collection = {
        name: 'test-collection',
        dimension: -1,
        similarity_metric: 'cosine'
      };

      expect(() => validateCollection(collection)).toThrow(ValidationError);
      expect(() => validateCollection(collection)).toThrow('Dimension must be a positive number');
    });

    it('should throw error for non-number dimension', () => {
      const collection = {
        name: 'test-collection',
        dimension: '384',
        similarity_metric: 'cosine'
      };

      expect(() => validateCollection(collection)).toThrow(ValidationError);
      expect(() => validateCollection(collection)).toThrow('Dimension must be a positive number');
    });

    it('should throw error for missing similarity metric', () => {
      const collection = {
        name: 'test-collection',
        dimension: 384
      };

      expect(() => validateCollection(collection)).toThrow(ValidationError);
      expect(() => validateCollection(collection)).toThrow('Invalid similarity metric');
    });

    it('should throw error for invalid similarity metric', () => {
      const collection = {
        name: 'test-collection',
        dimension: 384,
        similarity_metric: 'invalid'
      };

      expect(() => validateCollection(collection)).toThrow(ValidationError);
      expect(() => validateCollection(collection)).toThrow('Invalid similarity metric');
    });

    it('should validate all valid similarity metrics', () => {
      const validMetrics = ['cosine', 'euclidean', 'dot_product'];
      
      validMetrics.forEach(metric => {
        const collection = {
          name: 'test-collection',
          dimension: 384,
          similarity_metric: metric
        };

        expect(() => validateCollection(collection)).not.toThrow();
      });
    });

    it('should validate large dimension', () => {
      const collection = {
        name: 'large-collection',
        dimension: 4096,
        similarity_metric: 'cosine'
      };

      expect(() => validateCollection(collection)).not.toThrow();
    });
  });

  describe('validateCreateCollectionRequest', () => {
    it('should validate a correct create collection request', () => {
      const request = {
        name: 'test-collection',
        dimension: 384,
        similarity_metric: 'cosine',
        description: 'Test collection'
      };

      expect(() => validateCreateCollectionRequest(request)).not.toThrow();
    });

    it('should validate request without optional fields', () => {
      const request = {
        name: 'test-collection',
        dimension: 384
      };

      expect(() => validateCreateCollectionRequest(request)).not.toThrow();
    });

    it('should throw error for missing name', () => {
      const request = {
        dimension: 384
      };

      expect(() => validateCreateCollectionRequest(request)).toThrow(ValidationError);
      expect(() => validateCreateCollectionRequest(request)).toThrow('Collection name must be a non-empty string');
    });

    it('should throw error for empty name', () => {
      const request = {
        name: '',
        dimension: 384
      };

      expect(() => validateCreateCollectionRequest(request)).toThrow(ValidationError);
      expect(() => validateCreateCollectionRequest(request)).toThrow('Collection name must be a non-empty string');
    });

    it('should throw error for missing dimension', () => {
      const request = {
        name: 'test-collection'
      };

      expect(() => validateCreateCollectionRequest(request)).toThrow(ValidationError);
      expect(() => validateCreateCollectionRequest(request)).toThrow('Dimension must be a positive number');
    });

    it('should throw error for invalid dimension', () => {
      const request = {
        name: 'test-collection',
        dimension: -1
      };

      expect(() => validateCreateCollectionRequest(request)).toThrow(ValidationError);
      expect(() => validateCreateCollectionRequest(request)).toThrow('Dimension must be a positive number');
    });

    it('should throw error for invalid similarity metric', () => {
      const request = {
        name: 'test-collection',
        dimension: 384,
        similarity_metric: 'invalid'
      };

      expect(() => validateCreateCollectionRequest(request)).toThrow(ValidationError);
      expect(() => validateCreateCollectionRequest(request)).toThrow('Invalid similarity metric');
    });

    it('should validate all valid similarity metrics', () => {
      const validMetrics = ['cosine', 'euclidean', 'dot_product'];
      
      validMetrics.forEach(metric => {
        const request = {
          name: 'test-collection',
          dimension: 384,
          similarity_metric: metric
        };

        expect(() => validateCreateCollectionRequest(request)).not.toThrow();
      });
    });
  });
});
