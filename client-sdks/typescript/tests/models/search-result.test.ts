/**
 * Tests for SearchResult model validation.
 */

import { validateSearchResult, validateSearchResponse } from '../../src/models/search-result';
import { ValidationError } from '../../src/exceptions';

describe('SearchResult Model Validation', () => {
  describe('validateSearchResult', () => {
    it('should validate a correct search result', () => {
      const result = {
        id: 'test-id',
        score: 0.95,
        data: [0.1, 0.2, 0.3, 0.4],
        metadata: { source: 'test.pdf' }
      };

      expect(() => validateSearchResult(result)).not.toThrow();
    });

    it('should validate search result without metadata', () => {
      const result = {
        id: 'test-id',
        score: 0.95,
        data: [0.1, 0.2, 0.3]
      };

      expect(() => validateSearchResult(result)).not.toThrow();
    });

    it('should throw error for missing ID', () => {
      const result = {
        score: 0.95,
        data: [0.1, 0.2, 0.3]
      } as any;

      expect(() => validateSearchResult(result)).toThrow(ValidationError);
      expect(() => validateSearchResult(result)).toThrow('Search result ID must be a non-empty string');
    });

    it('should throw error for empty ID', () => {
      const result = {
        id: '',
        score: 0.95,
        data: [0.1, 0.2, 0.3]
      };

      expect(() => validateSearchResult(result)).toThrow(ValidationError);
      expect(() => validateSearchResult(result)).toThrow('Search result ID must be a non-empty string');
    });

    it('should throw error for non-string ID', () => {
      const result = {
        id: 123,
        score: 0.95,
        data: [0.1, 0.2, 0.3]
      } as any;

      expect(() => validateSearchResult(result)).toThrow(ValidationError);
      expect(() => validateSearchResult(result)).toThrow('Search result ID must be a non-empty string');
    });

    it('should throw error for missing score', () => {
      const result = {
        id: 'test-id',
        data: [0.1, 0.2, 0.3]
      } as any;

      expect(() => validateSearchResult(result)).toThrow(ValidationError);
      expect(() => validateSearchResult(result)).toThrow('Search result score must be a valid number');
    });

    it('should throw error for non-number score', () => {
      const result = {
        id: 'test-id',
        score: '0.95',
        data: [0.1, 0.2, 0.3]
      } as any;

      expect(() => validateSearchResult(result)).toThrow(ValidationError);
      expect(() => validateSearchResult(result)).toThrow('Search result score must be a valid number');
    });

    it('should throw error for NaN score', () => {
      const result = {
        id: 'test-id',
        score: NaN,
        data: [0.1, 0.2, 0.3]
      };

      expect(() => validateSearchResult(result)).toThrow(ValidationError);
      expect(() => validateSearchResult(result)).toThrow('Search result score must be a valid number');
    });

    it('should validate negative score', () => {
      const result = {
        id: 'test-id',
        score: -0.1,
        data: [0.1, 0.2, 0.3]
      };

      expect(() => validateSearchResult(result)).not.toThrow();
    });

    it('should validate score greater than 1', () => {
      const result = {
        id: 'test-id',
        score: 1.5,
        data: [0.1, 0.2, 0.3]
      };

      expect(() => validateSearchResult(result)).not.toThrow();
    });

    it('should throw error for missing data', () => {
      const result = {
        id: 'test-id',
        score: 0.95
      } as any;

      expect(() => validateSearchResult(result)).toThrow(ValidationError);
      expect(() => validateSearchResult(result)).toThrow('Search result data must be a non-empty array');
    });

    it('should throw error for empty data array', () => {
      const result = {
        id: 'test-id',
        score: 0.95,
        data: []
      };

      expect(() => validateSearchResult(result)).toThrow(ValidationError);
      expect(() => validateSearchResult(result)).toThrow('Search result data must be a non-empty array');
    });

    it('should throw error for non-array data', () => {
      const result = {
        id: 'test-id',
        score: 0.95,
        data: 'not-an-array'
      } as any;

      expect(() => validateSearchResult(result)).toThrow(ValidationError);
      expect(() => validateSearchResult(result)).toThrow('Search result data must be a non-empty array');
    });

    it('should throw error for invalid number in data', () => {
      const result = {
        id: 'test-id',
        score: 0.95,
        data: [0.1, 'invalid', 0.3]
      } as any;

      expect(() => validateSearchResult(result)).toThrow(ValidationError);
      expect(() => validateSearchResult(result)).toThrow('Search result data must contain only valid numbers');
    });

    it('should validate large search result', () => {
      const result = {
        id: 'large-result',
        score: 0.95,
        data: Array.from({ length: 1000 }, (_, i) => i * 0.001)
      };

      expect(() => validateSearchResult(result)).not.toThrow();
    });
  });

  describe('validateSearchResponse', () => {
    it('should validate a correct search response', () => {
      const response = {
        results: [
          {
            id: 'result-1',
            score: 0.95,
            data: [0.1, 0.2, 0.3]
          },
          {
            id: 'result-2',
            score: 0.90,
            data: [0.4, 0.5, 0.6]
          }
        ],
        total: 2
      };

      expect(() => validateSearchResponse(response)).not.toThrow();
    });

    it('should validate empty search response', () => {
      const response = {
        results: [],
        total: 0
      };

      expect(() => validateSearchResponse(response)).not.toThrow();
    });

    it('should throw error for missing results', () => {
      const response = {
        total: 0
      } as any;

      expect(() => validateSearchResponse(response)).toThrow(ValidationError);
      expect(() => validateSearchResponse(response)).toThrow('Search response results must be an array');
    });

    it('should throw error for non-array results', () => {
      const response = {
        results: 'not-an-array',
        total: 0
      } as any;

      expect(() => validateSearchResponse(response)).toThrow(ValidationError);
      expect(() => validateSearchResponse(response)).toThrow('Search response results must be an array');
    });

    it('should throw error for missing total', () => {
      const response = {
        results: []
      } as any;

      expect(() => validateSearchResponse(response)).toThrow(ValidationError);
      expect(() => validateSearchResponse(response)).toThrow('Search response total must be a non-negative number');
    });

    it('should throw error for negative total', () => {
      const response = {
        results: [],
        total: -1
      };

      expect(() => validateSearchResponse(response)).toThrow(ValidationError);
      expect(() => validateSearchResponse(response)).toThrow('Search response total must be a non-negative number');
    });

    it('should throw error for non-number total', () => {
      const response = {
        results: [],
        total: '0'
      } as any;

      expect(() => validateSearchResponse(response)).toThrow(ValidationError);
      expect(() => validateSearchResponse(response)).toThrow('Search response total must be a non-negative number');
    });

    it('should throw error for invalid result in results array', () => {
      const response = {
        results: [
          {
            id: 'valid-result',
            score: 0.95,
            data: [0.1, 0.2, 0.3]
          },
          {
            id: '', // Invalid result
            score: 0.90,
            data: [0.4, 0.5, 0.6]
          }
        ],
        total: 2
      };

      expect(() => validateSearchResponse(response)).toThrow(ValidationError);
      expect(() => validateSearchResponse(response)).toThrow('Invalid search result at index 1');
    });

    it('should validate large search response', () => {
      const response = {
        results: Array.from({ length: 100 }, (_, i) => ({
          id: `result-${i}`,
          score: 0.95 - i * 0.001,
          data: Array.from({ length: 100 }, (_, j) => j * 0.001)
        })),
        total: 100
      };

      expect(() => validateSearchResponse(response)).not.toThrow();
    });
  });
});
