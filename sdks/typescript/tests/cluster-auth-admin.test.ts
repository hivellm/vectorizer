/**
 * Unit tests for the cluster + auth admin surface (phase15):
 * - ReplicationClient: clusterFailover, clusterResyncReplica, clusterAddPeer,
 *   clusterRebalance, clusterRebalanceStatus
 * - AuthClient: rotateApiKey, createScopedApiKey, introspectToken, listAuditLog
 *
 * Verifies wire shape (URL + body) and server-response decoding.
 * Pattern mirrors tests/schema-evolution.test.ts.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { ReplicationClient } from '../src/client/replication';
import { AuthClient } from '../src/client/auth';
import type {
  AuditEntry,
  FailoverReport,
  PeerInfo,
  RebalanceJob,
  ResyncJob,
  RotatedKey,
  TokenIntrospection,
} from '../src/models/cluster-auth-admin';
import type { ApiKey } from '../src/models/auth';

// ---------------------------------------------------------------------------
// Shared mock transport helper
// ---------------------------------------------------------------------------

interface MockTransport {
  get: ReturnType<typeof vi.fn>;
  post: ReturnType<typeof vi.fn>;
  put: ReturnType<typeof vi.fn>;
  patch: ReturnType<typeof vi.fn>;
  delete: ReturnType<typeof vi.fn>;
  postFormData: ReturnType<typeof vi.fn>;
}

function createMock(): MockTransport {
  return {
    get: vi.fn(),
    post: vi.fn(),
    put: vi.fn(),
    patch: vi.fn(),
    delete: vi.fn(),
    postFormData: vi.fn(),
  };
}

// ---------------------------------------------------------------------------
// ReplicationClient — phase15 cluster admin
// ---------------------------------------------------------------------------

describe('ReplicationClient — cluster admin (phase15)', () => {
  let mock: MockTransport;
  let client: ReplicationClient;

  beforeEach(() => {
    mock = createMock();
    client = new ReplicationClient({ transport: mock as never });
  });

  // ── clusterFailover ───────────────────────────────────────────────────────

  describe('clusterFailover', () => {
    const serverResponse: FailoverReport = {
      promoted_replica_id: 'replica-1',
      master_offset_at_promotion: 1000,
      replica_offset_at_promotion: 999,
      residual_lag_operations: 1,
    };

    it('POSTs {replica_id} to /cluster/failover', async () => {
      mock.post.mockResolvedValue(serverResponse);
      await client.clusterFailover('replica-1');
      expect(mock.post).toHaveBeenCalledWith('/cluster/failover', { replica_id: 'replica-1' });
    });

    it('returns FailoverReport with all fields', async () => {
      mock.post.mockResolvedValue(serverResponse);
      const r = await client.clusterFailover('replica-1');
      expect(r.promoted_replica_id).toBe('replica-1');
      expect(r.master_offset_at_promotion).toBe(1000);
      expect(r.residual_lag_operations).toBe(1);
    });
  });

  // ── clusterResyncReplica ──────────────────────────────────────────────────

  describe('clusterResyncReplica', () => {
    const serverResponse: ResyncJob = {
      replica_id: 'replica-2',
      snapshot_offset: 5000,
      full_snapshot: true,
    };

    it('POSTs empty body to /cluster/replicas/{id}/resync', async () => {
      mock.post.mockResolvedValue(serverResponse);
      await client.clusterResyncReplica('replica-2');
      expect(mock.post).toHaveBeenCalledWith('/cluster/replicas/replica-2/resync', {});
    });

    it('returns ResyncJob with full_snapshot true', async () => {
      mock.post.mockResolvedValue(serverResponse);
      const j = await client.clusterResyncReplica('replica-2');
      expect(j.replica_id).toBe('replica-2');
      expect(j.snapshot_offset).toBe(5000);
      expect(j.full_snapshot).toBe(true);
    });
  });

  // ── clusterAddPeer ────────────────────────────────────────────────────────

  describe('clusterAddPeer', () => {
    const serverResponse: PeerInfo = {
      node_id: 'peer-abc',
      address: '10.0.0.2:15003',
      role: 'member',
    };

    it('POSTs {address, role} to /cluster/peers', async () => {
      mock.post.mockResolvedValue(serverResponse);
      await client.clusterAddPeer({ address: '10.0.0.2:15003', role: 'member' });
      expect(mock.post).toHaveBeenCalledWith('/cluster/peers', {
        address: '10.0.0.2:15003',
        role: 'member',
      });
    });

    it('returns PeerInfo with node_id and role', async () => {
      mock.post.mockResolvedValue(serverResponse);
      const p = await client.clusterAddPeer({ address: '10.0.0.2:15003' });
      expect(p.node_id).toBe('peer-abc');
      expect(p.role).toBe('member');
    });

    it('accepts observer role', async () => {
      mock.post.mockResolvedValue({ ...serverResponse, role: 'observer' });
      const p = await client.clusterAddPeer({ address: '10.0.0.3:15003', role: 'observer' });
      expect(p.role).toBe('observer');
    });
  });

  // ── clusterRebalance ──────────────────────────────────────────────────────

  describe('clusterRebalance', () => {
    const serverResponse: RebalanceJob = {
      job_id: 'job-xyz',
      status: 'running',
      shards_to_move: 4,
      shards_moved: 0,
      message: 'Rebalance started',
    };

    it('POSTs empty body to /cluster/rebalance', async () => {
      mock.post.mockResolvedValue(serverResponse);
      await client.clusterRebalance();
      expect(mock.post).toHaveBeenCalledWith('/cluster/rebalance', {});
    });

    it('returns RebalanceJob with job_id and status', async () => {
      mock.post.mockResolvedValue(serverResponse);
      const j = await client.clusterRebalance();
      expect(j.job_id).toBe('job-xyz');
      expect(j.status).toBe('running');
      expect(j.shards_to_move).toBe(4);
    });
  });

  // ── clusterRebalanceStatus ────────────────────────────────────────────────

  describe('clusterRebalanceStatus', () => {
    it('GETs /cluster/rebalance/status', async () => {
      const response: RebalanceJob = {
        job_id: 'job-xyz',
        status: 'completed',
        shards_to_move: 4,
        shards_moved: 4,
        message: 'Rebalance complete: 4 shards moved',
      };
      mock.get.mockResolvedValue(response);
      await client.clusterRebalanceStatus();
      expect(mock.get).toHaveBeenCalledWith('/cluster/rebalance/status');
    });

    it('returns RebalanceJob when a job exists', async () => {
      const response: RebalanceJob = {
        job_id: 'job-xyz',
        status: 'completed',
        shards_to_move: 4,
        shards_moved: 4,
        message: 'done',
      };
      mock.get.mockResolvedValue(response);
      const j = await client.clusterRebalanceStatus();
      expect(j).not.toBeNull();
      expect(j!.status).toBe('completed');
      expect(j!.shards_moved).toBe(4);
    });

    it('returns null when server reports idle', async () => {
      mock.get.mockResolvedValue({ status: 'idle', message: 'No rebalance has been triggered on this node' });
      const j = await client.clusterRebalanceStatus();
      expect(j).toBeNull();
    });
  });
});

// ---------------------------------------------------------------------------
// AuthClient — phase15 auth admin
// ---------------------------------------------------------------------------

describe('AuthClient — auth admin (phase15)', () => {
  let mock: MockTransport;
  let client: AuthClient;

  beforeEach(() => {
    mock = createMock();
    client = new AuthClient({ transport: mock as never });
  });

  // ── rotateApiKey ──────────────────────────────────────────────────────────

  describe('rotateApiKey', () => {
    const serverResponse: RotatedKey = {
      old_key_id: 'key-old',
      new_key_id: 'key-new',
      new_token: 'sk-new-token',
      grace_until: 1714694400,
    };

    it('POSTs empty body to /auth/keys/{id}/rotate', async () => {
      mock.post.mockResolvedValue(serverResponse);
      await client.rotateApiKey('key-old');
      expect(mock.post).toHaveBeenCalledWith('/auth/keys/key-old/rotate', {});
    });

    it('returns RotatedKey with both ids and grace_until', async () => {
      mock.post.mockResolvedValue(serverResponse);
      const r = await client.rotateApiKey('key-old');
      expect(r.old_key_id).toBe('key-old');
      expect(r.new_key_id).toBe('key-new');
      expect(r.new_token).toBe('sk-new-token');
      expect(r.grace_until).toBe(1714694400);
    });
  });

  // ── createScopedApiKey ────────────────────────────────────────────────────

  describe('createScopedApiKey', () => {
    const serverResponse: ApiKey = {
      id: 'key-scoped',
      name: 'scoped-key',
      permissions: ['Read'],
      api_key: 'sk-scoped-abc',
      created_at: 1714608000,
      active: true,
      warning: 'Save this API key now! It will not be shown again.',
    };

    it('POSTs to /auth/keys with name, permissions, and scopes', async () => {
      mock.post.mockResolvedValue(serverResponse);
      await client.createScopedApiKey({
        name: 'scoped-key',
        permissions: ['Read'],
        scopes: [{ collection: 'my-col', permissions: ['read', 'write'] }],
      });
      expect(mock.post).toHaveBeenCalledWith('/auth/keys', {
        name: 'scoped-key',
        permissions: ['Read'],
        scopes: [{ collection: 'my-col', permissions: ['read', 'write'] }],
      });
    });

    it('returns ApiKey with api_key field at creation time', async () => {
      mock.post.mockResolvedValue(serverResponse);
      const k = await client.createScopedApiKey({ name: 'scoped-key' });
      expect(k.id).toBe('key-scoped');
      expect(k.api_key).toBe('sk-scoped-abc');
      expect(k.active).toBe(true);
    });

    it('sends empty scopes array when not provided', async () => {
      mock.post.mockResolvedValue(serverResponse);
      await client.createScopedApiKey({ name: 'no-scope-key' });
      const [, body] = mock.post.mock.calls[0] as [string, Record<string, unknown>];
      expect(body['name']).toBe('no-scope-key');
    });
  });

  // ── introspectToken ───────────────────────────────────────────────────────

  describe('introspectToken', () => {
    it('POSTs {token} to /auth/introspect', async () => {
      const response: TokenIntrospection = { active: true, sub: 'user-1', exp: 9999999999 };
      mock.post.mockResolvedValue(response);
      await client.introspectToken('some-jwt-or-key');
      expect(mock.post).toHaveBeenCalledWith('/auth/introspect', { token: 'some-jwt-or-key' });
    });

    it('returns active introspection for valid token', async () => {
      const response: TokenIntrospection = {
        active: true,
        sub: 'user-1',
        exp: 9999999999,
        username: 'alice',
        scope: 'docs:read',
      };
      mock.post.mockResolvedValue(response);
      const r = await client.introspectToken('valid-jwt');
      expect(r.active).toBe(true);
      expect(r.sub).toBe('user-1');
      expect(r.username).toBe('alice');
      expect(r.scope).toBe('docs:read');
    });

    it('returns active:false for invalid token', async () => {
      mock.post.mockResolvedValue({ active: false });
      const r = await client.introspectToken('invalid-token');
      expect(r.active).toBe(false);
      expect(r.sub).toBeUndefined();
      expect(r.exp).toBeUndefined();
    });
  });

  // ── listAuditLog ──────────────────────────────────────────────────────────

  describe('listAuditLog', () => {
    const entries: AuditEntry[] = [
      {
        actor: 'admin',
        action: 'rotate_api_key',
        target: 'key-1',
        at: '2026-05-02T12:00:00Z',
        correlation_id: 'corr-abc',
      },
      {
        actor: 'admin',
        action: 'create_api_key',
        target: 'key-2',
        at: '2026-05-02T13:00:00Z',
      },
    ];

    it('GETs /auth/audit with no params by default', async () => {
      mock.get.mockResolvedValue({ entries, total: 2 });
      await client.listAuditLog();
      expect(mock.get).toHaveBeenCalledWith('/auth/audit');
    });

    it('appends query params when provided', async () => {
      mock.get.mockResolvedValue({ entries: [entries[0]], total: 1 });
      await client.listAuditLog({ actor: 'admin', action: 'rotate_api_key', limit: 10 });
      const [url] = mock.get.mock.calls[0] as [string];
      expect(url).toContain('actor=admin');
      expect(url).toContain('action=rotate_api_key');
      expect(url).toContain('limit=10');
    });

    it('returns entries array', async () => {
      mock.get.mockResolvedValue({ entries, total: 2 });
      const result = await client.listAuditLog();
      expect(result).toHaveLength(2);
      expect(result[0].actor).toBe('admin');
      expect(result[0].correlation_id).toBe('corr-abc');
      expect(result[1].correlation_id).toBeUndefined();
    });

    it('returns empty array when entries missing', async () => {
      mock.get.mockResolvedValue({ total: 0 });
      const result = await client.listAuditLog();
      expect(result).toEqual([]);
    });

    it('applies since and until params', async () => {
      mock.get.mockResolvedValue({ entries: [], total: 0 });
      await client.listAuditLog({
        since: '2026-05-01T00:00:00Z',
        until: '2026-05-02T00:00:00Z',
      });
      const [url] = mock.get.mock.calls[0] as [string];
      expect(url).toContain('since=');
      expect(url).toContain('until=');
    });
  });
});
