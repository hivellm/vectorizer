<template>
  <div class="p-8">
    <div class="connection-manager">
    <!-- Add Connection Form -->
    <div v-if="showAddConnectionForm" class="bg-bg-secondary border border-border rounded-xl p-6 mb-6">
      <div class="flex items-center justify-between mb-6">
        <h3 class="text-xl font-semibold text-text-primary">{{ editingConnection ? 'Edit Connection' : 'Add New Connection' }}</h3>
        <button @click="cancelForm" class="p-2 text-text-secondary hover:text-text-primary hover:bg-bg-hover rounded transition-colors">
          <i class="fas fa-times"></i>
        </button>
      </div>
      <div>
        <form @submit.prevent="saveConnection" class="space-y-4">
          <div>
            <label for="conn-name" class="block text-sm font-medium text-text-primary mb-2">Connection Name</label>
            <input
              id="conn-name"
              v-model="connectionForm.name"
              type="text"
              class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors"
              placeholder="My Vectorizer"
              required
            />
          </div>

          <div>
            <label class="block text-sm font-medium text-text-primary mb-2">Connection Type</label>
            <div class="flex gap-4">
              <label class="flex items-center gap-2 cursor-pointer">
                <input v-model="connectionForm.type" type="radio" value="local" class="text-blue-500" />
                <span class="text-sm text-text-primary">Local</span>
              </label>
              <label class="flex items-center gap-2 cursor-pointer">
                <input v-model="connectionForm.type" type="radio" value="remote" class="text-blue-500" />
                <span class="text-sm text-text-primary">Remote</span>
              </label>
            </div>
          </div>

          <div class="grid grid-cols-2 gap-4">
            <div>
              <label for="conn-host" class="block text-sm font-medium text-text-primary mb-2">Host</label>
              <input
                id="conn-host"
                v-model="connectionForm.host"
                type="text"
                class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors"
                placeholder="localhost"
                required
              />
            </div>

            <div>
              <label for="conn-port" class="block text-sm font-medium text-text-primary mb-2">Port</label>
              <input
                id="conn-port"
                v-model.number="connectionForm.port"
                type="number"
                class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors"
                placeholder="3030"
                required
              />
            </div>
          </div>

          <div>
            <label for="conn-token" class="block text-sm font-medium text-text-primary mb-2">API Token (Optional)</label>
            <input
              id="conn-token"
              v-model="connectionForm.token"
              type="password"
              class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors"
              placeholder="Enter API token if required"
            />
          </div>

          <div class="flex items-center justify-between pt-4">
            <button type="button" @click="testConnection" :disabled="testing" class="px-4 py-2 text-sm font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
              <i :class="['fas', testing ? 'fa-spinner fa-spin' : 'fa-plug', 'mr-2']"></i>
              {{ testing ? 'Testing...' : 'Test Connection' }}
            </button>
            <div class="flex gap-2">
              <button type="button" @click="cancelForm" class="px-4 py-2 text-sm font-medium bg-transparent text-text-secondary border border-border rounded hover:bg-bg-hover hover:text-text-primary transition-colors">Cancel</button>
              <button type="submit" :disabled="saving" class="px-4 py-2 text-sm font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
                <i :class="['fas', saving ? 'fa-spinner fa-spin' : 'fa-save', 'mr-2']"></i>
                {{ saving ? 'Saving...' : 'Save' }}
              </button>
            </div>
          </div>
        </form>
      </div>
    </div>

    <!-- Connections List -->
    <div class="space-y-4">
      <div v-if="connections.length === 0 && !showAddConnectionForm" class="flex flex-col items-center justify-center py-16 text-text-secondary">
        <i class="fas fa-network-wired text-4xl mb-4"></i>
        <h3 class="text-lg font-medium text-text-primary mb-2">No Connections</h3>
        <p class="text-sm text-text-secondary">Click "New Connection" in the top bar to add your first connection</p>
      </div>

      <div v-for="connection in connections" :key="connection.id" class="bg-bg-secondary border border-border rounded-xl p-6">
        <div class="flex items-center justify-between mb-4">
          <div class="flex items-center gap-3">
            <h3 class="text-lg font-semibold text-text-primary">{{ connection.name }}</h3>
            <span class="px-2 py-1 text-xs font-medium rounded uppercase" :class="connection.type === 'local' ? 'bg-blue-500/20 text-blue-400' : 'bg-purple-500/20 text-purple-400'">
              {{ connection.type }}
            </span>
          </div>
          <div class="flex items-center gap-2">
            <span :class="['w-2 h-2 rounded-full', connection.status === 'online' ? 'bg-success' : 'bg-text-muted']"></span>
            <span class="text-sm" :class="connection.status === 'online' ? 'text-success' : 'text-text-muted'">{{ connection.status }}</span>
          </div>
        </div>

        <div class="grid grid-cols-2 gap-4 mb-4 text-sm">
          <div>
            <span class="text-text-secondary">Endpoint:</span>
            <span class="text-text-primary ml-2">{{ connection.host }}:{{ connection.port }}</span>
          </div>
          <div>
            <span class="text-text-secondary">Authentication:</span>
            <span class="text-text-primary ml-2">{{ connection.auth?.token ? 'Token configured' : 'None' }}</span>
          </div>
        </div>

        <div class="flex gap-2">
          <button
            @click="setActive(connection.id)"
            :disabled="connection.active"
            class="px-3 py-1.5 text-xs font-medium rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            :class="connection.active ? 'bg-success/20 text-success' : 'bg-bg-tertiary text-text-primary border border-border hover:bg-bg-hover'"
          >
            <i class="fas fa-check mr-1"></i>
            {{ connection.active ? 'Active' : 'Set Active' }}
          </button>
          <button @click="editConnection(connection)" class="px-3 py-1.5 text-xs font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors">
            <i class="fas fa-edit mr-1"></i> Edit
          </button>
          <button @click="removeConnection(connection.id)" class="px-3 py-1.5 text-xs font-medium bg-error/20 text-error border border-error rounded hover:bg-error/30 transition-colors">
            <i class="fas fa-trash mr-1"></i> Delete
          </button>
        </div>
      </div>
    </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted } from 'vue';
import { storeToRefs } from 'pinia';
import { useConnectionsStore } from '../stores/connections';
import { useVectorizerStore } from '../stores/vectorizer';
import { useDialog } from '../composables/useDialog';
import type { Connection } from '@shared/types';

const connectionsStore = useConnectionsStore();
const vectorizerStore = useVectorizerStore();
const dialog = useDialog();

const { connections } = storeToRefs(connectionsStore);

const showAddConnectionForm = ref(false);
const editingConnection = ref<Connection | null>(null);
const testing = ref(false);
const saving = ref(false);

// Listen for add-connection event from App.vue
function handleAddConnection(): void {
  showAddConnectionForm.value = true;
}

onMounted(() => {
  window.addEventListener('add-connection', handleAddConnection);
});

onUnmounted(() => {
  window.removeEventListener('add-connection', handleAddConnection);
});

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
      await dialog.alert('Connection successful! The server is reachable.', 'Connection Test');
    } else {
      await dialog.alert('Connection failed: Server returned an error.', 'Connection Test Failed');
    }
  } catch (error) {
    await dialog.alert(
      `Connection failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
      'Connection Test Failed'
    );
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
  const conn = connections.value.find((c: Connection) => c.id === id);
  if (conn) {
    await vectorizerStore.initializeClient(conn.host, conn.port, conn.auth?.token);
  }
}

async function removeConnection(id: string): Promise<void> {
  const confirmed = await dialog.confirm(
    'Are you sure you want to delete this connection? This action cannot be undone.',
    'Delete Connection'
  );
  
  if (confirmed) {
    connectionsStore.removeConnection(id);
  }
}
</script>
