/**
 * Retry-After header parser tests for the TypeScript SDK
 * (issue #263, phase9 §7).
 *
 * The full retry loop is exercised end-to-end at the server level by
 * `crates/vectorizer-server/tests/backpressure_429.rs`; here we lock
 * in the value-parsing edges (default, cap, zero, junk) that
 * determine how aggressively the SDK backs off.
 */

import { describe, expect, it } from 'vitest';
import { parseRetryAfterSeconds } from '../src/utils/http-client';

describe('parseRetryAfterSeconds', () => {
  it('returns the default for null/undefined', () => {
    expect(parseRetryAfterSeconds(null)).toBe(1);
    expect(parseRetryAfterSeconds(undefined)).toBe(1);
  });

  it('returns the default for empty/whitespace', () => {
    expect(parseRetryAfterSeconds('')).toBe(1);
    expect(parseRetryAfterSeconds('   ')).toBe(1);
  });

  it('returns the default for zero (avoid busy-loop)', () => {
    expect(parseRetryAfterSeconds('0')).toBe(1);
  });

  it('returns the default for unparseable strings', () => {
    expect(parseRetryAfterSeconds('not-a-number')).toBe(1);
  });

  it('passes through small values verbatim', () => {
    expect(parseRetryAfterSeconds('3')).toBe(3);
    expect(parseRetryAfterSeconds('7')).toBe(7);
    expect(parseRetryAfterSeconds(' 5 ')).toBe(5);
  });

  it('caps large values at 30 (mirrors Rust + Python SDKs)', () => {
    // If this assertion ever flips, audit RETRY_AFTER_MAX_SECONDS
    // in src/utils/http-client.ts first.
    expect(parseRetryAfterSeconds('3600')).toBe(30);
    expect(parseRetryAfterSeconds('31')).toBe(30);
  });
});
