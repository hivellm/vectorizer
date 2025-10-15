<template>
  <div class="p-8">
    <!-- Projects Configuration -->
    <div class="bg-bg-secondary border border-border rounded-xl p-6">
      <div class="flex items-center justify-between mb-6">
        <div class="flex items-center gap-4 flex-1">
          <h2 class="text-xl font-semibold text-text-primary">Projects</h2>
          <div class="flex-1 max-w-md">
            <input
              v-model="searchFilter"
              type="text"
              placeholder="Search projects..."
              class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm"
            />
    </div>
        </div>
        <div class="flex gap-2">
          <button @click="addProject" class="px-4 py-2 text-sm font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors">
            <i class="fas fa-plus mr-2"></i>
            Add Project
          </button>
          <button @click="saveWorkspaceConfig" :disabled="saving" class="px-4 py-2 text-sm font-medium rounded hover:bg-bg-hover transition-colors disabled:opacity-50 disabled:cursor-not-allowed" :class="hasUnsavedChanges ? 'bg-warning/20 text-warning border border-warning' : 'bg-bg-tertiary text-text-primary border border-border'">
            <i :class="['fas', saving ? 'fa-spinner fa-spin' : 'fa-save', 'mr-2']"></i>
            {{ saving ? 'Saving...' : hasUnsavedChanges ? 'Save Changes *' : 'Save Configuration' }}
      </button>
        </div>
    </div>

      <div v-if="filteredProjects.length === 0 && workspaceConfig.projects.length === 0" class="flex flex-col items-center justify-center py-16 text-text-secondary">
        <i class="fas fa-folder-open text-4xl mb-4"></i>
        <h3 class="text-lg font-medium text-text-primary mb-2">No Projects</h3>
        <p class="text-sm">Add your first project to get started</p>
        </div>

      <div v-else-if="filteredProjects.length === 0" class="flex flex-col items-center justify-center py-16 text-text-secondary">
        <i class="fas fa-search text-4xl mb-4"></i>
        <h3 class="text-lg font-medium text-text-primary mb-2">No Results</h3>
        <p class="text-sm">No projects match your search</p>
        </div>

      <div v-else class="space-y-4">
        <div v-for="(project, projectIndex) in filteredProjects" :key="project.name" class="bg-bg-tertiary border border-border rounded-lg p-4">
          <div class="flex items-start justify-between mb-4">
            <div class="flex-1 grid grid-cols-3 gap-4">
              <div>
                <label class="block text-xs font-medium text-text-secondary mb-1">Project Name</label>
                <input
                  v-model="project.name"
                  type="text"
                  class="w-full px-3 py-2 bg-bg-primary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm"
                  placeholder="project-name"
                />
              </div>
              <div>
                <label class="block text-xs font-medium text-text-secondary mb-1">Path</label>
                <input
                  v-model="project.path"
                  type="text"
                  class="w-full px-3 py-2 bg-bg-primary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm"
                  placeholder="../project-path"
                />
              </div>
              <div>
                <label class="block text-xs font-medium text-text-secondary mb-1">Description</label>
                <input
                  v-model="project.description"
                  type="text"
                  class="w-full px-3 py-2 bg-bg-primary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm"
                  placeholder="Project description"
                />
                </div>
              </div>
            <button @click="removeProjectByName(project.name)" class="ml-4 p-2 text-error hover:bg-error/20 rounded transition-colors">
                <i class="fas fa-trash"></i>
              </button>
            </div>
            
          <!-- Collections -->
          <div class="mt-4 pt-4 border-t border-border">
            <div class="flex items-center justify-between mb-3">
              <h3 class="text-sm font-semibold text-text-primary">Collections</h3>
              <button @click="addCollection(project.name)" class="px-3 py-1.5 text-xs font-medium bg-bg-primary text-text-secondary border border-border rounded hover:bg-bg-hover hover:text-text-primary transition-colors">
                <i class="fas fa-plus mr-1"></i>
                Add Collection
              </button>
            </div>

            <div v-if="!project.collections || project.collections.length === 0" class="text-sm text-text-secondary py-2">
              No collections configured
            </div>

            <div v-else class="space-y-2">
              <div v-for="(collection, collectionIndex) in project.collections" :key="collection.name" class="bg-bg-primary border border-border rounded">
                <!-- Collection Header (Always Visible) -->
                <div class="flex items-center justify-between p-3 cursor-pointer hover:bg-bg-hover transition-colors" @click="toggleCollection(project.name, collectionIndex)">
                  <div class="flex items-center gap-2 flex-1 min-w-0">
                    <i :class="['fas fa-chevron-right text-xs text-text-muted transition-transform', { 'rotate-90': isCollectionExpanded(project.name, collectionIndex) }]"></i>
                    <span class="text-sm font-medium text-text-primary truncate">{{ collection.name || 'Unnamed Collection' }}</span>
                    <span class="text-xs text-text-secondary truncate">{{ collection.description }}</span>
                  </div>
                  <button @click.stop="removeCollection(project.name, collectionIndex)" class="p-1.5 text-error hover:bg-error/20 rounded transition-colors">
                    <i class="fas fa-trash text-xs"></i>
                  </button>
            </div>

                <!-- Collection Details (Collapsed by default) -->
                <div v-if="isCollectionExpanded(project.name, collectionIndex)" class="p-3 pt-0 space-y-3">
                  <div class="grid grid-cols-2 gap-3">
                    <div>
                      <label class="block text-xs font-medium text-text-secondary mb-1">Collection Name</label>
                      <input
                        v-model="collection.name"
                        type="text"
                        @click.stop
                        class="w-full px-2 py-1.5 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-xs"
                        placeholder="collection-name"
                      />
                    </div>
                    <div>
                      <label class="block text-xs font-medium text-text-secondary mb-1">Description</label>
            <input
                        v-model="collection.description"
              type="text"
                        @click.stop
                        class="w-full px-2 py-1.5 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-xs"
                        placeholder="Collection description"
            />
                    </div>
              </div>

                  <div class="grid grid-cols-2 gap-3">
                    <div>
                      <label class="block text-xs font-medium text-text-secondary mb-1">Include Patterns</label>
                      <textarea
                        v-model="collection.include_patterns_str"
                        @input="updateIncludePatterns(project.name, collectionIndex)"
                        @click.stop
                        rows="3"
                        class="w-full px-2 py-1.5 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-xs font-mono"
                        placeholder="**/*.md&#10;**/*.ts&#10;src/**/*"
                      ></textarea>
              </div>
                    <div>
                      <label class="block text-xs font-medium text-text-secondary mb-1">Exclude Patterns</label>
                      <textarea
                        v-model="collection.exclude_patterns_str"
                        @input="updateExcludePatterns(project.name, collectionIndex)"
                        @click.stop
                        rows="3"
                        class="w-full px-2 py-1.5 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-xs font-mono"
                        placeholder="node_modules/**&#10;dist/**&#10;**/*.log"
                      ></textarea>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useVectorizerStore } from '../stores/vectorizer';
import { useDialog } from '../composables/useDialog';

interface Collection {
  name: string;
  description: string;
  include_patterns: string[];
  exclude_patterns: string[];
  include_patterns_str?: string;
  exclude_patterns_str?: string;
}

interface Project {
  name: string;
  path: string;
  description: string;
  collections: Collection[];
}

interface WorkspaceConfig {
  projects: Project[];
}

const vectorizerStore = useVectorizerStore();
const dialog = useDialog();
const saving = ref(false);
const searchFilter = ref('');
const hasUnsavedChanges = ref(false);

const workspaceConfig = ref<WorkspaceConfig>({
  projects: []
});

const filteredProjects = computed(() => {
  if (!searchFilter.value.trim()) {
    return workspaceConfig.value.projects;
  }
  const search = searchFilter.value.toLowerCase();
  return workspaceConfig.value.projects.filter(p => 
    p.name.toLowerCase().includes(search) ||
    p.description.toLowerCase().includes(search) ||
    p.path.toLowerCase().includes(search)
  );
});

const expandedCollections = ref<Set<string>>(new Set());

function toggleCollection(projectName: string, collectionIndex: number): void {
  const key = `${projectName}-${collectionIndex}`;
  if (expandedCollections.value.has(key)) {
    expandedCollections.value.delete(key);
  } else {
    expandedCollections.value.add(key);
  }
}

function isCollectionExpanded(projectName: string, collectionIndex: number): boolean {
  return expandedCollections.value.has(`${projectName}-${collectionIndex}`);
}

function addProject(): void {
  const timestamp = Date.now();
  const newProject = {
    name: `new-project-${timestamp}`,
    path: `../new-project-${timestamp}`,
    description: 'New Project Description',
    collections: []
  };
  workspaceConfig.value.projects = [...workspaceConfig.value.projects, newProject];
  hasUnsavedChanges.value = true;
  
  // Set filter to show only the new project
  searchFilter.value = `new-project-${timestamp}`;
  
  // Scroll to top to see the filtered result
  setTimeout(() => {
    const container = document.querySelector('.overflow-y-auto');
    if (container) {
      container.scrollTop = 0;
    }
  }, 100);
}

async function removeProjectByName(projectName: string): Promise<void> {
  const confirmed = await dialog.confirm(
    `Are you sure you want to remove project "${projectName}"?`,
    'Remove Project'
  );
  if (confirmed) {
    const index = workspaceConfig.value.projects.findIndex(p => p.name === projectName);
    if (index !== -1) {
      workspaceConfig.value.projects.splice(index, 1);
      hasUnsavedChanges.value = true;
    }
    searchFilter.value = '';
  }
}

function addCollection(projectName: string): void {
  const project = workspaceConfig.value.projects.find(p => p.name === projectName);
  if (!project) return;
  
  if (!project.collections) {
    project.collections = [];
  }
  
  const timestamp = Date.now();
  project.collections.push({
    name: `new-collection-${timestamp}`,
    description: 'New Collection Description',
    include_patterns: [],
    exclude_patterns: [],
    include_patterns_str: '',
    exclude_patterns_str: ''
  });
  hasUnsavedChanges.value = true;
}

async function removeCollection(projectName: string, collectionIndex: number): Promise<void> {
  const project = workspaceConfig.value.projects.find(p => p.name === projectName);
  if (!project) return;
  
  const collectionName = project.collections[collectionIndex].name;
  const confirmed = await dialog.confirm(
    `Remove collection "${collectionName}"?`,
    'Remove Collection'
  );
  if (confirmed) {
    project.collections.splice(collectionIndex, 1);
    hasUnsavedChanges.value = true;
  }
}

function updateIncludePatterns(projectName: string, collectionIndex: number): void {
  const project = workspaceConfig.value.projects.find(p => p.name === projectName);
  if (!project) return;
  const collection = project.collections[collectionIndex];
  collection.include_patterns = collection.include_patterns_str?.split('\n').filter((p: string) => p.trim()) || [];
}

function updateExcludePatterns(projectName: string, collectionIndex: number): void {
  const project = workspaceConfig.value.projects.find(p => p.name === projectName);
  if (!project) return;
  const collection = project.collections[collectionIndex];
  collection.exclude_patterns = collection.exclude_patterns_str?.split('\n').filter((p: string) => p.trim()) || [];
}

async function loadWorkspaceConfig(): Promise<void> {
  try {
    const response = await fetch('/api/workspace/config');
    if (!response.ok) {
      throw new Error(`Failed to load workspace config: ${response.statusText}`);
    }
    
    const config = await response.json();
    workspaceConfig.value = config;
    
    // Convert arrays to strings for textarea editing
    if (config.projects) {
      config.projects.forEach((project: Project) => {
        project.collections?.forEach((collection: Collection) => {
          collection.include_patterns_str = collection.include_patterns?.join('\n') || '';
          collection.exclude_patterns_str = collection.exclude_patterns?.join('\n') || '';
        });
      });
    }
  } catch (error) {
    console.error('Failed to load workspace configuration:', error);
    await dialog.alert(
      `Failed to load workspace configuration: ${error instanceof Error ? error.message : 'Unknown error'}`,
      'Error'
    );
  }
}

async function saveWorkspaceConfig(): Promise<void> {
  saving.value = true;
  try {
    // Clean up temporary string fields before saving
    const configToSave = JSON.parse(JSON.stringify(workspaceConfig.value));
    configToSave.projects.forEach((project: Project) => {
      project.collections?.forEach((collection: Collection) => {
        delete collection.include_patterns_str;
        delete collection.exclude_patterns_str;
      });
    });

    const response = await fetch('/api/workspace/config', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(configToSave)
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Failed to save: ${error}`);
    }

    const result = await response.json();
    hasUnsavedChanges.value = false;
    await dialog.alert(
      result.message || 'Workspace configuration saved successfully!',
      'Success'
    );
  } catch (error) {
    console.error('Failed to save workspace configuration:', error);
    await dialog.alert(
      `Failed to save configuration: ${error instanceof Error ? error.message : 'Unknown error'}`,
      'Error'
    );
  } finally {
    saving.value = false;
  }
}

onMounted(async () => {
  console.log('WorkspaceManager mounted!');
  await loadWorkspaceConfig();
});
</script>
