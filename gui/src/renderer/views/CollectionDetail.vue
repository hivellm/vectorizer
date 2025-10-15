<template>
  <div class="collection-detail">
    <div class="page-header">
      <div class="header-content">
        <h1><i class="fas fa-database"></i> {{ collectionName }}</h1>
        <p>{{ collectionInfo?.vector_count || 0 }} vectors</p>
      </div>
      <div class="header-actions">
        <button @click="showInsertModal = true" class="btn btn-primary">
          <i class="fas fa-plus"></i> Insert Data
        </button>
        <button @click="deleteCollection" class="btn btn-danger">
          <i class="fas fa-trash"></i> Delete Collection
        </button>
      </div>
    </div>

    <!-- Search Section -->
    <div class="search-section card">
      <div class="card-header">
        <h2><i class="fas fa-search"></i> Search</h2>
      </div>
      <div class="card-body">
        <div class="search-controls">
          <div class="search-input-group">
            <select v-model="searchType" class="search-type-select">
              <option value="search_vectors">Basic Search</option>
              <option value="semantic_search">Semantic Search</option>
              <option value="intelligent_search">Intelligent Search</option>
              <option value="discover">Discover</option>
            </select>
            <input
              v-model="searchQuery"
              type="text"
              class="search-input"
              placeholder="Enter your search query..."
              @keyup.enter="performSearch"
            />
            <button @click="performSearch" :disabled="searching || !searchQuery.trim()" class="btn btn-primary">
              <i :class="['fas', searching ? 'fa-spinner fa-spin' : 'fa-search']"></i>
              Search
            </button>
          </div>

          <div class="search-options">
            <label class="option-label">
              Results:
              <input v-model.number="searchLimit" type="number" min="1" max="100" class="limit-input" />
            </label>
          </div>
        </div>

        <!-- Search Results -->
        <div v-if="searchResults.length > 0" class="search-results">
          <div class="results-header">
            <h3>Results ({{ searchResults.length }})</h3>
            <span class="search-time">{{ searchTime }}ms</span>
          </div>

          <div class="results-list">
            <div v-for="result in searchResults" :key="result.id" class="result-item">
              <div class="result-header">
                <div class="result-score">
                  <span class="score-badge">{{ (result.score! * 100).toFixed(1) }}%</span>
                </div>
                <div class="result-meta">
                  <span class="file-path">{{ result.payload?.file_path || 'Unknown' }}</span>
                  <span class="chunk-info">Chunk {{ result.payload?.chunk_index || 0 }}</span>
                </div>
              </div>
              <div class="result-content">
                <p>{{ result.payload?.content || JSON.stringify(result.payload) }}</p>
              </div>
              <div class="result-actions">
                <button @click="viewVectorDetails(result)" class="btn-icon btn-xs">
                  <i class="fas fa-eye"></i>
                </button>
                <button @click="deleteVector(result.id)" class="btn-icon btn-xs">
                  <i class="fas fa-trash"></i>
                </button>
              </div>
            </div>
          </div>
        </div>

        <div v-else-if="searchPerformed" class="no-results">
          <i class="fas fa-search"></i>
          <p>No results found</p>
        </div>
      </div>
    </div>

    <!-- Insert Modal -->
    <div v-if="showInsertModal" class="modal-overlay" @click.self="showInsertModal = false">
      <div class="modal">
        <div class="modal-header">
          <h2>Insert Data</h2>
          <button @click="showInsertModal = false" class="btn-icon">
            <i class="fas fa-times"></i>
          </button>
        </div>
        <div class="modal-body">
          <div class="form-group">
            <label>Insert Type</label>
            <div class="radio-group">
              <label class="radio-label">
                <input v-model="insertType" type="radio" value="text" />
                <span>Text</span>
              </label>
              <label class="radio-label">
                <input v-model="insertType" type="radio" value="files" />
                <span>Files</span>
              </label>
            </div>
          </div>

          <div v-if="insertType === 'text'" class="form-group">
            <label>Text Content</label>
            <textarea
              v-model="insertText"
              class="form-control"
              rows="10"
              placeholder="Enter text content to index..."
            ></textarea>
          </div>

          <div v-else class="form-group">
            <button @click="selectFilesToInsert" class="btn btn-secondary">
              <i class="fas fa-file-upload"></i>
              Select Files
            </button>
            <div v-if="selectedFiles.length > 0" class="selected-files">
              <p>{{ selectedFiles.length }} file(s) selected</p>
              <ul class="file-list">
                <li v-for="(file, index) in selectedFiles" :key="index">{{ file }}</li>
              </ul>
            </div>
          </div>
        </div>
        <div class="modal-footer">
          <button @click="showInsertModal = false" class="btn btn-secondary">Cancel</button>
          <button @click="insertData" :disabled="inserting" class="btn btn-primary">
            <i :class="['fas', inserting ? 'fa-spinner fa-spin' : 'fa-upload']"></i>
            {{ inserting ? 'Inserting...' : 'Insert' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { storeToRefs } from 'pinia';
import { useVectorizerStore } from '../stores/vectorizer';
import type { SearchResult, SearchType } from '@shared/types';

interface Props {
  name: string;
}

const props = defineProps<Props>();
const router = useRouter();
const vectorizerStore = useVectorizerStore();

const { collections, isConnected } = storeToRefs(vectorizerStore);

const collectionName = computed(() => props.name);
const collectionInfo = computed(() => 
  collections.value.find(c => c.name === collectionName.value)
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
    searchResults.value = [];
  } finally {
    searching.value = false;
  }
}

async function selectFilesToInsert(): Promise<void> {
  if (!window.electron) {
    alert('This feature requires Electron. Please run the app in Electron mode.');
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
      alert('Text inserted successfully');
      insertText.value = '';
    } else if (insertType.value === 'files' && selectedFiles.value.length > 0) {
      // Read files and batch insert
      // This would need Node.js file reading in the main process
      alert('File insertion not yet implemented');
    }
    
    showInsertModal.value = false;
    await vectorizerStore.loadCollections();
  } catch (error) {
    alert(`Insert failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
  } finally {
    inserting.value = false;
  }
}

function viewVectorDetails(vector: SearchResult): void {
  console.log('Vector details:', vector);
  // TODO: Show modal with full vector details
}

async function deleteVector(vectorId: string): Promise<void> {
  if (confirm('Are you sure you want to delete this vector?')) {
    // TODO: Implement delete vector
    console.log('Delete vector:', vectorId);
  }
}

async function deleteCollection(): Promise<void> {
  if (confirm(`Are you sure you want to delete collection "${collectionName.value}"?`)) {
    try {
      await vectorizerStore.deleteCollection(collectionName.value);
      router.push('/');
    } catch (error) {
      alert(`Failed to delete collection: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }
}

onMounted(() => {
  if (!isConnected.value) {
    router.push('/connections');
  }
});
</script>

<style scoped>
.collection-detail {
  padding: 2rem;
}

.search-section {
  margin-bottom: 2rem;
}

.search-controls {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.search-input-group {
  display: flex;
  gap: 0.5rem;
}

.search-type-select {
  padding: 0.5rem;
  border: 1px solid #d1d5db;
  border-radius: 4px;
  min-width: 180px;
}

.search-input {
  flex: 1;
  padding: 0.5rem;
  border: 1px solid #d1d5db;
  border-radius: 4px;
}

.search-options {
  display: flex;
  gap: 1rem;
}

.limit-input {
  width: 80px;
  padding: 0.25rem 0.5rem;
  border: 1px solid #d1d5db;
  border-radius: 4px;
}

.results-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin: 1.5rem 0 1rem;
}

.search-time {
  color: #6b7280;
  font-size: 0.875rem;
}

.results-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.result-item {
  border: 1px solid #e5e7eb;
  border-radius: 6px;
  padding: 1rem;
}

.result-header {
  display: flex;
  gap: 1rem;
  margin-bottom: 0.75rem;
}

.score-badge {
  background: #10b981;
  color: white;
  padding: 0.25rem 0.75rem;
  border-radius: 12px;
  font-size: 0.875rem;
  font-weight: 600;
}

.result-meta {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  font-size: 0.875rem;
  color: #6b7280;
}

.result-content p {
  margin: 0;
  line-height: 1.6;
}

.result-actions {
  margin-top: 0.75rem;
  display: flex;
  gap: 0.5rem;
}

.btn-icon {
  border: none;
  background: transparent;
  cursor: pointer;
  padding: 0.5rem;
  color: #6b7280;
}

.btn-icon:hover {
  color: #1a1a2e;
}

.btn-xs {
  padding: 0.25rem 0.5rem;
}

.no-results {
  text-align: center;
  padding: 2rem;
  color: #6b7280;
}

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal {
  background: white;
  border-radius: 8px;
  max-width: 600px;
  width: 100%;
  max-height: 80vh;
  overflow: auto;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1.5rem;
  border-bottom: 1px solid #e5e7eb;
}

.modal-body {
  padding: 1.5rem;
}

.modal-footer {
  padding: 1.5rem;
  border-top: 1px solid #e5e7eb;
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
}

.selected-files {
  margin-top: 1rem;
  padding: 1rem;
  background: #f9fafb;
  border-radius: 4px;
}

.file-list {
  margin-top: 0.5rem;
  max-height: 200px;
  overflow-y: auto;
}

.file-list li {
  font-size: 0.875rem;
  padding: 0.25rem 0;
  color: #6b7280;
}
</style>

