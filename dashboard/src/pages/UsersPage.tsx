/**
 * Users Management Page
 * Admin-only page to manage dashboard users
 */

import { useEffect, useState } from 'react';
import { useApiClient } from '@/hooks/useApiClient';
import { useAuth } from '@/contexts/AuthContext';
import Card from '@/components/ui/Card';
import Button from '@/components/ui/Button';
import Modal from '@/components/ui/Modal';
import { Input } from '@/components/ui/Input';
import { Select, SelectOption } from '@/components/ui/Select';
import PasswordStrengthIndicator from '@/components/ui/PasswordStrengthIndicator';
import { useToastContext } from '@/providers/ToastProvider';
import LoadingState from '@/components/LoadingState';
import { formatDate } from '@/utils/formatters';

interface User {
  user_id: string;
  username: string;
  roles: string[];
  created_at?: string;
}

function UsersPage() {
  const api = useApiClient();
  const { user: currentUser, token } = useAuth();
  const toast = useToastContext();

  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showPasswordModal, setShowPasswordModal] = useState(false);
  const [showDeleteModal, setShowDeleteModal] = useState(false);
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

  useEffect(() => {
    loadUsers();
  }, []);

  const loadUsers = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await api.get<{ users: User[] }>('/auth/users', {
        headers: { Authorization: `Bearer ${token}` },
      });
      setUsers(data.users || []);
    } catch (err) {
      console.error('Error loading users:', err);
      setError(err instanceof Error ? err.message : 'Failed to load users');
    } finally {
      setLoading(false);
    }
  };

  const handleCreateUser = async () => {
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
      }, {
        headers: { Authorization: `Bearer ${token}` },
      });
      toast.success(`User "${createForm.username}" created successfully`);
      setShowCreateModal(false);
      setCreateForm({ username: '', password: '', confirmPassword: '', role: 'Viewer' });
      loadUsers();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to create user');
    } finally {
      setSubmitting(false);
    }
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
      }, {
        headers: { Authorization: `Bearer ${token}` },
      });
      toast.success('Password changed successfully');
      setShowPasswordModal(false);
      setPasswordForm({ newPassword: '', confirmPassword: '' });
      setSelectedUser(null);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to change password');
    } finally {
      setSubmitting(false);
    }
  };

  const handleDeleteUser = async () => {
    if (!selectedUser) return;

    setSubmitting(true);
    try {
      await api.delete(`/auth/users/${selectedUser.username}`, {
        headers: { Authorization: `Bearer ${token}` },
      });
      toast.success(`User "${selectedUser.username}" deleted`);
      setShowDeleteModal(false);
      setSelectedUser(null);
      loadUsers();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to delete user');
    } finally {
      setSubmitting(false);
    }
  };

  const openPasswordModal = (user: User) => {
    setSelectedUser(user);
    setPasswordForm({ newPassword: '', confirmPassword: '' });
    setShowPasswordModal(true);
  };

  const openDeleteModal = (user: User) => {
    setSelectedUser(user);
    setShowDeleteModal(true);
  };

  if (loading) {
    return <LoadingState message="Loading users..." />;
  }

  if (error) {
    return (
      <div className="p-6">
        <Card>
          <div className="p-6 text-center">
            <p className="text-red-500 mb-4">{error}</p>
            <Button onClick={loadUsers}>Retry</Button>
          </div>
        </Card>
      </div>
    );
  }

  return (
    <div className="p-4 md:p-6 space-y-6">
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
        <div>
          <h1 className="text-2xl font-semibold text-neutral-900 dark:text-white">
            User Management
          </h1>
          <p className="text-sm text-neutral-500 dark:text-neutral-400 mt-1">
            Manage dashboard users and their access
          </p>
        </div>
        <Button onClick={() => setShowCreateModal(true)}>
          + Create User
        </Button>
      </div>

      <Card>
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-neutral-200 dark:border-neutral-700">
                <th className="text-left py-3 px-4 text-sm font-medium text-neutral-500 dark:text-neutral-400">Username</th>
                <th className="text-left py-3 px-4 text-sm font-medium text-neutral-500 dark:text-neutral-400">Role</th>
                <th className="text-left py-3 px-4 text-sm font-medium text-neutral-500 dark:text-neutral-400">Created</th>
                <th className="text-right py-3 px-4 text-sm font-medium text-neutral-500 dark:text-neutral-400">Actions</th>
              </tr>
            </thead>
            <tbody>
              {users.length === 0 ? (
                <tr>
                  <td colSpan={4} className="py-8 text-center text-neutral-500">
                    No users found
                  </td>
                </tr>
              ) : (
                users.map((user) => (
                  <tr key={user.user_id} className="border-b border-neutral-100 dark:border-neutral-800 last:border-0">
                    <td className="py-3 px-4">
                      <div className="flex items-center gap-3">
                        <div className="w-8 h-8 rounded-full bg-indigo-600 flex items-center justify-center">
                          <span className="text-sm font-medium text-white">
                            {user.username.charAt(0).toUpperCase()}
                          </span>
                        </div>
                        <div>
                          <p className="font-medium text-neutral-900 dark:text-white">{user.username}</p>
                          {currentUser?.username === user.username && (
                            <span className="text-xs text-indigo-600 dark:text-indigo-400">(You)</span>
                          )}
                        </div>
                      </div>
                    </td>
                    <td className="py-3 px-4">
                      {user.roles.map((role) => (
                        <span
                          key={role}
                          className={`inline-flex px-2 py-0.5 text-xs font-medium rounded ${
                            role === 'Admin'
                              ? 'bg-indigo-100 text-indigo-700 dark:bg-indigo-900/30 dark:text-indigo-400'
                              : 'bg-neutral-100 text-neutral-700 dark:bg-neutral-800 dark:text-neutral-300'
                          }`}
                        >
                          {role}
                        </span>
                      ))}
                    </td>
                    <td className="py-3 px-4 text-sm text-neutral-500 dark:text-neutral-400">
                      {user.created_at ? formatDate(user.created_at) : '-'}
                    </td>
                    <td className="py-3 px-4">
                      <div className="flex justify-end gap-2">
                        <Button variant="secondary" size="sm" onClick={() => openPasswordModal(user)} title="Change Password">
                          Password
                        </Button>
                        <Button
                          variant="danger"
                          size="sm"
                          onClick={() => openDeleteModal(user)}
                          disabled={user.username === 'admin' || user.username === currentUser?.username}
                        >
                          Delete
                        </Button>
                      </div>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </Card>

      <Modal isOpen={showCreateModal} onClose={() => setShowCreateModal(false)} title="Create User">
        <div className="space-y-4">
          <Input label="Username" value={createForm.username} onChange={(e) => setCreateForm({ ...createForm, username: e.target.value })} placeholder="Enter username" />
          <div>
            <Input label="Password" type="password" value={createForm.password} onChange={(e) => setCreateForm({ ...createForm, password: e.target.value })} placeholder="Enter password" />
            <PasswordStrengthIndicator password={createForm.password} />
          </div>
          <Input label="Confirm Password" type="password" value={createForm.confirmPassword} onChange={(e) => setCreateForm({ ...createForm, confirmPassword: e.target.value })} placeholder="Confirm password" />
          <Select label="Role" value={createForm.role} onChange={(value) => setCreateForm({ ...createForm, role: value })}>
            <SelectOption id="Viewer" value="Viewer">Viewer - Read-only access</SelectOption>
            <SelectOption id="Admin" value="Admin">Admin - Full access</SelectOption>
          </Select>
          <div className="flex justify-end gap-3 pt-4">
            <Button variant="secondary" onClick={() => setShowCreateModal(false)}>Cancel</Button>
            <Button onClick={handleCreateUser} disabled={submitting}>{submitting ? 'Creating...' : 'Create User'}</Button>
          </div>
        </div>
      </Modal>

      <Modal isOpen={showPasswordModal} onClose={() => setShowPasswordModal(false)} title={`Change Password for ${selectedUser?.username || ''}`}>
        <div className="space-y-4">
          <div>
            <Input label="New Password" type="password" value={passwordForm.newPassword} onChange={(e) => setPasswordForm({ ...passwordForm, newPassword: e.target.value })} placeholder="Enter new password" />
            <PasswordStrengthIndicator password={passwordForm.newPassword} />
          </div>
          <Input label="Confirm Password" type="password" value={passwordForm.confirmPassword} onChange={(e) => setPasswordForm({ ...passwordForm, confirmPassword: e.target.value })} placeholder="Confirm new password" />
          <div className="flex justify-end gap-3 pt-4">
            <Button variant="secondary" onClick={() => setShowPasswordModal(false)}>Cancel</Button>
            <Button onClick={handleChangePassword} disabled={submitting}>{submitting ? 'Changing...' : 'Change Password'}</Button>
          </div>
        </div>
      </Modal>

      <Modal isOpen={showDeleteModal} onClose={() => setShowDeleteModal(false)} title="Delete User">
        <div className="space-y-4">
          <p className="text-neutral-600 dark:text-neutral-400">
            Are you sure you want to delete user <strong>{selectedUser?.username}</strong>? This action cannot be undone.
          </p>
          <div className="flex justify-end gap-3 pt-4">
            <Button variant="secondary" onClick={() => setShowDeleteModal(false)}>Cancel</Button>
            <Button variant="danger" onClick={handleDeleteUser} disabled={submitting}>{submitting ? 'Deleting...' : 'Delete User'}</Button>
          </div>
        </div>
      </Modal>
    </div>
  );
}

export default UsersPage;
