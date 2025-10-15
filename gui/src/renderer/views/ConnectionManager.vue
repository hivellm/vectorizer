<template>
  <div class="connection-manager">
    <div class="page-header">
      <h1><i class="fas fa-network-wired"></i> Connection Manager</h1>
      <button @click="showAddConnectionForm = true" class="btn btn-primary">
        <i class="fas fa-plus"></i> New Connection
      </button>
    </div>

    <!-- Add Connection Form -->
    <div v-if="showAddConnectionForm" class="card form-card">
      <div class="card-header">
        <h3>{{ editingConnection ? 'Edit Connection' : 'Add New Connection' }}</h3>
        <button @click="cancelForm" class="btn-icon">
          <i class="fas fa-times"></i>
        </button>
      </div>
      <div class="card-body">
        <form @submit.prevent="saveConnection">
          <div class="form-group">
            <label for="conn-name">Connection Name</label>
            <input
              id="conn-name"
              v-model="connectionForm.name"
              type="text"
              class="form-control"
              placeholder="My Vectorizer"
              required
            />
          </div>

          <div class="form-group">
            <label>Connection Type</label>
            <div class="radio-group">
              <label class="radio-label">
                <input v-model="connectionForm.type" type="radio" value="local" />
                <span>Local</span>
              </label>
              <label class="radio-label">
                <input v-model="connectionForm.type" type="radio" value="remote" />
                <span>Remote</span>
              </label>
            </div>
          </div>

          <div class="form-row">
            <div class="form-group">
              <label for="conn-host">Host</label>
              <input
                id="conn-host"
                v-model="connectionForm.host"
                type="text"
                class="form-control"
                placeholder="localhost"
                required
              />
            </div>

            <div class="form-group">
              <label for="conn-port">Port</label>
              <input
                id="conn-port"
                v-model.number="connectionForm.port"
                type="number"
                class="form-control"
                placeholder="3030"
                required
              />
            </div>
          </div>

          <div class="form-group">
            <label for="conn-token">API Token (Optional)</label>
            <input
              id="conn-token"
              v-model="connectionForm.token"
              type="password"
              class="form-control"
              placeholder="Enter API token if required"
            />
          </div>

          <div class="form-actions">
            <button type="button" @click="testConnection" :disabled="testing" class="btn btn-secondary">
              <i :class="['fas', testing ? 'fa-spinner fa-spin' : 'fa-plug']"></i>
              {{ testing ? 'Testing...' : 'Test Connection' }}
            </button>
            <div class="form-actions-right">
              <button type="button" @click="cancelForm" class="btn btn-secondary">Cancel</button>
              <button type="submit" :disabled="saving" class="btn btn-primary">
                <i :class="['fas', saving ? 'fa-spinner fa-spin' : 'fa-save']"></i>
                {{ saving ? 'Saving...' : 'Save' }}
              </button>
            </div>
          </div>
        </form>
      </div>
    </div>

    <!-- Connections List -->
    <div class="connections-list">
      <div v-if="connections.length === 0" class="empty-state">
        <i class="fas fa-network-wired"></i>
        <h3>No Connections</h3>
        <p>Add your first connection to get started</p>
      </div>

      <div v-for="connection in connections" :key="connection.id" class="connection-card">
        <div class="connection-header">
          <div class="connection-info">
            <h3>{{ connection.name }}</h3>
            <span :class="['connection-type-badge', connection.type]">
              {{ connection.type }}
            </span>
          </div>
          <div :class="['connection-status', connection.status]">
            <span class="status-dot"></span>
            <span>{{ connection.status }}</span>
          </div>
        </div>

        <div class="connection-details">
          <div class="detail-item">
            <span class="detail-label">Endpoint:</span>
            <span class="detail-value">{{ connection.host }}:{{ connection.port }}</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">Authentication:</span>
            <span class="detail-value">{{ connection.auth?.token ? 'Token configured' : 'None' }}</span>
          </div>
        </div>

        <div class="connection-actions">
          <button
            @click="setActive(connection.id)"
            :disabled="connection.active"
            class="btn btn-sm btn-primary"
          >
            <i class="fas fa-check"></i>
            {{ connection.active ? 'Active' : 'Set Active' }}
          </button>
          <button @click="editConnection(connection)" class="btn btn-sm btn-secondary">
            <i class="fas fa-edit"></i> Edit
          </button>
          <button @click="removeConnection(connection.id)" class="btn btn-sm btn-danger">
            <i class="fas fa-trash"></i> Delete
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue';
import { storeToRefs } from 'pinia';
import { useConnectionsStore } from '../stores/connections';
import { useVectorizerStore } from '../stores/vectorizer';
import type { Connection } from '@shared/types';

const connectionsStore = useConnectionsStore();
const vectorizerStore = useVectorizerStore();

const { connections } = storeToRefs(connectionsStore);

const showAddConnectionForm = ref(false);
const editingConnection = ref<Connection | null>(null);
const testing = ref(false);
const saving = ref(false);

const connectionForm = reactive({
  name: '',
  type: 'local' as 'local' | 'remote',
  host: 'localhost',
  port: 15002,
  token: ''
});

function cancelForm(): void {
  showAddConnectionForm.value = false;
  editingConnection.value = null;
  resetForm();
}

function resetForm(): void {
  connectionForm.name = '';
  connectionForm.type = 'local';
  connectionForm.host = 'localhost';
  connectionForm.port = 3030;
  connectionForm.token = '';
}

function editConnection(connection: Connection): void {
  editingConnection.value = connection;
  connectionForm.name = connection.name;
  connectionForm.type = connection.type;
  connectionForm.host = connection.host;
  connectionForm.port = connection.port;
  connectionForm.token = connection.auth?.token || '';
  showAddConnectionForm.value = true;
}

async function testConnection(): Promise<void> {
  testing.value = true;
  try {
    const response = await fetch(`http://${connectionForm.host}:${connectionForm.port}/api/status`, {
      method: 'GET',
      headers: connectionForm.token ? {
        'Authorization': `Bearer ${connectionForm.token}`
      } : {},
      signal: AbortSignal.timeout(5000)
    });

    if (response.ok) {
      alert('Connection successful!');
    } else {
      alert('Connection failed: Server returned error');
    }
  } catch (error) {
    alert(`Connection failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
  } finally {
    testing.value = false;
  }
}

function saveConnection(): void {
  saving.value = true;
  try {
    if (editingConnection.value) {
      connectionsStore.updateConnection(editingConnection.value.id, {
        name: connectionForm.name,
        type: connectionForm.type,
        host: connectionForm.host,
        port: connectionForm.port,
        auth: connectionForm.token ? { token: connectionForm.token } : undefined
      });
    } else {
      connectionsStore.addConnection({
        name: connectionForm.name,
        type: connectionForm.type,
        host: connectionForm.host,
        port: connectionForm.port,
        auth: connectionForm.token ? { token: connectionForm.token } : undefined,
        active: connections.value.length === 0 // Set as active if first connection
      });
    }

    cancelForm();
  } finally {
    saving.value = false;
  }
}

async function setActive(id: string): Promise<void> {
  await connectionsStore.setActiveConnection(id);
  
  // Reinitialize vectorizer client
  const conn = connections.value.find(c => c.id === id);
  if (conn) {
    await vectorizerStore.initializeClient(conn.host, conn.port, conn.auth?.token);
  }
}

function removeConnection(id: string): void {
  if (confirm('Are you sure you want to delete this connection?')) {
    connectionsStore.removeConnection(id);
  }
}
</script>

<style scoped>
.connection-manager {
  padding: 2rem;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 2rem;
}

.card {
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  margin-bottom: 1.5rem;
}

.form-card {
  max-width: 600px;
}

.form-group {
  margin-bottom: 1rem;
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
}

.form-control {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid #d1d5db;
  border-radius: 4px;
  font-size: 0.875rem;
}

.radio-group {
  display: flex;
  gap: 1rem;
}

.radio-label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
}

.form-actions {
  display: flex;
  justify-content: space-between;
  margin-top: 1.5rem;
}

.form-actions-right {
  display: flex;
  gap: 0.5rem;
}

.connections-list {
  display: grid;
  gap: 1rem;
}

.connection-card {
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  padding: 1.5rem;
}

.connection-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}

.connection-info {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.connection-type-badge {
  padding: 0.25rem 0.75rem;
  border-radius: 12px;
  font-size: 0.75rem;
  font-weight: 600;
}

.connection-type-badge.local {
  background: #dbeafe;
  color: #1e40af;
}

.connection-type-badge.remote {
  background: #fef3c7;
  color: #92400e;
}

.connection-status {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.875rem;
}

.connection-status.online {
  color: #10b981;
}

.connection-status.offline {
  color: #ef4444;
}

.connection-details {
  display: grid;
  gap: 0.5rem;
  margin-bottom: 1rem;
  padding-bottom: 1rem;
  border-bottom: 1px solid #e5e7eb;
}

.detail-item {
  display: flex;
  gap: 0.5rem;
}

.detail-label {
  font-weight: 600;
  color: #6b7280;
}

.connection-actions {
  display: flex;
  gap: 0.5rem;
}

.btn-sm {
  padding: 0.375rem 0.75rem;
  font-size: 0.8125rem;
}

.btn-secondary {
  background: #6b7280;
  color: white;
}

.btn-secondary:hover {
  background: #4b5563;
}

.btn-danger {
  background: #ef4444;
  color: white;
}

.btn-danger:hover {
  background: #dc2626;
}

.empty-state {
  text-align: center;
  padding: 3rem;
  color: #6b7280;
}

.empty-state i {
  font-size: 3rem;
  margin-bottom: 1rem;
  color: #d1d5db;
}
</style>

