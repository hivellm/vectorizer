/**
 * `RpcClient`: connect, hello, call, ping, close.
 *
 * The transport is Thunder's (`@hivehub/thunder`): one TCP connection per
 * client, responses demultiplexed by frame id into per-call promises,
 * bounded in-flight, per-call timeouts, lazy re-dial and typed errors. What
 * lives here is Vectorizer's shape on top of it — the `vectorizer://`
 * protocol config, the HELLO payload/response types, and the error classes
 * the typed wrappers in {@link ./commands} throw.
 *
 * Auth is **per-connection sticky** per wire spec § 4, and Thunder carries
 * credentials in the connection handshake (`AUTH`) rather than in a command.
 * {@link RpcClient.hello} therefore re-dials when its payload carries a
 * token or an API key, so the credentials reach the session later commands
 * run under; the HELLO command itself still runs, because the server answers
 * it with the capability list and auth flags this client surfaces.
 */

import {
  AuthError,
  Client,
  Config,
  ConnectionError,
  DecodeError,
  FrameTooLargeError,
  ServerError,
  ThunderError,
  TimeoutError,
  type ClientOptions,
  type Credentials,
} from '@hivehub/thunder';

import { DEFAULT_RPC_PORT, Endpoint, parseEndpoint } from './endpoint';
import { Value, VectorizerValue, asArray, asBool, asInt, asStr, mapGet } from './types';

/**
 * Frame-body cap, matching the server's listener so neither end rejects a
 * frame the other is willing to send.
 */
const MAX_FRAME_BYTES = 512 * 1024 * 1024;

/**
 * How Vectorizer uses the Thunder wire — the client half of the server's
 * `vectorizer_config()`: `vectorizer` scheme, `AUTH`-command handshake, no
 * HELLO negotiation (the `HELLO` *command* is Vectorizer's own), RESP3-style
 * error prefixes.
 */
export function protocolConfig(): Config {
  return Config.standard()
    .withScheme('vectorizer')
    .withPort(DEFAULT_RPC_PORT)
    .withHandshake('auth_command')
    .withHelloStyle('not_used')
    .withPush('reserved')
    .withErrorCodes('resp3_prefixes')
    .withMaxFrameBytes(MAX_FRAME_BYTES);
}

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
 * The connection failed: the dial was refused, the write failed, or the
 * peer went away while the call was pending. The client re-dials lazily on
 * the next call; a dial that cannot be re-established keeps throwing this.
 */
export class RpcConnectionClosed extends RpcClientError {
  constructor(message = 'connection closed before response') {
    super(message);
    this.name = 'RpcConnectionClosed';
  }
}

/**
 * The server refused the session's credentials — `NOAUTH` (no `AUTH` sent,
 * or HELLO issued without credentials against an auth-enabled server),
 * `WRONGPASS`, or `NOPERM` for an admin-only command.
 */
export class RpcNotAuthenticated extends RpcClientError {
  constructor(
    message = 'HELLO must succeed before any data-plane command can be issued',
  ) {
    super(message);
    this.name = 'RpcNotAuthenticated';
  }
}

/** The connect or per-call timeout elapsed. */
export class RpcTimeout extends RpcClientError {
  constructor(message = 'call timed out') {
    super(message);
    this.name = 'RpcTimeout';
  }
}

/**
 * The peer sent a malformed or oversized frame; the connection is poisoned
 * and the next call re-dials.
 */
export class RpcProtocolError extends RpcClientError {
  constructor(message: string) {
    super(message);
    this.name = 'RpcProtocolError';
  }
}

/** Map a typed Thunder error onto this SDK's error classes. */
function mapThunderError(err: unknown): Error {
  if (err instanceof AuthError) return new RpcNotAuthenticated(err.message);
  if (err instanceof ServerError) return new RpcServerError(err.message);
  if (err instanceof ConnectionError) return new RpcConnectionClosed(err.message);
  if (err instanceof TimeoutError) return new RpcTimeout(err.message);
  if (err instanceof FrameTooLargeError || err instanceof DecodeError) {
    return new RpcProtocolError(err.message);
  }
  if (err instanceof ThunderError) return new RpcClientError(err.message);
  return err instanceof Error ? err : new RpcClientError(String(err));
}

/**
 * HELLO request payload.
 *
 * At least one of `token` / `apiKey` should be populated when the server has
 * auth enabled: those credentials travel in the connection handshake, so
 * passing them to {@link RpcClient.hello} is what authenticates the session.
 * When the server runs in single-user mode (`auth.enabled: false`) the
 * listener is open, credentials are accepted-but-ignored, and the connection
 * runs as the implicit local admin.
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

/** Options for {@link RpcClient.connect}. */
export interface ConnectOptions {
  /** Connect timeout in ms. Defaults to 10_000. */
  timeoutMs?: number;
  /** Per-call timeout in ms. Defaults to 30_000. */
  callTimeoutMs?: number;
}

/**
 * One TCP connection to a Vectorizer RPC server.
 *
 * Construct via {@link RpcClient.connect} (raw `host:port`) or
 * {@link RpcClient.connectUrl} (`vectorizer://` URL). Issue {@link hello}
 * with credentials when the server enforces auth.
 *
 * Concurrency-safe: multiple concurrent `await client.X()` calls multiplex
 * over the one connection and complete in server order.
 */
export class RpcClient {
  private client: Client;
  private readonly endpoint: string;
  private options: ClientOptions;

  private constructor(client: Client, endpoint: string, options: ClientOptions) {
    this.client = client;
    this.endpoint = endpoint;
    this.options = options;
  }

  /**
   * Dial `address` — `host:port`, or any form
   * {@link parseEndpoint} accepts.
   *
   * Does NOT authenticate: pass credentials to {@link hello}, which
   * re-dials with them in the handshake.
   */
  static async connect(
    address: string,
    options: ConnectOptions = {},
  ): Promise<RpcClient> {
    const clientOptions: ClientOptions = {
      connectTimeoutMs: options.timeoutMs ?? 10_000,
      callTimeoutMs: options.callTimeoutMs ?? 30_000,
      clientName: 'vectorizer-sdk-typescript',
    };
    const client = await RpcClient.dial(address, clientOptions);
    return new RpcClient(client, address, clientOptions);
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

  private static async dial(endpoint: string, options: ClientOptions): Promise<Client> {
    try {
      return await Client.connect(endpoint, protocolConfig(), options);
    } catch (err) {
      throw mapThunderError(err);
    }
  }

  /**
   * Issue the HELLO handshake and return the server's capability list and
   * auth flags.
   *
   * When `payload` carries a token or an API key, the connection is
   * re-dialed so those credentials travel in Thunder's `AUTH` handshake —
   * that is what authenticates the session every later command runs under.
   * A credential-free payload reuses the existing connection.
   */
  async hello(payload: HelloPayload = {}): Promise<HelloResponse> {
    const credentials = helloCredentials(payload);
    if (credentials !== undefined) {
      const options: ClientOptions = { ...this.options, credentials };
      if (payload.clientName !== undefined) {
        options.clientName = payload.clientName;
      }
      const fresh = await RpcClient.dial(this.endpoint, options);
      const previous = this.client;
      this.client = fresh;
      this.options = options;
      await previous.close();
    }
    return parseHelloResponse(await this.call('HELLO', [helloPayloadToValue(payload)]));
  }

  /** Health check. Auth-exempt per wire spec § 4 — works pre-HELLO. */
  async ping(): Promise<string> {
    const s = asStr(await this.call('PING', []));
    if (s === null) {
      throw new RpcServerError('PING returned non-string payload');
    }
    return s;
  }

  /**
   * Dispatch a generic command. Most callers should reach for a typed
   * wrapper from {@link ./commands} instead.
   *
   * The server gates un-authenticated sessions, so a data-plane command on
   * a session that never authenticated throws {@link RpcNotAuthenticated}.
   */
  async call(command: string, args: VectorizerValue[] = []): Promise<VectorizerValue> {
    try {
      return await this.client.call(command, args);
    } catch (err) {
      throw mapThunderError(err);
    }
  }

  /**
   * Returns `true` once the connection's handshake authenticated. Always
   * `false` against an open (single-user) server, which authenticates
   * nobody because it gates nothing.
   */
  isAuthenticated(): boolean {
    return this.client.isAuthenticated;
  }

  /**
   * Close the connection. In-flight calls receive
   * {@link RpcConnectionClosed}.
   */
  async close(): Promise<void> {
    await this.client.close();
  }
}

// ── helpers ────────────────────────────────────────────────────────

/** The handshake credentials a HELLO payload carries, if any. */
function helloCredentials(payload: HelloPayload): Credentials | undefined {
  if (payload.token !== undefined) {
    return { type: 'token', token: payload.token };
  }
  if (payload.apiKey !== undefined) {
    return { type: 'apiKey', apiKey: payload.apiKey };
  }
  return undefined;
}

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
