/**
 * End-to-end integration tests for `RpcClient`.
 *
 * Spins up an in-test server on `127.0.0.1:0` that speaks the
 * VectorizerRPC wire format using the SDK's own codec + types
 * (because the production server isn't available as a TS dependency)
 * and drives it from {@link RpcClient} to prove:
 *
 * - HELLO handshake produces the expected {@link HelloResponse} shape.
 * - `PING` works pre-HELLO (auth-exempt per wire spec § 4).
 * - A data-plane command before HELLO returns
 *   {@link RpcNotAuthenticated} from the local gate.
 * - Concurrent calls on the same connection are demultiplexed by
 *   `Request.id` correctly.
 * - Typed wrappers (`listCollections`, `getCollectionInfo`,
 *   `searchBasic`) round-trip through the codec.
 * - `connectUrl` accepts the canonical `vectorizer://` form and
 *   rejects REST URLs with a clear error.
 */

import * as net from 'node:net';
import { afterEach, beforeEach, describe, expect, test } from 'vitest';

import {
  RpcClient,
  RpcNotAuthenticated,
  RpcServerError,
} from '../../src/rpc/client';
import '../../src/rpc/commands'; // attach typed wrappers
import { encodeFrame, FrameReader } from '../../src/rpc/codec';
import {
  Request,
  Response,
  Value,
  VectorizerValue,
  asStr,
  responseErr,
  responseOk,
  responseToMsgpack,
  valueFromMsgpack,
} from '../../src/rpc/types';

// ─────────────────────────────────────────────────────────────────────
// In-test fake-server fixture
// ─────────────────────────────────────────────────────────────────────

function buildHelloResponse(rid: number): Response {
  return responseOk(
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
  return responseOk(
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
  return responseOk(
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

function dispatch(req: Request, state: { authenticated: boolean }): Response {
  const cmd = req.command;
  if (cmd === 'HELLO') {
    state.authenticated = true;
    return buildHelloResponse(req.id);
  }
  if (cmd === 'PING') return responseOk(req.id, Value.str('PONG'));
  if (!state.authenticated) {
    return responseErr(
      req.id,
      `authentication required: send HELLO first (${cmd})`,
    );
  }
  if (cmd === 'collections.list') {
    return responseOk(
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
  return responseErr(req.id, `unknown command '${cmd}'`);
}

interface FakeServer {
  port: number;
  close(): Promise<void>;
}

function spawnFakeServer(): Promise<FakeServer> {
  return new Promise((resolve) => {
    const server = net.createServer((socket) => {
      const reader = new FrameReader();
      const state = { authenticated: false };
      socket.on('data', (chunk) => {
        reader.push(chunk);
        let frames: unknown[];
        try {
          frames = reader.drain();
        } catch {
          socket.destroy();
          return;
        }
        for (const raw of frames) {
          if (!Array.isArray(raw) || raw.length !== 3) continue;
          const req: Request = {
            id: Number(raw[0]),
            command: String(raw[1]),
            args: (raw[2] as unknown[]).map(valueFromMsgpack),
          };
          const resp = dispatch(req, state);
          socket.write(encodeFrame(responseToMsgpack(resp)));
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

    client.close();
  });

  test('data-plane call before HELLO is rejected locally', async () => {
    const client = await RpcClient.connect(address);
    await expect(client.listCollections()).rejects.toBeInstanceOf(
      RpcNotAuthenticated,
    );
    client.close();
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
    client.close();
  });

  test('connectUrl accepts the vectorizer:// scheme', async () => {
    const client = await RpcClient.connectUrl(`vectorizer://${address}`);
    expect(await client.ping()).toBe('PONG');
    client.close();
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
