<template>
  <div class="p-8">
    <!-- Collection Header -->
    <div class="bg-bg-secondary border border-border rounded-xl p-6 mb-6">
      <div class="flex items-center justify-between mb-4">
        <div>
          <h2 class="text-2xl font-semibold text-text-primary mb-2">{{ collectionName }}</h2>
          <p class="text-sm text-text-secondary">{{ formatNumber(collectionInfo?.vector_count || 0) }} vectors • Dimension: {{ collectionInfo?.dimension || 'N/A' }}</p>
        </div>
        <div class="flex gap-2">
          <button @click="showInsertModal = true" class="px-4 py-2 text-sm font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors">
            <i class="fas fa-plus mr-2"></i> Insert Data
          </button>
          <button @click="deleteCollection" class="px-4 py-2 text-sm font-medium bg-error/20 text-error border border-error rounded hover:bg-error/30 transition-colors">
            <i class="fas fa-trash mr-2"></i> Delete
          </button>
        </div>
      </div>
    </div>

    <!-- Search Section -->
    <div class="bg-bg-secondary border border-border rounded-xl p-6 mb-6">
      <h3 class="text-lg font-semibold text-text-primary mb-4">Search Vectors</h3>
      
      <div class="space-y-4">
        <div class="flex gap-3">
          <select v-model="searchType" class="px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary focus:outline-none focus:border-border-light transition-colors text-sm min-w-[180px]">
            <option value="search_vectors">Basic Search</option>
            <option value="semantic_search">Semantic Search</option>
            <option value="intelligent_search">Intelligent Search</option>
            <option value="discover">Discover</option>
          </select>
          
          <input
            v-model="searchQuery"
            type="text"
            class="flex-1 px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm"
            placeholder="Enter your search query..."
            @keyup.enter="performSearch"
          />
          
          <input
            v-model.number="searchLimit"
            type="number"
            min="1"
            max="100"
            class="w-20 px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary focus:outline-none focus:border-border-light transition-colors text-sm"
            title="Results limit"
          />
          
          <button 
            @click="performSearch" 
            :disabled="searching || !searchQuery.trim()" 
            class="px-4 py-2 text-sm font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <i :class="['fas', searching ? 'fa-spinner fa-spin' : 'fa-search', 'mr-2']"></i>
            Search
          </button>
        </div>

        <!-- Loading State -->
        <div v-if="loadingVectors" class="flex flex-col items-center justify-center py-12 text-text-secondary">
          <i class="fas fa-spinner fa-spin text-4xl mb-4"></i>
          <p class="text-sm">Loading vectors...</p>
        </div>

        <!-- Search Results -->
        <div v-else-if="searchResults.length > 0" class="mt-6">
          <div class="flex items-center justify-between mb-4">
            <h4 class="text-sm font-semibold text-text-primary">{{ searchQuery ? 'Search Results' : 'Recent Vectors' }} ({{ searchResults.length }})</h4>
            <span v-if="searchTime > 0" class="text-xs text-text-muted">{{ searchTime }}ms</span>
          </div>

          <div class="space-y-3">
            <div v-for="result in searchResults" :key="result.id" class="bg-bg-tertiary border border-border rounded-lg p-4">
              <div class="flex items-start justify-between mb-3">
                <div class="flex items-center gap-3 flex-1">
                  <!-- Score badge (only show if score exists) -->
                  <span v-if="result.score !== undefined" class="px-3 py-1 text-xs font-semibold bg-success/20 text-success rounded-full">
                    {{ (result.score * 100).toFixed(1) }}%
                  </span>
                  <!-- Vector ID badge for list view -->
                  <span v-else class="px-3 py-1 text-xs font-semibold bg-bg-primary text-text-secondary rounded-full">
                    Vector
                  </span>
                  <div class="flex-1 min-w-0">
                    <div class="text-sm font-medium text-text-primary truncate">
                      {{ getVectorTitle(result) }}
                    </div>
                    <div class="text-xs text-text-secondary">
                      {{ getVectorSubtitle(result) }}
                    </div>
                  </div>
                </div>
                <div class="flex gap-1">
                  <button @click="viewVectorDetails(result)" class="p-2 text-text-secondary hover:text-text-primary hover:bg-bg-hover rounded transition-colors">
                    <i class="fas fa-eye text-xs"></i>
                  </button>
                  <button @click="deleteVector(result.id)" class="p-2 text-error hover:bg-error/20 rounded transition-colors">
                    <i class="fas fa-trash text-xs"></i>
                  </button>
                </div>
              </div>
              
              <!-- Content preview -->
              <div class="text-sm text-text-primary leading-relaxed pl-16">
                <MonacoEditor
                  :value="getContentPreview(result)"
                  :language="getContentLanguage(result)"
                  height="120px"
                />
              </div>
            </div>
          </div>
        </div>

        <div v-else-if="searchPerformed" class="flex flex-col items-center justify-center py-12 text-text-secondary">
          <i class="fas fa-search text-4xl mb-4"></i>
          <p class="text-sm">No results found</p>
        </div>
      </div>
    </div>

    <!-- Insert Modal -->
    <div v-if="showInsertModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-modal" @click.self="showInsertModal = false">
      <div class="bg-bg-secondary border border-border rounded-xl w-full max-w-2xl mx-4 shadow-xl max-h-[80vh] overflow-auto">
        <div class="flex items-center justify-between p-6 border-b border-border">
          <h2 class="text-lg font-semibold text-text-primary">Insert Data</h2>
          <button @click="showInsertModal = false" class="p-2 text-text-secondary hover:text-text-primary hover:bg-bg-hover rounded transition-colors">
            <i class="fas fa-times"></i>
          </button>
        </div>
        
        <div class="p-6 space-y-4">
          <div>
            <label class="block text-sm font-medium text-text-primary mb-3">Insert Type</label>
            <div class="flex gap-4">
              <label class="flex items-center gap-2 cursor-pointer">
                <input v-model="insertType" type="radio" value="text" class="cursor-pointer" />
                <span class="text-sm text-text-primary">Text</span>
              </label>
              <label class="flex items-center gap-2 cursor-pointer">
                <input v-model="insertType" type="radio" value="files" class="cursor-pointer" />
                <span class="text-sm text-text-primary">Files</span>
              </label>
            </div>
          </div>

          <div v-if="insertType === 'text'">
            <label class="block text-sm font-medium text-text-primary mb-2">Text Content</label>
            <textarea
              v-model="insertText"
              class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors resize-none"
              rows="10"
              placeholder="Enter text content to index..."
            ></textarea>
          </div>

          <div v-else>
            <button @click="selectFilesToInsert" class="px-4 py-2 text-sm font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors">
              <i class="fas fa-file-upload mr-2"></i>
              Select Files
            </button>
            <div v-if="selectedFiles.length > 0" class="mt-4 p-4 bg-bg-tertiary border border-border rounded">
              <p class="text-sm text-text-primary mb-2">{{ selectedFiles.length }} file(s) selected</p>
              <ul class="max-h-48 overflow-y-auto space-y-1">
                <li v-for="(file, index) in selectedFiles" :key="index" class="text-xs text-text-secondary">{{ file }}</li>
              </ul>
            </div>
          </div>
        </div>
        
        <div class="flex items-center justify-end gap-2 p-6 border-t border-border">
          <button @click="showInsertModal = false" class="px-4 py-2 text-sm font-medium bg-transparent text-text-secondary border border-border rounded hover:bg-bg-hover hover:text-text-primary transition-colors">Cancel</button>
          <button @click="insertData" :disabled="inserting" class="px-4 py-2 text-sm font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
            <i :class="['fas', inserting ? 'fa-spinner fa-spin' : 'fa-upload', 'mr-2']"></i>
            {{ inserting ? 'Inserting...' : 'Insert' }}
          </button>
        </div>
      </div>
    </div>
  </div>

  <!-- Vector Details Modal -->
  <div v-if="selectedVector" class="fixed inset-0 bg-black/50 flex items-center justify-center z-modal" @click.self="selectedVector = null">
    <div class="bg-bg-secondary border border-border rounded-xl w-full max-w-4xl mx-4 shadow-xl max-h-[80vh] overflow-auto">
      <div class="flex items-center justify-between p-6 border-b border-border">
        <h2 class="text-lg font-semibold text-text-primary">Vector Details</h2>
        <button @click="selectedVector = null" class="p-2 text-text-secondary hover:text-text-primary hover:bg-bg-hover rounded transition-colors">
          <i class="fas fa-times"></i>
        </button>
      </div>
      
      <div class="p-6 space-y-6">
        <!-- Vector Info -->
        <div>
          <h3 class="text-sm font-semibold text-text-primary mb-3">Vector Information</h3>
          <div class="bg-bg-primary border border-border rounded p-4 space-y-2">
            <div class="flex justify-between">
              <span class="text-sm text-text-secondary">ID:</span>
              <span class="text-sm font-mono text-text-primary">{{ selectedVector.id }}</span>
            </div>
            <div v-if="selectedVector.score !== undefined" class="flex justify-between">
              <span class="text-sm text-text-secondary">Score:</span>
              <span class="text-sm text-text-primary">{{ (selectedVector.score * 100).toFixed(2) }}%</span>
            </div>
            <div v-if="selectedVector.vector" class="flex justify-between">
              <span class="text-sm text-text-secondary">Dimensions:</span>
              <span class="text-sm text-text-primary">{{ selectedVector.vector.length }}</span>
            </div>
          </div>
        </div>

        <!-- Payload -->
        <div v-if="selectedVector.payload">
          <h3 class="text-sm font-semibold text-text-primary mb-3">Payload</h3>
          <MonacoEditor
            :value="JSON.stringify(selectedVector.payload, null, 2)"
            language="json"
            height="200px"
          />
        </div>

        <!-- Content -->
        <div v-if="selectedVector.payload?.content">
          <h3 class="text-sm font-semibold text-text-primary mb-3">Content</h3>
          <MonacoEditor
            :value="selectedVector.payload.content"
            :language="getContentLanguage(selectedVector)"
            height="300px"
          />
        </div>

        <!-- Vector Data (truncated) -->
        <div v-if="selectedVector.vector">
          <h3 class="text-sm font-semibold text-text-primary mb-3">Vector Data (first 10 dimensions)</h3>
          <div class="bg-bg-primary border border-border rounded p-4">
            <div class="text-xs text-text-primary font-mono">
              [{{ selectedVector.vector.slice(0, 10).map((v: number) => v.toFixed(6)).join(', ') }}{{ selectedVector.vector.length > 10 ? ', ...' : '' }}]
            </div>
            <div class="text-xs text-text-secondary mt-2">
              Total dimensions: {{ selectedVector.vector.length }}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { useRouter } from 'vue-router';
import { storeToRefs } from 'pinia';
import { useVectorizerStore } from '../stores/vectorizer';
import { useDialog } from '../composables/useDialog';
import MonacoEditor from '../components/MonacoEditor.vue';
import type { SearchResult, SearchType } from '@shared/types';

interface Props {
  name: string;
}

const props = defineProps<Props>();
const router = useRouter();
const vectorizerStore = useVectorizerStore();
const dialog = useDialog();

const { collections, isConnected } = storeToRefs(vectorizerStore);

const collectionName = computed(() => props.name);
const collectionInfo = computed(() => 
  collections.value.find((c: any) => c.name === collectionName.value)
);

const searchQuery = ref('');
const searchType = ref<SearchType>('search_vectors');
const searchLimit = ref(10);
const searching = ref(false);
const searchResults = ref<SearchResult[]>([]);
const searchPerformed = ref(false);
const searchTime = ref(0);

const showInsertModal = ref(false);
const insertType = ref<'text' | 'files'>('text');
const insertText = ref('');
const selectedFiles = ref<string[]>([]);
const inserting = ref(false);

// Pre-load initial vectors
const loadingVectors = ref(false);

// Vector details modal
const selectedVector = ref<SearchResult | null>(null);

async function performSearch(): Promise<void> {
  if (!searchQuery.value.trim()) return;

  searching.value = true;
  searchPerformed.value = true;
  const startTime = Date.now();

  try {
    searchResults.value = await vectorizerStore.search(
      collectionName.value,
      searchQuery.value,
      searchLimit.value
    );
    searchTime.value = Date.now() - startTime;
  } catch (error) {
    console.error('Search error:', error);
    await dialog.alert(
      `Search failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
      'Search Error'
    );
    searchResults.value = [];
  } finally {
    searching.value = false;
  }
}

async function selectFilesToInsert(): Promise<void> {
  if (!window.electron) {
    await dialog.alert(
      'This feature requires Electron. Please run the app in Electron mode.',
      'Feature Unavailable'
    );
    return;
  }
  
  const files = await window.electron.selectFiles();
  selectedFiles.value = files;
}

async function insertData(): Promise<void> {
  inserting.value = true;
  try {
    if (insertType.value === 'text' && insertText.value.trim()) {
      await vectorizerStore.insertText(collectionName.value, insertText.value);
      await dialog.alert('Text inserted successfully!', 'Success');
      insertText.value = '';
    } else if (insertType.value === 'files' && selectedFiles.value.length > 0) {
      await dialog.alert('File insertion not yet implemented.', 'Not Implemented');
    }
    
    showInsertModal.value = false;
    await vectorizerStore.loadCollections();
  } catch (error) {
    await dialog.alert(
      `Insert failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
      'Insert Error'
    );
  } finally {
    inserting.value = false;
  }
}

function viewVectorDetails(vector: SearchResult): void {
  selectedVector.value = vector;
}

async function deleteVector(vectorId: string): Promise<void> {
  const confirmed = await dialog.confirm(
    'Are you sure you want to delete this vector?',
    'Delete Vector'
  );
  
  if (confirmed) {
    // TODO: Implement delete vector API call
    console.log('Delete vector:', vectorId);
    await dialog.alert('Vector deletion not yet implemented.', 'Not Implemented');
  }
}

async function deleteCollection(): Promise<void> {
  const confirmed = await dialog.confirm(
    `Are you sure you want to delete collection "${collectionName.value}"? This action cannot be undone.`,
    'Delete Collection'
  );
  
  if (confirmed) {
    try {
      await vectorizerStore.deleteCollection(collectionName.value);
      await dialog.alert('Collection deleted successfully!', 'Success');
      router.push('/');
    } catch (error) {
      await dialog.alert(
        `Failed to delete collection: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'Error'
      );
    }
  }
}

async function loadInitialVectors(): Promise<void> {
  loadingVectors.value = true;
  try {
    const vectors = await vectorizerStore.listVectors(collectionName.value, 50, 0);
    searchResults.value = vectors;
    searchPerformed.value = true;
  } catch (error) {
    console.error('Failed to load initial vectors:', error);
  } finally {
    loadingVectors.value = false;
  }
}

function formatNumber(num: number): string {
  return new Intl.NumberFormat().format(num);
}

function getVectorTitle(result: SearchResult): string {
  if (result.payload?.file_path) {
    const fileName = result.payload.file_path.split('/').pop() || result.payload.file_path;
    return fileName;
  }
  return result.id.substring(0, 8) + '...';
}

function getVectorSubtitle(result: SearchResult): string {
  if (result.payload?.chunk_index !== undefined) {
    return `Chunk ${result.payload.chunk_index}`;
  }
  if (result.payload?.file_path) {
    return result.payload.file_path;
  }
  return `ID: ${result.id}`;
}

function getContentPreview(result: SearchResult): string {
  // Try to get content from payload
  if (result.payload?.content) {
    const content = result.payload.content;
    // Truncate long content
    if (content.length > 200) {
      return content.substring(0, 200) + '...';
    }
    return content;
  }
  
  // If no content, show payload structure
  if (result.payload) {
    const payloadStr = JSON.stringify(result.payload, null, 2);
    if (payloadStr.length > 200) {
      return payloadStr.substring(0, 200) + '...';
    }
    return payloadStr;
  }
  
  // Fallback to vector ID
  return `Vector ID: ${result.id}`;
}

function getContentLanguage(result: SearchResult): string {
  // Determine language based on file extension
  if (result.payload?.file_extension) {
    const ext = result.payload.file_extension.toLowerCase();
    switch (ext) {
      case '.js':
      case '.jsx':
        return 'javascript';
      case '.ts':
      case '.tsx':
        return 'typescript';
      case '.py':
        return 'python';
      case '.rs':
        return 'rust';
      case '.go':
        return 'go';
      case '.java':
        return 'java';
      case '.cpp':
      case '.c':
        return 'cpp';
      case '.css':
        return 'css';
      case '.html':
        return 'html';
      case '.json':
        return 'json';
      case '.yaml':
      case '.yml':
        return 'yaml';
      case '.md':
        return 'markdown';
      case '.xml':
        return 'xml';
      case '.sql':
        return 'sql';
      default:
        return 'text';
    }
  }
  
  // Try to detect language from content
  if (result.payload?.content) {
    const content = result.payload.content;
    if (content.includes('function') || content.includes('const') || content.includes('let')) {
      return 'javascript';
    }
    if (content.includes('import') && content.includes('from')) {
      return 'typescript';
    }
    if (content.includes('def ') || content.includes('import ')) {
      return 'python';
    }
    if (content.includes('fn ') || content.includes('struct ')) {
      return 'rust';
    }
    if (content.includes('package ') || content.includes('public class')) {
      return 'java';
    }
  }
  
  return 'text';
}

// Watch for collection name changes
watch(() => props.name, async (newName: string, oldName: string) => {
  if (newName !== oldName) {
    // Clear previous results
    searchResults.value = [];
    searchPerformed.value = false;
    searchQuery.value = '';
    searchTime.value = 0;
    
    // Load new collection vectors
    await loadInitialVectors();
  }
});

onMounted(async () => {
  if (!isConnected.value) {
    router.push('/connections');
    return;
  }
  
  // Pre-load 50 vectors
  await loadInitialVectors();
});
</script>
