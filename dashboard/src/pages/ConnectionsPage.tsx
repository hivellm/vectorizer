/**
 * Connections page - Manage database connections
 */

import { useState, useEffect } from 'react';
import { useConnections, Connection } from '@/hooks/useConnections';
import Card from '@/components/ui/Card';
import Button from '@/components/ui/Button';
import Modal from '@/components/ui/Modal';
import { Input } from '@/components/ui/Input';
import { useToastContext } from '@/providers/ToastProvider';
import { CheckCircle, XCircle, RefreshCw01, Plus, Trash01, Edit01, AlertCircle } from '@untitledui/icons';

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
    setActiveConnection,
  } = useConnections();
  const toast = useToastContext();

  const [showAddModal, setShowAddModal] = useState(false);
  const [showEditModal, setShowEditModal] = useState(false);
  const [editingConnection, setEditingConnection] = useState<Connection | null>(null);
  const [formData, setFormData] = useState({
    name: '',
    host: 'localhost',
    port: 15002,
    type: 'local' as 'local' | 'remote',
    token: '',
  });
  const [checking, setChecking] = useState<string | null>(null);

  useEffect(() => {
    // Check health of all connections on mount
    if (connections.length > 0) {
      checkAllConnectionsHealth();
    }
  }, []);

  const handleAdd = () => {
    setFormData({
      name: '',
      host: 'localhost',
      port: 15002,
      type: 'local',
      token: '',
    });
    setShowAddModal(true);
  };

  const handleEdit = (connection: Connection) => {
    setEditingConnection(connection);
    setFormData({
      name: connection.name,
      host: connection.host,
      port: connection.port,
      type: connection.type,
      token: connection.auth?.token || '',
    });
    setShowEditModal(true);
  };

  const handleSave = async () => {
    if (!formData.name || !formData.host || !formData.port) {
      toast.error('Please fill in all required fields');
      return;
    }

    try {
      if (showAddModal) {
        const id = addConnection({
          name: formData.name,
          host: formData.host,
          port: formData.port,
          type: formData.type,
          auth: formData.token ? { token: formData.token } : undefined,
        });
        toast.success('Connection added successfully');
        // Check health after adding
        setTimeout(() => checkConnectionHealth(id), 500);
      } else if (editingConnection) {
        updateConnection(editingConnection.id, {
          name: formData.name,
          host: formData.host,
          port: formData.port,
          type: formData.type,
          auth: formData.token ? { token: formData.token } : undefined,
        });
        toast.success('Connection updated successfully');
        // Check health after updating
        setTimeout(() => checkConnectionHealth(editingConnection.id), 500);
      }
      setShowAddModal(false);
      setShowEditModal(false);
      setEditingConnection(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to save connection');
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

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'online':
        return <CheckCircle className="w-5 h-5 text-green-600 dark:text-green-400" />;
      case 'offline':
        return <XCircle className="w-5 h-5 text-red-600 dark:text-red-400" />;
      case 'connecting':
        return <RefreshCw01 className="w-5 h-5 text-yellow-600 dark:text-yellow-400 animate-spin" />;
      default:
        return <AlertCircle className="w-5 h-5 text-neutral-400 dark:text-neutral-500" />;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'online':
        return 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400';
      case 'offline':
        return 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400';
      case 'connecting':
        return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400';
      default:
        return 'bg-neutral-100 text-neutral-800 dark:bg-neutral-800 dark:text-neutral-400';
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-12">
        <RefreshCw01 className="w-8 h-8 text-neutral-400 dark:text-neutral-500 animate-spin" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-xl sm:text-2xl font-bold text-neutral-900 dark:text-white">Connections</h1>
          <p className="text-sm sm:text-base text-neutral-600 dark:text-neutral-400 mt-1">
            Manage connections to Vectorizer servers
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="secondary" size="sm" onClick={checkAllConnectionsHealth}>
            <RefreshCw01 className="w-4 h-4 mr-2" />
            Check All
          </Button>
          <Button variant="primary" size="sm" onClick={handleAdd}>
            <Plus className="w-4 h-4 mr-2" />
            New Connection
          </Button>
        </div>
      </div>

      {/* Connections List */}
      {connections.length === 0 ? (
        <Card>
          <div className="text-center py-12">
            <div className="w-16 h-16 mx-auto mb-4 bg-neutral-100 dark:bg-neutral-800 rounded-full flex items-center justify-center">
              <AlertCircle className="w-8 h-8 text-neutral-400 dark:text-neutral-500" />
            </div>
            <h3 className="text-lg font-semibold text-neutral-900 dark:text-white mb-2">
              No Connections
            </h3>
            <p className="text-sm text-neutral-500 dark:text-neutral-400 mb-6">
              Add your first connection to get started
            </p>
            <Button variant="primary" onClick={handleAdd}>
              <Plus className="w-4 h-4 mr-2" />
              Add Connection
            </Button>
          </div>
        </Card>
      ) : (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
          {connections.map((connection) => (
            <Card key={connection.id}>
              <div className="space-y-4">
                {/* Header */}
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <h3 className="text-lg font-semibold text-neutral-900 dark:text-white">
                      {connection.name}
                    </h3>
                    <span
                      className={`px-2 py-1 text-xs font-medium rounded uppercase ${
                        connection.type === 'local'
                          ? 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-400'
                          : 'bg-purple-100 text-purple-800 dark:bg-purple-900/20 dark:text-purple-400'
                      }`}
                    >
                      {connection.type}
                    </span>
                  </div>
                  <div className="flex items-center gap-2">
                    {getStatusIcon(connection.status)}
                    <span
                      className={`text-xs font-medium px-2 py-1 rounded ${getStatusColor(connection.status)}`}
                    >
                      {connection.status}
                    </span>
                  </div>
                </div>

                {/* Details */}
                <div className="grid grid-cols-2 gap-4 text-sm">
                  <div>
                    <span className="text-neutral-500 dark:text-neutral-400">Endpoint:</span>
                    <p className="text-neutral-900 dark:text-white font-mono mt-1">
                      {connection.host}:{connection.port}
                    </p>
                  </div>
                  <div>
                    <span className="text-neutral-500 dark:text-neutral-400">Authentication:</span>
                    <p className="text-neutral-900 dark:text-white mt-1">
                      {connection.auth?.token ? 'Token configured' : 'None'}
                    </p>
                  </div>
                </div>

                {/* Actions */}
                <div className="flex items-center gap-2 pt-2 border-t border-neutral-200 dark:border-neutral-800">
                  <Button
                    variant="secondary"
                    size="sm"
                    onClick={() => handleCheckHealth(connection.id)}
                    disabled={checking === connection.id}
                    isLoading={checking === connection.id}
                  >
                    <RefreshCw01 className="w-4 h-4 mr-2" />
                    Check Health
                  </Button>
                  {activeConnectionId !== connection.id && (
                    <Button
                      variant="secondary"
                      size="sm"
                      onClick={() => setActiveConnection(connection.id)}
                    >
                      Set Active
                    </Button>
                  )}
                  {activeConnectionId === connection.id && (
                    <span className="text-xs font-medium text-primary-600 dark:text-primary-400">
                      Active
                    </span>
                  )}
                  <Button
                    variant="secondary"
                    size="sm"
                    onClick={() => handleEdit(connection)}
                  >
                    <Edit01 className="w-4 h-4" />
                  </Button>
                  <Button
                    variant="danger"
                    size="sm"
                    onClick={() => handleDelete(connection.id)}
                  >
                    <Trash01 className="w-4 h-4" />
                  </Button>
                </div>
              </div>
            </Card>
          ))}
        </div>
      )}

      {/* Add Connection Modal */}
      <Modal
        isOpen={showAddModal}
        onClose={() => setShowAddModal(false)}
        title="Add Connection"
        size="md"
        footer={
          <>
            <Button variant="secondary" onClick={() => setShowAddModal(false)}>
              Cancel
            </Button>
            <Button variant="primary" onClick={handleSave}>
              Add Connection
            </Button>
          </>
        }
      >
        <div className="space-y-4">
          <Input
            label="Name"
            value={formData.name}
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
            placeholder="My Local Server"
            required
          />
          <div className="grid grid-cols-2 gap-4">
            <Input
              label="Host"
              value={formData.host}
              onChange={(e) => setFormData({ ...formData, host: e.target.value })}
              placeholder="localhost"
              required
            />
            <Input
              label="Port"
              type="number"
              value={formData.port}
              onChange={(e) => setFormData({ ...formData, port: parseInt(e.target.value) || 15002 })}
              placeholder="15002"
              required
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-2">
              Type
            </label>
            <select
              value={formData.type}
              onChange={(e) => setFormData({ ...formData, type: e.target.value as 'local' | 'remote' })}
              className="w-full px-3 py-2 border border-neutral-300 dark:border-neutral-800 rounded-lg bg-white dark:bg-neutral-900 text-neutral-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-primary-500"
            >
              <option value="local">Local</option>
              <option value="remote">Remote</option>
            </select>
          </div>
          <Input
            label="Auth Token (Optional)"
            type="password"
            value={formData.token}
            onChange={(e) => setFormData({ ...formData, token: e.target.value })}
            placeholder="Enter authentication token"
          />
        </div>
      </Modal>

      {/* Edit Connection Modal */}
      <Modal
        isOpen={showEditModal}
        onClose={() => {
          setShowEditModal(false);
          setEditingConnection(null);
        }}
        title="Edit Connection"
        size="md"
        footer={
          <>
            <Button
              variant="secondary"
              onClick={() => {
                setShowEditModal(false);
                setEditingConnection(null);
              }}
            >
              Cancel
            </Button>
            <Button variant="primary" onClick={handleSave}>
              Save Changes
            </Button>
          </>
        }
      >
        <div className="space-y-4">
          <Input
            label="Name"
            value={formData.name}
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
            placeholder="My Local Server"
            required
          />
          <div className="grid grid-cols-2 gap-4">
            <Input
              label="Host"
              value={formData.host}
              onChange={(e) => setFormData({ ...formData, host: e.target.value })}
              placeholder="localhost"
              required
            />
            <Input
              label="Port"
              type="number"
              value={formData.port}
              onChange={(e) => setFormData({ ...formData, port: parseInt(e.target.value) || 15002 })}
              placeholder="15002"
              required
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-2">
              Type
            </label>
            <select
              value={formData.type}
              onChange={(e) => setFormData({ ...formData, type: e.target.value as 'local' | 'remote' })}
              className="w-full px-3 py-2 border border-neutral-300 dark:border-neutral-800 rounded-lg bg-white dark:bg-neutral-900 text-neutral-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-primary-500"
            >
              <option value="local">Local</option>
              <option value="remote">Remote</option>
            </select>
          </div>
          <Input
            label="Auth Token (Optional)"
            type="password"
            value={formData.token}
            onChange={(e) => setFormData({ ...formData, token: e.target.value })}
            placeholder="Enter authentication token"
          />
        </div>
      </Modal>
    </div>
  );
}

export default ConnectionsPage;
