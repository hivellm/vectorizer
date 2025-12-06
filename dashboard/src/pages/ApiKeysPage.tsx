/**
 * API Keys Management Page
 * Manage API keys for programmatic access
 */

import { useEffect, useState } from 'react';
import { useApiClient } from '@/hooks/useApiClient';
import { useAuth } from '@/contexts/AuthContext';
import Card from '@/components/ui/Card';
import Button from '@/components/ui/Button';
import Modal from '@/components/ui/Modal';
import { Input } from '@/components/ui/Input';
import { Select, SelectOption } from '@/components/ui/Select';
import { useToastContext } from '@/providers/ToastProvider';
import LoadingState from '@/components/LoadingState';
import { formatDate } from '@/utils/formatters';

interface ApiKey {
  id: string;
  name: string;
  key_prefix: string;
  permissions: string[];
  created_at: string;
  last_used_at?: string;
  expires_at?: string;
}

interface CreateKeyResponse {
  id: string;
  key: string;
  name: string;
  permissions: string[];
}

function ApiKeysPage() {
  const api = useApiClient();
  const { token } = useAuth();
  const toast = useToastContext();

  const [keys, setKeys] = useState<ApiKey[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showKeyModal, setShowKeyModal] = useState(false);
  const [showDeleteModal, setShowDeleteModal] = useState(false);
  const [selectedKey, setSelectedKey] = useState<ApiKey | null>(null);
  const [newKeyValue, setNewKeyValue] = useState<string | null>(null);

  const [createForm, setCreateForm] = useState({
    name: '',
    permissions: ['read'],
    expiresIn: '',
  });

  const [submitting, setSubmitting] = useState(false);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    loadKeys();
  }, []);

  const loadKeys = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await api.get<{ keys: ApiKey[] }>('/auth/keys', {
        headers: { Authorization: `Bearer ${token}` },
      });
      setKeys(data.keys || []);
    } catch (err) {
      console.error('Error loading API keys:', err);
      setError(err instanceof Error ? err.message : 'Failed to load API keys');
    } finally {
      setLoading(false);
    }
  };

  const handleCreateKey = async () => {
    if (!createForm.name.trim()) {
      toast.error('Key name is required');
      return;
    }

    setSubmitting(true);
    try {
      const data = await api.post<CreateKeyResponse>('/auth/keys', {
        name: createForm.name,
        permissions: createForm.permissions,
        expires_in_days: createForm.expiresIn ? parseInt(createForm.expiresIn) : null,
      }, {
        headers: { Authorization: `Bearer ${token}` },
      });

      setNewKeyValue(data.key);
      setShowCreateModal(false);
      setShowKeyModal(true);
      setCreateForm({ name: '', permissions: ['read'], expiresIn: '' });
      loadKeys();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to create API key');
    } finally {
      setSubmitting(false);
    }
  };

  const handleDeleteKey = async () => {
    if (!selectedKey) return;

    setSubmitting(true);
    try {
      await api.delete(`/auth/keys/${selectedKey.id}`, {
        headers: { Authorization: `Bearer ${token}` },
      });
      toast.success('API key revoked');
      setShowDeleteModal(false);
      setSelectedKey(null);
      loadKeys();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to revoke API key');
    } finally {
      setSubmitting(false);
    }
  };

  const copyToClipboard = async () => {
    if (!newKeyValue) return;
    try {
      await navigator.clipboard.writeText(newKeyValue);
      setCopied(true);
      toast.success('API key copied to clipboard');
      setTimeout(() => setCopied(false), 2000);
    } catch {
      toast.error('Failed to copy');
    }
  };

  const openDeleteModal = (key: ApiKey) => {
    setSelectedKey(key);
    setShowDeleteModal(true);
  };

  const togglePermission = (perm: string) => {
    if (createForm.permissions.includes(perm)) {
      setCreateForm({
        ...createForm,
        permissions: createForm.permissions.filter(p => p !== perm),
      });
    } else {
      setCreateForm({
        ...createForm,
        permissions: [...createForm.permissions, perm],
      });
    }
  };

  if (loading) {
    return <LoadingState message="Loading API keys..." />;
  }

  if (error) {
    return (
      <div className="p-6">
        <Card>
          <div className="p-6 text-center">
            <p className="text-red-500 mb-4">{error}</p>
            <Button onClick={loadKeys}>Retry</Button>
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
            API Keys
          </h1>
          <p className="text-sm text-neutral-500 dark:text-neutral-400 mt-1">
            Manage API keys for programmatic access to Vectorizer
          </p>
        </div>
        <Button onClick={() => setShowCreateModal(true)}>
          + Create API Key
        </Button>
      </div>

      <Card>
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-neutral-200 dark:border-neutral-700">
                <th className="text-left py-3 px-4 text-sm font-medium text-neutral-500 dark:text-neutral-400">Name</th>
                <th className="text-left py-3 px-4 text-sm font-medium text-neutral-500 dark:text-neutral-400">Key</th>
                <th className="text-left py-3 px-4 text-sm font-medium text-neutral-500 dark:text-neutral-400">Permissions</th>
                <th className="text-left py-3 px-4 text-sm font-medium text-neutral-500 dark:text-neutral-400">Created</th>
                <th className="text-left py-3 px-4 text-sm font-medium text-neutral-500 dark:text-neutral-400">Last Used</th>
                <th className="text-right py-3 px-4 text-sm font-medium text-neutral-500 dark:text-neutral-400">Actions</th>
              </tr>
            </thead>
            <tbody>
              {keys.length === 0 ? (
                <tr>
                  <td colSpan={6} className="py-8 text-center text-neutral-500">
                    No API keys found. Create one to get started.
                  </td>
                </tr>
              ) : (
                keys.map((key) => (
                  <tr key={key.id} className="border-b border-neutral-100 dark:border-neutral-800 last:border-0">
                    <td className="py-3 px-4">
                      <p className="font-medium text-neutral-900 dark:text-white">{key.name}</p>
                    </td>
                    <td className="py-3 px-4">
                      <code className="text-sm bg-neutral-100 dark:bg-neutral-800 px-2 py-1 rounded">
                        {key.key_prefix}...
                      </code>
                    </td>
                    <td className="py-3 px-4">
                      <div className="flex flex-wrap gap-1">
                        {key.permissions.map((perm) => (
                          <span
                            key={perm}
                            className={`inline-flex px-2 py-0.5 text-xs font-medium rounded ${
                              perm === 'admin'
                                ? 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400'
                                : perm === 'write'
                                ? 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400'
                                : 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400'
                            }`}
                          >
                            {perm}
                          </span>
                        ))}
                      </div>
                    </td>
                    <td className="py-3 px-4 text-sm text-neutral-500 dark:text-neutral-400">
                      {formatDate(key.created_at)}
                    </td>
                    <td className="py-3 px-4 text-sm text-neutral-500 dark:text-neutral-400">
                      {key.last_used_at ? formatDate(key.last_used_at) : 'Never'}
                    </td>
                    <td className="py-3 px-4">
                      <div className="flex justify-end">
                        <Button variant="danger" size="sm" onClick={() => openDeleteModal(key)}>
                          Revoke
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

      {/* Create Key Modal */}
      <Modal isOpen={showCreateModal} onClose={() => setShowCreateModal(false)} title="Create API Key">
        <div className="space-y-4">
          <Input
            label="Key Name"
            value={createForm.name}
            onChange={(e) => setCreateForm({ ...createForm, name: e.target.value })}
            placeholder="e.g., Production API Key"
          />

          <div>
            <label className="block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-2">
              Permissions
            </label>
            <div className="space-y-2">
              {['read', 'write', 'admin'].map((perm) => (
                <label key={perm} className="flex items-center gap-2 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={createForm.permissions.includes(perm)}
                    onChange={() => togglePermission(perm)}
                    className="rounded border-neutral-300 text-indigo-600 focus:ring-indigo-500"
                  />
                  <span className="text-sm text-neutral-700 dark:text-neutral-300 capitalize">
                    {perm}
                    {perm === 'read' && ' - View collections and search'}
                    {perm === 'write' && ' - Create, update, delete data'}
                    {perm === 'admin' && ' - Full administrative access'}
                  </span>
                </label>
              ))}
            </div>
          </div>

          <Select
            label="Expiration (optional)"
            value={createForm.expiresIn}
            onChange={(value) => setCreateForm({ ...createForm, expiresIn: value })}
          >
            <SelectOption id="never" value="">Never expires</SelectOption>
            <SelectOption id="7" value="7">7 days</SelectOption>
            <SelectOption id="30" value="30">30 days</SelectOption>
            <SelectOption id="90" value="90">90 days</SelectOption>
            <SelectOption id="365" value="365">1 year</SelectOption>
          </Select>

          <div className="flex justify-end gap-3 pt-4">
            <Button variant="secondary" onClick={() => setShowCreateModal(false)}>Cancel</Button>
            <Button onClick={handleCreateKey} disabled={submitting}>
              {submitting ? 'Creating...' : 'Create Key'}
            </Button>
          </div>
        </div>
      </Modal>

      {/* Show New Key Modal */}
      <Modal isOpen={showKeyModal} onClose={() => { setShowKeyModal(false); setNewKeyValue(null); }} title="API Key Created">
        <div className="space-y-4">
          <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
            <p className="text-sm text-yellow-800 dark:text-yellow-200 font-medium mb-2">
              Important: Copy your API key now!
            </p>
            <p className="text-sm text-yellow-700 dark:text-yellow-300">
              This is the only time you will see this key. Store it securely.
            </p>
          </div>

          <div className="relative">
            <code className="block w-full p-3 bg-neutral-100 dark:bg-neutral-800 rounded-lg text-sm font-mono break-all">
              {newKeyValue}
            </code>
            <Button
              variant="secondary"
              size="sm"
              className="absolute top-2 right-2"
              onClick={copyToClipboard}
            >
              {copied ? 'Copied!' : 'Copy'}
            </Button>
          </div>

          <div className="flex justify-end pt-4">
            <Button onClick={() => { setShowKeyModal(false); setNewKeyValue(null); }}>
              Done
            </Button>
          </div>
        </div>
      </Modal>

      {/* Delete Confirmation Modal */}
      <Modal isOpen={showDeleteModal} onClose={() => setShowDeleteModal(false)} title="Revoke API Key">
        <div className="space-y-4">
          <p className="text-neutral-600 dark:text-neutral-400">
            Are you sure you want to revoke the API key <strong>{selectedKey?.name}</strong>?
            Any applications using this key will immediately lose access.
          </p>
          <div className="flex justify-end gap-3 pt-4">
            <Button variant="secondary" onClick={() => setShowDeleteModal(false)}>Cancel</Button>
            <Button variant="danger" onClick={handleDeleteKey} disabled={submitting}>
              {submitting ? 'Revoking...' : 'Revoke Key'}
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  );
}

export default ApiKeysPage;
