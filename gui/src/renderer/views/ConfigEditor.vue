<template>
  <div class="config-editor">
    <div class="page-header">
      <h1><i class="fas fa-cog"></i> Configuration</h1>
      <div class="header-actions">
        <button @click="loadConfig" :disabled="loading" class="btn btn-secondary">
          <i :class="['fas', loading ? 'fa-spinner fa-spin' : 'fa-sync']"></i>
          Reload
        </button>
        <button @click="saveAndRestart" :disabled="saving" class="btn btn-primary">
          <i :class="['fas', saving ? 'fa-spinner fa-spin' : 'fa-save']"></i>
          Save & Restart
        </button>
      </div>
    </div>

    <div class="config-tabs">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        :class="['tab-button', { active: activeTab === tab.id }]"
        @click="activeTab = tab.id"
      >
        <i :class="tab.icon"></i>
        {{ tab.label }}
      </button>
    </div>

    <div class="config-content card">
      <div class="card-body">
        <!-- General Tab -->
        <div v-if="activeTab === 'general'" class="config-section">
          <h3>Server Settings</h3>
          <div class="form-group">
            <label for="cfg-host">Host</label>
            <input
              id="cfg-host"
              v-model="config.server.host"
              type="text"
              class="form-control"
              @input="markDirty"
            />
          </div>

          <div class="form-group">
            <label for="cfg-port">Port</label>
            <input
              id="cfg-port"
              v-model.number="config.server.port"
              type="number"
              class="form-control"
              @input="markDirty"
            />
          </div>

          <h3>Authentication</h3>
          <div class="form-group">
            <label class="checkbox-label">
              <input v-model="config.server.auth.enabled" type="checkbox" @change="markDirty" />
              <span>Enable Authentication</span>
            </label>
          </div>

          <div v-if="config.server.auth.enabled" class="form-group">
            <label for="cfg-token">API Token</label>
            <input
              id="cfg-token"
              v-model="config.server.auth.token"
              type="password"
              class="form-control"
              @input="markDirty"
            />
          </div>
        </div>

        <!-- Storage Tab -->
        <div v-if="activeTab === 'storage'" class="config-section">
          <h3>Storage Settings</h3>
          <div class="form-group">
            <label for="cfg-datadir">Data Directory</label>
            <div class="input-group">
              <input
                id="cfg-datadir"
                v-model="config.storage.data_dir"
                type="text"
                class="form-control"
                @input="markDirty"
              />
              <button @click="selectDataDirectory" class="btn btn-secondary">
                <i class="fas fa-folder-open"></i> Browse
              </button>
            </div>
          </div>

          <div class="form-group">
            <label for="cfg-cache">Cache Size (MB)</label>
            <input
              id="cfg-cache"
              v-model.number="config.storage.cache_size"
              type="number"
              class="form-control"
              @input="markDirty"
            />
          </div>
        </div>

        <!-- Embedding Tab -->
        <div v-if="activeTab === 'embedding'" class="config-section">
          <h3>Embedding Settings</h3>
          <div class="form-group">
            <label for="cfg-provider">Provider</label>
            <select id="cfg-provider" v-model="config.embedding.provider" class="form-control" @change="markDirty">
              <option value="fastembed">FastEmbed (Local)</option>
              <option value="openai">OpenAI</option>
              <option value="ollama">Ollama</option>
              <option value="custom">Custom</option>
            </select>
          </div>

          <div class="form-group">
            <label for="cfg-model">Model</label>
            <input
              id="cfg-model"
              v-model="config.embedding.model"
              type="text"
              class="form-control"
              @input="markDirty"
            />
          </div>

          <div class="form-group">
            <label for="cfg-dimension">Dimension</label>
            <select id="cfg-dimension" v-model.number="config.embedding.dimension" class="form-control" @change="markDirty">
              <option :value="384">384</option>
              <option :value="512">512</option>
              <option :value="768">768</option>
              <option :value="1536">1536</option>
            </select>
          </div>
        </div>

        <!-- Performance Tab -->
        <div v-if="activeTab === 'performance'" class="config-section">
          <h3>Performance Settings</h3>
          <div class="form-group">
            <label for="cfg-threads">Worker Threads</label>
            <input
              id="cfg-threads"
              v-model.number="config.performance.threads"
              type="number"
              class="form-control"
              @input="markDirty"
            />
          </div>

          <div class="form-group">
            <label for="cfg-batch">Batch Size</label>
            <input
              id="cfg-batch"
              v-model.number="config.performance.batch_size"
              type="number"
              class="form-control"
              @input="markDirty"
            />
          </div>
        </div>

        <!-- YAML Tab -->
        <div v-if="activeTab === 'yaml'" class="config-section">
          <h3>YAML Configuration</h3>
          <p class="config-help">Edit the raw YAML configuration file. Be careful with syntax!</p>
          <textarea
            v-model="yamlContent"
            class="yaml-editor"
            @input="markDirty"
          ></textarea>
        </div>
      </div>
    </div>

    <!-- Dirty Indicator -->
    <div v-if="isDirty" class="dirty-indicator">
      <i class="fas fa-exclamation-circle"></i>
      Unsaved changes
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue';
import * as yaml from 'js-yaml';
import type { VectorizerConfig } from '@shared/types';

const activeTab = ref('general');
const loading = ref(false);
const saving = ref(false);
const isDirty = ref(false);
const yamlContent = ref('');

const tabs = [
  { id: 'general', label: 'General', icon: 'fas fa-server' },
  { id: 'storage', label: 'Storage', icon: 'fas fa-database' },
  { id: 'embedding', label: 'Embedding', icon: 'fas fa-brain' },
  { id: 'performance', label: 'Performance', icon: 'fas fa-tachometer-alt' },
  { id: 'yaml', label: 'YAML', icon: 'fas fa-code' }
];

const config = reactive<VectorizerConfig>({
  server: {
    host: '0.0.0.0',
    port: 15002,
    auth: {
      enabled: false,
      token: ''
    }
  },
  storage: {
    data_dir: './data',
    cache_size: 1024
  },
  embedding: {
    provider: 'fastembed',
    model: 'BAAI/bge-small-en-v1.5',
    dimension: 384
  },
  performance: {
    threads: 4,
    batch_size: 100
  }
});

function markDirty(): void {
  isDirty.value = true;
}

async function loadConfig(): Promise<void> {
  loading.value = true;
  try {
    // TODO: Load config from API
    // const response = await api.getConfig();
    // Object.assign(config, response.data);
    
    // Update YAML view
    yamlContent.value = yaml.dump(config);
    isDirty.value = false;
  } catch (error) {
    alert(`Failed to load config: ${error instanceof Error ? error.message : 'Unknown error'}`);
  } finally {
    loading.value = false;
  }
}

async function selectDataDirectory(): Promise<void> {
  const path = await window.electron.selectDirectory();
  if (path) {
    config.storage.data_dir = path;
    markDirty();
  }
}

async function saveAndRestart(): Promise<void> {
  if (!confirm('This will save the configuration and restart the vectorizer. Continue?')) {
    return;
  }

  saving.value = true;
  try {
    // Parse YAML if on YAML tab
    if (activeTab.value === 'yaml') {
      try {
        const parsed = yaml.load(yamlContent.value) as VectorizerConfig;
        Object.assign(config, parsed);
      } catch (error) {
        alert('Invalid YAML syntax');
        return;
      }
    }

    // TODO: Save config via API
    // await api.updateConfig(config);
    // await api.restartServer();

    isDirty.value = false;
    alert('Configuration saved successfully. Vectorizer is restarting...');
  } catch (error) {
    alert(`Failed to save config: ${error instanceof Error ? error.message : 'Unknown error'}`);
  } finally {
    saving.value = false;
  }
}

onMounted(() => {
  loadConfig();
});
</script>

<style scoped>
.config-editor {
  padding: 2rem;
}

.config-tabs {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 1.5rem;
  border-bottom: 2px solid #e5e7eb;
}

.tab-button {
  padding: 0.75rem 1.5rem;
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  color: #6b7280;
  font-weight: 500;
  transition: all 0.2s;
  margin-bottom: -2px;
}

.tab-button:hover {
  color: #1a1a2e;
}

.tab-button.active {
  color: #3b82f6;
  border-bottom-color: #3b82f6;
}

.config-section {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.config-section h3 {
  margin: 0;
  padding-bottom: 0.75rem;
  border-bottom: 1px solid #e5e7eb;
}

.yaml-editor {
  width: 100%;
  min-height: 400px;
  font-family: 'Courier New', monospace;
  font-size: 0.875rem;
  padding: 1rem;
  border: 1px solid #d1d5db;
  border-radius: 4px;
  resize: vertical;
}

.config-help {
  margin-bottom: 1rem;
  color: #f59e0b;
  font-size: 0.875rem;
}

.dirty-indicator {
  position: fixed;
  bottom: 2rem;
  right: 2rem;
  background: #f59e0b;
  color: white;
  padding: 0.75rem 1.5rem;
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-weight: 500;
}
</style>

