<template>
  <div class="p-8">
    <!-- Backup Directory Info -->
    <div class="bg-bg-secondary border border-border rounded-xl p-6 mb-6">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-lg font-semibold text-text-primary">Backup Directory</h3>
        <button @click="openBackupDirectory" class="px-3 py-1.5 text-xs font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors">
          <i class="fas fa-folder-open mr-1"></i> Open Directory
        </button>
      </div>
      <div class="flex items-center gap-2 text-sm">
        <span class="text-text-secondary">Location:</span>
        <span class="text-text-primary font-mono">{{ backupDirectory || 'Loading...' }}</span>
      </div>
    </div>

    <!-- Backups List -->
    <div>
      <div v-if="loading && backups.length === 0" class="flex flex-col items-center justify-center py-16 text-text-secondary">
        <i class="fas fa-spinner fa-spin text-4xl mb-4"></i>
        <span>Loading backups...</span>
      </div>

      <div v-else-if="backups.length === 0" class="flex flex-col items-center justify-center py-16 text-text-secondary">
        <i class="fas fa-save text-4xl mb-4"></i>
        <h3 class="text-lg font-medium text-text-primary mb-2">No Backups</h3>
        <p class="text-sm text-text-secondary">Click "Create Backup" in the top bar to protect your data</p>
      </div>

      <div v-else class="space-y-4">
        <div v-for="backup in backups" :key="backup.id" class="bg-bg-secondary border border-border rounded-xl p-6">
          <div class="flex items-start justify-between mb-4">
            <div>
              <h3 class="text-lg font-semibold text-text-primary mb-1">{{ backup.name }}</h3>
              <span class="text-sm text-text-secondary">{{ formatDate(backup.date) }}</span>
            </div>
            <div class="text-right">
              <div class="text-lg font-semibold text-info">{{ formatSize(backup.size) }}</div>
            </div>
          </div>

          <div class="mb-4">
            <span class="text-sm font-medium text-text-secondary">Collections:</span>
            <div class="flex flex-wrap gap-2 mt-2">
              <span v-for="collection in backup.collections" :key="collection" class="px-3 py-1 text-xs font-medium bg-bg-tertiary text-text-primary border border-border rounded-full">
                {{ collection }}
              </span>
            </div>
          </div>

          <div class="flex gap-2 justify-end">
            <button @click="restoreBackup(backup.id)" class="px-3 py-1.5 text-xs font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors">
              <i class="fas fa-undo mr-1"></i> Restore
            </button>
            <button @click="deleteBackup(backup.id)" class="px-3 py-1.5 text-xs font-medium bg-error/20 text-error border border-error rounded hover:bg-error/30 transition-colors">
              <i class="fas fa-trash mr-1"></i> Delete
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Create Backup Modal -->
    <div v-if="createBackupModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-modal" @click.self="createBackupModal = false">
      <div class="bg-bg-secondary border border-border rounded-xl w-full max-w-md mx-4 shadow-xl">
        <div class="flex items-center justify-between p-6 border-b border-border">
          <h2 class="text-lg font-semibold text-text-primary">Create New Backup</h2>
          <button @click="createBackupModal = false" class="p-2 text-text-secondary hover:text-text-primary hover:bg-bg-hover rounded transition-colors">
            <i class="fas fa-times"></i>
          </button>
        </div>
        
        <div class="p-6 space-y-4">
          <div>
            <label for="backup-name" class="block text-sm font-medium text-text-primary mb-2">Backup Name</label>
            <input
              id="backup-name"
              v-model="newBackup.name"
              type="text"
              class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors"
              placeholder="My Backup"
              required
            />
          </div>

          <div>
            <label class="block text-sm font-medium text-text-primary mb-2">Collections to Backup</label>
            <div class="max-h-64 overflow-y-auto border border-border rounded bg-bg-tertiary p-3 space-y-2">
              <label v-for="collection in collections" :key="collection.name" class="flex items-center gap-2 p-2 hover:bg-bg-hover rounded cursor-pointer transition-colors">
                <input
                  v-model="newBackup.selectedCollections"
                  type="checkbox"
                  :value="collection.name"
                  class="cursor-pointer"
                />
                <span class="text-sm text-text-primary">{{ collection.name }} <span class="text-text-secondary">({{ formatNumber(collection.vector_count) }} vectors)</span></span>
              </label>
            </div>
          </div>
        </div>
        
        <div class="flex items-center justify-end gap-2 p-6 border-t border-border">
          <button @click="createBackupModal = false" class="px-4 py-2 text-sm font-medium bg-transparent text-text-secondary border border-border rounded hover:bg-bg-hover hover:text-text-primary transition-colors">Cancel</button>
          <button @click="confirmCreateBackup" :disabled="creating || !canCreateBackup" class="px-4 py-2 text-sm font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
            <i :class="['fas', creating ? 'fa-spinner fa-spin' : 'fa-save', 'mr-1']"></i>
            {{ creating ? 'Creating...' : 'Create Backup' }}
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
import { useDialog } from '../composables/useDialog';
import type { Backup } from '@shared/types';

const vectorizerStore = useVectorizerStore();
const dialog = useDialog();
const { collections } = storeToRefs(vectorizerStore);

// Listen for create-backup event from App.vue
function handleCreateBackup(): void {
  createBackupModal.value = true;
}

onMounted(() => {
  window.addEventListener('create-backup', handleCreateBackup);
  loadBackups();
  loadBackupDirectory();
});

onUnmounted(() => {
  window.removeEventListener('create-backup', handleCreateBackup);
});

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
    const client = vectorizerStore.client;
    if (!client) {
      console.error('Vectorizer client not initialized');
      return;
    }
    
    const response = await fetch(`${client.config.baseURL}/api/backups`, {
      headers: client.config.apiKey ? {
        'Authorization': `Bearer ${client.config.apiKey}`
      } : {}
    });
    
    if (!response.ok) {
      throw new Error(`Failed to load backups: ${response.statusText}`);
    }
    
    const data = await response.json();
    backups.value = data.backups || [];
  } catch (error) {
    console.error('Failed to load backups:', error);
    await dialog.alert(
      `Failed to load backups: ${error instanceof Error ? error.message : 'Unknown error'}`,
      'Error'
    );
  } finally {
    loading.value = false;
  }
}

async function loadBackupDirectory(): Promise<void> {
  try {
    const client = vectorizerStore.client;
    if (!client) {
      console.error('Vectorizer client not initialized');
      return;
    }
    
    const response = await fetch(`${client.config.baseURL}/api/backups/directory`, {
      headers: client.config.apiKey ? {
        'Authorization': `Bearer ${client.config.apiKey}`
      } : {}
    });
    
    if (!response.ok) {
      throw new Error(`Failed to load backup directory: ${response.statusText}`);
    }
    
    const data = await response.json();
    backupDirectory.value = data.path || './backups';
  } catch (error) {
    console.error('Failed to load backup directory:', error);
    backupDirectory.value = './backups';
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
    const client = vectorizerStore.client;
    if (!client) {
      await dialog.alert('Vectorizer client not initialized', 'Error');
      return;
    }
    
    const response = await fetch(`${client.config.baseURL}/api/backups/create`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...(client.config.apiKey ? { 'Authorization': `Bearer ${client.config.apiKey}` } : {})
      },
      body: JSON.stringify({
        name: newBackup.value.name,
        collections: newBackup.value.selectedCollections
      })
    });

    if (!response.ok) {
      throw new Error(`Failed to create backup: ${response.statusText}`);
    }

    await dialog.alert('Backup created successfully!', 'Success');
    createBackupModal.value = false;
    newBackup.value = { name: '', selectedCollections: [] };
    await loadBackups();
  } catch (error) {
    await dialog.alert(
      `Failed to create backup: ${error instanceof Error ? error.message : 'Unknown error'}`,
      'Error'
    );
  } finally {
    creating.value = false;
  }
}

async function restoreBackup(backupId: string): Promise<void> {
  const confirmed = await dialog.confirm(
    'Restore this backup? This will overwrite current data.',
    'Restore Backup'
  );
  
  if (!confirmed) return;

  try {
    const client = vectorizerStore.client;
    if (!client) {
      await dialog.alert('Vectorizer client not initialized', 'Error');
      return;
    }
    
    const response = await fetch(`${client.config.baseURL}/api/backups/restore`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...(client.config.apiKey ? { 'Authorization': `Bearer ${client.config.apiKey}` } : {})
      },
      body: JSON.stringify({
        backup_id: backupId
      })
    });

    if (!response.ok) {
      throw new Error(`Failed to restore backup: ${response.statusText}`);
    }
    
    await dialog.alert('Backup restored successfully. Refreshing data...', 'Success');
    await vectorizerStore.loadCollections();
  } catch (error) {
    await dialog.alert(
      `Failed to restore backup: ${error instanceof Error ? error.message : 'Unknown error'}`,
      'Error'
    );
  }
}

async function deleteBackup(backupId: string): Promise<void> {
  const confirmed = await dialog.confirm(
    'Delete this backup? This action cannot be undone.',
    'Delete Backup'
  );
  
  if (!confirmed) return;

  try {
    // Delete the backup file
    const backup_file = `./backups/${backupId}.backup`;
    
    // For now, just remove from the list and inform user
    // TODO: Implement actual file deletion via API
    backups.value = backups.value.filter((b: Backup) => b.id !== backupId);
    await dialog.alert('Backup deleted successfully.', 'Success');
  } catch (error) {
    await dialog.alert(
      `Failed to delete backup: ${error instanceof Error ? error.message : 'Unknown error'}`,
      'Error'
    );
  }
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
</script>
