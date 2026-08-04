/**
 * End-to-end integration tests for `RpcClient`.
 *
 * Spins up an in-test server on `127.0.0.1:0` that speaks the wire through
 * Thunder's own codec — the same one the client and the production server
 * use — and drives it from {@link RpcClient} to prove:
 *
 * - HELLO produces the expected {@link HelloResponse} shape.
 * - `PING` works pre-HELLO (auth-exempt per wire spec § 4).
 * - A data-plane command on an un-credentialed session against an
 *   auth-enabled server throws {@link RpcNotAuthenticated}.
 * - A credential-carrying HELLO authenticates the session via `AUTH`, so
 *   later commands pass the gate.
 * - Concurrent calls on the same connection are demultiplexed by frame id.
 * - Typed wrappers (`listCollections`, `getCollectionInfo`, `searchBasic`)
 *   round-trip over the wire.
 * - `connectUrl` accepts the canonical `vectorizer://` form and rejects
 *   REST URLs with a clear error.
 */

import * as net from 'node:net';
import { afterEach, beforeEach, describe, expect, test } from 'vitest';

import {
  RpcClient,
  RpcNotAuthenticated,
  RpcServerError,
} from '../../src/rpc/client';
import '../../src/rpc/commands'; // attach typed wrappers
import {
  FrameReader,
  decodeRequestBody,
  encodeResponse,
} from '@hivehub/thunder';
import { Request, Response, Value, asStr } from '../../src/rpc/types';

// ─────────────────────────────────────────────────────────────────────
// In-test fake-server fixture
// ─────────────────────────────────────────────────────────────────────

/** The one credential the fake server accepts. */
const GOOD_TOKEN = 'good-token';

/** Commands the server answers before the session authenticates. */
const PRE_AUTH = new Set(['PING', 'HELLO', 'AUTH', 'QUIT']);

function buildHelloResponse(rid: number): Response {
  return Response.ok(
    rid,
    Value.map([
      [Value.str('server_version'), Value.str('test-fixture/0.0.0')],
      [Value.str('protocol_version'), Value.int(1)],
      [Value.str('authenticated'), Value.bool(true)],
      [Value.str('admin'), Value.bool(true)],
      [
        Value.str('capabilities'),
        Value.array([
          Value.str('PING'),
          Value.str('collections.list'),
          Value.str('collections.get_info'),
          Value.str('vectors.get'),
          Value.str('search.basic'),
        ]),
      ],
    ]),
  );
}

function buildCollectionInfoResponse(rid: number, name: string): Response {
  return Response.ok(
    rid,
    Value.map([
      [Value.str('name'), Value.str(name)],
      [Value.str('vector_count'), Value.int(42)],
      [Value.str('document_count'), Value.int(10)],
      [Value.str('dimension'), Value.int(384)],
      [Value.str('metric'), Value.str('Cosine')],
      [Value.str('created_at'), Value.str('2026-04-19T00:00:00Z')],
      [Value.str('updated_at'), Value.str('2026-04-19T00:00:00Z')],
    ]),
  );
}

function buildSearchBasicResponse(rid: number): Response {
  return Response.ok(
    rid,
    Value.array([
      Value.map([
        [Value.str('id'), Value.str('vec-0')],
        [Value.str('score'), Value.float(0.95)],
        [Value.str('payload'), Value.str('{"title":"hit one"}')],
      ]),
      Value.map([
        [Value.str('id'), Value.str('vec-1')],
        [Value.str('score'), Value.float(0.81)],
      ]),
    ]),
  );
}

interface SessionState {
  authenticated: boolean;
}

/**
 * The server side of the `auth_command` handshake plus Vectorizer's command
 * catalog, reduced to what these tests exercise. `authRequired` mirrors the
 * deployment posture: `false` opens the listener (single-user mode), `true`
 * refuses un-credentialed sessions with `NOAUTH`, which the client
 * classifies as an auth error.
 */
function dispatch(
  req: Request,
  state: SessionState,
  authRequired: boolean,
): Response {
  const cmd = req.command;
  if (cmd === 'AUTH') {
    const secret = req.args[0] !== undefined ? asStr(req.args[0]) : null;
    if (secret !== GOOD_TOKEN) {
      return Response.err(req.id, 'WRONGPASS invalid credentials');
    }
    state.authenticated = true;
    return Response.ok(req.id, Value.str('OK'));
  }
  if (authRequired && !state.authenticated && !PRE_AUTH.has(cmd)) {
    return Response.err(req.id, 'NOAUTH authentication required');
  }
  if (cmd === 'HELLO') return buildHelloResponse(req.id);
  if (cmd === 'PING') return Response.ok(req.id, Value.str('PONG'));
  if (cmd === 'collections.list') {
    return Response.ok(
      req.id,
      Value.array([Value.str('alpha-docs'), Value.str('beta-source')]),
    );
  }
  if (cmd === 'collections.get_info') {
    let name = 'unknown';
    if (req.args[0] !== undefined) {
      const s = asStr(req.args[0]);
      if (s !== null) name = s;
    }
    return buildCollectionInfoResponse(req.id, name);
  }
  if (cmd === 'search.basic') {
    return buildSearchBasicResponse(req.id);
  }
  return Response.err(req.id, `unknown command '${cmd}'`);
}

interface FakeServer {
  port: number;
  close(): Promise<void>;
}

function spawnFakeServer(authRequired = false): Promise<FakeServer> {
  return new Promise((resolve) => {
    const server = net.createServer((socket) => {
      const reader = new FrameReader();
      const state: SessionState = { authenticated: false };
      socket.on('data', (chunk: Buffer) => {
        reader.push(chunk);
        for (;;) {
          let body: Uint8Array | null;
          try {
            body = reader.nextBody();
          } catch {
            socket.destroy();
            return;
          }
          if (body === null) break;
          const req = decodeRequestBody(body);
          socket.write(encodeResponse(dispatch(req, state, authRequired)));
        }
      });
      socket.on('error', () => {});
    });
    server.listen(0, '127.0.0.1', () => {
      const addr = server.address();
      if (addr === null || typeof addr === 'string') {
        throw new Error('listener address unavailable');
      }
      resolve({
        port: addr.port,
        close: () =>
          new Promise<void>((closeResolve) => {
            server.close(() => closeResolve());
            // Force-disconnect any lingering sockets so close() resolves.
            // (server.close waits for connections to close — daemon
            // sockets from earlier tests can hold it open otherwise.)
            server.unref();
          }),
      });
    });
  });
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

describe('RpcClient — integration with fake server', () => {
  let server: FakeServer;
  let address: string;

  beforeEach(async () => {
    server = await spawnFakeServer();
    address = `127.0.0.1:${server.port}`;
  });

  afterEach(async () => {
    await server.close();
  });

  test('hello + ping + typed commands', async () => {
    const client = await RpcClient.connect(address);

    // PING is auth-exempt per wire spec § 4.
    expect(await client.ping()).toBe('PONG');

    const hello = await client.hello({ clientName: 'rpc-integration-test' });
    expect(hello.authenticated).toBe(true);
    expect(hello.admin).toBe(true);
    expect(hello.protocolVersion).toBe(1);
    expect(hello.serverVersion).toBe('test-fixture/0.0.0');
    expect(hello.capabilities).toContain('collections.list');

    const cols = await client.listCollections();
    expect(cols).toEqual(['alpha-docs', 'beta-source']);

    const info = await client.getCollectionInfo('alpha-docs');
    expect(info.name).toBe('alpha-docs');
    expect(info.vectorCount).toBe(42);
    expect(info.dimension).toBe(384);
    expect(info.metric).toBe('Cosine');

    const hits = await client.searchBasic('alpha-docs', 'anything', 10);
    expect(hits).toHaveLength(2);
    expect(hits[0]!.id).toBe('vec-0');
    expect(hits[0]!.score).toBeCloseTo(0.95, 9);
    expect(hits[0]!.payload).toBe('{"title":"hit one"}');
    expect(hits[1]!.id).toBe('vec-1');
    expect(hits[1]!.payload).toBeNull();

    await client.close();
  });

  test('concurrent calls on one connection are demultiplexed by id', async () => {
    const client = await RpcClient.connect(address);
    await client.hello({ clientName: 'concurrent-test' });

    // Fire 16 list_collections concurrently. If demuxing were broken,
    // calls would either hang or deliver the wrong payload.
    const results = await Promise.all(
      Array.from({ length: 16 }, () => client.listCollections()),
    );
    for (const cols of results) {
      expect(cols).toEqual(['alpha-docs', 'beta-source']);
    }
    await client.close();
  });

  test('connectUrl accepts the vectorizer:// scheme', async () => {
    const client = await RpcClient.connectUrl(`vectorizer://${address}`);
    expect(await client.ping()).toBe('PONG');
    await client.close();
  });

  test('connectUrl rejects http:// schemes with a clear error', async () => {
    await expect(
      RpcClient.connectUrl('http://localhost:15002'),
    ).rejects.toBeInstanceOf(RpcServerError);
    try {
      await RpcClient.connectUrl('http://localhost:15002');
    } catch (err) {
      const msg = String(err);
      expect(msg).toContain('REST URL');
      expect(msg).toContain('HTTP client');
    }
  });
});

describe('RpcClient — auth-enabled server', () => {
  let server: FakeServer;
  let address: string;

  beforeEach(async () => {
    server = await spawnFakeServer(true);
    address = `127.0.0.1:${server.port}`;
  });

  afterEach(async () => {
    await server.close();
  });

  test('data-plane call without credentials is rejected', async () => {
    const client = await RpcClient.connect(address);
    expect(client.isAuthenticated()).toBe(false);
    await expect(client.listCollections()).rejects.toBeInstanceOf(
      RpcNotAuthenticated,
    );
    await client.close();
  });

  test('hello with a token authenticates the session', async () => {
    const client = await RpcClient.connect(address);
    const hello = await client.hello({
      clientName: 'rpc-integration-test',
      token: GOOD_TOKEN,
    });
    expect(hello.authenticated).toBe(true);
    expect(client.isAuthenticated()).toBe(true);

    const cols = await client.listCollections();
    expect(cols).toHaveLength(2);
    await client.close();
  });

  test('bad credentials fail the handshake', async () => {
    const client = await RpcClient.connect(address);
    await expect(
      client.hello({ token: 'wrong-token' }),
    ).rejects.toBeInstanceOf(RpcNotAuthenticated);
    await client.close();
  });
});
