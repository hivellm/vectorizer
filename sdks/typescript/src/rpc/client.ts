/**
 * `RpcClient`: connect, hello, call, ping, close.
 *
 * The client owns one TCP connection to the server. It uses Node's
 * `net.Socket` for the transport and demultiplexes responses by
 * `Request.id` into per-call promise mailboxes so concurrent
 * in-flight calls on the same connection don't block each other.
 *
 * Auth is **per-connection sticky** per wire spec § 4: the first
 * frame on a connection MUST be `HELLO`; every subsequent call
 * inherits the auth state. The local `call` method enforces this so
 * callers see a clear typed error instead of a server-side string.
 */

import * as net from 'node:net';

import { encodeFrame, FrameReader } from './codec';
import { Endpoint, parseEndpoint } from './endpoint';
import {
  Request,
  Response,
  ResponseResult,
  Value,
  VectorizerValue,
  asArray,
  asBool,
  asInt,
  asStr,
  mapGet,
  requestToMsgpack,
  responseFromMsgpack,
} from './types';

/** Base error for all RPC client failures. */
export class RpcClientError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'RpcClientError';
  }
}

/** The server returned `Result::Err(message)` for the call. */
export class RpcServerError extends RpcClientError {
  constructor(message: string) {
    super(message);
    this.name = 'RpcServerError';
  }
}

/**
 * The reader closed before the response arrived. Either the peer
 * closed cleanly (EOF) or an I/O error tore down the socket. The
 * client's connection is unusable; build a new one.
 */
export class RpcConnectionClosed extends RpcClientError {
  constructor(message = 'connection closed before response') {
    super(message);
    this.name = 'RpcConnectionClosed';
  }
}

/**
 * A data-plane command was issued before HELLO succeeded. The server
 * would also reject this; the client surfaces it locally so the
 * offending caller sees a clear error without burning a network
 * round-trip.
 */
export class RpcNotAuthenticated extends RpcClientError {
  constructor() {
    super('HELLO must succeed before any data-plane command can be issued');
    this.name = 'RpcNotAuthenticated';
  }
}

/**
 * HELLO request payload — sent as the FIRST frame on a connection.
 *
 * At least one of `token` / `apiKey` should be populated when the
 * server has auth enabled. When the server runs in single-user mode
 * (`auth.enabled: false`), credentials are accepted-but-ignored and
 * the connection runs as the implicit local admin.
 */
export interface HelloPayload {
  clientName?: string;
  token?: string;
  apiKey?: string;
  /** Wire spec protocol version. Defaults to 1. */
  version?: number;
}

/** Decoded HELLO success payload from the server. */
export interface HelloResponse {
  serverVersion: string;
  protocolVersion: number;
  authenticated: boolean;
  admin: boolean;
  capabilities: string[];
}

/** Auth-exempt commands per wire spec § 4. */
const AUTH_EXEMPT = new Set(['HELLO', 'PING']);

/** Options for {@link RpcClient.connect}. */
export interface ConnectOptions {
  /** Connect timeout in ms. Defaults to 10_000. */
  timeoutMs?: number;
}

interface PendingCall {
  resolve: (value: VectorizerValue) => void;
  reject: (err: Error) => void;
}

/**
 * One TCP connection to a Vectorizer RPC server.
 *
 * Construct via {@link RpcClient.connect} (raw `host:port`) or
 * {@link RpcClient.connectUrl} (`vectorizer://` URL). Always issue
 * {@link hello} before any data-plane call.
 *
 * Coroutine-safe: multiple concurrent `await client.X()` calls
 * serialize on a writer queue and demultiplex by `Request.id` into
 * per-call promises.
 */
export class RpcClient {
  private socket: net.Socket;
  private reader = new FrameReader();
  private pending = new Map<number, PendingCall>();
  private nextId = 1;
  private authenticated = false;
  private closed = false;
  private writeQueue: Promise<void> = Promise.resolve();

  private constructor(socket: net.Socket) {
    this.socket = socket;
    this.socket.setNoDelay(true);
    this.socket.on('data', (chunk) => this.onData(chunk));
    this.socket.on('error', (err) => this.onError(err));
    this.socket.on('close', () => this.onClose());
  }

  /**
   * Open a TCP connection to `address` (`host:port`).
   *
   * Does NOT send HELLO — callers MUST `await client.hello(...)`
   * before any data-plane command, or the server will reject it.
   */
  static connect(address: string, options: ConnectOptions = {}): Promise<RpcClient> {
    const [host, port] = splitHostPort(address);
    const timeoutMs = options.timeoutMs ?? 10_000;
    return new Promise((resolve, reject) => {
      const socket = net.createConnection({ host, port });
      const onError = (err: Error): void => {
        socket.removeAllListeners();
        socket.destroy();
        reject(new RpcConnectionClosed(`connect failed: ${err.message}`));
      };
      const timer = setTimeout(() => {
        onError(new Error(`connection timed out after ${timeoutMs}ms`));
      }, timeoutMs);
      socket.once('connect', () => {
        clearTimeout(timer);
        socket.removeListener('error', onError);
        resolve(new RpcClient(socket));
      });
      socket.once('error', onError);
    });
  }

  /**
   * Parse a `vectorizer://host[:port]` URL and dial it.
   *
   * REST URLs (`http(s)://`) are rejected with a clear error
   * pointing the caller at the HTTP client.
   */
  static async connectUrl(url: string, options: ConnectOptions = {}): Promise<RpcClient> {
    const ep: Endpoint = parseEndpoint(url);
    if (ep.kind === 'rpc') {
      return RpcClient.connect(`${ep.host}:${ep.port}`, options);
    }
    throw new RpcServerError(
      `RpcClient cannot dial REST URL '${ep.url}'; ` +
        `use the HTTP client (VectorizerClient) instead, ` +
        `or pass a 'vectorizer://' URL`,
    );
  }

  /**
   * Issue the HELLO handshake. Must be the first call on a fresh
   * connection. Returns the server's capability list and auth flags.
   */
  async hello(payload: HelloPayload = {}): Promise<HelloResponse> {
    const value = helloPayloadToValue(payload);
    const result = await this.rawCall('HELLO', [value]);
    const parsed = parseHelloResponse(result);
    if (parsed.authenticated) {
      this.authenticated = true;
    }
    return parsed;
  }

  /** Health check. Auth-exempt per wire spec § 4 — works pre-HELLO. */
  async ping(): Promise<string> {
    const result = await this.rawCall('PING', []);
    const s = asStr(result);
    if (s === null) {
      throw new RpcServerError('PING returned non-string payload');
    }
    return s;
  }

  /**
   * Dispatch a generic command. Most callers should reach for a
   * typed wrapper from {@link ./commands} instead.
   *
   * Enforces the local auth gate: data-plane commands throw
   * {@link RpcNotAuthenticated} before sending if HELLO hasn't
   * succeeded.
   */
  async call(command: string, args: VectorizerValue[] = []): Promise<VectorizerValue> {
    if (!AUTH_EXEMPT.has(command) && !this.authenticated) {
      throw new RpcNotAuthenticated();
    }
    return this.rawCall(command, args);
  }

  /** Returns `true` once HELLO has succeeded on this connection. */
  isAuthenticated(): boolean {
    return this.authenticated;
  }

  /**
   * Close the connection. In-flight calls receive
   * {@link RpcConnectionClosed}.
   */
  close(): void {
    if (this.closed) return;
    this.closed = true;
    try {
      this.socket.end();
    } catch {
      // ignore
    }
    this.socket.destroy();
    this.failAllPending(new RpcConnectionClosed('client closed'));
  }

  // ── internals ────────────────────────────────────────────────────
  private allocId(): number {
    const rid = this.nextId;
    this.nextId = (this.nextId + 1) & 0xffffffff;
    if (this.nextId === 0) this.nextId = 1;
    return rid;
  }

  private async rawCall(
    command: string,
    args: VectorizerValue[],
  ): Promise<VectorizerValue> {
    if (this.closed) {
      throw new RpcConnectionClosed('client closed');
    }
    const id = this.allocId();
    const req: Request = { id, command, args };
    const frame = encodeFrame(requestToMsgpack(req));

    const responsePromise = new Promise<VectorizerValue>((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
    });

    // Serialise writes — concurrent calls must not interleave bytes.
    this.writeQueue = this.writeQueue.then(
      () =>
        new Promise<void>((resolve, reject) => {
          if (!this.socket.write(frame, (err) => (err ? reject(err) : resolve()))) {
            // Backpressure — flushed callback above still fires.
          }
        }),
    );
    try {
      await this.writeQueue;
    } catch (err) {
      this.pending.delete(id);
      throw new RpcConnectionClosed(
        `send failed: ${err instanceof Error ? err.message : String(err)}`,
      );
    }

    return responsePromise;
  }

  private onData(chunk: Buffer): void {
    this.reader.push(chunk);
    let frames: unknown[];
    try {
      frames = this.reader.drain();
    } catch (err) {
      this.failAllPending(
        new RpcClientError(
          `frame reader error: ${err instanceof Error ? err.message : String(err)}`,
        ),
      );
      this.socket.destroy();
      return;
    }
    for (const raw of frames) {
      let resp: Response;
      try {
        resp = responseFromMsgpack(raw);
      } catch (_err) {
        // Skip malformed frames — log via stderr would be too noisy.
        // The pending caller will eventually time out or get
        // ConnectionClosed when the socket dies.
        continue;
      }
      const pending = this.pending.get(resp.id);
      if (pending === undefined) continue;
      this.pending.delete(resp.id);
      const r: ResponseResult = resp.result;
      if (r.kind === 'Ok') {
        pending.resolve(r.value);
      } else {
        pending.reject(new RpcServerError(r.message));
      }
    }
  }

  private onError(_err: Error): void {
    // The 'close' handler will run too — failure is centralised there.
  }

  private onClose(): void {
    this.closed = true;
    this.failAllPending(new RpcConnectionClosed());
  }

  private failAllPending(err: Error): void {
    const pending = Array.from(this.pending.values());
    this.pending.clear();
    for (const p of pending) p.reject(err);
  }
}

// ── helpers ────────────────────────────────────────────────────────

function helloPayloadToValue(payload: HelloPayload): VectorizerValue {
  const pairs: Array<[VectorizerValue, VectorizerValue]> = [
    [Value.str('version'), Value.int(payload.version ?? 1)],
  ];
  if (payload.token !== undefined) {
    pairs.push([Value.str('token'), Value.str(payload.token)]);
  }
  if (payload.apiKey !== undefined) {
    pairs.push([Value.str('api_key'), Value.str(payload.apiKey)]);
  }
  if (payload.clientName !== undefined) {
    pairs.push([Value.str('client_name'), Value.str(payload.clientName)]);
  }
  return Value.map(pairs);
}

function parseHelloResponse(value: VectorizerValue): HelloResponse {
  const sv = mapGet(value, 'server_version');
  const pv = mapGet(value, 'protocol_version');
  const au = mapGet(value, 'authenticated');
  const ad = mapGet(value, 'admin');
  const caps = mapGet(value, 'capabilities');
  const capsArr: string[] = [];
  const arr = caps !== null ? asArray(caps) : null;
  if (arr !== null) {
    for (const v of arr) {
      const s = asStr(v);
      if (s !== null) capsArr.push(s);
    }
  }
  return {
    serverVersion: (sv !== null && asStr(sv)) || '',
    protocolVersion: (pv !== null && asInt(pv)) || 0,
    authenticated: (au !== null && asBool(au)) || false,
    admin: (ad !== null && asBool(ad)) || false,
    capabilities: capsArr,
  };
}

/**
 * Split a `host:port` string. IPv6 literals (`[::1]:1234`) are
 * handled specially so the colons inside the brackets aren't
 * treated as port separators.
 */
function splitHostPort(address: string): [string, number] {
  if (address.startsWith('[')) {
    const close = address.indexOf(']');
    if (close < 0) {
      throw new Error(`unterminated IPv6 literal in address: ${address}`);
    }
    const host = address.slice(1, close);
    const rest = address.slice(close + 1);
    if (!rest.startsWith(':')) {
      throw new Error(`expected ':<port>' after IPv6 literal in address: ${address}`);
    }
    return [host, Number(rest.slice(1))];
  }
  const colon = address.lastIndexOf(':');
  if (colon < 0) {
    throw new Error(`address must include ':<port>', got ${address}`);
  }
  return [address.slice(0, colon), Number(address.slice(colon + 1))];
}
