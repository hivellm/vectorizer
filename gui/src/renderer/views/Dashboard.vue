<template>
  <div class="p-8">
    <!-- Stats Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
      <StatCard
        icon="fas fa-layer-group"
        :value="collections.length"
        label="Collections"
        variant="primary"
      />
      
      <StatCard
        icon="fas fa-vector-square"
        :value="formatNumber(totalVectors)"
        label="Total Vectors"
        variant="success"
      />
      
      <StatCard
        icon="fas fa-cube"
        :value="avgDimension"
        label="Avg Dimension"
        variant="info"
      />
      
      <StatCard
        icon="fas fa-link"
        :value="connections.length"
        label="Connections"
        variant="secondary"
      />
    </div>

    <!-- Quick Actions -->
    <div class="bg-bg-secondary border border-border rounded-xl p-6">
      <h2 class="text-xl font-semibold text-text-primary mb-6">Quick Actions</h2>
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <button @click="$router.push('/workspace')" class="bg-bg-tertiary border border-border rounded-lg p-6 text-left hover:bg-bg-hover hover:border-border-light transition-colors group">
          <i class="fas fa-folder-plus text-2xl text-text-secondary group-hover:text-text-primary mb-3 block"></i>
          <h3 class="text-lg font-medium text-text-primary mb-2">Add Directory</h3>
          <p class="text-sm text-text-secondary">Index a new directory</p>
        </button>

        <button @click="createCollection" class="bg-bg-tertiary border border-border rounded-lg p-6 text-left hover:bg-bg-hover hover:border-border-light transition-colors group">
          <i class="fas fa-database text-2xl text-text-secondary group-hover:text-text-primary mb-3 block"></i>
          <h3 class="text-lg font-medium text-text-primary mb-2">New Collection</h3>
          <p class="text-sm text-text-secondary">Create a collection</p>
        </button>

        <button @click="$router.push('/backups')" class="bg-bg-tertiary border border-border rounded-lg p-6 text-left hover:bg-bg-hover hover:border-border-light transition-colors group">
          <i class="fas fa-save text-2xl text-text-secondary group-hover:text-text-primary mb-3 block"></i>
          <h3 class="text-lg font-medium text-text-primary mb-2">Create Backup</h3>
          <p class="text-sm text-text-secondary">Backup your data</p>
        </button>

        <button @click="refreshData" class="bg-bg-tertiary border border-border rounded-lg p-6 text-left hover:bg-bg-hover hover:border-border-light transition-colors group">
          <i class="fas fa-sync text-2xl text-text-secondary group-hover:text-text-primary mb-3 block"></i>
          <h3 class="text-lg font-medium text-text-primary mb-2">Refresh Data</h3>
          <p class="text-sm text-text-secondary">Reload all data</p>
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
import StatCard from '../components/StatCard.vue';

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
const wasOffline = ref(false);
const showReconnectToast = ref(false);

async function checkVectorizerStatus(): Promise<void> {
  try {
    if (!window.electron) {
      // Running in web mode - assume offline
      vectorizerOnline.value = false;
      vectorizerStatus.value = null;
      wasOffline.value = true;
      return;
    }
    
    const status = await window.electron.vectorizer.getStatus();
    const isNowOnline = status.online;
    
    // Detect if Vectorizer came back online
    if (isNowOnline && wasOffline.value) {
      console.log('Vectorizer came back online, refreshing data...');
      await refreshData();
      wasOffline.value = false;
    } else if (!isNowOnline) {
      wasOffline.value = true;
    }
    
    vectorizerOnline.value = isNowOnline;
    if (isNowOnline) {
      vectorizerStatus.value = {
        version: status.version,
        uptime: status.uptime
      };
    } else {
      vectorizerStatus.value = null;
    }
  } catch (error) {
    console.error('Failed to get Vectorizer status:', error);
    vectorizerOnline.value = false;
    vectorizerStatus.value = null;
    wasOffline.value = true;
  }
}

async function startVectorizer(): Promise<void> {
  starting.value = true;
  try {
    if (!window.electron) {
      alert('This feature requires Electron. Please run the app in Electron mode.');
      return;
    }
    
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
  await connectionsStore.loadConnections();
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
  refreshData();
  
  // Check status every 5 seconds for faster detection
  statusInterval.value = setInterval(checkVectorizerStatus, 5000);
});

onUnmounted(() => {
  if (statusInterval.value) {
    clearInterval(statusInterval.value);
  }
});
</script>

