/**
 * Backups page — console-themed restyle.
 *
 * Visual restyle only: behaviour (loading list of backups, creating a
 * new backup with a chosen subset of collections, and restoring an
 * existing backup into a target collection) is preserved from the
 * pre-redesign version. The redesign brief has no dedicated mockup for
 * Backups, so this page applies the established Phase 3 recipe:
 *   - `.page` + `.page-head` shell with title/sub + toolbar buttons
 *   - console `Card` / `CardHead` / `CardBody`, `Tbl` / `Th` / `Td`
 *   - `Kpi` cards for the headline metrics
 *   - `StatusPill` / `Pill` for backup status / collection chips
 *   - `.btn` actions with `Icons.*`
 *   - no Tailwind utility classes, no `dark:` variants
 *
 * The legacy "Create Backup" and "Restore Backup" modals are rendered
 * as inline panels below the table — flagged with `// TODO(actions)`
 * until the console design ships a modal primitive (matches the pattern
 * established by FileWatcherPage).
 */

import { useEffect, useState } from 'react';
import { useApiClient } from '@/hooks/useApiClient';
import { useCollections } from '@/hooks/useCollections';
import { useToastContext } from '@/providers/ToastProvider';
import { formatNumber, formatDate } from '@/utils/formatters';
import {
  Icons,
  Pill,
  StatusPill,
  Card,
  CardHead,
  CardBody,
  Kpi,
  Tbl,
  Th,
  Td,
} from '@/components/console';

interface Backup {
  id: string;
  name: string;
  date: string;
  size: number;
  collections: string[];
}

interface CollectionOption {
  name: string;
  vector_count?: number;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

function BackupsPage() {
  const api = useApiClient();
  const { listCollections } = useCollections();
  const toast = useToastContext();

  const [backups, setBackups] = useState<Backup[]>([]);
  const [collections, setCollections] = useState<CollectionOption[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [createOpen, setCreateOpen] = useState(false);
  const [restoreOpen, setRestoreOpen] = useState(false);
  const [selectedBackup, setSelectedBackup] = useState<Backup | null>(null);
  const [createForm, setCreateForm] = useState({
    name: '',
    collections: [] as string[],
  });
  const [restoreForm, setRestoreForm] = useState({ collection: '' });
  const [creating, setCreating] = useState(false);
  const [restoring, setRestoring] = useState(false);

  const loadData = async () => {
    setLoading(true);
    setError(null);
    try {
      const [backupsData, collectionsData] = await Promise.all([
        api.get<unknown>('/backups'),
        listCollections(),
      ]);

      // Handle both bare-array and `{ backups: [...] }` payloads.
      const list: Backup[] = Array.isArray(backupsData)
        ? (backupsData as Backup[])
        : Array.isArray((backupsData as { backups?: Backup[] })?.backups)
        ? (backupsData as { backups: Backup[] }).backups
        : [];

      setBackups(list);
      setCollections(Array.isArray(collectionsData) ? collectionsData : []);
    } catch (err) {
      console.error('Error loading backups:', err);
      setError(err instanceof Error ? err.message : 'Failed to load backups');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const openCreate = () => {
    setCreateForm({
      name: `backup-${new Date().toISOString().split('T')[0]}`,
      collections: [],
    });
    setCreateOpen(true);
  };

  const handleCreate = async () => {
    if (!createForm.name.trim()) {
      toast.error('Please enter a backup name');
      return;
    }
    if (createForm.collections.length === 0) {
      toast.error('Please select at least one collection');
      return;
    }

    setCreating(true);
    try {
      await api.post('/backups/create', {
        name: createForm.name,
        collections: createForm.collections,
      });
      toast.success('Backup created successfully');
      setCreateOpen(false);
      await loadData();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to create backup');
    } finally {
      setCreating(false);
    }
  };

  const openRestore = (backup: Backup) => {
    setSelectedBackup(backup);
    setRestoreForm({ collection: backup.collections[0] ?? '' });
    setRestoreOpen(true);
  };

  const handleRestore = async () => {
    if (!selectedBackup) return;
    if (
      !window.confirm(
        `Are you sure you want to restore backup "${selectedBackup.name}"? This will overwrite existing data in the selected collection.`,
      )
    ) {
      return;
    }

    setRestoring(true);
    try {
      await api.post('/backups/restore', {
        backup_id: selectedBackup.id,
        collection: restoreForm.collection || selectedBackup.collections[0],
      });
      toast.success('Backup restored successfully');
      setRestoreOpen(false);
      setSelectedBackup(null);
      await loadData();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to restore backup');
    } finally {
      setRestoring(false);
    }
  };

  const totalBytes = backups.reduce((s, b) => s + (b.size ?? 0), 0);
  const lastBackup = backups
    .map((b) => b.date)
    .filter(Boolean)
    .sort()
    .pop();
  const lastBackupLabel = lastBackup ? formatDate(lastBackup) : '—';

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Backups</h1>
          <p className="page-sub">Manage database backups and restorations</p>
        </div>
        <div className="row" style={{ gap: 8 }}>
          <button className="btn" onClick={loadData} disabled={loading}>
            <Icons.refresh size={13} />
            Refresh
          </button>
          <button className="btn primary" onClick={openCreate}>
            <Icons.plus size={13} />
            Create backup
          </button>
        </div>
      </div>

      {error && (
        <div style={{ marginBottom: 14 }}>
          <Card>
            <CardBody>
              <div className="row" style={{ gap: 8 }}>
                <Pill tone="red">error</Pill>
                <span style={{ color: 'var(--text-2)' }}>{error}</span>
              </div>
            </CardBody>
          </Card>
        </div>
      )}

      <Card>
        <CardHead title="Overview" sub={loading && backups.length === 0 ? 'loading…' : undefined} />
        <CardBody>
          <div className="row" style={{ gap: 24, flexWrap: 'wrap' }}>
            <Kpi label="Total backups" value={formatNumber(backups.length)} />
            <Kpi
              label="Total size"
              value={formatBytes(totalBytes)}
              accent={totalBytes > 0 ? 'teal' : 'none'}
            />
            <Kpi label="Last backup" value={lastBackupLabel} />
            <Kpi label="Collections tracked" value={formatNumber(collections.length)} />
          </div>
        </CardBody>
      </Card>

      <div style={{ height: 14 }} />

      <Card>
        <CardHead
          title="Backups"
          sub={backups.length > 0 ? `${backups.length} stored` : undefined}
        />
        <CardBody tight>
          {backups.length === 0 && !loading ? (
            <div style={{ padding: 24, color: 'var(--text-2)', textAlign: 'center' }}>
              No backups yet · Create one above to protect your data.
            </div>
          ) : (
            <Tbl>
              <thead>
                <tr>
                  <Th>Name</Th>
                  <Th>Created</Th>
                  <Th>Size</Th>
                  <Th>Collections</Th>
                  <Th>Status</Th>
                  <Th />
                </tr>
              </thead>
              <tbody>
                {backups.map((backup) => (
                  <tr key={backup.id}>
                    <Td>
                      <div className="row" style={{ gap: 8 }}>
                        <Icons.database size={13} className="muted" />
                        <span style={{ fontWeight: 500 }}>{backup.name}</span>
                      </div>
                      <div
                        className="mono"
                        style={{ fontSize: 11, color: 'var(--text-3)', marginTop: 2 }}
                      >
                        {backup.id.substring(0, 8)}…
                      </div>
                    </Td>
                    <Td className="num muted">{backup.date ? formatDate(backup.date) : '—'}</Td>
                    <Td className="num">{formatBytes(backup.size ?? 0)}</Td>
                    <Td>
                      <div className="row" style={{ gap: 4, flexWrap: 'wrap' }}>
                        {backup.collections.length === 0 ? (
                          <span style={{ color: 'var(--text-3)' }}>—</span>
                        ) : (
                          backup.collections.map((col) => (
                            <Pill key={col} tone="muted" className="mono">
                              {col}
                            </Pill>
                          ))
                        )}
                      </div>
                    </Td>
                    <Td>
                      <StatusPill status="healthy" />
                    </Td>
                    <Td>
                      <div className="row" style={{ gap: 4, justifyContent: 'flex-end' }}>
                        <button
                          className="btn sm primary"
                          onClick={() => openRestore(backup)}
                          aria-label={`Restore ${backup.name}`}
                        >
                          <Icons.refresh size={11} />
                          Restore
                        </button>
                      </div>
                    </Td>
                  </tr>
                ))}
              </tbody>
            </Tbl>
          )}
        </CardBody>
      </Card>

      {createOpen && (
        <>
          <div style={{ height: 14 }} />
          {/* TODO(actions): replace inline panel with a real modal once
              the console design ships a modal primitive. */}
          <Card>
            <CardHead
              title="Create backup"
              right={
                <button className="btn sm" onClick={() => setCreateOpen(false)}>
                  <Icons.x size={11} />
                  Close
                </button>
              }
            />
            <CardBody>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                <label
                  className="col"
                  style={{ display: 'flex', flexDirection: 'column', gap: 6 }}
                >
                  <span style={{ color: 'var(--text-2)', fontSize: 12 }}>Backup name</span>
                  <input
                    className="input"
                    type="text"
                    value={createForm.name}
                    onChange={(e) =>
                      setCreateForm({ ...createForm, name: e.target.value })
                    }
                    placeholder="backup-2026-05-01"
                  />
                </label>
                <div className="col" style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                  <span style={{ color: 'var(--text-2)', fontSize: 12 }}>
                    Collections to back up
                  </span>
                  {collections.length === 0 ? (
                    <div style={{ color: 'var(--text-3)', fontSize: 12 }}>
                      No collections available
                    </div>
                  ) : (
                    <div
                      style={{
                        display: 'flex',
                        flexDirection: 'column',
                        gap: 6,
                        maxHeight: 220,
                        overflowY: 'auto',
                        padding: 10,
                        border: '1px solid var(--border)',
                        borderRadius: 4,
                        background: 'var(--surface-2)',
                      }}
                    >
                      {collections.map((col) => (
                        <label
                          key={col.name}
                          className="row"
                          style={{ gap: 8, alignItems: 'center', cursor: 'pointer' }}
                        >
                          <input
                            type="checkbox"
                            checked={createForm.collections.includes(col.name)}
                            onChange={(e) => {
                              if (e.target.checked) {
                                setCreateForm({
                                  ...createForm,
                                  collections: [...createForm.collections, col.name],
                                });
                              } else {
                                setCreateForm({
                                  ...createForm,
                                  collections: createForm.collections.filter(
                                    (c) => c !== col.name,
                                  ),
                                });
                              }
                            }}
                          />
                          <span>{col.name}</span>
                          <span className="num muted" style={{ fontSize: 11 }}>
                            {formatNumber(col.vector_count ?? 0)} vectors
                          </span>
                        </label>
                      ))}
                    </div>
                  )}
                </div>
                <div className="row" style={{ gap: 8, marginTop: 4 }}>
                  <button
                    className="btn primary"
                    onClick={handleCreate}
                    disabled={
                      creating ||
                      !createForm.name.trim() ||
                      createForm.collections.length === 0
                    }
                  >
                    <Icons.check size={13} />
                    {creating ? 'Creating…' : 'Create backup'}
                  </button>
                  <button className="btn" onClick={() => setCreateOpen(false)}>
                    Cancel
                  </button>
                </div>
              </div>
            </CardBody>
          </Card>
        </>
      )}

      {restoreOpen && selectedBackup && (
        <>
          <div style={{ height: 14 }} />
          {/* TODO(actions): replace inline panel with a real modal once
              the console design ships a modal primitive. */}
          <Card>
            <CardHead
              title={`Restore backup · ${selectedBackup.name}`}
              right={
                <button
                  className="btn sm"
                  onClick={() => {
                    setRestoreOpen(false);
                    setSelectedBackup(null);
                  }}
                >
                  <Icons.x size={11} />
                  Close
                </button>
              }
            />
            <CardBody>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                <div className="row" style={{ gap: 8, alignItems: 'flex-start' }}>
                  <Pill tone="amber">warning</Pill>
                  <span style={{ color: 'var(--text-2)', fontSize: 12 }}>
                    Restoring this backup will overwrite all data in the selected
                    collection. This action cannot be undone.
                  </span>
                </div>
                <div style={{ color: 'var(--text-2)', fontSize: 12 }}>
                  Collections in backup: {selectedBackup.collections.join(', ') || '—'}
                </div>
                <label
                  className="col"
                  style={{ display: 'flex', flexDirection: 'column', gap: 6 }}
                >
                  <span style={{ color: 'var(--text-2)', fontSize: 12 }}>
                    Target collection
                  </span>
                  <select
                    className="input"
                    value={restoreForm.collection}
                    onChange={(e) => setRestoreForm({ collection: e.target.value })}
                  >
                    <option value="">Select collection…</option>
                    {selectedBackup.collections.map((col) => (
                      <option key={col} value={col}>
                        {col}
                      </option>
                    ))}
                  </select>
                </label>
                <div className="row" style={{ gap: 8, marginTop: 4 }}>
                  <button
                    className="btn primary"
                    onClick={handleRestore}
                    disabled={restoring || !restoreForm.collection}
                  >
                    <Icons.refresh size={13} />
                    {restoring ? 'Restoring…' : 'Restore backup'}
                  </button>
                  <button
                    className="btn"
                    onClick={() => {
                      setRestoreOpen(false);
                      setSelectedBackup(null);
                    }}
                  >
                    Cancel
                  </button>
                </div>
              </div>
            </CardBody>
          </Card>
        </>
      )}
    </div>
  );
}

export default BackupsPage;
