import { useEffect, useState } from 'react';
import { useApiClient } from '@/hooks/useApiClient';
import { useToastContext } from '@/providers/ToastProvider';
import {
  Icons,
  Pill,
  Sparkline,
  Card,
  CardHead,
  CardBody,
  Tbl,
  Th,
  Td,
} from '@/components/console';
import { formatNumber, formatDate } from '@/utils/formatters';

/** Mirrors `ApiKeyInfo` in `auth_handlers/types.rs` exactly. */
interface ApiKey {
  id: string;
  name: string;
  permissions: string[];
  usage_count: number;
  usage_24h: number;
  created_at: number;
  last_used: number | null;
  expires_at: number | null;
  active: boolean;
  scopes?: string[];
}

/** Response shape from `POST /auth/keys` (one-shot, key returned once). */
interface CreateApiKeyResponse {
  api_key: string;
  id: string;
  name: string;
  permissions: string[];
  expires_at: number | null;
  warning: string;
}

/** Response shape from `GET /auth/keys/{id}/usage?window=14`. */
interface KeyUsageBucket {
  date: string;
  count: number;
}

interface KeyUsageResponse {
  key: ApiKey;
  buckets: KeyUsageBucket[];
  window_total: number;
}

const PERMISSION_MATRIX: ReadonlyArray<readonly [string, boolean, boolean, boolean, boolean, boolean]> = [
  ['Admin',     true, true, true,  true,  true],
  ['ReadWrite', true, true, true,  false, false],
  ['Mcp',       true, true, false, false, false],
  ['ReadOnly',  true, false, false, false, false],
];

const PERMISSION_HEADERS = ['List/Search', 'Insert/Update', 'Delete', 'Reindex', 'Admin endpoints'] as const;

type RoleTone = 'magenta' | 'teal' | 'amber' | 'muted';

function rolePillTone(role: string): RoleTone {
  if (role === 'Admin') return 'magenta';
  if (role === 'ReadOnly') return 'muted';
  if (role === 'Mcp') return 'amber';
  return 'teal';
}

/** UI-only helper — derives a display label from `permissions[]`. */
function deriveRole(k: ApiKey): string {
  const perms = k.permissions ?? [];
  if (perms.includes('admin')) return 'Admin';
  if (perms.includes('write')) return 'ReadWrite';
  if (perms.includes('mcp')) return 'Mcp';
  if (perms.includes('read')) return 'ReadOnly';
  return 'ReadOnly';
}

/** Format a unix-second timestamp as a readable date, or em-dash when null. */
function formatUnixDate(ts: number | null | undefined): string {
  if (ts === null || ts === undefined) return '—';
  return formatDate(new Date(ts * 1000));
}

function ApiKeysPage() {
  const api = useApiClient();
  const toast = useToastContext();
  const [keys, setKeys] = useState<ApiKey[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Create key panel state
  const [createOpen, setCreateOpen] = useState(false);
  const [createForm, setCreateForm] = useState({ name: '', permission: 'read', expires_in: '' });
  const [creating, setCreating] = useState(false);

  // One-shot key material from the create/rotate response — cleared on dismiss
  const [newKeyMaterial, setNewKeyMaterial] = useState<string | null>(null);

  // Usage sparkline modal state
  const [usageModalKeyId, setUsageModalKeyId] = useState<string | null>(null);
  const [usageBuckets, setUsageBuckets] = useState<KeyUsageBucket[]>([]);
  const [usageTotal, setUsageTotal] = useState(0);
  const [usageLoading, setUsageLoading] = useState(false);

  const loadKeys = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await api.get<{ keys?: ApiKey[] }>('/auth/keys');
      setKeys(Array.isArray(data?.keys) ? data.keys : []);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load API keys');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    let cancelled = false;
    (async () => {
      setLoading(true);
      setError(null);
      try {
        const data = await api.get<{ keys?: ApiKey[] }>('/auth/keys');
        if (cancelled) return;
        setKeys(Array.isArray(data?.keys) ? data.keys : []);
      } catch (err) {
        if (cancelled) return;
        setError(err instanceof Error ? err.message : 'Failed to load API keys');
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => { cancelled = true; };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const handleCreate = async () => {
    if (!createForm.name.trim()) {
      toast.error('Key name is required');
      return;
    }
    setCreating(true);
    try {
      const body: { name: string; permissions: string[]; expires_in?: number } = {
        name: createForm.name.trim(),
        permissions: [createForm.permission],
      };
      if (createForm.expires_in) {
        const secs = parseInt(createForm.expires_in, 10);
        if (!isNaN(secs) && secs > 0) body.expires_in = secs;
      }
      const resp = await api.post<CreateApiKeyResponse>('/auth/keys', body);
      setNewKeyMaterial(resp.api_key);
      setCreateOpen(false);
      setCreateForm({ name: '', permission: 'read', expires_in: '' });
      await loadKeys();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to create API key');
    } finally {
      setCreating(false);
    }
  };

  const handleDelete = async (k: ApiKey) => {
    if (!window.confirm(`Delete API key "${k.name}"? This cannot be undone.`)) return;
    try {
      await api.delete(`/auth/keys/${k.id}`);
      toast.success(`API key "${k.name}" deleted`);
      await loadKeys();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to delete API key');
    }
  };

  const handleRotate = async (k: ApiKey) => {
    if (!window.confirm(`Rotate API key "${k.name}"? The old key will stop working immediately.`)) return;
    try {
      const resp = await api.post<CreateApiKeyResponse>(`/auth/keys/${k.id}/rotate`, {});
      setNewKeyMaterial(resp.api_key);
      await loadKeys();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to rotate API key');
    }
  };

  const openUsageModal = async (k: ApiKey) => {
    setUsageModalKeyId(k.id);
    setUsageBuckets([]);
    setUsageTotal(0);
    setUsageLoading(true);
    try {
      const resp = await api.get<KeyUsageResponse>(`/auth/keys/${k.id}/usage`, {
        params: { window: 14 },
      });
      setUsageBuckets(Array.isArray(resp?.buckets) ? resp.buckets : []);
      setUsageTotal(resp?.window_total ?? 0);
    } catch {
      // Endpoint not available — show empty chart without crashing
    } finally {
      setUsageLoading(false);
    }
  };

  const totalCalls = keys.reduce((s, k) => s + (k.usage_count ?? 0), 0);
  const total24h = keys.reduce((s, k) => s + (k.usage_24h ?? 0), 0);

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">API Keys</h1>
          <p className="page-sub">
            Localhost-only management · {keys.length} active keys ·{' '}
            {formatNumber(totalCalls)} total calls · {formatNumber(total24h)} today
          </p>
        </div>
        <div className="row" style={{ gap: 8 }}>
          <button className="btn" onClick={loadKeys} disabled={loading}>
            <Icons.refresh size={13} />
            Refresh
          </button>
          <button className="btn primary" onClick={() => setCreateOpen(true)}>
            <Icons.plus size={13} />
            Generate key
          </button>
        </div>
      </div>

      {/* One-shot new-key panel — surfaced after create or rotate */}
      {newKeyMaterial && (
        <>
          <div style={{ height: 14 }} />
          <Card>
            <CardHead
              title="Save your new API key"
              right={
                <button className="btn sm" onClick={() => setNewKeyMaterial(null)}>
                  <Icons.x size={11} />
                  Dismiss
                </button>
              }
            />
            <CardBody>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
                <div
                  className="row"
                  style={{ gap: 8, padding: '8px 12px', background: 'var(--surface-2)', borderRadius: 4, border: '1px solid var(--border)' }}
                >
                  <Pill tone="amber">warning</Pill>
                  <span style={{ color: 'var(--text-2)', fontSize: 12 }}>
                    This key will not be shown again. Copy it now and store it securely.
                  </span>
                </div>
                <div
                  className="mono"
                  style={{
                    fontSize: 13,
                    padding: '10px 12px',
                    background: 'var(--surface-3)',
                    borderRadius: 4,
                    border: '1px solid var(--border)',
                    wordBreak: 'break-all',
                    color: 'var(--text)',
                    userSelect: 'all',
                  }}
                >
                  {newKeyMaterial}
                </div>
                <div className="row" style={{ gap: 8, justifyContent: 'flex-end' }}>
                  <button
                    className="btn sm"
                    onClick={() => {
                      navigator.clipboard.writeText(newKeyMaterial).then(
                        () => toast.success('Copied to clipboard'),
                        () => toast.error('Copy failed — select the key text above'),
                      );
                    }}
                  >
                    <Icons.copy size={11} />
                    Copy
                  </button>
                  <button className="btn sm" onClick={() => setNewKeyMaterial(null)}>
                    <Icons.check size={11} />
                    I saved it
                  </button>
                </div>
              </div>
            </CardBody>
          </Card>
        </>
      )}

      <div style={{ height: 14 }} />

      <Card>
        <CardHead title="Keys" sub={loading ? 'loading…' : `${keys.length} active`} />
        <CardBody tight>
          {error && (
            <div style={{ padding: 14 }}>
              <Pill tone="red">{error}</Pill>
            </div>
          )}
          {!error && keys.length === 0 && !loading && (
            <div style={{ padding: 24, color: 'var(--text-2)', textAlign: 'center' }}>
              No API keys yet. Generate one above.
            </div>
          )}
          {keys.length > 0 && (
            <Tbl>
              <thead>
                <tr>
                  <Th>Name</Th>
                  <Th>Key</Th>
                  <Th>Role</Th>
                  <Th>Total calls</Th>
                  <Th>Last 24h</Th>
                  <Th>Last used</Th>
                  <Th>Created</Th>
                  <Th />
                </tr>
              </thead>
              <tbody>
                {keys.map((k) => {
                  const role = deriveRole(k);
                  return (
                    <tr key={k.id}>
                      <Td>
                        <div className="row" style={{ gap: 8 }}>
                          <Icons.shield size={13} className="muted" />
                          <span style={{ fontWeight: 500 }}>{k.name}</span>
                          {!k.active && (
                            <Pill tone="muted" className="mono">inactive</Pill>
                          )}
                        </div>
                      </Td>
                      {/* Server never returns key material on list */}
                      <Td className="mono" style={{ fontSize: 11, color: 'var(--text-3)' }}>
                        —
                      </Td>
                      <Td>
                        <Pill tone={rolePillTone(role)} className="mono">
                          {role}
                        </Pill>
                      </Td>
                      <Td className="num">{formatNumber(k.usage_count)}</Td>
                      <Td className="num">{formatNumber(k.usage_24h)}</Td>
                      <Td className="num muted">{formatUnixDate(k.last_used)}</Td>
                      <Td className="num muted">{formatUnixDate(k.created_at)}</Td>
                      <Td>
                        <div className="row" style={{ gap: 4, justifyContent: 'flex-end' }}>
                          <button
                            className="btn sm"
                            onClick={() => openUsageModal(k)}
                            aria-label={`Usage for ${k.name}`}
                          >
                            Usage
                          </button>
                          <button
                            className="btn sm"
                            onClick={() => handleRotate(k)}
                            aria-label={`Rotate ${k.name}`}
                          >
                            <Icons.refresh size={11} />
                          </button>
                          <button
                            className="btn sm"
                            onClick={() => handleDelete(k)}
                            aria-label={`Delete ${k.name}`}
                          >
                            <Icons.trash size={11} />
                          </button>
                        </div>
                      </Td>
                    </tr>
                  );
                })}
              </tbody>
            </Tbl>
          )}
        </CardBody>
      </Card>

      {/* Usage sparkline modal (inline panel) */}
      {usageModalKeyId !== null && (
        <>
          <div style={{ height: 14 }} />
          <Card>
            <CardHead
              title={`14-day usage · ${keys.find((k) => k.id === usageModalKeyId)?.name ?? usageModalKeyId}`}
              right={
                <button className="btn sm" onClick={() => setUsageModalKeyId(null)}>
                  <Icons.x size={11} />
                  Close
                </button>
              }
            />
            <CardBody>
              {usageLoading ? (
                <div style={{ color: 'var(--text-3)', fontSize: 12 }}>Loading…</div>
              ) : usageBuckets.length === 0 ? (
                <div style={{ color: 'var(--text-3)', fontSize: 12 }}>No usage data in this window.</div>
              ) : (
                <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
                  <div className="row" style={{ gap: 8, alignItems: 'center' }}>
                    <Sparkline
                      data={usageBuckets.map((b) => b.count)}
                      width={240}
                      height={40}
                      ariaLabel="14-day usage trend"
                    />
                    <span style={{ color: 'var(--text-2)', fontSize: 12 }}>
                      {formatNumber(usageTotal)} calls over 14 days
                    </span>
                  </div>
                  <div
                    style={{
                      display: 'grid',
                      gridTemplateColumns: 'repeat(auto-fill, minmax(130px, 1fr))',
                      gap: 4,
                    }}
                  >
                    {usageBuckets.map((b) => (
                      <div
                        key={b.date}
                        className="mono"
                        style={{ fontSize: 11, color: 'var(--text-3)' }}
                      >
                        {b.date}: {formatNumber(b.count)}
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </CardBody>
          </Card>
        </>
      )}

      {/* Create key panel */}
      {createOpen && (
        <>
          <div style={{ height: 14 }} />
          <Card>
            <CardHead
              title="Generate API key"
              right={
                <button className="btn sm" onClick={() => setCreateOpen(false)}>
                  <Icons.x size={11} />
                  Close
                </button>
              }
            />
            <CardBody>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                <label style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                  <span style={{ color: 'var(--text-2)', fontSize: 12 }}>Key name</span>
                  <input
                    className="input"
                    type="text"
                    value={createForm.name}
                    onChange={(e) => setCreateForm({ ...createForm, name: e.target.value })}
                  />
                </label>
                <label style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                  <span style={{ color: 'var(--text-2)', fontSize: 12 }}>Permissions</span>
                  <select
                    className="input"
                    value={createForm.permission}
                    onChange={(e) => setCreateForm({ ...createForm, permission: e.target.value })}
                  >
                    <option value="read">ReadOnly</option>
                    <option value="write">ReadWrite</option>
                    <option value="mcp">Mcp</option>
                    <option value="admin">Admin</option>
                  </select>
                </label>
                <label style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                  <span style={{ color: 'var(--text-2)', fontSize: 12 }}>
                    Expires in (seconds, blank = never)
                  </span>
                  <input
                    className="input"
                    type="number"
                    min={0}
                    value={createForm.expires_in}
                    onChange={(e) => setCreateForm({ ...createForm, expires_in: e.target.value })}
                  />
                </label>
                <div className="row" style={{ gap: 8, marginTop: 4, justifyContent: 'flex-end' }}>
                  <button className="btn" onClick={() => setCreateOpen(false)} disabled={creating}>
                    Cancel
                  </button>
                  <button className="btn primary" onClick={handleCreate} disabled={creating}>
                    <Icons.plus size={11} />
                    {creating ? 'Generating…' : 'Generate key'}
                  </button>
                </div>
              </div>
            </CardBody>
          </Card>
        </>
      )}

      <div style={{ height: 14 }} />

      <Card>
        <CardHead title="Permission matrix" />
        <CardBody tight>
          <Tbl>
            <thead>
              <tr>
                <Th>Role</Th>
                {PERMISSION_HEADERS.map((h) => (
                  <Th key={h}>{h}</Th>
                ))}
              </tr>
            </thead>
            <tbody>
              {PERMISSION_MATRIX.map(([role, ...perms]) => (
                <tr key={role as string}>
                  <Td>
                    <Pill tone={rolePillTone(role as string)} className="mono">
                      {role}
                    </Pill>
                  </Td>
                  {(perms as boolean[]).map((v, j) => (
                    <Td key={j}>
                      {v ? (
                        <Icons.check size={14} style={{ color: 'var(--green)' }} />
                      ) : (
                        <Icons.x size={14} style={{ color: 'var(--text-3)' }} />
                      )}
                    </Td>
                  ))}
                </tr>
              ))}
            </tbody>
          </Tbl>
        </CardBody>
      </Card>
    </div>
  );
}

export default ApiKeysPage;
