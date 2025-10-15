<template>
  <div id="app" class="vectorizer-app">
    <!-- Sidebar -->
    <aside class="sidebar">
      <div class="sidebar-header">
        <img src="/icon.png" alt="Vectorizer" class="logo" />
        <h1>Vectorizer</h1>
      </div>

      <!-- Connection Selector -->
      <div class="connection-selector">
        <select v-model="activeConnectionId" @change="switchConnection" class="connection-select">
          <option v-if="connections.length === 0" value="">No connections</option>
          <option v-for="conn in connections" :key="conn.id" :value="conn.id">
            {{ conn.name }} ({{ conn.status }})
          </option>
        </select>
        <button @click="openConnectionManager" class="btn-icon" title="Manage Connections">
          <i class="fas fa-cog"></i>
        </button>
      </div>

      <!-- Collections List -->
      <div class="collections-section">
        <div class="section-header">
          <h3>Collections</h3>
          <button @click="createCollection" class="btn-icon" title="New Collection">
            <i class="fas fa-plus"></i>
          </button>
        </div>
        
        <div v-if="loading" class="loading-state">
          <i class="fas fa-spinner fa-spin"></i>
          <span>Loading...</span>
        </div>
        
        <div v-else-if="collections.length === 0" class="empty-state">
          <p>No collections</p>
        </div>
        
        <ul v-else class="collections-list">
          <li
            v-for="collection in collections"
            :key="collection.name"
            :class="['collection-item', { active: selectedCollection === collection.name }]"
            @click="selectCollection(collection.name)"
          >
            <div class="collection-info">
              <i class="fas fa-database"></i>
              <span class="collection-name">{{ collection.name }}</span>
            </div>
            <span class="collection-count">{{ formatNumber(collection.vector_count) }}</span>
          </li>
        </ul>
      </div>

      <!-- Navigation -->
      <nav class="nav-menu">
        <router-link to="/" class="nav-item">
          <i class="fas fa-home"></i>
          <span>Dashboard</span>
        </router-link>
        <router-link to="/workspace" class="nav-item">
          <i class="fas fa-folder-open"></i>
          <span>Workspace</span>
        </router-link>
        <router-link to="/config" class="nav-item">
          <i class="fas fa-cog"></i>
          <span>Configuration</span>
        </router-link>
        <router-link to="/logs" class="nav-item">
          <i class="fas fa-file-alt"></i>
          <span>Logs</span>
        </router-link>
        <router-link to="/backups" class="nav-item">
          <i class="fas fa-save"></i>
          <span>Backups</span>
        </router-link>
      </nav>
    </aside>

    <!-- Main Content -->
    <main class="main-content">
      <header class="top-bar">
        <div class="breadcrumb">
          <span>{{ pageTitle }}</span>
        </div>
        <div class="top-bar-actions">
          <div class="connection-status">
            <span :class="['status-dot', { online: isConnected }]"></span>
            <span>{{ isConnected ? 'Connected' : 'Disconnected' }}</span>
          </div>
        </div>
      </header>

      <div class="content-area">
        <router-view
          :selected-collection="selectedCollection"
          @select-collection="selectCollection"
        />
      </div>
    </main>

    <!-- Toast Notifications -->
    <ToastContainer />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { storeToRefs } from 'pinia';
import { useConnectionsStore } from './stores/connections';
import { useVectorizerStore } from './stores/vectorizer';
import ToastContainer from './components/ToastContainer.vue';

const router = useRouter();
const route = useRoute();

const connectionsStore = useConnectionsStore();
const vectorizerStore = useVectorizerStore();

const { connections, activeConnection } = storeToRefs(connectionsStore);
const { collections, loading, isConnected } = storeToRefs(vectorizerStore);

const activeConnectionId = ref<string | null>(null);
const selectedCollection = ref<string | null>(null);

const pageTitle = computed(() => {
  const titles: Record<string, string> = {
    '/': 'Dashboard',
    '/workspace': 'Workspace Manager',
    '/config': 'Configuration',
    '/logs': 'Logs',
    '/backups': 'Backups & Snapshots'
  };
  return titles[route.path] || 'Vectorizer GUI';
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

function openConnectionManager(): void {
  router.push('/connections');
}

function createCollection(): void {
  router.push('/collections/new');
}

function selectCollection(name: string): void {
  selectedCollection.value = name;
  router.push(`/collections/${name}`);
}

function formatNumber(num: number): string {
  return new Intl.NumberFormat().format(num);
}

onMounted(async () => {
  await connectionsStore.loadConnections();
  
  // Auto-select active connection
  if (activeConnection.value) {
    activeConnectionId.value = activeConnection.value.id;
    await switchConnection();
  }
});
</script>

<style scoped>
.vectorizer-app {
  display: grid;
  grid-template-columns: 280px 1fr;
  height: 100vh;
  overflow: hidden;
}

/* Sidebar Styles */
.sidebar {
  background: var(--bg-dark);
  color: white;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
}

.sidebar-header {
  padding: 1.5rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  display: flex;
  align-items: center;
  gap: 1rem;
}

.sidebar-header .logo {
  width: 40px;
  height: 40px;
  border-radius: 8px;
}

.sidebar-header h1 {
  font-size: 1.25rem;
  font-weight: 600;
  margin: 0;
}

.connection-selector {
  padding: 1rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  display: flex;
  gap: 0.5rem;
}

.connection-select {
  flex: 1;
  padding: 0.5rem;
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 4px;
  color: white;
  font-size: 0.875rem;
}

.collections-section {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}

.section-header h3 {
  font-size: 0.875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: rgba(255, 255, 255, 0.7);
  margin: 0;
}

.collections-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.collection-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem 1rem;
  cursor: pointer;
  border-radius: 6px;
  transition: background 0.2s;
  margin-bottom: 0.25rem;
}

.collection-item:hover {
  background: rgba(255, 255, 255, 0.1);
}

.collection-item.active {
  background: var(--color-primary);
}

.collection-info {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  flex: 1;
  min-width: 0;
}

.collection-name {
  font-size: 0.875rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.collection-count {
  font-size: 0.75rem;
  color: rgba(255, 255, 255, 0.7);
  flex-shrink: 0;
}

.loading-state,
.empty-state {
  text-align: center;
  padding: 2rem 1rem;
  color: rgba(255, 255, 255, 0.5);
  font-size: 0.875rem;
}

.nav-menu {
  padding: 1rem;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem 1rem;
  color: white;
  text-decoration: none;
  border-radius: 6px;
  transition: background 0.2s;
  margin-bottom: 0.25rem;
}

.nav-item:hover {
  background: rgba(255, 255, 255, 0.1);
}

.nav-item.router-link-active {
  background: var(--color-primary);
}

/* Main Content Styles */
.main-content {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg-secondary);
}

.top-bar {
  background: white;
  padding: 1rem 2rem;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.breadcrumb {
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--text-primary);
}

.top-bar-actions {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.connection-status {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.status-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--color-danger);
}

.status-dot.online {
  background: var(--color-success);
}

.content-area {
  flex: 1;
  overflow-y: auto;
  background: var(--bg-secondary);
}

.btn-icon {
  border: none;
  background: rgba(255, 255, 255, 0.1);
  color: white;
  cursor: pointer;
  padding: 0.5rem;
  border-radius: 4px;
  transition: background 0.2s;
}

.btn-icon:hover {
  background: rgba(255, 255, 255, 0.2);
}
</style>

