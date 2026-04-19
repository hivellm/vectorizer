/**
 * Canonical URL parser for the SDK's connection string.
 *
 * Mirrors the Rust SDK at `sdks/rust/src/rpc/endpoint.rs` and the
 * Python SDK at `sdks/python/rpc/endpoint.py` so polyglot projects
 * share a single contract:
 *
 * - `vectorizer://host:port` → RPC on the given port.
 * - `vectorizer://host` (no port) → RPC on default port 15503.
 * - `host:port` (no scheme) → RPC.
 * - `http://host:port` / `https://host:port` → REST (legacy fallback).
 * - Anything else → throws {@link EndpointParseError}.
 *
 * URLs that carry credentials in the userinfo section
 * (`user:pass@host`) are REJECTED. Credentials cross the wire in the
 * HELLO handshake, NOT in the URL — this avoids accidentally logging
 * or shell-history-saving a token-bearing URL.
 */

/** Default RPC port. Matches the server's `RpcConfig::default_port()`. */
export const DEFAULT_RPC_PORT = 15503;

/** Default REST port. Matches the server's `ServerConfig::default()`. */
export const DEFAULT_HTTP_PORT = 15002;

/** Discriminated union returned by {@link parseEndpoint}. */
export type Endpoint =
  | { kind: 'rpc'; host: string; port: number }
  | { kind: 'rest'; url: string };

/** Raised when {@link parseEndpoint} cannot interpret the URL. */
export class EndpointParseError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'EndpointParseError';
  }
}

/**
 * Parse a connection string into a typed {@link Endpoint}.
 *
 * Throws {@link EndpointParseError} for empty input, unsupported
 * schemes, malformed authorities, and URLs carrying credentials in
 * the userinfo section.
 */
export function parseEndpoint(url: string): Endpoint {
  if (typeof url !== 'string') {
    throw new EndpointParseError(
      `endpoint URL must be a string, got ${typeof url}`,
    );
  }
  const trimmed = url.trim();
  if (trimmed.length === 0) {
    throw new EndpointParseError('endpoint URL is empty');
  }

  // Recognise an explicit scheme by splitting on the FIRST "://".
  const schemeIdx = trimmed.indexOf('://');
  if (schemeIdx >= 0) {
    const scheme = trimmed.slice(0, schemeIdx).toLowerCase();
    const rest = trimmed.slice(schemeIdx + 3);
    if (scheme === 'vectorizer') {
      return parseRpcAuthority(rest);
    }
    if (scheme === 'http' || scheme === 'https') {
      return parseRest(scheme, rest, trimmed);
    }
    throw new EndpointParseError(
      `unsupported URL scheme '${trimmed.slice(0, schemeIdx)}'; ` +
        `expected 'vectorizer', 'http', or 'https'`,
    );
  }

  // No scheme — treat as bare host[:port] for RPC.
  return parseRpcAuthority(trimmed);
}

function parseRpcAuthority(authority: string): Endpoint {
  if (authority.length === 0) {
    throw new EndpointParseError(
      `invalid authority in URL '${authority}': missing host`,
    );
  }
  if (authority.includes('@')) {
    throw new EndpointParseError(
      'URL carries credentials in the userinfo section; ' +
        'pass credentials to the HELLO handshake instead of embedding them in the URL',
    );
  }

  // Trim a trailing path; the RPC scheme has no notion of paths.
  let hostPort = authority;
  for (const sep of ['/', '?', '#']) {
    const idx = hostPort.indexOf(sep);
    if (idx >= 0) hostPort = hostPort.slice(0, idx);
  }
  if (hostPort.length === 0) {
    throw new EndpointParseError(
      `invalid authority in URL '${authority}': missing host`,
    );
  }

  // IPv6 literal: [::1] or [::1]:port. Bracket-aware split.
  if (hostPort.startsWith('[')) {
    const close = hostPort.indexOf(']');
    if (close < 0) {
      throw new EndpointParseError(
        `invalid authority in URL '${authority}': unterminated IPv6 literal '['`,
      );
    }
    const host = hostPort.slice(0, close + 1);
    const after = hostPort.slice(close + 1);
    if (after.length === 0) {
      return { kind: 'rpc', host, port: DEFAULT_RPC_PORT };
    }
    if (!after.startsWith(':')) {
      throw new EndpointParseError(
        `invalid authority in URL '${authority}': ` +
          `expected ':<port>' after IPv6 literal, got '${after}'`,
      );
    }
    return { kind: 'rpc', host, port: parsePort(after.slice(1), authority) };
  }

  // Hostname or IPv4. Split on the LAST colon so a port-like suffix
  // wins over an unrelated colon in any future hostname extension.
  const colon = hostPort.lastIndexOf(':');
  if (colon >= 0) {
    const host = hostPort.slice(0, colon);
    const portStr = hostPort.slice(colon + 1);
    if (host.length === 0) {
      throw new EndpointParseError(
        `invalid authority in URL '${authority}': missing host before ':<port>'`,
      );
    }
    return { kind: 'rpc', host, port: parsePort(portStr, authority) };
  }

  return { kind: 'rpc', host: hostPort, port: DEFAULT_RPC_PORT };
}

function parseRest(scheme: string, rest: string, raw: string): Endpoint {
  if (rest.length === 0) {
    throw new EndpointParseError(
      `invalid authority in URL '${raw}': missing host`,
    );
  }
  if (rest.includes('@')) {
    throw new EndpointParseError(
      'URL carries credentials in the userinfo section; ' +
        'pass credentials to the HELLO handshake instead of embedding them in the URL',
    );
  }
  return { kind: 'rest', url: `${scheme}://${rest}` };
}

function parsePort(portStr: string, authority: string): number {
  if (!/^\d+$/.test(portStr)) {
    throw new EndpointParseError(
      `invalid authority in URL '${authority}': invalid port: '${portStr}' is not a number`,
    );
  }
  const port = Number(portStr);
  if (!Number.isInteger(port) || port < 0 || port > 65535) {
    throw new EndpointParseError(
      `invalid authority in URL '${authority}': port ${port} is out of range 0..65535`,
    );
  }
  return port;
}
