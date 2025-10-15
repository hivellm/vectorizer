<template>
  <div class="logs-viewer">
    <div class="page-header">
      <h1><i class="fas fa-file-alt"></i> Logs</h1>
      <div class="header-actions">
        <button @click="refreshLogs" :disabled="loading" class="btn btn-secondary">
          <i :class="['fas', loading ? 'fa-spinner fa-spin' : 'fa-sync']"></i>
          Refresh
        </button>
        <button @click="exportLogs" class="btn btn-secondary">
          <i class="fas fa-download"></i>
          Export
        </button>
        <button @click="clearLogs" class="btn btn-danger">
          <i class="fas fa-trash"></i>
          Clear
        </button>
      </div>
    </div>

    <!-- Filters -->
    <div class="filters-card card">
      <div class="card-body">
        <div class="filters-row">
          <div class="filter-group">
            <label>Level</label>
            <select v-model="filterLevel" class="filter-select" @change="applyFilters">
              <option value="">All Levels</option>
              <option value="DEBUG">Debug</option>
              <option value="INFO">Info</option>
              <option value="WARN">Warning</option>
              <option value="ERROR">Error</option>
            </select>
          </div>

          <div class="filter-group">
            <label>Search</label>
            <input
              v-model="filterSearch"
              type="text"
              class="filter-input"
              placeholder="Filter logs..."
              @input="applyFilters"
            />
          </div>

          <div class="filter-group">
            <label>Lines</label>
            <input
              v-model.number="maxLines"
              type="number"
              min="10"
              max="1000"
              class="filter-input"
              style="width: 100px"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- Logs List -->
    <div class="logs-card card">
      <div class="card-body">
        <div v-if="loading && logs.length === 0" class="loading-state">
          <i class="fas fa-spinner fa-spin"></i>
          <span>Loading logs...</span>
        </div>

        <div v-else-if="filteredLogs.length === 0" class="empty-state">
          <i class="fas fa-file-alt"></i>
          <h3>No Logs</h3>
          <p>{{ logs.length === 0 ? 'No logs available' : 'No logs match your filters' }}</p>
        </div>

        <div v-else class="logs-container">
          <div
            v-for="(log, index) in filteredLogs"
            :key="index"
            :class="['log-entry', `log-${log.level.toLowerCase()}`]"
          >
            <div class="log-timestamp">{{ formatTimestamp(log.timestamp) }}</div>
            <div :class="['log-level', `level-${log.level.toLowerCase()}`]">
              {{ log.level }}
            </div>
            <div class="log-message">{{ log.message }}</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import type { LogEntry, LogLevel } from '@shared/types';

const logs = ref<LogEntry[]>([]);
const filterLevel = ref<LogLevel | ''>('');
const filterSearch = ref('');
const maxLines = ref(100);
const loading = ref(false);
const pollInterval = ref<NodeJS.Timeout | null>(null);

const filteredLogs = computed(() => {
  let filtered = logs.value;

  if (filterLevel.value) {
    filtered = filtered.filter(log => log.level === filterLevel.value);
  }

  if (filterSearch.value) {
    const search = filterSearch.value.toLowerCase();
    filtered = filtered.filter(log => 
      log.message.toLowerCase().includes(search) ||
      log.level.toLowerCase().includes(search)
    );
  }

  return filtered.slice(-maxLines.value);
});

async function refreshLogs(): Promise<void> {
  loading.value = true;
  try {
    const vectorizerLogs = await window.electron.vectorizer.getLogs();
    logs.value = vectorizerLogs.map(log => ({
      timestamp: log.timestamp,
      level: log.level as LogLevel,
      message: log.message,
      source: 'vectorizer'
    }));
  } catch (error) {
    console.error('Failed to load logs:', error);
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
  const content = logs.value.map(log => 
    `[${log.timestamp}] [${log.level}] ${log.message}`
  ).join('\n');

  const blob = new Blob([content], { type: 'text/plain' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `vectorizer-logs-${new Date().toISOString()}.txt`;
  a.click();
  URL.revokeObjectURL(url);
}

function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp);
  return date.toLocaleString();
}

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

<style scoped>
.logs-viewer {
  padding: 2rem;
}

.filters-card {
  margin-bottom: 1.5rem;
}

.filters-row {
  display: flex;
  gap: 1rem;
  align-items: flex-end;
}

.filter-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.filter-select,
.filter-input {
  padding: 0.5rem;
  border: 1px solid #d1d5db;
  border-radius: 4px;
  font-size: 0.875rem;
}

.logs-container {
  font-family: 'Courier New', monospace;
  font-size: 0.875rem;
  max-height: 600px;
  overflow-y: auto;
  background: #1a1a2e;
  border-radius: 4px;
  padding: 1rem;
}

.log-entry {
  display: grid;
  grid-template-columns: 200px 80px 1fr;
  gap: 1rem;
  padding: 0.5rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.log-timestamp {
  color: #9ca3af;
}

.log-level {
  font-weight: 700;
  text-transform: uppercase;
}

.level-debug {
  color: #9ca3af;
}

.level-info {
  color: #60a5fa;
}

.level-warn {
  color: #fbbf24;
}

.level-error {
  color: #f87171;
}

.log-message {
  color: #e5e7eb;
  word-break: break-word;
}

.loading-state,
.empty-state {
  text-align: center;
  padding: 3rem;
  color: #6b7280;
}
</style>

