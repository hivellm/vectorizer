/**
 * Backups page - Manage backups
 */

import { useEffect, useState } from 'react';
import { useApiClient } from '@/hooks/useApiClient';
import { useCollections } from '@/hooks/useCollections';
import Card from '@/components/ui/Card';
import Button from '@/components/ui/Button';
import Modal from '@/components/ui/Modal';
import { Select } from '@/components/ui/Select';
import { useToastContext } from '@/providers/ToastProvider';
import LoadingState from '@/components/LoadingState';
import { formatNumber, formatDate } from '@/utils/formatters';
import { Plus, RefreshCw01, AlertCircle, CheckCircle } from '@untitledui/icons';

interface Backup {
  id: string;
  name: string;
  date: string;
  size: number;
  collections: string[];
}

function BackupsPage() {
  const api = useApiClient();
  const { listCollections } = useCollections();
  const toast = useToastContext();

  const [backups, setBackups] = useState<Backup[]>([]);
  const [collections, setCollections] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showRestoreModal, setShowRestoreModal] = useState(false);
  const [selectedBackup, setSelectedBackup] = useState<Backup | null>(null);
  const [createForm, setCreateForm] = useState({
    name: '',
    collections: [] as string[],
  });
  const [restoreForm, setRestoreForm] = useState({
    collection: '',
  });
  const [creating, setCreating] = useState(false);
  const [restoring, setRestoring] = useState(false);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    setLoading(true);
    setError(null);
    try {
      const [backupsData, collectionsData] = await Promise.all([
        api.get<any>('/api/backups'),
        listCollections(),
      ]);
      
      // Handle different response formats
      const backupsList = Array.isArray(backupsData) 
        ? backupsData 
        : Array.isArray(backupsData.backups) 
        ? backupsData.backups 
        : [];
      
      setBackups(backupsList);
      setCollections(Array.isArray(collectionsData) ? collectionsData : []);
    } catch (err) {
      console.error('Error loading backups:', err);
      setError(err instanceof Error ? err.message : 'Failed to load backups');
    } finally {
      setLoading(false);
    }
  };

  const handleCreate = () => {
    setCreateForm({
      name: `backup-${new Date().toISOString().split('T')[0]}`,
      collections: [],
    });
    setShowCreateModal(true);
  };

  const handleCreateBackup = async () => {
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
      await api.post('/api/backups/create', {
        name: createForm.name,
        collections: createForm.collections,
      });
      toast.success('Backup created successfully');
      setShowCreateModal(false);
      await loadData();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to create backup');
    } finally {
      setCreating(false);
    }
  };

  const handleRestore = (backup: Backup) => {
    setSelectedBackup(backup);
    setRestoreForm({ collection: '' });
    setShowRestoreModal(true);
  };

  const handleRestoreBackup = async () => {
    if (!selectedBackup) return;

    if (!window.confirm(
      `Are you sure you want to restore backup "${selectedBackup.name}"? This will overwrite existing data in the selected collection.`
    )) {
      return;
    }

    setRestoring(true);
    try {
      await api.post('/api/backups/restore', {
        backup_id: selectedBackup.id,
        collection: restoreForm.collection || selectedBackup.collections[0],
      });
      toast.success('Backup restored successfully');
      setShowRestoreModal(false);
      setSelectedBackup(null);
      await loadData();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to restore backup');
    } finally {
      setRestoring(false);
    }
  };

  const formatBytes = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  };

  if (loading && backups.length === 0) {
    return <LoadingState message="Loading backups..." />;
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-xl sm:text-2xl font-bold text-neutral-900 dark:text-white">Backups</h1>
          <p className="text-sm sm:text-base text-neutral-600 dark:text-neutral-400 mt-1">
            Manage database backups and restorations
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="secondary" size="sm" onClick={loadData}>
            <RefreshCw01 className="w-4 h-4 mr-2" />
            Refresh
          </Button>
          <Button variant="primary" size="sm" onClick={handleCreate}>
            <Plus className="w-4 h-4 mr-2" />
            Create Backup
          </Button>
        </div>
      </div>

      {/* Error Message */}
      {error && (
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
          <div className="flex items-center gap-2">
            <AlertCircle className="w-5 h-5 text-red-600 dark:text-red-400" />
            <p className="text-sm text-red-800 dark:text-red-300">{error}</p>
          </div>
        </div>
      )}

      {/* Backups List */}
      {backups.length === 0 ? (
        <Card>
          <div className="text-center py-12">
            <div className="w-16 h-16 mx-auto mb-4 bg-neutral-100 dark:bg-neutral-800 rounded-full flex items-center justify-center">
              <svg className="w-8 h-8 text-neutral-400 dark:text-neutral-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
              </svg>
            </div>
            <h3 className="text-lg font-semibold text-neutral-900 dark:text-white mb-2">
              No Backups
            </h3>
            <p className="text-sm text-neutral-500 dark:text-neutral-400 mb-6">
              Create your first backup to protect your data
            </p>
            <Button variant="primary" onClick={handleCreate}>
              <Plus className="w-4 h-4 mr-2" />
              Create Backup
            </Button>
          </div>
        </Card>
      ) : (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
          {backups.map((backup) => (
            <Card key={backup.id}>
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <div>
                    <h3 className="text-lg font-semibold text-neutral-900 dark:text-white">
                      {backup.name}
                    </h3>
                    <p className="text-sm text-neutral-500 dark:text-neutral-400 mt-1">
                      {formatDate(backup.date)}
                    </p>
                  </div>
                  <div className="w-10 h-10 bg-green-100 dark:bg-green-900/20 rounded-lg flex items-center justify-center">
                    <CheckCircle className="w-5 h-5 text-green-600 dark:text-green-400" />
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-4 text-sm">
                  <div>
                    <span className="text-neutral-500 dark:text-neutral-400">Size:</span>
                    <p className="text-neutral-900 dark:text-white font-medium mt-1">
                      {formatBytes(backup.size)}
                    </p>
                  </div>
                  <div>
                    <span className="text-neutral-500 dark:text-neutral-400">Collections:</span>
                    <p className="text-neutral-900 dark:text-white font-medium mt-1">
                      {backup.collections.length}
                    </p>
                  </div>
                </div>

                {backup.collections.length > 0 && (
                  <div>
                    <span className="text-sm text-neutral-500 dark:text-neutral-400">Collections:</span>
                    <div className="mt-2 flex flex-wrap gap-2">
                      {backup.collections.map((col, idx) => (
                        <span
                          key={idx}
                          className="px-2 py-1 text-xs font-medium bg-neutral-100 dark:bg-neutral-800 text-neutral-700 dark:text-neutral-300 rounded"
                        >
                          {col}
                        </span>
                      ))}
                    </div>
                  </div>
                )}

                <div className="flex items-center gap-2 pt-2 border-t border-neutral-200 dark:border-neutral-800">
                  <Button
                    variant="primary"
                    size="sm"
                    onClick={() => handleRestore(backup)}
                  >
                    Restore
                  </Button>
                  <div className="flex-1 text-xs text-neutral-500 dark:text-neutral-400">
                    ID: {backup.id.substring(0, 8)}...
                  </div>
                </div>
              </div>
            </Card>
          ))}
        </div>
      )}

      {/* Create Backup Modal */}
      <Modal
        isOpen={showCreateModal}
        onClose={() => setShowCreateModal(false)}
        title="Create Backup"
        size="md"
        footer={
          <>
            <Button variant="secondary" onClick={() => setShowCreateModal(false)}>
              Cancel
            </Button>
            <Button
              variant="primary"
              onClick={handleCreateBackup}
              disabled={creating || !createForm.name.trim() || createForm.collections.length === 0}
              isLoading={creating}
            >
              Create Backup
            </Button>
          </>
        }
      >
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-2">
              Backup Name
            </label>
            <input
              type="text"
              value={createForm.name}
              onChange={(e) => setCreateForm({ ...createForm, name: e.target.value })}
              placeholder="backup-2024-01-01"
              className="w-full px-3 py-2 border border-neutral-300 dark:border-neutral-800 rounded-lg bg-white dark:bg-neutral-900 text-neutral-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-primary-500"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-2">
              Collections to Backup
            </label>
            <div className="space-y-2 max-h-60 overflow-y-auto border border-neutral-200 dark:border-neutral-800 rounded-lg p-3">
              {collections.length === 0 ? (
                <p className="text-sm text-neutral-500 dark:text-neutral-400">
                  No collections available
                </p>
              ) : (
                collections.map((col) => (
                  <label key={col.name} className="flex items-center gap-2 cursor-pointer">
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
                            collections: createForm.collections.filter((c) => c !== col.name),
                          });
                        }
                      }}
                      className="w-4 h-4 text-primary-600 rounded focus:ring-primary-500"
                    />
                    <span className="text-sm text-neutral-700 dark:text-neutral-300">
                      {col.name} ({formatNumber(col.vector_count || 0)} vectors)
                    </span>
                  </label>
                ))
              )}
            </div>
          </div>
        </div>
      </Modal>

      {/* Restore Backup Modal */}
      <Modal
        isOpen={showRestoreModal}
        onClose={() => {
          setShowRestoreModal(false);
          setSelectedBackup(null);
        }}
        title="Restore Backup"
        size="md"
        footer={
          <>
            <Button
              variant="secondary"
              onClick={() => {
                setShowRestoreModal(false);
                setSelectedBackup(null);
              }}
            >
              Cancel
            </Button>
            <Button
              variant="primary"
              onClick={handleRestoreBackup}
              disabled={restoring || !restoreForm.collection}
              isLoading={restoring}
            >
              Restore Backup
            </Button>
          </>
        }
      >
        {selectedBackup && (
          <div className="space-y-4">
            <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
              <div className="flex items-start gap-3">
                <AlertCircle className="w-5 h-5 text-yellow-600 dark:text-yellow-400 mt-0.5 flex-shrink-0" />
                <div>
                  <h3 className="text-sm font-semibold text-yellow-800 dark:text-yellow-300 mb-1">
                    Warning
                  </h3>
                  <p className="text-sm text-yellow-700 dark:text-yellow-400">
                    Restoring this backup will overwrite all data in the selected collection. This action cannot be undone.
                  </p>
                </div>
              </div>
            </div>
            <div>
              <p className="text-sm text-neutral-600 dark:text-neutral-400 mb-2">
                Backup: <strong className="text-neutral-900 dark:text-white">{selectedBackup.name}</strong>
              </p>
              <p className="text-sm text-neutral-600 dark:text-neutral-400 mb-4">
                Collections in backup: {selectedBackup.collections.join(', ')}
              </p>
              <Select
                label="Target Collection"
                value={restoreForm.collection}
                onChange={(value) => setRestoreForm({ collection: value })}
                placeholder="Select collection to restore to..."
              >
                <Select.Option id="" value="">
                  Select collection...
                </Select.Option>
                {selectedBackup.collections.map((col) => (
                  <Select.Option key={col} id={col} value={col}>
                    {col}
                  </Select.Option>
                ))}
              </Select>
            </div>
          </div>
        )}
      </Modal>
    </div>
  );
}

export default BackupsPage;
