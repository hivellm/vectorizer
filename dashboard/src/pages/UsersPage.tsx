/**
 * Users page — console-themed restyle.
 *
 * Visual restyle only: behaviour (loading the user list, creating a
 * user, changing a user password, and deleting a user) is preserved
 * from the pre-redesign version. The redesign brief has no dedicated
 * mockup for Users, so this page applies the established Phase 3
 * recipe:
 *   - `.page` + `.page-head` shell with title/sub + toolbar buttons
 *   - console `Card` / `CardHead` / `CardBody`, `Tbl` / `Th` / `Td`
 *   - `Pill` for the role chips (Admin = magenta, write/User = teal,
 *     ReadOnly/read = muted)
 *   - `.btn` actions with `Icons.*`
 *   - no Tailwind utility classes, no `dark:` variants
 *   - drop `@untitledui/icons` and `@/components/ui/*` imports
 *
 * The legacy "Create user", "Change password" and "Delete user"
 * modals are rendered as inline panels below the table — flagged with
 * `// TODO(actions)` until the console design ships a modal primitive
 * (matches the modal-deferral pattern from BackupsPage / FileWatcher).
 */

import { useEffect, useState } from 'react';
import { useApiClient } from '@/hooks/useApiClient';
import { useAuth } from '@/contexts/AuthContext';
import { useToastContext } from '@/providers/ToastProvider';
import { formatDate } from '@/utils/formatters';
import {
  Icons,
  Pill,
  type PillTone,
  Card,
  CardHead,
  CardBody,
  Tbl,
  Th,
  Td,
} from '@/components/console';

interface User {
  user_id: string;
  username: string;
  roles: string[];
  created_at?: string;
  last_login_at?: string | null;
}

// Role names map to redesign tones. Backend currently emits PascalCase
// (`Admin`, `User`, `ReadOnly`, ...) — the brief mocks reference the
// lowercase shorthand (`admin`, `write`, `read`). Both are recognised.
function rolePillTone(role: string): PillTone {
  const r = role.toLowerCase();
  if (r === 'admin') return 'magenta';
  if (r === 'write' || r === 'user' || r === 'apiuser' || r === 'service') {
    return 'teal';
  }
  if (r === 'read' || r === 'readonly' || r === 'viewer') return 'muted';
  return 'default';
}

function UsersPage() {
  const api = useApiClient();
  const { user: currentUser, token } = useAuth();
  const toast = useToastContext();

  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [createOpen, setCreateOpen] = useState(false);
  const [passwordOpen, setPasswordOpen] = useState(false);
  const [deleteOpen, setDeleteOpen] = useState(false);
  const [selectedUser, setSelectedUser] = useState<User | null>(null);

  const [createForm, setCreateForm] = useState({
    username: '',
    password: '',
    confirmPassword: '',
    role: 'Viewer',
  });
  const [passwordForm, setPasswordForm] = useState({
    newPassword: '',
    confirmPassword: '',
  });

  const [submitting, setSubmitting] = useState(false);

  const authHeader = token ? { Authorization: `Bearer ${token}` } : undefined;

  const loadUsers = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await api.get<{ users?: User[] }>('/auth/users', {
        headers: authHeader,
      });
      setUsers(Array.isArray(data?.users) ? data.users : []);
    } catch (err) {
      console.error('Error loading users:', err);
      setError(err instanceof Error ? err.message : 'Failed to load users');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadUsers();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const openCreate = () => {
    setCreateForm({ username: '', password: '', confirmPassword: '', role: 'Viewer' });
    setCreateOpen(true);
  };

  const handleCreate = async () => {
    if (!createForm.username.trim()) {
      toast.error('Username is required');
      return;
    }
    if (createForm.password.length < 6) {
      toast.error('Password must be at least 6 characters');
      return;
    }
    if (createForm.password !== createForm.confirmPassword) {
      toast.error('Passwords do not match');
      return;
    }

    setSubmitting(true);
    try {
      await api.post('/auth/users', {
        username: createForm.username,
        password: createForm.password,
        roles: [createForm.role],
      }, { headers: authHeader });
      toast.success(`User "${createForm.username}" created successfully`);
      setCreateOpen(false);
      await loadUsers();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to create user');
    } finally {
      setSubmitting(false);
    }
  };

  const openPassword = (user: User) => {
    setSelectedUser(user);
    setPasswordForm({ newPassword: '', confirmPassword: '' });
    setPasswordOpen(true);
  };

  const handleChangePassword = async () => {
    if (!selectedUser) return;

    if (passwordForm.newPassword.length < 6) {
      toast.error('Password must be at least 6 characters');
      return;
    }
    if (passwordForm.newPassword !== passwordForm.confirmPassword) {
      toast.error('Passwords do not match');
      return;
    }

    setSubmitting(true);
    try {
      await api.put(`/auth/users/${selectedUser.username}/password`, {
        new_password: passwordForm.newPassword,
      }, { headers: authHeader });
      toast.success('Password changed successfully');
      setPasswordOpen(false);
      setSelectedUser(null);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to change password');
    } finally {
      setSubmitting(false);
    }
  };

  const openDelete = (user: User) => {
    setSelectedUser(user);
    setDeleteOpen(true);
  };

  const handleDelete = async () => {
    if (!selectedUser) return;

    setSubmitting(true);
    try {
      await api.delete(`/auth/users/${selectedUser.username}`, {
        headers: authHeader,
      });
      toast.success(`User "${selectedUser.username}" deleted`);
      setDeleteOpen(false);
      setSelectedUser(null);
      await loadUsers();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to delete user');
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Users</h1>
          <p className="page-sub">
            Manage dashboard users and their access · {users.length} active
          </p>
        </div>
        <div className="row" style={{ gap: 8 }}>
          <button className="btn" onClick={loadUsers} disabled={loading}>
            <Icons.refresh size={13} />
            Refresh
          </button>
          <button className="btn primary" onClick={openCreate}>
            <Icons.plus size={13} />
            Create user
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
        <CardHead
          title="Users"
          sub={loading ? 'loading…' : users.length > 0 ? `${users.length} active` : undefined}
        />
        <CardBody tight>
          {users.length === 0 && !loading && !error ? (
            <div style={{ padding: 24, color: 'var(--text-2)', textAlign: 'center' }}>
              No users yet · Create one above to grant dashboard access.
            </div>
          ) : (
            <Tbl>
              <thead>
                <tr>
                  <Th>Username</Th>
                  <Th>Roles</Th>
                  <Th>Created</Th>
                  <Th>Last login</Th>
                  <Th>Actions</Th>
                </tr>
              </thead>
              <tbody>
                {users.map((user) => {
                  const isCurrent = currentUser?.username === user.username;
                  const isAdminUser = user.username === 'admin';
                  return (
                    <tr key={user.user_id}>
                      <Td>
                        <div className="row" style={{ gap: 8 }}>
                          <Icons.shield size={13} className="muted" />
                          <span style={{ fontWeight: 500 }}>{user.username}</span>
                          {isCurrent && (
                            <Pill tone="teal" className="mono">
                              you
                            </Pill>
                          )}
                        </div>
                      </Td>
                      <Td>
                        <div className="row" style={{ gap: 4, flexWrap: 'wrap' }}>
                          {user.roles.length === 0 ? (
                            <span style={{ color: 'var(--text-3)' }}>—</span>
                          ) : (
                            user.roles.map((role) => (
                              <Pill key={role} tone={rolePillTone(role)} className="mono">
                                {role}
                              </Pill>
                            ))
                          )}
                        </div>
                      </Td>
                      <Td className="num muted">
                        {user.created_at ? formatDate(user.created_at) : '—'}
                      </Td>
                      <Td className="num muted">
                        {user.last_login_at ? formatDate(user.last_login_at) : '—'}
                      </Td>
                      <Td>
                        <div className="row" style={{ gap: 4, justifyContent: 'flex-end' }}>
                          <button
                            className="btn sm"
                            onClick={() => openPassword(user)}
                            aria-label={`Change password for ${user.username}`}
                          >
                            <Icons.keys size={11} />
                            Password
                          </button>
                          <button
                            className="btn sm"
                            onClick={() => openDelete(user)}
                            disabled={isAdminUser || isCurrent}
                            aria-label={`Delete ${user.username}`}
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

      {createOpen && (
        <>
          <div style={{ height: 14 }} />
          {/* TODO(actions): replace inline panel with a real modal once
              the console design ships a modal primitive. */}
          <Card>
            <CardHead
              title="Create user"
              right={
                <button className="btn sm" onClick={() => setCreateOpen(false)}>
                  <Icons.x size={11} />
                  Close
                </button>
              }
            />
            <CardBody>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                <label className="col" style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                  <span style={{ color: 'var(--text-2)', fontSize: 12 }}>Username</span>
                  <input
                    className="input"
                    type="text"
                    value={createForm.username}
                    onChange={(e) => setCreateForm({ ...createForm, username: e.target.value })}
                    placeholder="alice"
                  />
                </label>
                <label className="col" style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                  <span style={{ color: 'var(--text-2)', fontSize: 12 }}>Password</span>
                  <input
                    className="input"
                    type="password"
                    value={createForm.password}
                    onChange={(e) => setCreateForm({ ...createForm, password: e.target.value })}
                    placeholder="At least 6 characters"
                  />
                </label>
                <label className="col" style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                  <span style={{ color: 'var(--text-2)', fontSize: 12 }}>Confirm password</span>
                  <input
                    className="input"
                    type="password"
                    value={createForm.confirmPassword}
                    onChange={(e) =>
                      setCreateForm({ ...createForm, confirmPassword: e.target.value })
                    }
                    placeholder="Repeat password"
                  />
                </label>
                <label className="col" style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                  <span style={{ color: 'var(--text-2)', fontSize: 12 }}>Role</span>
                  <select
                    className="input"
                    value={createForm.role}
                    onChange={(e) => setCreateForm({ ...createForm, role: e.target.value })}
                  >
                    <option value="Viewer">Viewer — Read-only access</option>
                    <option value="Admin">Admin — Full access</option>
                  </select>
                </label>
                <div className="row" style={{ gap: 8, marginTop: 4, justifyContent: 'flex-end' }}>
                  <button
                    className="btn"
                    onClick={() => setCreateOpen(false)}
                    disabled={submitting}
                  >
                    Cancel
                  </button>
                  <button className="btn primary" onClick={handleCreate} disabled={submitting}>
                    <Icons.plus size={11} />
                    {submitting ? 'Creating…' : 'Create user'}
                  </button>
                </div>
              </div>
            </CardBody>
          </Card>
        </>
      )}

      {passwordOpen && selectedUser && (
        <>
          <div style={{ height: 14 }} />
          {/* TODO(actions): replace inline panel with a real modal once
              the console design ships a modal primitive. */}
          <Card>
            <CardHead
              title={`Change password — ${selectedUser.username}`}
              right={
                <button className="btn sm" onClick={() => setPasswordOpen(false)}>
                  <Icons.x size={11} />
                  Close
                </button>
              }
            />
            <CardBody>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                <label className="col" style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                  <span style={{ color: 'var(--text-2)', fontSize: 12 }}>New password</span>
                  <input
                    className="input"
                    type="password"
                    value={passwordForm.newPassword}
                    onChange={(e) =>
                      setPasswordForm({ ...passwordForm, newPassword: e.target.value })
                    }
                    placeholder="At least 6 characters"
                  />
                </label>
                <label className="col" style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                  <span style={{ color: 'var(--text-2)', fontSize: 12 }}>Confirm password</span>
                  <input
                    className="input"
                    type="password"
                    value={passwordForm.confirmPassword}
                    onChange={(e) =>
                      setPasswordForm({ ...passwordForm, confirmPassword: e.target.value })
                    }
                    placeholder="Repeat password"
                  />
                </label>
                <div className="row" style={{ gap: 8, marginTop: 4, justifyContent: 'flex-end' }}>
                  <button
                    className="btn"
                    onClick={() => setPasswordOpen(false)}
                    disabled={submitting}
                  >
                    Cancel
                  </button>
                  <button
                    className="btn primary"
                    onClick={handleChangePassword}
                    disabled={submitting}
                  >
                    <Icons.keys size={11} />
                    {submitting ? 'Changing…' : 'Change password'}
                  </button>
                </div>
              </div>
            </CardBody>
          </Card>
        </>
      )}

      {deleteOpen && selectedUser && (
        <>
          <div style={{ height: 14 }} />
          {/* TODO(actions): replace inline panel with a real modal once
              the console design ships a modal primitive. */}
          <Card>
            <CardHead
              title="Delete user"
              right={
                <button className="btn sm" onClick={() => setDeleteOpen(false)}>
                  <Icons.x size={11} />
                  Close
                </button>
              }
            />
            <CardBody>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                <p style={{ color: 'var(--text-2)' }}>
                  Are you sure you want to delete user{' '}
                  <strong style={{ color: 'var(--text-1)' }}>
                    {selectedUser.username}
                  </strong>
                  ? This action cannot be undone.
                </p>
                <div className="row" style={{ gap: 8, marginTop: 4, justifyContent: 'flex-end' }}>
                  <button
                    className="btn"
                    onClick={() => setDeleteOpen(false)}
                    disabled={submitting}
                  >
                    Cancel
                  </button>
                  <button className="btn primary" onClick={handleDelete} disabled={submitting}>
                    <Icons.trash size={11} />
                    {submitting ? 'Deleting…' : 'Delete user'}
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

export default UsersPage;
