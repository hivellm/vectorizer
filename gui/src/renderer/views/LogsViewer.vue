<template>
  <div class="p-8">
    <!-- Header -->
    <div class="flex items-center justify-between mb-6">
      <h2 class="text-xl font-semibold text-text-primary">Logs</h2>
      <div class="flex items-center gap-3">
        <button @click="refreshLogs" :disabled="loading" class="px-4 py-2 bg-bg-tertiary border border-border rounded text-text-primary hover:bg-bg-hover hover:border-border-light transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
          <i :class="['fas', 'mr-2', loading ? 'fa-spinner fa-spin' : 'fa-sync']"></i>
          Refresh
        </button>
        <button @click="exportLogs" class="px-4 py-2 bg-bg-tertiary border border-border rounded text-text-primary hover:bg-bg-hover hover:border-border-light transition-colors">
          <i class="fas fa-download mr-2"></i>
          Export
        </button>
        <button @click="clearLogs" class="px-4 py-2 bg-error text-white border border-error rounded hover:bg-error/80 transition-colors">
          <i class="fas fa-trash mr-2"></i>
          Clear
        </button>
      </div>
    </div>

    <!-- Filters -->
    <div class="bg-bg-secondary border border-border rounded-xl p-6 mb-6">
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div>
          <label class="block text-xs font-medium text-text-secondary mb-2">Level</label>
          <select v-model="filterLevel" @change="applyFilters" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary focus:outline-none focus:border-border-light transition-colors text-sm">
            <option value="">All Levels</option>
            <option value="DEBUG">Debug</option>
            <option value="INFO">Info</option>
            <option value="WARN">Warning</option>
            <option value="ERROR">Error</option>
          </select>
        </div>

        <div>
          <label class="block text-xs font-medium text-text-secondary mb-2">Search</label>
          <input
            v-model="filterSearch"
            type="text"
            class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm"
            placeholder="Filter logs..."
            @input="applyFilters"
          />
        </div>

        <div>
          <label class="block text-xs font-medium text-text-secondary mb-2">Max Lines</label>
          <input
            v-model.number="maxLines"
            type="number"
            min="100"
            max="10000"
            step="100"
            class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm"
            @input="applyFilters"
          />
        </div>
      </div>
    </div>

    <!-- Logs Container -->
    <div class="bg-bg-secondary border border-border rounded-xl overflow-hidden">
      <div class="flex items-center justify-between p-4 border-b border-border">
        <div class="flex items-center gap-4">
          <span class="text-sm text-text-secondary">Showing {{ filteredLogs.length }} logs</span>
          <div class="flex items-center gap-2">
            <CustomCheckbox v-model="autoScroll" label="Auto-scroll" />
            <CustomCheckbox v-model="showTimestamps" label="Timestamps" />
          </div>
        </div>
        <div class="flex items-center gap-2">
          <span class="text-xs text-text-muted">Last updated: {{ lastUpdate }}</span>
          <div v-if="loading" class="flex items-center gap-2 text-text-secondary">
            <i class="fas fa-spinner fa-spin"></i>
            <span class="text-sm">Loading...</span>
          </div>
        </div>
      </div>

      <!-- Logs List -->
      <div ref="logsContainer" class="h-96 overflow-y-auto bg-bg-primary">
        <div v-if="filteredLogs.length === 0" class="flex items-center justify-center h-full text-text-muted">
          <div class="text-center">
            <i class="fas fa-file-alt text-4xl mb-4"></i>
            <p class="text-lg">No logs found</p>
            <p class="text-sm">Try adjusting your filters or refresh the logs</p>
          </div>
        </div>
        
        <div v-else class="p-4 space-y-1">
          <div
            v-for="(log, index) in filteredLogs"
            :key="index"
            class="flex items-start gap-3 p-2 rounded hover:bg-bg-hover transition-colors"
            :class="getLogLevelClass(log.level)"
          >
            <div class="flex-shrink-0 w-16 text-xs text-text-muted">
              <span v-if="showTimestamps">{{ formatTimestamp(log.timestamp) }}</span>
            </div>
            
            <div class="flex-shrink-0">
              <span class="inline-flex items-center px-2 py-1 rounded text-xs font-medium" :class="getLevelBadgeClass(log.level)">
                {{ log.level }}
              </span>
            </div>
            
            <div class="flex-1 min-w-0">
              <div class="text-sm text-text-primary break-words">{{ log.message }}</div>
              <div v-if="log.source" class="text-xs text-text-muted mt-1">{{ log.source }}</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue';
import { useVectorizerStore } from '../stores/vectorizer';
import CustomCheckbox from '../components/CustomCheckbox.vue';

type LogLevel = 'DEBUG' | 'INFO' | 'WARN' | 'ERROR';

interface LogEntry {
  timestamp: string;
  level: LogLevel;
  message: string;
  source?: string;
}

const vectorizerStore = useVectorizerStore();

// State
const logs = ref<LogEntry[]>([]);
const loading = ref(false);
const filterLevel = ref<LogLevel | ''>('');
const filterSearch = ref('');
const maxLines = ref(1000);
const autoScroll = ref(true);
const showTimestamps = ref(true);
const logsContainer = ref<HTMLElement | null>(null);
const pollInterval = ref<NodeJS.Timeout | null>(null);
const lastUpdate = ref('');

// Computed
const filteredLogs = computed(() => {
  let filtered = logs.value;
  
  if (filterLevel.value) {
    filtered = filtered.filter(log => log.level === filterLevel.value);
  }
  
  if (filterSearch.value) {
    const search = filterSearch.value.toLowerCase();
    filtered = filtered.filter(log => 
      log.message.toLowerCase().includes(search) ||
      log.source?.toLowerCase().includes(search)
    );
  }
  
  return filtered.slice(-maxLines.value);
});

// Methods
async function refreshLogs(): Promise<void> {
  loading.value = true;
  try {
    const client = vectorizerStore.client;
    
    if (!client) {
      console.warn('Vectorizer client not initialized');
      logs.value = [];
      return;
    }
    
    // Load logs from API
    const response = await fetch(`${client.config.baseURL}/api/logs`);
    
    if (!response.ok) {
      throw new Error(`Failed to load logs: ${response.statusText}`);
    }
    
    const data = await response.json();
    const vectorizerLogs = data.logs || [];
    
    logs.value = vectorizerLogs.map((log: any) => ({
      timestamp: log.timestamp || new Date().toISOString(),
      level: (log.level || 'INFO') as LogLevel,
      message: log.message || log.msg || String(log),
      source: 'vectorizer'
    }));
    
    lastUpdate.value = new Date().toLocaleTimeString();
    
    // Auto-scroll to bottom if enabled
    if (autoScroll.value) {
      await nextTick();
      scrollToBottom();
    }
  } catch (error) {
    console.error('Failed to load logs:', error);
    logs.value = [];
  } finally {
    loading.value = false;
  }
}

function applyFilters(): void {
  // Filters are applied reactively via computed property
}

function clearLogs(): void {
  if (confirm('Clear all logs from display?')) {
    logs.value = [];
  }
}

function exportLogs(): void {
  const dataStr = JSON.stringify(logs.value, null, 2);
  const dataBlob = new Blob([dataStr], { type: 'application/json' });
  const url = URL.createObjectURL(dataBlob);
  const link = document.createElement('a');
  link.href = url;
  link.download = `vectorizer-logs-${new Date().toISOString().split('T')[0]}.json`;
  link.click();
  URL.revokeObjectURL(url);
}

function getLogLevelClass(level: LogLevel): string {
  switch (level) {
    case 'ERROR':
      return 'border-l-4 border-error bg-error/5';
    case 'WARN':
      return 'border-l-4 border-warning bg-warning/5';
    case 'DEBUG':
      return 'border-l-4 border-info bg-info/5';
    default:
      return '';
  }
}

function getLevelBadgeClass(level: LogLevel): string {
  switch (level) {
    case 'ERROR':
      return 'bg-error text-white';
    case 'WARN':
      return 'bg-warning text-white';
    case 'INFO':
      return 'bg-info text-white';
    case 'DEBUG':
      return 'bg-bg-tertiary text-text-secondary';
    default:
      return 'bg-bg-tertiary text-text-secondary';
  }
}

function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp);
  return date.toLocaleTimeString();
}

function scrollToBottom(): void {
  if (logsContainer.value) {
    logsContainer.value.scrollTop = logsContainer.value.scrollHeight;
  }
}

// Watch for auto-scroll changes
watch(autoScroll, (newValue) => {
  if (newValue) {
    nextTick(() => scrollToBottom());
  }
});

// Lifecycle
onMounted(() => {
  refreshLogs();
  
  // Poll logs every 5 seconds
  pollInterval.value = setInterval(refreshLogs, 5000);
});

onUnmounted(() => {
  if (pollInterval.value) {
    clearInterval(pollInterval.value);
  }
});
</script>