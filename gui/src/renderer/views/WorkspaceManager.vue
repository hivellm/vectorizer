<template>
  <div class="workspace-manager">
    <div class="page-header">
      <h1><i class="fas fa-folder-open"></i> Workspace Manager</h1>
      <button @click="addDirectory" class="btn btn-primary">
        <i class="fas fa-folder-plus"></i> Add Directory
      </button>
    </div>

    <div class="workspace-info card">
      <div class="card-header">
        <h2>Workspace Directories</h2>
        <div class="auto-save-indicator" :class="{ saving: autoSaving }">
          <i :class="['fas', autoSaving ? 'fa-spinner fa-spin' : 'fa-check-circle']"></i>
          <span>{{ autoSaving ? 'Saving...' : 'Saved' }}</span>
        </div>
      </div>
      <div class="card-body">
        <div v-if="workspaces.length === 0" class="empty-state">
          <i class="fas fa-folder-open"></i>
          <h3>No Workspace Directories</h3>
          <p>Add directories to start indexing files into collections</p>
        </div>

        <div v-else class="workspaces-list">
          <div v-for="workspace in workspaces" :key="workspace.path" class="workspace-item">
            <div class="workspace-header">
              <div class="workspace-info">
                <i class="fas fa-folder"></i>
                <div class="workspace-details">
                  <span class="workspace-path">{{ workspace.path }}</span>
                  <span class="workspace-collection">→ {{ workspace.collection_name }}</span>
                </div>
              </div>
              <button @click="removeWorkspace(workspace.path)" class="btn-icon btn-danger">
                <i class="fas fa-trash"></i>
              </button>
            </div>
            
            <div class="workspace-stats">
              <span class="stat-item">
                <i class="fas fa-file"></i>
                {{ workspace.indexed_files }} files indexed
              </span>
            </div>

            <!-- Indexing Progress -->
            <div v-if="getIndexingProgress(workspace.collection_name)" class="indexing-progress">
              <div class="progress-header">
                <span>Indexing...</span>
                <span class="progress-percent">
                  {{ getIndexingProgress(workspace.collection_name)!.progress.toFixed(0) }}%
                </span>
              </div>
              <div class="progress-bar">
                <div
                  class="progress-fill"
                  :style="{ width: getIndexingProgress(workspace.collection_name)!.progress + '%' }"
                ></div>
              </div>
              <div class="progress-info">
                <span>{{ getIndexingProgress(workspace.collection_name)!.files_processed || 0 }} files processed</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Add Directory Modal -->
    <div v-if="showAddModal" class="modal-overlay" @click.self="showAddModal = false">
      <div class="modal">
        <div class="modal-header">
          <h2>Add Workspace Directory</h2>
          <button @click="showAddModal = false" class="btn-icon">
            <i class="fas fa-times"></i>
          </button>
        </div>
        <div class="modal-body">
          <div class="form-group">
            <label>Directory Path</label>
            <div class="input-group">
              <input
                v-model="newWorkspace.path"
                type="text"
                class="form-control"
                placeholder="Select a directory..."
                readonly
              />
              <button @click="selectDirectory" class="btn btn-secondary">
                <i class="fas fa-folder-open"></i> Browse
              </button>
            </div>
          </div>

          <div class="form-group">
            <label>Target Collection</label>
            <select v-model="newWorkspace.collection" class="form-control">
              <option value="">-- Select Collection --</option>
              <option v-for="collection in collections" :key="collection.name" :value="collection.name">
                {{ collection.name }}
              </option>
            </select>
            <p class="form-help">Or create a new collection name</p>
            <input
              v-if="!newWorkspace.collection"
              v-model="newWorkspace.newCollectionName"
              type="text"
              class="form-control"
              placeholder="New collection name..."
            />
          </div>

          <div class="form-group">
            <label class="checkbox-label">
              <input v-model="newWorkspace.autoIndex" type="checkbox" />
              <span>Auto-index file changes</span>
            </label>
          </div>
        </div>
        <div class="modal-footer">
          <button @click="showAddModal = false" class="btn btn-secondary">Cancel</button>
          <button @click="confirmAddDirectory" :disabled="!canAddDirectory" class="btn btn-primary">
            <i class="fas fa-plus"></i> Add Directory
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { storeToRefs } from 'pinia';
import { useVectorizerStore } from '../stores/vectorizer';
import type { WorkspaceDirectory, IndexingProgress } from '@shared/types';
import { useDebounceFn } from '@vueuse/core';

const vectorizerStore = useVectorizerStore();
const { collections, isConnected } = storeToRefs(vectorizerStore);

const workspaces = ref<WorkspaceDirectory[]>([]);
const indexingProgress = ref<IndexingProgress | null>(null);
const autoSaving = ref(false);
const showAddModal = ref(false);
const progressInterval = ref<NodeJS.Timeout | null>(null);

const newWorkspace = ref({
  path: '',
  collection: '',
  newCollectionName: '',
  autoIndex: true
});

const canAddDirectory = computed(() => 
  newWorkspace.value.path && 
  (newWorkspace.value.collection || newWorkspace.value.newCollectionName)
);

function addDirectory(): void {
  showAddModal.value = true;
  newWorkspace.value = {
    path: '',
    collection: '',
    newCollectionName: '',
    autoIndex: true
  };
}

async function selectDirectory(): Promise<void> {
  const path = await window.electron.selectDirectory();
  if (path) {
    newWorkspace.value.path = path;
  }
}

const debouncedSave = useDebounceFn(async () => {
  autoSaving.value = true;
  try {
    // Save workspace configuration
    await window.electron.setStoreValue('workspaces', workspaces.value);
    
    // Force save all collections
    for (const workspace of workspaces.value) {
      // TODO: Call force-save API
    }
  } catch (error) {
    console.error('Auto-save failed:', error);
  } finally {
    autoSaving.value = false;
  }
}, 3000);

async function confirmAddDirectory(): Promise<void> {
  try {
    const collectionName = newWorkspace.value.collection || newWorkspace.value.newCollectionName;
    
    // Add workspace to vectorizer
    // TODO: Call API to add workspace
    
    workspaces.value.push({
      path: newWorkspace.value.path,
      collection_name: collectionName,
      indexed_files: 0,
      auto_index: newWorkspace.value.autoIndex
    });

    debouncedSave();
    showAddModal.value = false;
  } catch (error) {
    alert(`Failed to add directory: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
}

async function removeWorkspace(path: string): Promise<void> {
  if (confirm(`Remove workspace directory?\n${path}`)) {
    workspaces.value = workspaces.value.filter(w => w.path !== path);
    debouncedSave();
  }
}

function getIndexingProgress(collectionName: string) {
  return indexingProgress.value?.collections.find(c => c.name === collectionName);
}

async function loadWorkspaces(): Promise<void> {
  try {
    const stored = await window.electron.getStoreValue('workspaces');
    if (Array.isArray(stored)) {
      workspaces.value = stored as WorkspaceDirectory[];
    }
  } catch (error) {
    console.error('Failed to load workspaces:', error);
  }
}

async function loadIndexingProgress(): Promise<void> {
  // TODO: Load indexing progress from API
}

onMounted(() => {
  loadWorkspaces();
  loadIndexingProgress();
  
  // Poll indexing progress every 2 seconds
  progressInterval.value = setInterval(loadIndexingProgress, 2000);
});

onUnmounted(() => {
  if (progressInterval.value) {
    clearInterval(progressInterval.value);
  }
});
</script>

<style scoped>
.workspace-manager {
  padding: 2rem;
}

.workspace-info {
  margin-bottom: 2rem;
}

.auto-save-indicator {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.875rem;
  color: #10b981;
}

.auto-save-indicator.saving {
  color: #f59e0b;
}

.workspaces-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.workspace-item {
  border: 1px solid #e5e7eb;
  border-radius: 6px;
  padding: 1rem;
}

.workspace-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.75rem;
}

.workspace-info {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.workspace-details {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.workspace-path {
  font-weight: 600;
  color: #1a1a2e;
}

.workspace-collection {
  font-size: 0.875rem;
  color: #6b7280;
}

.workspace-stats {
  display: flex;
  gap: 1rem;
  font-size: 0.875rem;
  color: #6b7280;
}

.indexing-progress {
  margin-top: 1rem;
  padding-top: 1rem;
  border-top: 1px solid #e5e7eb;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 0.5rem;
  font-size: 0.875rem;
}

.progress-bar {
  height: 8px;
  background: #e5e7eb;
  border-radius: 4px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: #3b82f6;
  transition: width 0.3s ease;
}

.progress-info {
  margin-top: 0.5rem;
  font-size: 0.875rem;
  color: #6b7280;
}

.input-group {
  display: flex;
  gap: 0.5rem;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
}

.form-help {
  margin-top: 0.25rem;
  font-size: 0.875rem;
  color: #6b7280;
}
</style>

