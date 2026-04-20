/**
 * Regression guard for the per-surface client split (phase4).
 *
 * Each per-surface client must route its calls through whatever
 * `Transport` is injected at construction time — this is the contract
 * that lets `phase6_sdk-typescript-rpc` plug `RpcTransport` into the
 * same surface modules without rewriting any wrappers.
 *
 * The tests below build a `MockTransport implements Transport`,
 * register canned responses keyed by `<METHOD> <path>`, instantiate
 * each per-surface client with that transport, exercise one
 * representative method, and assert the call landed on the mock
 * instead of the default HTTP transport.
 */

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import {
  AdminClient,
  BaseClient,
  CollectionsClient,
  CoreClient,
  DiscoveryClient,
  FilesClient,
  GraphClient,
  QdrantClient,
  SearchClient,
  Transport,
  VectorizerClient,
  VectorsClient,
} from '../src/client';
import { HttpClient } from '../src/utils/http-client';

// Block any accidental fall-through to the real HTTP client. If the mock
// fails to intercept, this throws a clear error instead of attempting a
// network call.
vi.mock('../src/utils/http-client', () => {
  return {
    // vitest 4 rejects arrow-body implementations passed directly to
    // `vi.fn(...)`. Use `function` bodies here — the `MockTransport`
    // class wires every real call up below, so reaching these spies
    // means the test is misconfigured.
    HttpClient: vi.fn().mockImplementation(function () {
      return {
        get: vi.fn(function () {
          throw new Error('HttpClient.get called — MockTransport was not wired up');
        }),
        post: vi.fn(function () {
          throw new Error('HttpClient.post called — MockTransport was not wired up');
        }),
        put: vi.fn(function () {
          throw new Error('HttpClient.put called — MockTransport was not wired up');
        }),
        delete: vi.fn(function () {
          throw new Error('HttpClient.delete called — MockTransport was not wired up');
        }),
        postFormData: vi.fn(function () {
          throw new Error('HttpClient.postFormData called — MockTransport was not wired up');
        }),
      };
    }),
  };
});

class MockTransport implements Transport {
  public readonly calls: Array<{ method: string; url: string; data?: unknown }> = [];
  public readonly responses = new Map<string, unknown>();

  setResponse(method: string, url: string, value: unknown): void {
    this.responses.set(`${method.toUpperCase()} ${url}`, value);
  }

  private resolve<T>(method: string, url: string): T {
    const key = `${method.toUpperCase()} ${url}`;
    if (!this.responses.has(key)) {
      throw new Error(`MockTransport: no response registered for ${key}`);
    }
    return this.responses.get(key) as T;
  }

  async get<T = unknown>(url: string): Promise<T> {
    this.calls.push({ method: 'GET', url });
    return this.resolve<T>('GET', url);
  }

  async post<T = unknown>(url: string, data?: unknown): Promise<T> {
    this.calls.push({ method: 'POST', url, data });
    return this.resolve<T>('POST', url);
  }

  async put<T = unknown>(url: string, data?: unknown): Promise<T> {
    this.calls.push({ method: 'PUT', url, data });
    return this.resolve<T>('PUT', url);
  }

  async delete<T = unknown>(url: string): Promise<T> {
    this.calls.push({ method: 'DELETE', url });
    return this.resolve<T>('DELETE', url);
  }

  async postFormData<T = unknown>(url: string, formData: FormData): Promise<T> {
    this.calls.push({ method: 'POST_FORM', url, data: formData });
    return this.resolve<T>('POST_FORM', url);
  }
}

describe('per-surface client transport routing (phase4 regression guard)', () => {
  let mock: MockTransport;

  beforeEach(() => {
    vi.clearAllMocks();
    (HttpClient as unknown as ReturnType<typeof vi.fn>).mockClear();
    mock = new MockTransport();
  });

  afterEach(() => {
    // Confirm every test routed through the mock — never fell back to HTTP.
    expect(HttpClient).not.toHaveBeenCalled();
  });

  it('BaseClient stores the injected transport and skips HTTP construction', () => {
    const client = new BaseClient({ transport: mock });
    expect(client).toBeInstanceOf(BaseClient);
    expect(client.getProtocol()).toBe('http');
  });

  it('CoreClient.healthCheck routes through the mock', async () => {
    mock.setResponse('GET', '/health', { status: 'ok', timestamp: '2026-04-19T00:00:00Z' });
    const client = new CoreClient({ transport: mock });
    const result = await client.healthCheck();
    expect(result).toEqual({ status: 'ok', timestamp: '2026-04-19T00:00:00Z' });
    expect(mock.calls).toEqual([{ method: 'GET', url: '/health' }]);
  });

  it('CollectionsClient.listCollections routes through the mock', async () => {
    mock.setResponse('GET', '/collections', { collections: [] });
    const client = new CollectionsClient({ transport: mock });
    const result = await client.listCollections();
    expect(result).toEqual([]);
    expect(mock.calls[0]).toEqual({ method: 'GET', url: '/collections' });
  });

  it('VectorsClient.deleteVectors routes through the mock', async () => {
    mock.setResponse('POST', '/collections/c1/vectors/delete', { deleted: 2 });
    const client = new VectorsClient({ transport: mock });
    const result = await client.deleteVectors('c1', ['v1', 'v2']);
    expect(result).toEqual({ deleted: 2 });
    expect(mock.calls[0]).toEqual({
      method: 'POST',
      url: '/collections/c1/vectors/delete',
      data: { vector_ids: ['v1', 'v2'] },
    });
  });

  it('SearchClient.searchVectors routes through the mock', async () => {
    mock.setResponse('POST', '/search', { results: [{ id: 'v1', score: 0.9 }] });
    const client = new SearchClient({ transport: mock });
    const result = await client.searchVectors('c1', {
      query_vector: [0.1, 0.2, 0.3],
      limit: 5,
    });
    expect(result.results).toHaveLength(1);
    expect(mock.calls[0]?.method).toBe('POST');
    expect(mock.calls[0]?.url).toBe('/search');
  });

  it('DiscoveryClient.discover routes through the mock', async () => {
    mock.setResponse('POST', '/discover', { bullets: ['summary'] });
    const client = new DiscoveryClient({ transport: mock });
    const result = await client.discover({ query: 'rust async' });
    expect(result).toEqual({ bullets: ['summary'] });
    expect(mock.calls[0]?.url).toBe('/discover');
  });

  it('FilesClient.getUploadConfig routes through the mock', async () => {
    mock.setResponse('GET', '/files/config', {
      max_file_size_mb: 50,
      allowed_extensions: ['md', 'rs'],
    });
    const client = new FilesClient({ transport: mock });
    const result = await client.getUploadConfig();
    expect(result.max_file_size_mb).toBe(50);
    expect(mock.calls[0]).toEqual({ method: 'GET', url: '/files/config' });
  });

  it('GraphClient.listGraphNodes routes through the mock', async () => {
    mock.setResponse('GET', '/graph/nodes/c1', { nodes: [], count: 0 });
    const client = new GraphClient({ transport: mock });
    const result = await client.listGraphNodes('c1');
    expect(result).toEqual({ nodes: [], count: 0 });
    expect(mock.calls[0]).toEqual({ method: 'GET', url: '/graph/nodes/c1' });
  });

  it('QdrantClient.qdrantListCollections routes through the mock', async () => {
    mock.setResponse('GET', '/qdrant/collections', { result: { collections: [] } });
    const client = new QdrantClient({ transport: mock });
    await client.qdrantListCollections();
    expect(mock.calls[0]).toEqual({ method: 'GET', url: '/qdrant/collections' });
  });

  it('AdminClient.getStatus routes through the mock', async () => {
    mock.setResponse('GET', '/status', {
      status: 'ok',
      version: '3.0.0',
      uptime: 1,
      collections: 0,
      total_vectors: 0,
    });
    const client = new AdminClient({ transport: mock });
    const result = await client.getStatus();
    expect(result.version).toBe('3.0.0');
    expect(mock.calls[0]).toEqual({ method: 'GET', url: '/status' });
  });

  it('VectorizerClient facade routes every surface through the same mock', async () => {
    mock.setResponse('GET', '/health', { status: 'ok', timestamp: 't' });
    mock.setResponse('GET', '/collections', { collections: [] });
    mock.setResponse('POST', '/discover', { bullets: [] });
    mock.setResponse('GET', '/graph/nodes/c1', { nodes: [], count: 0 });
    mock.setResponse('GET', '/qdrant/collections', { result: {} });
    mock.setResponse('GET', '/status', {
      status: 'ok',
      version: '3.0.0',
      uptime: 1,
      collections: 0,
      total_vectors: 0,
    });

    const client = new VectorizerClient({ transport: mock });

    await client.healthCheck();
    await client.listCollections();
    await client.discover({ query: 'rust' });
    await client.listGraphNodes('c1');
    await client.qdrantListCollections();
    await client.getStatus();

    expect(mock.calls.map((c) => `${c.method} ${c.url}`)).toEqual([
      'GET /health',
      'GET /collections',
      'POST /discover',
      'GET /graph/nodes/c1',
      'GET /qdrant/collections',
      'GET /status',
    ]);
  });
});
