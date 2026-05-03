import { useEffect, useState } from 'react';
import { useApiClient } from '@/hooks/useApiClient';
import { useAuth } from '@/contexts/AuthContext';
import {
  Icons,
  Pill,
  Card,
  CardHead,
  CardBody,
  Tbl,
  Th,
  Td,
} from '@/components/console';
import { formatNumber } from '@/utils/formatters';

interface ApiKey {
  id: string;
  name: string;
  // Backend currently exposes `key_prefix`; some payloads may also carry
  // `masked` or a pre-formatted `key_preview`. Accept all three.
  key_prefix?: string;
  key_preview?: string;
  masked?: string;
  permissions?: string[];
  // The new design renders a Role pill. Backend may not return this yet —
  // derive from `permissions` when missing.
  role?: string;
  calls?: number;
  last_used_at?: string | null;
  created_at?: string;
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

function deriveRole(k: ApiKey): string {
  if (k.role) return k.role;
  const perms = k.permissions ?? [];
  if (perms.includes('admin')) return 'Admin';
  if (perms.includes('write')) return 'ReadWrite';
  if (perms.includes('read')) return 'ReadOnly';
  return 'ReadOnly';
}

function deriveKeyPreview(k: ApiKey): string {
  if (k.key_preview) return k.key_preview;
  if (k.masked) return k.masked;
  if (k.key_prefix) return `${k.key_prefix}…`;
  return '—';
}

function ApiKeysPage() {
  const api = useApiClient();
  const { token } = useAuth();
  const [keys, setKeys] = useState<ApiKey[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      setLoading(true);
      setError(null);
      try {
        const data = await api.get<{ keys?: ApiKey[] }>('/auth/keys', {
          headers: token ? { Authorization: `Bearer ${token}` } : undefined,
        });
        if (cancelled) return;
        setKeys(Array.isArray(data?.keys) ? data.keys : []);
      } catch (err) {
        if (cancelled) return;
        setError(err instanceof Error ? err.message : 'Failed to load API keys');
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const totalCalls = keys.reduce((s, k) => s + (k.calls ?? 0), 0);

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">API Keys</h1>
          <p className="page-sub">
            Localhost-only management · {keys.length} active keys ·{' '}
            {formatNumber(totalCalls)} calls in last 30d
          </p>
        </div>
        <div className="row" style={{ gap: 8 }}>
          <button className="btn">
            <Icons.refresh size={13} />
            Audit log
          </button>
          {/* TODO(actions): wire create-key modal */}
          <button className="btn primary">
            <Icons.plus size={13} />
            Generate key
          </button>
        </div>
      </div>

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
                  <Th>Calls (30d)</Th>
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
                        </div>
                      </Td>
                      <Td className="mono" style={{ fontSize: 11, color: 'var(--text-2)' }}>
                        {deriveKeyPreview(k)}
                      </Td>
                      <Td>
                        <Pill tone={rolePillTone(role)} className="mono">
                          {role}
                        </Pill>
                      </Td>
                      <Td className="num">{formatNumber(k.calls ?? 0)}</Td>
                      <Td className="num muted">{k.last_used_at ?? '—'}</Td>
                      <Td className="num muted">{k.created_at ?? '—'}</Td>
                      <Td>
                        <div className="row" style={{ gap: 4, justifyContent: 'flex-end' }}>
                          {/* TODO(actions): wire copy/delete */}
                          <button className="btn sm" aria-label={`Copy ${k.name}`}>
                            <Icons.copy size={11} />
                          </button>
                          <button className="btn sm" aria-label={`Delete ${k.name}`}>
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
