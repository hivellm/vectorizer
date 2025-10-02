/**
 * Tests for validation utilities.
 */

import {
  validateNonEmptyString,
  validatePositiveNumber,
  validateNonNegativeNumber,
  validateNumberRange,
  validateNumberArray,
  validateBoolean,
  validateObject,
  validateDate,
  validateEnum,
  validateUrl,
  validateRequired,
  validateOptional,
} from '../../src/utils/validation';
import { ValidationError } from '../../src/exceptions';

describe('Validation Utilities', () => {
  describe('validateNonEmptyString', () => {
    it('should validate non-empty string', () => {
      expect(validateNonEmptyString('test', 'field')).toBe('test');
    });

    it('should throw error for empty string', () => {
      expect(() => validateNonEmptyString('', 'field')).toThrow(ValidationError);
      expect(() => validateNonEmptyString('', 'field')).toThrow('field must be a non-empty string');
    });

    it('should throw error for whitespace-only string', () => {
      expect(() => validateNonEmptyString('   ', 'field')).toThrow(ValidationError);
      expect(() => validateNonEmptyString('   ', 'field')).toThrow('field must be a non-empty string');
    });

    it('should throw error for non-string value', () => {
      expect(() => validateNonEmptyString(123, 'field')).toThrow(ValidationError);
      expect(() => validateNonEmptyString(123, 'field')).toThrow('field must be a non-empty string');
    });

    it('should throw error for null', () => {
      expect(() => validateNonEmptyString(null, 'field')).toThrow(ValidationError);
    });

    it('should throw error for undefined', () => {
      expect(() => validateNonEmptyString(undefined, 'field')).toThrow(ValidationError);
    });
  });

  describe('validatePositiveNumber', () => {
    it('should validate positive number', () => {
      expect(validatePositiveNumber(5, 'field')).toBe(5);
      expect(validatePositiveNumber(0.1, 'field')).toBe(0.1);
    });

    it('should throw error for zero', () => {
      expect(() => validatePositiveNumber(0, 'field')).toThrow(ValidationError);
      expect(() => validatePositiveNumber(0, 'field')).toThrow('field must be a positive number');
    });

    it('should throw error for negative number', () => {
      expect(() => validatePositiveNumber(-1, 'field')).toThrow(ValidationError);
      expect(() => validatePositiveNumber(-1, 'field')).toThrow('field must be a positive number');
    });

    it('should throw error for NaN', () => {
      expect(() => validatePositiveNumber(NaN, 'field')).toThrow(ValidationError);
      expect(() => validatePositiveNumber(NaN, 'field')).toThrow('field must be a positive number');
    });

    it('should throw error for non-number', () => {
      expect(() => validatePositiveNumber('5', 'field')).toThrow(ValidationError);
      expect(() => validatePositiveNumber('5', 'field')).toThrow('field must be a positive number');
    });
  });

  describe('validateNonNegativeNumber', () => {
    it('should validate non-negative number', () => {
      expect(validateNonNegativeNumber(5, 'field')).toBe(5);
      expect(validateNonNegativeNumber(0, 'field')).toBe(0);
      expect(validateNonNegativeNumber(0.1, 'field')).toBe(0.1);
    });

    it('should throw error for negative number', () => {
      expect(() => validateNonNegativeNumber(-1, 'field')).toThrow(ValidationError);
      expect(() => validateNonNegativeNumber(-1, 'field')).toThrow('field must be a non-negative number');
    });

    it('should throw error for NaN', () => {
      expect(() => validateNonNegativeNumber(NaN, 'field')).toThrow(ValidationError);
      expect(() => validateNonNegativeNumber(NaN, 'field')).toThrow('field must be a non-negative number');
    });

    it('should throw error for non-number', () => {
      expect(() => validateNonNegativeNumber('5', 'field')).toThrow(ValidationError);
      expect(() => validateNonNegativeNumber('5', 'field')).toThrow('field must be a non-negative number');
    });
  });

  describe('validateNumberRange', () => {
    it('should validate number within range', () => {
      expect(validateNumberRange(5, 'field', 0, 10)).toBe(5);
      expect(validateNumberRange(0, 'field', 0, 10)).toBe(0);
      expect(validateNumberRange(10, 'field', 0, 10)).toBe(10);
    });

    it('should throw error for number below range', () => {
      expect(() => validateNumberRange(-1, 'field', 0, 10)).toThrow(ValidationError);
      expect(() => validateNumberRange(-1, 'field', 0, 10)).toThrow('field must be between 0 and 10');
    });

    it('should throw error for number above range', () => {
      expect(() => validateNumberRange(11, 'field', 0, 10)).toThrow(ValidationError);
      expect(() => validateNumberRange(11, 'field', 0, 10)).toThrow('field must be between 0 and 10');
    });

    it('should throw error for invalid number', () => {
      expect(() => validateNumberRange(NaN, 'field', 0, 10)).toThrow(ValidationError);
      expect(() => validateNumberRange(NaN, 'field', 0, 10)).toThrow('field must be a non-negative number');
    });
  });

  describe('validateNumberArray', () => {
    it('should validate number array', () => {
      const result = validateNumberArray([1, 2, 3], 'field');
      expect(result).toEqual([1, 2, 3]);
    });

    it('should throw error for empty array', () => {
      expect(() => validateNumberArray([], 'field')).toThrow(ValidationError);
      expect(() => validateNumberArray([], 'field')).toThrow('field must be a non-empty array');
    });

    it('should throw error for non-array', () => {
      expect(() => validateNumberArray('not-array', 'field')).toThrow(ValidationError);
      expect(() => validateNumberArray('not-array', 'field')).toThrow('field must be a non-empty array');
    });

    it('should throw error for array with invalid numbers', () => {
      expect(() => validateNumberArray([1, 'invalid', 3], 'field')).toThrow(ValidationError);
      expect(() => validateNumberArray([1, 'invalid', 3], 'field')).toThrow('field must contain only valid numbers');
    });

    it('should throw error for array with NaN', () => {
      expect(() => validateNumberArray([1, NaN, 3], 'field')).toThrow(ValidationError);
      expect(() => validateNumberArray([1, NaN, 3], 'field')).toThrow('field must contain only valid numbers');
    });

    it('should validate large array', () => {
      const largeArray = Array.from({ length: 1000 }, (_, i) => i);
      const result = validateNumberArray(largeArray, 'field');
      expect(result).toEqual(largeArray);
    });
  });

  describe('validateBoolean', () => {
    it('should validate boolean values', () => {
      expect(validateBoolean(true, 'field')).toBe(true);
      expect(validateBoolean(false, 'field')).toBe(false);
    });

    it('should throw error for non-boolean', () => {
      expect(() => validateBoolean('true', 'field')).toThrow(ValidationError);
      expect(() => validateBoolean('true', 'field')).toThrow('field must be a boolean');
    });

    it('should throw error for number', () => {
      expect(() => validateBoolean(1, 'field')).toThrow(ValidationError);
      expect(() => validateBoolean(1, 'field')).toThrow('field must be a boolean');
    });

    it('should throw error for null', () => {
      expect(() => validateBoolean(null, 'field')).toThrow(ValidationError);
      expect(() => validateBoolean(null, 'field')).toThrow('field must be a boolean');
    });
  });

  describe('validateObject', () => {
    it('should validate object', () => {
      const obj = { key: 'value' };
      expect(validateObject(obj, 'field')).toBe(obj);
    });

    it('should validate empty object', () => {
      const obj = {};
      expect(validateObject(obj, 'field')).toBe(obj);
    });

    it('should throw error for null', () => {
      expect(() => validateObject(null, 'field')).toThrow(ValidationError);
      expect(() => validateObject(null, 'field')).toThrow('field must be an object');
    });

    it('should throw error for array', () => {
      expect(() => validateObject([], 'field')).toThrow(ValidationError);
      expect(() => validateObject([], 'field')).toThrow('field must be an object');
    });

    it('should throw error for primitive', () => {
      expect(() => validateObject('string', 'field')).toThrow(ValidationError);
      expect(() => validateObject('string', 'field')).toThrow('field must be an object');
    });
  });

  describe('validateDate', () => {
    it('should validate Date object', () => {
      const date = new Date('2023-01-01');
      expect(validateDate(date, 'field')).toBe(date);
    });

    it('should validate date string', () => {
      const dateString = '2023-01-01';
      const result = validateDate(dateString, 'field');
      expect(result).toBeInstanceOf(Date);
      expect(result.getTime()).toBe(new Date(dateString).getTime());
    });

    it('should validate timestamp number', () => {
      const timestamp = 1672531200000; // 2023-01-01
      const result = validateDate(timestamp, 'field');
      expect(result).toBeInstanceOf(Date);
      expect(result.getTime()).toBe(timestamp);
    });

    it('should throw error for invalid date string', () => {
      expect(() => validateDate('invalid-date', 'field')).toThrow(ValidationError);
      expect(() => validateDate('invalid-date', 'field')).toThrow('field must be a valid date');
    });

    it('should throw error for invalid timestamp', () => {
      expect(() => validateDate(NaN, 'field')).toThrow(ValidationError);
      expect(() => validateDate(NaN, 'field')).toThrow('field must be a valid date');
    });

    it('should throw error for non-date value', () => {
      expect(() => validateDate({}, 'field')).toThrow(ValidationError);
      expect(() => validateDate({}, 'field')).toThrow('field must be a valid date');
    });
  });

  describe('validateEnum', () => {
    const validValues = ['option1', 'option2', 'option3'] as const;

    it('should validate valid enum value', () => {
      expect(validateEnum('option1', 'field', validValues)).toBe('option1');
      expect(validateEnum('option2', 'field', validValues)).toBe('option2');
      expect(validateEnum('option3', 'field', validValues)).toBe('option3');
    });

    it('should throw error for invalid enum value', () => {
      expect(() => validateEnum('invalid', 'field', validValues)).toThrow(ValidationError);
      expect(() => validateEnum('invalid', 'field', validValues)).toThrow('field must be one of: option1, option2, option3');
    });

    it('should throw error for case-sensitive mismatch', () => {
      expect(() => validateEnum('Option1', 'field', validValues)).toThrow(ValidationError);
      expect(() => validateEnum('Option1', 'field', validValues)).toThrow('field must be one of: option1, option2, option3');
    });
  });

  describe('validateUrl', () => {
    it('should validate valid URL', () => {
      expect(validateUrl('https://example.com', 'field')).toBe('https://example.com');
      expect(validateUrl('http://localhost:8080', 'field')).toBe('http://localhost:8080');
      expect(validateUrl('ws://localhost:8080/ws', 'field')).toBe('ws://localhost:8080/ws');
    });

    it('should throw error for invalid URL', () => {
      expect(() => validateUrl('not-a-url', 'field')).toThrow(ValidationError);
      expect(() => validateUrl('not-a-url', 'field')).toThrow('field must be a valid URL');
    });

    it('should throw error for empty string', () => {
      expect(() => validateUrl('', 'field')).toThrow(ValidationError);
      expect(() => validateUrl('', 'field')).toThrow('field must be a non-empty string');
    });

    it('should throw error for non-string', () => {
      expect(() => validateUrl(123, 'field')).toThrow(ValidationError);
      expect(() => validateUrl(123, 'field')).toThrow('field must be a non-empty string');
    });
  });

  describe('validateRequired', () => {
    it('should validate non-null, non-undefined values', () => {
      expect(validateRequired('value', 'field')).toBe('value');
      expect(validateRequired(0, 'field')).toBe(0);
      expect(validateRequired(false, 'field')).toBe(false);
      expect(validateRequired([], 'field')).toEqual([]);
      expect(validateRequired({}, 'field')).toEqual({});
    });

    it('should throw error for null', () => {
      expect(() => validateRequired(null, 'field')).toThrow(ValidationError);
      expect(() => validateRequired(null, 'field')).toThrow('field is required');
    });

    it('should throw error for undefined', () => {
      expect(() => validateRequired(undefined, 'field')).toThrow(ValidationError);
      expect(() => validateRequired(undefined, 'field')).toThrow('field is required');
    });
  });

  describe('validateOptional', () => {
    it('should return undefined for null', () => {
      expect(validateOptional(null, 'field', validateNonEmptyString)).toBeUndefined();
    });

    it('should return undefined for undefined', () => {
      expect(validateOptional(undefined, 'field', validateNonEmptyString)).toBeUndefined();
    });

    it('should validate non-null, non-undefined values', () => {
      expect(validateOptional('value', 'field', validateNonEmptyString)).toBe('value');
    });

    it('should throw error for invalid non-null value', () => {
      expect(() => validateOptional('', 'field', validateNonEmptyString)).toThrow(ValidationError);
      expect(() => validateOptional('', 'field', validateNonEmptyString)).toThrow('field must be a non-empty string');
    });
  });
});










