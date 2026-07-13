/**
 * Connections page — console-themed restyle.
 *
 * Visual restyle only: behaviour (managing the list of user-saved
 * Vectorizer server profiles via `useConnections` — local + remote
 * endpoints with optional bearer-token auth, per-row health checks,
 * setting an active connection) is preserved from the pre-redesign
 * version. The redesign brief has no dedicated mockup for Connections,
 * so this page applies the established Phase 3 recipe:
 *   - `.page` + `.page-head` shell with title/sub + toolbar buttons
 *   - `Kpi` strip with the headline counts (total / online / active)
 *   - real `<Tbl>` with rows-per-connection (Name, Endpoint, Type,
 *     Auth, Status, actions)
 *   - `StatusPill` for the health column, `Pill` for type / auth chips
 *   - `.btn` actions with `Icons.*`
 *   - no Tailwind utility classes, no `dark:` variants, no
 *     `@untitledui/icons` imports
 *
 * The legacy Add / Edit Modal dialogs are kept as inline panels with
 * `// TODO(actions)` notes until the console design ships a modal
 * primitive — same approach as BackupsPage / FileWatcherPage / UsersPage.
 */

import { useEffect, useState } from 'react';
import { useConnections, type Connection } from '@/hooks/useConnections';
import { useToastContext } from '@/providers/ToastProvider';
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

interface FormState {
  name: string;
  host: string;
  port: number;
  type: 'local' | 'remote';
  token: string;
}

const EMPTY_FORM: FormState = {
  name: '',
  host: 'localhost',
  port: 15002,
  type: 'local',
  token: '',
};

function ConnectionsPage() {
  const {
    connections,
    activeConnectionId,
    loading,
    addConnection,
    updateConnection,
    removeConnection,
    checkConnectionHealth,
    checkAllConnectionsHealth,
    testConnectionReachable,
    setActiveConnection,
  } = useConnections();
  const toast = useToastContext();

  const [addOpen, setAddOpen] = useState(false);
  const [editOpen, setEditOpen] = useState(false);
  const [editing, setEditing] = useState<Connection | null>(null);
  const [form, setForm] = useState<FormState>(EMPTY_FORM);
  const [checking, setChecking] = useState<string | null>(null);
  const [testingForm, setTestingForm] = useState(false);

  useEffect(() => {
    if (connections.length > 0) {
      checkAllConnectionsHealth();
    }
    // Mirror the legacy mount-only health probe: run once after the
    // initial load, not on every connections change.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const openAdd = () => {
    setForm(EMPTY_FORM);
    setAddOpen(true);
  };

  const openEdit = (conn: Connection) => {
    setEditing(conn);
    setForm({
      name: conn.name,
      host: conn.host,
      port: conn.port,
      type: conn.type,
      token: conn.auth?.token ?? '',
    });
    setEditOpen(true);
  };

  const closeAll = () => {
    setAddOpen(false);
    setEditOpen(false);
    setEditing(null);
  };

  const handleSave = () => {
    if (!form.name || !form.host || !form.port) {
      toast.error('Please fill in all required fields');
      return;
    }

    try {
      if (addOpen) {
        const id = addConnection({
          name: form.name,
          host: form.host,
          port: form.port,
          type: form.type,
          auth: form.token ? { token: form.token } : undefined,
        });
        toast.success('Connection added successfully');
        // Mirror the legacy 500ms-delayed health probe so the new row's
        // status is populated shortly after creation.
        setTimeout(() => checkConnectionHealth(id), 500);
      } else if (editing) {
        updateConnection(editing.id, {
          name: form.name,
          host: form.host,
          port: form.port,
          type: form.type,
          auth: form.token ? { token: form.token } : undefined,
        });
        toast.success('Connection updated successfully');
        setTimeout(() => checkConnectionHealth(editing.id), 500);
      }
      closeAll();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to save connection');
    }
  };

  const handleDelete = (id: string) => {
    if (window.confirm('Are you sure you want to delete this connection?')) {
      removeConnection(id);
      toast.success('Connection deleted');
    }
  };

  const handleCheckHealth = async (id: string) => {
    setChecking(id);
    try {
      await checkConnectionHealth(id);
    } finally {
      setChecking(null);
    }
  };

  const handleTestConnection = async () => {
    if (!form.host || !form.port) {
      toast.error('Enter a host and port before testing');
      return;
    }

    setTestingForm(true);
    try {
      const reachable = await testConnectionReachable(form.host, form.port, form.token || undefined);
      if (reachable) {
        toast.success(`Connection successful — ${form.host}:${form.port} is reachable`);
      } else {
        toast.error(`Connection failed — ${form.host}:${form.port} is not reachable`);
      }
    } finally {
      setTestingForm(false);
    }
  };

  // Map the localStorage health enum onto a StatusPill known status.
  // 'online' → healthy (green), 'connecting' → indexing (amber),
  // 'offline' → error (red). Anything else falls through as muted.
  const statusToPill = (status: Connection['status']): string => {
    switch (status) {
      case 'online':
        return 'healthy';
      case 'connecting':
        return 'indexing';
      case 'offline':
        return 'error';
      default:
        return status;
    }
  };

  const onlineCount = connections.filter((c) => c.status === 'online').length;
  const offlineCount = connections.filter((c) => c.status === 'offline').length;

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Connections</h1>
          <p className="page-sub">Manage connections to Vectorizer servers</p>
        </div>
        <div className="row" style={{ gap: 8 }}>
          <button
            className="btn"
            onClick={checkAllConnectionsHealth}
            disabled={loading || connections.length === 0}
          >
            <Icons.refresh size={13} />
            Check all
          </button>
          <button className="btn primary" onClick={openAdd}>
            <Icons.plus size={13} />
            New connection
          </button>
        </div>
      </div>

      <Card>
        <CardHead title="Overview" sub={loading ? 'loading…' : undefined} />
        <CardBody>
          <div className="row" style={{ gap: 24, flexWrap: 'wrap' }}>
            <Kpi label="Total connections" value={String(connections.length)} />
            <Kpi
              label="Online"
              value={String(onlineCount)}
              accent={onlineCount > 0 ? 'teal' : 'none'}
            />
            <Kpi
              label="Offline"
              value={String(offlineCount)}
              accent={offlineCount > 0 ? 'magenta' : 'none'}
            />
            <Kpi
              label="Active"
              value={
                activeConnectionId
                  ? connections.find((c) => c.id === activeConnectionId)?.name ?? '—'
                  : '—'
              }
            />
          </div>
        </CardBody>
      </Card>

      <div style={{ height: 14 }} />

      <Card>
        <CardHead
          title="Saved connections"
          sub={
            connections.length > 0
              ? `${connections.length} server${connections.length === 1 ? '' : 's'}`
              : undefined
          }
        />
        <CardBody tight>
          {loading && connections.length === 0 ? (
            <div style={{ padding: 24, color: 'var(--text-2)', textAlign: 'center' }}>
              Loading connections…
            </div>
          ) : connections.length === 0 ? (
            <div style={{ padding: 24, color: 'var(--text-2)', textAlign: 'center' }}>
              No connections yet · Add one above to point at a Vectorizer server.
            </div>
          ) : (
            <Tbl>
              <thead>
                <tr>
                  <Th>Name</Th>
                  <Th>Endpoint</Th>
                  <Th>Type</Th>
                  <Th>Auth</Th>
                  <Th>Status</Th>
                  <Th />
                </tr>
              </thead>
              <tbody>
                {connections.map((conn) => {
                  const isActive = activeConnectionId === conn.id;
                  return (
                    <tr key={conn.id}>
                      <Td>
                        <div className="row" style={{ gap: 8 }}>
                          <Icons.globe size={13} className="muted" />
                          <span style={{ fontWeight: 500 }}>{conn.name}</span>
                          {isActive && (
                            <Pill tone="teal">active</Pill>
                          )}
                        </div>
                      </Td>
                      <Td>
                        <span className="mono">
                          {conn.host}:{conn.port}
                        </span>
                      </Td>
                      <Td>
                        <Pill tone={conn.type === 'local' ? 'muted' : 'magenta'}>
                          {conn.type}
                        </Pill>
                      </Td>
                      <Td>
                        {conn.auth?.token ? (
                          <Pill tone="amber">token</Pill>
                        ) : (
                          <span style={{ color: 'var(--text-3)' }}>—</span>
                        )}
                      </Td>
                      <Td>
                        <StatusPill status={statusToPill(conn.status)} />
                      </Td>
                      <Td>
                        <div
                          className="row"
                          style={{ gap: 4, justifyContent: 'flex-end' }}
                        >
                          <button
                            className="btn sm"
                            onClick={() => handleCheckHealth(conn.id)}
                            disabled={checking === conn.id}
                            aria-label={`Check health of ${conn.name}`}
                          >
                            <Icons.refresh size={11} />
                            {checking === conn.id ? 'Checking…' : 'Check'}
                          </button>
                          {!isActive && (
                            <button
                              className="btn sm"
                              onClick={() => setActiveConnection(conn.id)}
                              aria-label={`Set ${conn.name} as active`}
                            >
                              <Icons.check size={11} />
                              Set active
                            </button>
                          )}
                          <button
                            className="btn sm"
                            onClick={() => openEdit(conn)}
                            aria-label={`Edit ${conn.name}`}
                          >
                            <Icons.settings size={11} />
                            Edit
                          </button>
                          <button
                            className="btn sm"
                            onClick={() => handleDelete(conn.id)}
                            aria-label={`Delete ${conn.name}`}
                          >
                            <Icons.trash size={11} />
                            Delete
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

      {(addOpen || editOpen) && (
        <>
          <div style={{ height: 14 }} />
          {/* TODO(actions): replace inline panel with a real modal once
              the console design ships a modal primitive. */}
          <Card>
            <CardHead
              title={addOpen ? 'Add connection' : `Edit connection · ${editing?.name ?? ''}`}
              right={
                <button className="btn sm" onClick={closeAll}>
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
                  <span style={{ color: 'var(--text-2)', fontSize: 12 }}>Name</span>
                  <input
                    className="input"
                    type="text"
                    value={form.name}
                    onChange={(e) => setForm({ ...form, name: e.target.value })}
                    placeholder="My local server"
                  />
                </label>
                <div
                  className="row"
                  style={{ gap: 12, flexWrap: 'wrap', alignItems: 'flex-end' }}
                >
                  <label
                    className="col"
                    style={{
                      display: 'flex',
                      flexDirection: 'column',
                      gap: 6,
                      flex: '1 1 200px',
                    }}
                  >
                    <span style={{ color: 'var(--text-2)', fontSize: 12 }}>Host</span>
                    <input
                      className="input"
                      type="text"
                      value={form.host}
                      onChange={(e) => setForm({ ...form, host: e.target.value })}
                      placeholder="localhost"
                    />
                  </label>
                  <label
                    className="col"
                    style={{
                      display: 'flex',
                      flexDirection: 'column',
                      gap: 6,
                      flex: '0 0 140px',
                    }}
                  >
                    <span style={{ color: 'var(--text-2)', fontSize: 12 }}>Port</span>
                    <input
                      className="input"
                      type="number"
                      value={form.port}
                      onChange={(e) =>
                        setForm({
                          ...form,
                          port: parseInt(e.target.value, 10) || 15002,
                        })
                      }
                      placeholder="15002"
                    />
                  </label>
                  <label
                    className="col"
                    style={{
                      display: 'flex',
                      flexDirection: 'column',
                      gap: 6,
                      flex: '0 0 140px',
                    }}
                  >
                    <span style={{ color: 'var(--text-2)', fontSize: 12 }}>Type</span>
                    <select
                      className="input"
                      value={form.type}
                      onChange={(e) =>
                        setForm({
                          ...form,
                          type: e.target.value as 'local' | 'remote',
                        })
                      }
                    >
                      <option value="local">Local</option>
                      <option value="remote">Remote</option>
                    </select>
                  </label>
                </div>
                <label
                  className="col"
                  style={{ display: 'flex', flexDirection: 'column', gap: 6 }}
                >
                  <span style={{ color: 'var(--text-2)', fontSize: 12 }}>
                    Auth token (optional)
                  </span>
                  <input
                    className="input"
                    type="password"
                    value={form.token}
                    onChange={(e) => setForm({ ...form, token: e.target.value })}
                    placeholder="Bearer token"
                  />
                </label>
                <div className="row" style={{ gap: 8, marginTop: 4 }}>
                  <button className="btn primary" onClick={handleSave}>
                    <Icons.check size={13} />
                    {addOpen ? 'Add connection' : 'Save changes'}
                  </button>
                  <button className="btn" onClick={handleTestConnection} disabled={testingForm}>
                    <Icons.zap size={13} />
                    {testingForm ? 'Testing…' : 'Test connection'}
                  </button>
                  <button className="btn" onClick={closeAll}>
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

export default ConnectionsPage;
