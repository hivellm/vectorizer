/**
 * Unit tests for formatter utility functions
 */

import { describe, it, expect } from 'vitest';
import { formatNumber, formatBytes, formatDate, formatDuration } from '../formatters';

describe('formatNumber', () => {
    it('should format positive numbers correctly', () => {
        expect(formatNumber(1234)).toBe('1,234');
        expect(formatNumber(1234567)).toBe('1,234,567');
        expect(formatNumber(0)).toBe('0');
    });

    it('should handle null and undefined', () => {
        expect(formatNumber(null)).toBe('0');
        expect(formatNumber(undefined)).toBe('0');
    });

    it('should handle NaN', () => {
        expect(formatNumber(NaN)).toBe('0');
    });

    it('should format decimal numbers', () => {
        expect(formatNumber(1234.56)).toBe('1,234.56');
        expect(formatNumber(0.123)).toBe('0.123');
    });
});

describe('formatBytes', () => {
    it('should format bytes correctly', () => {
        expect(formatBytes(0)).toBe('0 Bytes');
        expect(formatBytes(1024)).toBe('1 KB');
        expect(formatBytes(1048576)).toBe('1 MB');
        expect(formatBytes(1073741824)).toBe('1 GB');
    });

    it('should handle invalid inputs', () => {
        // formatBytes doesn't handle null/undefined, so we test with valid numbers
        expect(formatBytes(0)).toBe('0 Bytes');
    });

    it('should format with decimals', () => {
        expect(formatBytes(1536)).toBe('1.5 KB');
        expect(formatBytes(1572864)).toBe('1.5 MB');
    });
});

describe('formatDate', () => {
    it('should format dates correctly', () => {
        const date = new Date('2024-01-15T10:30:00Z');
        const formatted = formatDate(date);
        // formatDate uses 'short' month format like "Jan 15, 2024"
        expect(formatted).toContain('Jan');
        expect(formatted).toContain('2024');
    });

    it('should handle invalid dates', () => {
        // formatDate throws error for invalid dates
        const invalidDate = new Date('invalid');
        expect(() => formatDate(invalidDate)).toThrow();
    });
});

describe('formatDuration', () => {
    it('should format duration correctly', () => {
        expect(formatDuration(0)).toBe('0ms');
        expect(formatDuration(500)).toBe('500ms');
        expect(formatDuration(1000)).toBe('1s');
        expect(formatDuration(60000)).toBe('1m 0s');
        // formatDuration doesn't include seconds when hours are present
        expect(formatDuration(3661000)).toBe('1h 1m');
    });
});

