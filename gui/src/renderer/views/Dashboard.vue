<template>
  <div class="dashboard">
    <h1 class="page-title">
      <i class="fas fa-tachometer-alt"></i>
      Dashboard
    </h1>

    <!-- Vectorizer Status Card -->
    <div class="status-card">
      <div class="card-header">
        <h2>Vectorizer Status</h2>
        <div class="status-indicator">
          <span :class="['status-dot', { online: vectorizerOnline }]"></span>
          <span>{{ vectorizerOnline ? 'Online' : 'Offline' }}</span>
        </div>
      </div>
      
      <div class="card-body">
        <div v-if="vectorizerOnline" class="status-info">
          <div class="info-item">
            <span class="label">Version:</span>
            <span class="value">{{ vectorizerStatus?.version || 'Unknown' }}</span>
          </div>
          <div class="info-item">
            <span class="label">Uptime:</span>
            <span class="value">{{ formatUptime(vectorizerStatus?.uptime || 0) }}</span>
          </div>
        </div>
        
        <div v-else class="offline-actions">
          <p>Vectorizer is not running</p>
          <button @click="startVectorizer" :disabled="starting" class="btn btn-primary">
            <i :class="['fas', starting ? 'fa-spinner fa-spin' : 'fa-play']"></i>
            {{ starting ? 'Starting...' : 'Start Vectorizer' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Stats Grid -->
    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-icon">
          <i class="fas fa-layer-group"></i>
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ collections.length }}</div>
          <div class="stat-label">Collections</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon">
          <i class="fas fa-vector-square"></i>
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ formatNumber(totalVectors) }}</div>
          <div class="stat-label">Total Vectors</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon">
          <i class="fas fa-cube"></i>
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ avgDimension }}</div>
          <div class="stat-label">Avg Dimension</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon">
          <i class="fas fa-link"></i>
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ connections.length }}</div>
          <div class="stat-label">Connections</div>
        </div>
      </div>
    </div>

    <!-- Quick Actions -->
    <div class="quick-actions">
      <h2>Quick Actions</h2>
      <div class="actions-grid">
        <button @click="$router.push('/workspace')" class="action-card">
          <i class="fas fa-folder-plus"></i>
          <h3>Add Directory</h3>
          <p>Index a new directory</p>
        </button>

        <button @click="createCollection" class="action-card">
          <i class="fas fa-database"></i>
          <h3>New Collection</h3>
          <p>Create a collection</p>
        </button>

        <button @click="$router.push('/backups')" class="action-card">
          <i class="fas fa-save"></i>
          <h3>Create Backup</h3>
          <p>Backup your data</p>
        </button>

        <button @click="refreshData" class="action-card">
          <i class="fas fa-sync"></i>
          <h3>Refresh Data</h3>
          <p>Reload all data</p>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { storeToRefs } from 'pinia';
import { useConnectionsStore } from '../stores/connections';
import { useVectorizerStore } from '../stores/vectorizer';

const router = useRouter();
const connectionsStore = useConnectionsStore();
const vectorizerStore = useVectorizerStore();

const { connections } = storeToRefs(connectionsStore);
const { collections, loading, isConnected, totalVectors, avgDimension } = storeToRefs(vectorizerStore);

const vectorizerOnline = ref(false);
const vectorizerStatus = ref<{ version?: string; uptime?: number } | null>(null);
const starting = ref(false);
const selectedCollection = ref<string | null>(null);
const statusInterval = ref<NodeJS.Timeout | null>(null);

async function checkVectorizerStatus(): Promise<void> {
  try {
    const status = await window.electron.vectorizer.getStatus();
    vectorizerOnline.value = status.online;
    if (status.online) {
      vectorizerStatus.value = {
        version: status.version,
        uptime: status.uptime
      };
    }
  } catch (error) {
    vectorizerOnline.value = false;
    vectorizerStatus.value = null;
  }
}

async function startVectorizer(): Promise<void> {
  starting.value = true;
  try {
    const result = await window.electron.vectorizer.start();
    if (result.success) {
      // Wait for vectorizer to be ready
      await new Promise(resolve => setTimeout(resolve, 2000));
      await checkVectorizerStatus();
      await refreshData();
    } else {
      alert(`Failed to start vectorizer: ${result.message}`);
    }
  } catch (error) {
    console.error('Error starting vectorizer:', error);
    alert('Failed to start vectorizer');
  } finally {
    starting.value = false;
  }
}

async function refreshData(): Promise<void> {
  if (isConnected.value) {
    await vectorizerStore.loadCollections();
  }
  await checkVectorizerStatus();
}

function createCollection(): void {
  router.push('/collections/new');
}

function formatNumber(num: number): string {
  return new Intl.NumberFormat().format(num);
}

function formatUptime(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  } else if (minutes > 0) {
    return `${minutes}m ${secs}s`;
  } else {
    return `${secs}s`;
  }
}

onMounted(() => {
  checkVectorizerStatus();
  
  // Check status every 10 seconds
  statusInterval.value = setInterval(checkVectorizerStatus, 10000);
});

onUnmounted(() => {
  if (statusInterval.value) {
    clearInterval(statusInterval.value);
  }
});
</script>

<style scoped>
.dashboard {
  padding: 2rem;
}

.page-title {
  font-size: 2rem;
  margin-bottom: 2rem;
  color: #1a1a2e;
}

.status-card {
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  margin-bottom: 2rem;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1.5rem;
  border-bottom: 1px solid #e5e7eb;
}

.card-body {
  padding: 1.5rem;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.status-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: #ef4444;
}

.status-dot.online {
  background: #10b981;
}

.status-info {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1rem;
}

.info-item {
  display: flex;
  justify-content: space-between;
}

.offline-actions {
  text-align: center;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 1.5rem;
  margin-bottom: 2rem;
}

.stat-card {
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  padding: 1.5rem;
  display: flex;
  gap: 1rem;
}

.stat-icon {
  width: 48px;
  height: 48px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #eff6ff;
  color: #3b82f6;
  font-size: 1.5rem;
}

.stat-value {
  font-size: 2rem;
  font-weight: 700;
  color: #1a1a2e;
}

.stat-label {
  color: #6b7280;
  font-size: 0.875rem;
}

.quick-actions {
  margin-top: 2rem;
}

.actions-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1rem;
  margin-top: 1rem;
}

.action-card {
  background: white;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  padding: 1.5rem;
  text-align: center;
  cursor: pointer;
  transition: all 0.2s;
}

.action-card:hover {
  border-color: #3b82f6;
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.1);
}

.action-card i {
  font-size: 2rem;
  color: #3b82f6;
  margin-bottom: 0.5rem;
}

.action-card h3 {
  margin: 0.5rem 0;
  font-size: 1rem;
}

.action-card p {
  color: #6b7280;
  font-size: 0.875rem;
  margin: 0;
}

.btn {
  padding: 0.5rem 1rem;
  border-radius: 6px;
  border: none;
  cursor: pointer;
  font-size: 0.875rem;
  transition: all 0.2s;
}

.btn-primary {
  background: #3b82f6;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: #2563eb;
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>

