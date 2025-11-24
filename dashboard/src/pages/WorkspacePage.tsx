/**
 * Workspace page - Manage workspace configuration
 */

import { useEffect, useState } from 'react';
import { useWorkspace, WorkspaceConfig } from '@/hooks/useWorkspace';
import { useCollections } from '@/hooks/useCollections';
import Card from '@/components/ui/Card';
import Button from '@/components/ui/Button';
import Modal from '@/components/ui/Modal';
import { Input } from '@/components/ui/Input';
import { Select } from '@/components/ui/Select';
import CodeEditor from '@/components/ui/CodeEditor';
import { useToastContext } from '@/providers/ToastProvider';
import LoadingState from '@/components/LoadingState';
import { Plus, Trash01, RefreshCw01, Settings01 } from '@untitledui/icons';

function WorkspacePage() {
  const { getConfig, updateConfig, addWorkspace, removeWorkspace } = useWorkspace();
  const { listCollections } = useCollections();
  const toast = useToastContext();

  const [config, setConfig] = useState<WorkspaceConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showAddModal, setShowAddModal] = useState(false);
  const [showConfigModal, setShowConfigModal] = useState(false);
  const [formData, setFormData] = useState({
    path: '',
    collectionName: '',
  });
  const [collections, setCollections] = useState<any[]>([]);
  const [configJson, setConfigJson] = useState('');

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    setLoading(true);
    setError(null);
    try {
      const [configData, collectionsData] = await Promise.all([
        getConfig(),
        listCollections(),
      ]);
      setConfig(configData);
      setCollections(Array.isArray(collectionsData) ? collectionsData : []);
    } catch (err) {
      console.error('Error loading workspace data:', err);
      setError(err instanceof Error ? err.message : 'Failed to load workspace data');
    } finally {
      setLoading(false);
    }
  };

  const handleAdd = () => {
    setFormData({ path: '', collectionName: '' });
    setShowAddModal(true);
  };

  const handleSaveAdd = async () => {
    if (!formData.path || !formData.collectionName) {
      toast.error('Please fill in all fields');
      return;
    }

    try {
      await addWorkspace(formData.path, formData.collectionName);
      toast.success('Workspace added successfully');
      setShowAddModal(false);
      await loadData();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to add workspace');
    }
  };

  const handleRemove = async (path: string) => {
    if (!window.confirm(`Are you sure you want to remove workspace "${path}"?`)) {
      return;
    }

    try {
      await removeWorkspace(path);
      toast.success('Workspace removed successfully');
      await loadData();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to remove workspace');
    }
  };

  const handleOpenConfig = () => {
    if (config) {
      setConfigJson(JSON.stringify(config, null, 2));
      setShowConfigModal(true);
    }
  };

  const handleSaveConfig = async () => {
    try {
      const parsedConfig = JSON.parse(configJson);
      await updateConfig(parsedConfig);
      toast.success('Configuration updated successfully');
      setShowConfigModal(false);
      await loadData();
    } catch (err) {
      if (err instanceof SyntaxError) {
        toast.error('Invalid JSON format');
      } else {
        toast.error(err instanceof Error ? err.message : 'Failed to update configuration');
      }
    }
  };

  if (loading) {
    return <LoadingState message="Loading workspace..." />;
  }

  const projects = config?.projects || [];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-xl sm:text-2xl font-bold text-neutral-900 dark:text-white">Workspace</h1>
          <p className="text-sm sm:text-base text-neutral-600 dark:text-neutral-400 mt-1">
            Manage workspace projects and configuration
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="secondary" size="sm" onClick={loadData}>
            <RefreshCw01 className="w-4 h-4 mr-2" />
            Refresh
          </Button>
          <Button variant="secondary" size="sm" onClick={handleOpenConfig}>
            <Settings01 className="w-4 h-4 mr-2" />
            Edit Config
          </Button>
          <Button variant="primary" size="sm" onClick={handleAdd}>
            <Plus className="w-4 h-4 mr-2" />
            Add Project
          </Button>
        </div>
      </div>

      {/* Error Message */}
      {error && (
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
          <p className="text-sm text-red-800 dark:text-red-300">{error}</p>
        </div>
      )}

      {/* File Watcher Settings */}
      {config?.global_settings?.file_watcher && (
        <Card>
          <h2 className="text-lg font-semibold text-neutral-900 dark:text-white mb-4">
            File Watcher Settings
          </h2>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
            <div>
              <span className="text-sm text-neutral-500 dark:text-neutral-400">Auto Discovery</span>
              <p className="text-sm font-medium text-neutral-900 dark:text-white mt-1">
                {config.global_settings.file_watcher.auto_discovery ? 'Enabled' : 'Disabled'}
              </p>
            </div>
            <div>
              <span className="text-sm text-neutral-500 dark:text-neutral-400">Auto Update</span>
              <p className="text-sm font-medium text-neutral-900 dark:text-white mt-1">
                {config.global_settings.file_watcher.enable_auto_update ? 'Enabled' : 'Disabled'}
              </p>
            </div>
            <div>
              <span className="text-sm text-neutral-500 dark:text-neutral-400">Hot Reload</span>
              <p className="text-sm font-medium text-neutral-900 dark:text-white mt-1">
                {config.global_settings.file_watcher.hot_reload ? 'Enabled' : 'Disabled'}
              </p>
            </div>
            <div>
              <span className="text-sm text-neutral-500 dark:text-neutral-400">Watch Paths</span>
              <p className="text-sm font-medium text-neutral-900 dark:text-white mt-1">
                {config.global_settings.file_watcher.watch_paths?.length || 0}
              </p>
            </div>
          </div>
        </Card>
      )}

      {/* Projects List */}
      {projects.length === 0 ? (
        <Card>
          <div className="text-center py-12">
            <div className="w-16 h-16 mx-auto mb-4 bg-neutral-100 dark:bg-neutral-800 rounded-full flex items-center justify-center">
              <svg className="w-8 h-8 text-neutral-400 dark:text-neutral-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
              </svg>
            </div>
            <h3 className="text-lg font-semibold text-neutral-900 dark:text-white mb-2">
              No Projects
            </h3>
            <p className="text-sm text-neutral-500 dark:text-neutral-400 mb-6">
              Add your first workspace project to get started
            </p>
            <Button variant="primary" onClick={handleAdd}>
              <Plus className="w-4 h-4 mr-2" />
              Add Project
            </Button>
          </div>
        </Card>
      ) : (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
          {projects.map((project, index) => (
            <Card key={index}>
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <div className="w-10 h-10 bg-primary-100 dark:bg-primary-900/20 rounded-lg flex items-center justify-center">
                      <svg className="w-5 h-5 text-primary-600 dark:text-primary-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                      </svg>
                    </div>
                    <div>
                      <h3 className="text-lg font-semibold text-neutral-900 dark:text-white">
                        {project.name || 'Unnamed Project'}
                      </h3>
                      <p className="text-sm text-neutral-500 dark:text-neutral-400 font-mono">
                        {project.path}
                      </p>
                    </div>
                  </div>
                  <Button
                    variant="danger"
                    size="sm"
                    onClick={() => handleRemove(project.path)}
                  >
                    <Trash01 className="w-4 h-4" />
                  </Button>
                </div>

                {project.collections && project.collections.length > 0 && (
                  <div>
                    <span className="text-sm text-neutral-500 dark:text-neutral-400">Collections:</span>
                    <ul className="mt-2 space-y-1">
                      {project.collections.map((collection, idx) => (
                        <li key={idx} className="text-sm text-neutral-700 dark:text-neutral-300">
                          • {collection.name}
                          {collection.include_patterns && collection.include_patterns.length > 0 && (
                            <span className="text-xs text-neutral-500 dark:text-neutral-400 ml-2">
                              ({collection.include_patterns.length} patterns)
                            </span>
                          )}
                        </li>
                      ))}
                    </ul>
                  </div>
                )}
              </div>
            </Card>
          ))}
        </div>
      )}

      {/* Add Project Modal */}
      <Modal
        isOpen={showAddModal}
        onClose={() => setShowAddModal(false)}
        title="Add Workspace Project"
        size="md"
        footer={
          <>
            <Button variant="secondary" onClick={() => setShowAddModal(false)}>
              Cancel
            </Button>
            <Button variant="primary" onClick={handleSaveAdd}>
              Add Project
            </Button>
          </>
        }
      >
        <div className="space-y-4">
          <Input
            label="Project Path"
            value={formData.path}
            onChange={(e) => setFormData({ ...formData, path: e.target.value })}
            placeholder="/path/to/project"
            required
          />
          <div>
            <Select
              label="Collection Name"
              value={formData.collectionName}
              onChange={(value) => setFormData({ ...formData, collectionName: value })}
              placeholder="Select a collection..."
            >
              <Select.Option id="" value="">
                Select a collection...
              </Select.Option>
              {collections.map((col) => (
                <Select.Option key={col.name} id={col.name} value={col.name}>
                  {col.name}
                </Select.Option>
              ))}
            </Select>
          </div>
        </div>
      </Modal>

      {/* Config Editor Modal */}
      <Modal
        isOpen={showConfigModal}
        onClose={() => setShowConfigModal(false)}
        title="Edit Workspace Configuration"
        size="xl"
        footer={
          <>
            <Button variant="secondary" onClick={() => setShowConfigModal(false)}>
              Cancel
            </Button>
            <Button variant="primary" onClick={handleSaveConfig}>
              Save Configuration
            </Button>
          </>
        }
      >
        <div className="space-y-4">
          <p className="text-sm text-neutral-500 dark:text-neutral-400">
            Edit the workspace configuration in JSON format. Changes will be saved to vectorize-workspace.yml
          </p>
          <CodeEditor
            value={configJson}
            onChange={(value) => setConfigJson(value || '')}
            language="json"
            height="500px"
            readOnly={false}
          />
        </div>
      </Modal>
    </div>
  );
}

export default WorkspacePage;
