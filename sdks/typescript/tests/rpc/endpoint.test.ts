/**
 * Unit tests for `parseEndpoint` — the canonical URL parser.
 *
 * The contract is shared with the Rust + Python SDKs; the golden URL
 * forms here are the same ones tested in `sdks/rust/src/rpc/endpoint.rs`
 * and `sdks/python/tests/rpc/test_endpoint.py`. Keeping them aligned
 * across SDKs prevents subtle behaviour drift between language
 * runtimes.
 */

import { describe, expect, test } from 'vitest';

import {
  DEFAULT_RPC_PORT,
  EndpointParseError,
  parseEndpoint,
} from '../../src/rpc/endpoint';

describe('parseEndpoint', () => {
  test('vectorizer:// with explicit host and port', () => {
    expect(parseEndpoint('vectorizer://example.com:9000')).toEqual({
      kind: 'rpc',
      host: 'example.com',
      port: 9000,
    });
  });

  test('vectorizer:// without port defaults to 15503', () => {
    expect(parseEndpoint('vectorizer://example.com')).toEqual({
      kind: 'rpc',
      host: 'example.com',
      port: DEFAULT_RPC_PORT,
    });
    expect(DEFAULT_RPC_PORT).toBe(15503);
  });

  test('bare host:port (no scheme) routes to RPC', () => {
    expect(parseEndpoint('localhost:15503')).toEqual({
      kind: 'rpc',
      host: 'localhost',
      port: 15503,
    });
  });

  test('http:// and https:// route to REST', () => {
    expect(parseEndpoint('http://localhost:15002')).toEqual({
      kind: 'rest',
      url: 'http://localhost:15002',
    });
    expect(parseEndpoint('https://api.example.com')).toEqual({
      kind: 'rest',
      url: 'https://api.example.com',
    });
  });

  test('unsupported scheme is rejected by name', () => {
    expect(() => parseEndpoint('ftp://server.example.com')).toThrow(
      EndpointParseError,
    );
    try {
      parseEndpoint('ftp://server.example.com');
    } catch (e) {
      expect(String(e)).toContain('ftp');
      expect(String(e)).toContain('vectorizer');
    }
  });

  test('empty string is rejected', () => {
    expect(() => parseEndpoint('')).toThrow(/empty/);
    expect(() => parseEndpoint('   ')).toThrow(/empty/);
  });

  test('userinfo credentials are rejected', () => {
    // Both schemes — credentials must go through HELLO, not the URL,
    // to avoid logging or shell-history-saving a token-bearing URL.
    expect(() => parseEndpoint('vectorizer://user:pass@host:15503')).toThrow(
      /credentials/,
    );
    expect(() => parseEndpoint('https://user:secret@api.example.com')).toThrow(
      /credentials/,
    );
  });

  test('malformed port is rejected', () => {
    expect(() => parseEndpoint('vectorizer://host:not-a-port')).toThrow(/invalid port/);
  });

  test('IPv6 literal with port', () => {
    expect(parseEndpoint('vectorizer://[::1]:15503')).toEqual({
      kind: 'rpc',
      host: '[::1]',
      port: 15503,
    });
  });

  test('IPv6 literal without port defaults', () => {
    expect(parseEndpoint('vectorizer://[::1]')).toEqual({
      kind: 'rpc',
      host: '[::1]',
      port: DEFAULT_RPC_PORT,
    });
  });

  test('non-string input is rejected', () => {
    // @ts-expect-error — intentionally violating the type to test runtime guard.
    expect(() => parseEndpoint(15503)).toThrow(/must be a string/);
  });
});
