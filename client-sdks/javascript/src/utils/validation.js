/**
 * Validation utilities for the Hive Vectorizer SDK.
 */

import { ValidationError } from '../exceptions/index.js';

/**
 * Validates that a value is a non-empty string.
 */
export function validateNonEmptyString(value, fieldName) {
  if (typeof value !== 'string' || value.trim().length === 0) {
    throw new ValidationError(`${fieldName} must be a non-empty string`);
  }
  return value;
}

/**
 * Validates that a value is a positive number.
 */
export function validatePositiveNumber(value, fieldName) {
  if (typeof value !== 'number' || value <= 0 || isNaN(value)) {
    throw new ValidationError(`${fieldName} must be a positive number`);
  }
  return value;
}

/**
 * Validates that a value is a non-negative number.
 */
export function validateNonNegativeNumber(value, fieldName) {
  if (typeof value !== 'number' || value < 0 || isNaN(value)) {
    throw new ValidationError(`${fieldName} must be a non-negative number`);
  }
  return value;
}

/**
 * Validates that a value is a valid number between min and max.
 */
export function validateNumberRange(value, fieldName, min, max) {
  if (typeof value !== 'number' || isNaN(value)) {
    throw new ValidationError(`${fieldName} must be a valid number`);
  }
  if (value < min || value > max) {
    throw new ValidationError(`${fieldName} must be between ${min} and ${max}`);
  }
  return value;
}

/**
 * Validates that a value is a valid array of numbers.
 */
export function validateNumberArray(value, fieldName) {
  if (!Array.isArray(value) || value.length === 0) {
    throw new ValidationError(`${fieldName} must be a non-empty array`);
  }
  
  if (!value.every(x => typeof x === 'number' && isFinite(x))) {
    throw new ValidationError(`${fieldName} must contain only valid finite numbers`);
  }
  
  return value;
}

/**
 * Validates that a value is a valid boolean.
 */
export function validateBoolean(value, fieldName) {
  if (typeof value !== 'boolean') {
    throw new ValidationError(`${fieldName} must be a boolean`);
  }
  return value;
}

/**
 * Validates that a value is a valid object.
 */
export function validateObject(value, fieldName) {
  if (typeof value !== 'object' || value === null || Array.isArray(value)) {
    throw new ValidationError(`${fieldName} must be an object`);
  }
  return value;
}

/**
 * Validates that a value is a valid Date or can be converted to one.
 */
export function validateDate(value, fieldName) {
  if (value instanceof Date) {
    if (isNaN(value.getTime())) {
      throw new ValidationError(`${fieldName} must be a valid date`);
    }
    return value;
  }
  
  if (typeof value === 'string' || typeof value === 'number') {
    const date = new Date(value);
    if (isNaN(date.getTime())) {
      throw new ValidationError(`${fieldName} must be a valid date`);
    }
    return date;
  }
  
  throw new ValidationError(`${fieldName} must be a valid date`);
}

/**
 * Validates that a value is one of the allowed values.
 */
export function validateEnum(value, fieldName, allowedValues) {
  if (!allowedValues.includes(value)) {
    throw new ValidationError(`${fieldName} must be one of: ${allowedValues.join(', ')}`);
  }
  return value;
}

/**
 * Validates that a value is a valid URL.
 */
export function validateUrl(value, fieldName) {
  const url = validateNonEmptyString(value, fieldName);
  try {
    new URL(url);
    return url;
  } catch {
    throw new ValidationError(`${fieldName} must be a valid URL`);
  }
}

/**
 * Validates that a value is not null or undefined.
 */
export function validateRequired(value, fieldName) {
  if (value === null || value === undefined) {
    throw new ValidationError(`${fieldName} is required`);
  }
  return value;
}

/**
 * Validates that a value is optional (can be null, undefined, or the expected type).
 */
export function validateOptional(value, fieldName, validator) {
  if (value === null || value === undefined) {
    return undefined;
  }
  return validator(value, fieldName);
}
