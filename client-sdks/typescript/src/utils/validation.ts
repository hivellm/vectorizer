/**
 * Validation utilities for the Hive Vectorizer SDK.
 */

import { ValidationError } from '../exceptions';

/**
 * Validates that a value is a non-empty string.
 */
export function validateNonEmptyString(value: unknown, fieldName: string): string {
  if (typeof value !== 'string' || value.trim().length === 0) {
    throw new ValidationError(`${fieldName} must be a non-empty string`);
  }
  return value;
}

/**
 * Validates that a value is a positive number.
 */
export function validatePositiveNumber(value: unknown, fieldName: string): number {
  if (typeof value !== 'number' || value <= 0 || isNaN(value)) {
    throw new ValidationError(`${fieldName} must be a positive number`);
  }
  return value;
}

/**
 * Validates that a value is a non-negative number.
 */
export function validateNonNegativeNumber(value: unknown, fieldName: string): number {
  if (typeof value !== 'number' || value < 0 || isNaN(value)) {
    throw new ValidationError(`${fieldName} must be a non-negative number`);
  }
  return value;
}

/**
 * Validates that a value is a valid number between min and max.
 */
export function validateNumberRange(value: unknown, fieldName: string, min: number, max: number): number {
  if (typeof value !== 'number' || isNaN(value)) {
    throw new ValidationError(`${fieldName} must be a non-negative number`);
  }
  if (value < min || value > max) {
    throw new ValidationError(`${fieldName} must be between ${min} and ${max}`);
  }
  return value;
}

/**
 * Validates that a value is a valid array of numbers.
 */
export function validateNumberArray(value: unknown, fieldName: string): number[] {
  if (!Array.isArray(value) || value.length === 0) {
    throw new ValidationError(`${fieldName} must be a non-empty array`);
  }
  
  if (!value.every(x => typeof x === 'number' && !isNaN(x))) {
    throw new ValidationError(`${fieldName} must contain only valid numbers`);
  }
  
  return value;
}

/**
 * Validates that a value is a valid boolean.
 */
export function validateBoolean(value: unknown, fieldName: string): boolean {
  if (typeof value !== 'boolean') {
    throw new ValidationError(`${fieldName} must be a boolean`);
  }
  return value;
}

/**
 * Validates that a value is a valid object.
 */
export function validateObject(value: unknown, fieldName: string): Record<string, unknown> {
  if (typeof value !== 'object' || value === null || Array.isArray(value)) {
    throw new ValidationError(`${fieldName} must be an object`);
  }
  return value as Record<string, unknown>;
}

/**
 * Validates that a value is a valid Date or can be converted to one.
 */
export function validateDate(value: unknown, fieldName: string): Date {
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
export function validateEnum<T>(value: unknown, fieldName: string, allowedValues: readonly T[]): T {
  if (!allowedValues.includes(value as T)) {
    throw new ValidationError(`${fieldName} must be one of: ${allowedValues.join(', ')}`);
  }
  return value as T;
}

/**
 * Validates that a value is a valid URL.
 */
export function validateUrl(value: unknown, fieldName: string): string {
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
export function validateRequired<T>(value: T | null | undefined, fieldName: string): T {
  if (value === null || value === undefined) {
    throw new ValidationError(`${fieldName} is required`);
  }
  return value;
}

/**
 * Validates that a value is optional (can be null, undefined, or the expected type).
 */
export function validateOptional<T>(
  value: T | null | undefined,
  fieldName: string,
  validator: (val: T, name: string) => T
): T | undefined {
  if (value === null || value === undefined) {
    return undefined;
  }
  return validator(value, fieldName);
}
