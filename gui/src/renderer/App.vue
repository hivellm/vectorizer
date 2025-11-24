<template>
  <div id="app" class="flex flex-col h-screen bg-bg-primary text-text-primary">
    <!-- Custom Titlebar -->
    <div class="h-8 bg-bg-secondary border-b border-border flex items-center justify-between px-4 drag-region flex-shrink-0">
      <div class="flex items-center gap-2 text-xs text-text-secondary">
        <i class="fas fa-cube"></i>
        <span>Vectorizer GUI</span>
      </div>
      <div class="flex items-center gap-1 no-drag">
        <button @click="minimizeWindow" class="p-1 w-8 h-8 hover:bg-bg-hover transition-colors rounded">
          <i class="fas fa-window-minimize text-xs"></i>
        </button>
        <button @click="maximizeWindow" class="p-1 w-8 h-8 hover:bg-bg-hover transition-colors rounded">
          <i class="fas fa-window-maximize text-xs"></i>
        </button>
        <button @click="closeWindow" class="p-1 w-8 h-8 hover:bg-red-600 hover:text-white transition-colors rounded">
          <i class="fas fa-times text-xs"></i>
        </button>
      </div>
    </div>

    <!-- Main Content -->
    <div class="flex flex-1 min-h-0">
    <!-- Sidebar -->
    <aside class="w-64 bg-bg-secondary border-r border-border flex flex-col">

      <!-- Connection Selector -->
      <div class="h-14 flex items-center px-4 border-b border-border flex-shrink-0">
        <div class="connection-dropdown relative w-full" :class="{ 'z-dropdown': isDropdownOpen }">
          <button 
            @click="connections.length === 0 ? openConnectionManager() : toggleDropdown()" 
            class="w-full flex items-center justify-between gap-3 p-3 rounded-lg bg-bg-tertiary hover:bg-bg-hover transition-colors cursor-pointer"
          >
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 whitespace-nowrap overflow-hidden text-ellipsis">
                <span class="text-sm font-medium text-text-primary">{{ activeConnection?.name || 'Select Connection' }}</span>
                <span class="text-text-muted">•</span>
                <span :class="['w-2 h-2 rounded-full', activeConnection?.status === 'online' ? 'bg-success' : 'bg-text-muted']"></span>
                <span class="text-xs text-text-secondary">{{ activeConnection?.status || 'No connections' }}</span>
              </div>
            </div>
            <i class="fas fa-chevron-down text-xs text-text-muted transition-transform" :class="{ 'rotate-180': isDropdownOpen }"></i>
          </button>
          
          <div v-if="isDropdownOpen" class="absolute top-full left-0 right-0 mt-1 bg-bg-elevated border border-border rounded-lg shadow-lg z-dropdown">
            <div v-if="connections.length === 0" class="p-4 text-center text-text-secondary">
              <i class="fas fa-exclamation-circle mb-2 block"></i>
              <span class="text-sm">No connections available</span>
            </div>
            <div v-else>
              <div 
                v-for="conn in connections" 
                :key="conn.id"
                :class="['flex items-center justify-between p-3 hover:bg-bg-hover cursor-pointer transition-colors', { 'bg-bg-hover': activeConnectionId === conn.id }]"
                @click="selectConnection(conn.id)"
              >
                <div class="flex-1 min-w-0">
                  <div class="text-sm font-medium text-text-primary truncate">{{ conn.name }}</div>
                  <div class="flex items-center gap-2 text-xs text-text-secondary">
                    <span :class="['w-1.5 h-1.5 rounded-full', conn.status === 'online' ? 'bg-success' : 'bg-text-muted']"></span>
                    {{ conn.host }}:{{ conn.port }}
                  </div>
                </div>
                <div v-if="activeConnectionId === conn.id" class="text-success">
                  <i class="fas fa-check text-sm"></i>
                </div>
              </div>
            </div>
            
            <div class="border-t border-border p-3">
              <button @click="openConnectionManager" class="w-full flex items-center gap-2 px-3 py-2 text-sm text-text-secondary hover:text-text-primary hover:bg-bg-hover rounded transition-colors">
                <i class="fas fa-cog"></i>
                Manage Connections
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Collections List -->
      <div class="flex-1 flex flex-col min-h-0">
        <div class="flex items-center justify-between p-4 border-b border-border flex-shrink-0">
          <h3 class="text-sm font-semibold text-text-primary">Collections</h3>
          <div class="flex gap-1">
            <button @click="refreshCollections" class="p-1 text-text-secondary hover:text-text-primary hover:bg-bg-hover rounded transition-colors" title="Refresh Collections">
              <i class="fas fa-sync text-sm" :class="{ 'fa-spin': loading }"></i>
            </button>
            <button @click="createCollection" class="p-1 text-text-secondary hover:text-text-primary hover:bg-bg-hover rounded transition-colors" title="New Collection">
              <i class="fas fa-plus text-sm"></i>
            </button>
          </div>
        </div>
        
        <div v-if="loading" class="flex flex-col items-center justify-center py-12 text-text-secondary flex-1">
          <i class="fas fa-spinner fa-spin text-2xl mb-3"></i>
          <span class="text-sm">Loading...</span>
        </div>
        
        <div v-else-if="collections.length === 0" class="flex flex-col items-center justify-center py-12 text-text-secondary flex-1">
          <p class="text-sm">No collections</p>
        </div>
        
        <div v-else class="flex-1 overflow-y-auto min-h-0">
          <ul>
            <li
              v-for="collection in collections"
              :key="collection.name"
              :class="['flex items-center justify-between p-3 hover:bg-bg-hover cursor-pointer transition-colors', { 'bg-bg-hover': selectedCollection === collection.name }]"
              @click="selectCollection(collection.name)"
            >
              <div class="flex items-center gap-3 flex-1 min-w-0">
                <i class="fas fa-database text-text-muted"></i>
                <span class="text-sm text-text-primary truncate">{{ collection.name }}</span>
              </div>
              <span class="text-xs text-text-secondary">{{ formatNumber(collection.vector_count) }}</span>
            </li>
          </ul>
        </div>
      </div>

      <!-- Navigation -->
      <nav class="p-4 border-t border-border">
        <router-link to="/" class="flex items-center gap-3 py-2 text-text-secondary hover:text-text-primary transition-colors cursor-pointer text-sm">
          <i class="fas fa-home w-4 text-center"></i>
          <span>Dashboard</span>
        </router-link>
        <router-link to="/workspace" class="flex items-center gap-3 py-2 text-text-secondary hover:text-text-primary transition-colors cursor-pointer text-sm">
          <i class="fas fa-folder-open w-4 text-center"></i>
          <span>Workspace</span>
        </router-link>
        <router-link to="/config" class="flex items-center gap-3 py-2 text-text-secondary hover:text-text-primary transition-colors cursor-pointer text-sm">
          <i class="fas fa-cog w-4 text-center"></i>
          <span>Configuration</span>
        </router-link>
        <router-link to="/logs" class="flex items-center gap-3 py-2 text-text-secondary hover:text-text-primary transition-colors cursor-pointer text-sm">
          <i class="fas fa-file-alt w-4 text-center"></i>
          <span>Logs</span>
        </router-link>
        <router-link to="/backups" class="flex items-center gap-3 py-2 text-text-secondary hover:text-text-primary transition-colors cursor-pointer text-sm">
          <i class="fas fa-save w-4 text-center"></i>
          <span>Backups</span>
        </router-link>
        <router-link to="/graph" class="flex items-center gap-3 py-2 text-text-secondary hover:text-text-primary transition-colors cursor-pointer text-sm">
          <i class="fas fa-project-diagram w-4 text-center"></i>
          <span>Graph</span>
        </router-link>
      </nav>
    </aside>

    <!-- Main Content -->
    <main class="flex-1 flex flex-col">
      <header class="h-14 border-b border-border flex items-center justify-between px-6 bg-bg-secondary">
        <div class="flex items-center gap-3">
          <i :class="[pageIcon, 'text-text-secondary']"></i>
          <span class="text-lg font-semibold text-text-primary">{{ pageTitle }}</span>
        </div>
        <div class="flex items-center gap-2">
          <!-- Page-specific actions -->
          <div v-if="route.path === '/config'" class="flex items-center gap-2">
            <button @click="reloadConfig" :disabled="configLoading" class="px-3 py-1.5 text-xs font-medium bg-transparent text-text-secondary border border-border rounded hover:bg-bg-hover hover:text-text-primary transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
              <i :class="['fas', configLoading ? 'fa-spinner fa-spin' : 'fa-sync']"></i>
              Reload
            </button>
            <button @click="saveConfig" :disabled="configSaving" class="px-3 py-1.5 text-xs font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
              <i :class="['fas', configSaving ? 'fa-spinner fa-spin' : 'fa-save']"></i>
              Save & Restart
            </button>
          </div>
          
          <div v-if="route.path === '/workspace'" class="flex items-center gap-2">
            <button @click="addDirectory" class="px-3 py-1.5 text-xs font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors">
              <i class="fas fa-folder-plus"></i>
              Add Directory
            </button>
          </div>
          
          <div v-if="route.path === '/logs'" class="flex items-center gap-2">
            <button @click="refreshLogs" :disabled="logsLoading" class="px-3 py-1.5 text-xs font-medium bg-transparent text-text-secondary border border-border rounded hover:bg-bg-hover hover:text-text-primary transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
              <i :class="['fas', logsLoading ? 'fa-spinner fa-spin' : 'fa-sync']"></i>
              Refresh
            </button>
            <button @click="exportLogs" class="px-3 py-1.5 text-xs font-medium bg-transparent text-text-secondary border border-border rounded hover:bg-bg-hover hover:text-text-primary transition-colors">
              <i class="fas fa-download"></i>
              Export
            </button>
            <button @click="clearLogs" class="px-3 py-1.5 text-xs font-medium bg-error text-white border border-error rounded hover:bg-red-600 transition-colors">
              <i class="fas fa-trash"></i>
              Clear
            </button>
          </div>
          
          <div v-if="route.path === '/backups'" class="flex items-center gap-2">
            <button @click="createBackup" class="px-3 py-1.5 text-xs font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors">
              <i class="fas fa-plus"></i>
              Create Backup
            </button>
          </div>
          
          <div v-if="route.path === '/connections'" class="flex items-center gap-2">
            <button @click="addConnection" class="px-3 py-1.5 text-xs font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors">
              <i class="fas fa-plus"></i>
              New Connection
            </button>
          </div>
          
          <!-- Global connection status -->
          <div class="flex items-center gap-2 ml-4 pl-4 border-l border-border">
            <span :class="['w-2 h-2 rounded-full', isConnected ? 'bg-success' : 'bg-text-muted']"></span>
            <span class="text-xs text-text-secondary">{{ isConnected ? 'Connected' : 'Disconnected' }}</span>
          </div>
        </div>
      </header>

      <div class="flex-1 overflow-y-auto">
        <router-view
          :selected-collection="selectedCollection"
          @select-collection="selectCollection"
        />
      </div>
    </main>

    <!-- Toast Notifications -->
    <ToastContainer />

    <!-- Global Dialog -->
    <div v-if="dialog.isDialogOpen.value" class="fixed inset-0 bg-black/50 flex items-center justify-center z-modal" @click.self="dialog.close">
      <div class="bg-bg-secondary border border-border rounded-xl w-full max-w-md mx-4 shadow-xl">
        <div class="flex items-center justify-between p-6 border-b border-border">
          <h2 class="text-lg font-semibold text-text-primary">{{ dialog.dialogOptions.value.title }}</h2>
          <button @click="dialog.close" class="p-2 text-text-secondary hover:text-text-primary hover:bg-bg-hover rounded transition-colors">
            <i class="fas fa-times"></i>
          </button>
        </div>
        
        <div class="p-6">
          <p class="text-sm text-text-primary whitespace-pre-wrap">{{ dialog.dialogOptions.value.message }}</p>
        </div>
        
        <div class="flex items-center justify-end gap-2 p-6 border-t border-border">
          <button v-if="dialog.dialogOptions.value.type === 'confirm'" @click="dialog.handleCancel" class="px-4 py-2 text-sm font-medium bg-transparent text-text-secondary border border-border rounded hover:bg-bg-hover hover:text-text-primary transition-colors">
            {{ dialog.dialogOptions.value.cancelText }}
          </button>
          <button @click="dialog.handleConfirm" class="px-4 py-2 text-sm font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors">
            {{ dialog.dialogOptions.value.confirmText }}
          </button>
        </div>
      </div>
    </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { storeToRefs } from 'pinia';
import { useConnectionsStore } from './stores/connections';
import { useVectorizerStore } from './stores/vectorizer';
import ToastContainer from './components/ToastContainer.vue';
import { useDialog } from './composables/useDialog';

const dialog = useDialog();

const router = useRouter();
const route = useRoute();

const connectionsStore = useConnectionsStore();
const vectorizerStore = useVectorizerStore();

const { connections, activeConnection } = storeToRefs(connectionsStore);
const { collections, loading, isConnected } = storeToRefs(vectorizerStore);

const activeConnectionId = ref<string | null>(null);
const selectedCollection = ref<string | null>(null);
const isDropdownOpen = ref(false);

// Page action states
const configLoading = ref(false);
const configSaving = ref(false);
const logsLoading = ref(false);

const pageTitle = computed(() => {
  const titles: Record<string, string> = {
    '/': 'Dashboard',
    '/workspace': 'Workspace Manager',
    '/config': 'Configuration',
    '/logs': 'Logs',
    '/backups': 'Backups & Snapshots',
    '/connections': 'Connection Manager',
    '/graph': 'Graph Relationships'
  };
  return titles[route.path] || 'Vectorizer GUI';
});

const pageIcon = computed(() => {
  const icons: Record<string, string> = {
    '/': 'fas fa-tachometer-alt',
    '/workspace': 'fas fa-folder-open',
    '/config': 'fas fa-cog',
    '/logs': 'fas fa-file-alt',
    '/backups': 'fas fa-save',
    '/connections': 'fas fa-network-wired',
    '/graph': 'fas fa-project-diagram'
  };
  return icons[route.path] || 'fas fa-cube';
});

async function switchConnection(): Promise<void> {
  if (!activeConnectionId.value) return;
  
  await connectionsStore.setActiveConnection(activeConnectionId.value);
  
  // Initialize vectorizer client with new connection
  const conn = activeConnection.value;
  if (conn) {
    await vectorizerStore.initializeClient(conn.host, conn.port, conn.auth?.token);
    await vectorizerStore.loadCollections();
  }
}

function toggleDropdown(): void {
  isDropdownOpen.value = !isDropdownOpen.value;
  console.log('Dropdown toggled:', isDropdownOpen.value);
}

async function selectConnection(connectionId: string): Promise<void> {
  activeConnectionId.value = connectionId;
  isDropdownOpen.value = false;
  await switchConnection();
  // Auto-reload collections after switching
  await vectorizerStore.loadCollections();
}

function openConnectionManager(): void {
  router.push('/connections');
}

// Page action functions
async function reloadConfig(): Promise<void> {
  configLoading.value = true;
  try {
    // Emit event to ConfigEditor component
    window.dispatchEvent(new CustomEvent('reload-config'));
  } finally {
    configLoading.value = false;
  }
}

async function saveConfig(): Promise<void> {
  configSaving.value = true;
  try {
    // Emit event to ConfigEditor component
    window.dispatchEvent(new CustomEvent('save-config'));
  } finally {
    configSaving.value = false;
  }
}

function addDirectory(): void {
  window.dispatchEvent(new CustomEvent('add-directory'));
}

async function refreshLogs(): Promise<void> {
  logsLoading.value = true;
  try {
    window.dispatchEvent(new CustomEvent('refresh-logs'));
  } finally {
    logsLoading.value = false;
  }
}

function exportLogs(): void {
  window.dispatchEvent(new CustomEvent('export-logs'));
}

function clearLogs(): void {
  window.dispatchEvent(new CustomEvent('clear-logs'));
}

function createBackup(): void {
  window.dispatchEvent(new CustomEvent('create-backup'));
}

function addConnection(): void {
  window.dispatchEvent(new CustomEvent('add-connection'));
}

function createCollection(): void {
  router.push('/collections/new');
}

function selectCollection(name: string): void {
  selectedCollection.value = name;
  router.push(`/collections/${name}`);
}

async function refreshCollections(): Promise<void> {
  if (activeConnection.value) {
    await vectorizerStore.loadCollections();
  }
}

function formatNumber(num: number): string {
  return new Intl.NumberFormat().format(num);
}

function minimizeWindow(): void {
  window.electron?.windowMinimize();
}

function maximizeWindow(): void {
  window.electron?.windowMaximize();
}

function closeWindow(): void {
  window.electron?.windowClose();
}

onMounted(async () => {
  await connectionsStore.loadConnections();
  
  // Auto-open connection manager if no connections exist
  if (connections.value.length === 0) {
    router.push('/connections');
    // Trigger the add connection form
    setTimeout(() => {
      window.dispatchEvent(new CustomEvent('add-connection'));
    }, 100);
  } else {
    // Auto-select active connection
    if (activeConnection.value) {
      activeConnectionId.value = activeConnection.value.id;
      await switchConnection();
    }
  }
  
  // Close dropdown when clicking outside
  document.addEventListener('click', (event) => {
    const target = event.target as HTMLElement;
    if (!target.closest('.connection-dropdown')) {
      isDropdownOpen.value = false;
    }
  });
});
</script>

<style scoped>
.drag-region {
  -webkit-app-region: drag;
}
.no-drag {
  -webkit-app-region: no-drag;
}
</style>
