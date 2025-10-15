<template>
  <div class="backup-manager">
    <div class="page-header">
      <h1><i class="fas fa-save"></i> Backups & Snapshots</h1>
      <button @click="createBackupModal = true" class="btn btn-primary">
        <i class="fas fa-plus"></i> Create Backup
      </button>
    </div>

    <!-- Backup Directory Info -->
    <div class="backup-info card">
      <div class="card-header">
        <h2>Backup Directory</h2>
        <button @click="openBackupDirectory" class="btn btn-secondary">
          <i class="fas fa-folder-open"></i> Open Directory
        </button>
      </div>
      <div class="card-body">
        <div class="info-item">
          <span class="label">Location:</span>
          <span class="value">{{ backupDirectory || 'Loading...' }}</span>
        </div>
      </div>
    </div>

    <!-- Backups List -->
    <div class="backups-list">
      <h2>Available Backups</h2>
      
      <div v-if="loading && backups.length === 0" class="loading-state">
        <i class="fas fa-spinner fa-spin"></i>
        <span>Loading backups...</span>
      </div>

      <div v-else-if="backups.length === 0" class="empty-state">
        <i class="fas fa-save"></i>
        <h3>No Backups</h3>
        <p>Create your first backup to protect your data</p>
      </div>

      <div v-else class="backup-cards">
        <div v-for="backup in backups" :key="backup.id" class="backup-card">
          <div class="backup-header">
            <div class="backup-info">
              <h3>{{ backup.name }}</h3>
              <span class="backup-date">{{ formatDate(backup.date) }}</span>
            </div>
            <div class="backup-size">
              {{ formatSize(backup.size) }}
            </div>
          </div>

          <div class="backup-collections">
            <span class="label">Collections:</span>
            <div class="collection-tags">
              <span v-for="collection in backup.collections" :key="collection" class="collection-tag">
                {{ collection }}
              </span>
            </div>
          </div>

          <div class="backup-actions">
            <button @click="restoreBackup(backup.id)" class="btn btn-sm btn-primary">
              <i class="fas fa-undo"></i> Restore
            </button>
            <button @click="deleteBackup(backup.id)" class="btn btn-sm btn-danger">
              <i class="fas fa-trash"></i> Delete
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Create Backup Modal -->
    <div v-if="createBackupModal" class="modal-overlay" @click.self="createBackupModal = false">
      <div class="modal">
        <div class="modal-header">
          <h2>Create New Backup</h2>
          <button @click="createBackupModal = false" class="btn-icon">
            <i class="fas fa-times"></i>
          </button>
        </div>
        <div class="modal-body">
          <div class="form-group">
            <label for="backup-name">Backup Name</label>
            <input
              id="backup-name"
              v-model="newBackup.name"
              type="text"
              class="form-control"
              placeholder="My Backup"
              required
            />
          </div>

          <div class="form-group">
            <label>Collections to Backup</label>
            <div class="checkbox-list">
              <label v-for="collection in collections" :key="collection.name" class="checkbox-label">
                <input
                  v-model="newBackup.selectedCollections"
                  type="checkbox"
                  :value="collection.name"
                />
                <span>{{ collection.name }} ({{ formatNumber(collection.vector_count) }} vectors)</span>
              </label>
            </div>
          </div>
        </div>
        <div class="modal-footer">
          <button @click="createBackupModal = false" class="btn btn-secondary">Cancel</button>
          <button @click="confirmCreateBackup" :disabled="creating || !canCreateBackup" class="btn btn-primary">
            <i :class="['fas', creating ? 'fa-spinner fa-spin' : 'fa-save']"></i>
            {{ creating ? 'Creating...' : 'Create Backup' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { storeToRefs } from 'pinia';
import { useVectorizerStore } from '../stores/vectorizer';
import type { Backup } from '@shared/types';

const vectorizerStore = useVectorizerStore();
const { collections } = storeToRefs(vectorizerStore);

const backups = ref<Backup[]>([]);
const backupDirectory = ref<string | null>(null);
const loading = ref(false);
const createBackupModal = ref(false);
const creating = ref(false);

const newBackup = ref({
  name: '',
  selectedCollections: [] as string[]
});

const canCreateBackup = computed(() => 
  newBackup.value.name.trim() && newBackup.value.selectedCollections.length > 0
);

async function loadBackups(): Promise<void> {
  loading.value = true;
  try {
    // TODO: Load backups from API
    // const response = await api.listBackups();
    // backups.value = response.data.backups;
    
    // Mock data for now
    backups.value = [];
  } catch (error) {
    console.error('Failed to load backups:', error);
  } finally {
    loading.value = false;
  }
}

async function loadBackupDirectory(): Promise<void> {
  try {
    // TODO: Get backup directory from API
    // const response = await api.getBackupDirectory();
    // backupDirectory.value = response.data.path;
    
    // Default for now
    backupDirectory.value = './backups';
  } catch (error) {
    console.error('Failed to load backup directory:', error);
  }
}

async function openBackupDirectory(): Promise<void> {
  if (!backupDirectory.value) return;
  
  // TODO: Open directory in file explorer
  console.log('Open directory:', backupDirectory.value);
}

async function confirmCreateBackup(): Promise<void> {
  creating.value = true;
  try {
    // TODO: Create backup via API
    // const response = await api.createBackup(
    //   newBackup.value.name,
    //   newBackup.value.selectedCollections
    // );

    alert('Backup created successfully');
    createBackupModal.value = false;
    newBackup.value = { name: '', selectedCollections: [] };
    await loadBackups();
  } catch (error) {
    alert(`Failed to create backup: ${error instanceof Error ? error.message : 'Unknown error'}`);
  } finally {
    creating.value = false;
  }
}

async function restoreBackup(backupId: string): Promise<void> {
  if (!confirm('Restore this backup? This will overwrite current data.')) {
    return;
  }

  try {
    // TODO: Restore backup via API
    // await api.restoreBackup(backupId);
    
    alert('Backup restored successfully. Refreshing data...');
    await vectorizerStore.loadCollections();
  } catch (error) {
    alert(`Failed to restore backup: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
}

async function deleteBackup(backupId: string): Promise<void> {
  if (!confirm('Delete this backup? This cannot be undone.')) {
    return;
  }

  try {
    // TODO: Delete backup via API
    backups.value = backups.value.filter(b => b.id !== backupId);
    alert('Backup deleted successfully');
  } catch (error) {
    alert(`Failed to delete backup: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
}

async function refreshLogs(): Promise<void> {
  await loadBackups();
}

function formatDate(dateString: string): string {
  const date = new Date(dateString);
  return date.toLocaleString();
}

function formatSize(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB'];
  let size = bytes;
  let unitIndex = 0;

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }

  return `${size.toFixed(2)} ${units[unitIndex]}`;
}

function formatNumber(num: number): string {
  return new Intl.NumberFormat().format(num);
}

onMounted(() => {
  loadBackups();
  loadBackupDirectory();
});

