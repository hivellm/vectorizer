<template>
  <div class="p-8 h-full flex flex-col">
    <!-- Header -->
    <div class="bg-bg-secondary border border-border rounded-xl p-6 mb-6">
      <div class="flex items-center justify-between mb-4">
        <div>
          <h2 class="text-2xl font-semibold text-text-primary mb-2">Graph Relationships</h2>
          <p class="text-sm text-text-secondary">Visualize relationships between vectors</p>
        </div>
        <div class="flex gap-2">
          <button
            @click="showCreateEdgeModal = true"
            :disabled="!selectedCollection || loading"
            class="px-4 py-2 text-sm font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <i class="fas fa-plus mr-2"></i>Create Edge
          </button>
          <button
            @click="showPathFinderModal = true"
            :disabled="!selectedCollection || loading"
            class="px-4 py-2 text-sm font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <i class="fas fa-route mr-2"></i>Find Path
          </button>
          <button
            @click="showDiscoveryModal = true"
            :disabled="!selectedCollection || loading"
            class="px-4 py-2 text-sm font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <i class="fas fa-search mr-2"></i>Discover Edges
          </button>
          <button
            @click="refreshGraph"
            :disabled="loading || !selectedCollection"
            class="px-4 py-2 text-sm font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <i :class="['fas', loading ? 'fa-spinner fa-spin' : 'fa-sync', 'mr-2']"></i>Refresh
          </button>
        </div>
      </div>

      <!-- Controls -->
      <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div>
          <label class="block text-sm font-medium text-text-secondary mb-1">Collection</label>
          <select
            v-model="selectedCollection"
            class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary focus:outline-none focus:border-border-light transition-colors text-sm"
          >
            <option value="">Select collection</option>
            <option v-for="col in collections" :key="col.name" :value="col.name">{{ col.name }}</option>
          </select>
        </div>
        <div>
          <label class="block text-sm font-medium text-text-secondary mb-1">Relationship Type</label>
          <select
            v-model="relationshipFilter"
            class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary focus:outline-none focus:border-border-light transition-colors text-sm"
          >
            <option value="all">All Types</option>
            <option v-for="type in relationshipTypes" :key="type" :value="type">{{ type }}</option>
          </select>
        </div>
        <div>
          <label class="block text-sm font-medium text-text-secondary mb-1">Search Node</label>
          <div class="relative">
            <i class="fas fa-search absolute left-3 top-1/2 transform -translate-y-1/2 text-text-muted text-sm"></i>
            <input
              v-model="searchQuery"
              type="text"
              class="w-full pl-10 pr-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm"
              placeholder="Search by ID or type..."
            />
          </div>
        </div>
        <div class="flex items-end gap-2">
          <button
            @click="fitGraph"
            class="px-4 py-2 text-sm font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors"
          >
            Fit
          </button>
          <button
            @click="resetView"
            class="px-4 py-2 text-sm font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors"
          >
            Reset
          </button>
        </div>
      </div>
    </div>

    <!-- Graph Visualization -->
    <div class="flex-1 bg-bg-secondary border border-border rounded-xl p-6 relative min-h-0">
      <div v-if="loading" class="absolute inset-0 flex flex-col items-center justify-center bg-bg-secondary/80 z-10 backdrop-blur-sm">
        <i class="fas fa-spinner fa-spin text-4xl text-text-secondary mb-4"></i>
        <p class="text-sm text-text-secondary">Loading graph data...</p>
      </div>

      <div v-else-if="!selectedCollection" class="flex flex-col items-center justify-center h-full text-text-secondary">
        <i class="fas fa-project-diagram text-6xl mb-4"></i>
        <p class="text-sm">Select a collection to view graph</p>
      </div>

      <div v-else-if="filteredNodes.length === 0" class="flex flex-col items-center justify-center h-full text-text-secondary">
        <i class="fas fa-project-diagram text-6xl mb-4"></i>
        <p class="text-sm">No graph data available for this collection</p>
        <button
          v-if="nodes.length === 0"
          @click="showDiscoveryModal = true"
          class="mt-4 px-4 py-2 text-sm font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors"
        >
          <i class="fas fa-search mr-2"></i>Discover Edges
        </button>
      </div>

      <div v-else class="h-full relative">
        <!-- vis-network Graph Container -->
        <div ref="networkContainer" class="w-full h-full border border-border rounded"></div>
        
        <!-- Rendering indicator -->
        <div v-if="rendering" class="absolute inset-0 flex flex-col items-center justify-center bg-bg-secondary/80 z-10 backdrop-blur-sm">
          <i class="fas fa-spinner fa-spin text-4xl text-text-secondary mb-4"></i>
          <p class="text-sm text-text-secondary">Rendering graph...</p>
        </div>

        <!-- Node Details Panel -->
        <div
          v-if="selectedNodeData"
          class="absolute top-4 right-4 bg-bg-elevated border border-border rounded-lg p-4 max-w-sm shadow-lg z-20"
        >
          <div class="flex items-center justify-between mb-3">
            <h3 class="font-semibold text-text-primary">Node Details</h3>
            <button @click="selectedNode = null" class="text-text-secondary hover:text-text-primary">
              <i class="fas fa-times"></i>
            </button>
          </div>
          <div class="space-y-2 text-sm">
            <div>
              <span class="font-medium text-text-secondary">ID:</span>
              <span class="text-text-primary ml-2">{{ selectedNodeData.id }}</span>
            </div>
            <div>
              <span class="font-medium text-text-secondary">Type:</span>
              <span class="text-text-primary ml-2">{{ selectedNodeData.node_type }}</span>
            </div>
            <div class="flex gap-2 mt-3">
              <button
                @click="showNeighborsModal = true; neighborsNodeId = selectedNodeData.id"
                class="px-3 py-1.5 text-xs font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors"
              >
                <i class="fas fa-users mr-1"></i>Neighbors
              </button>
              <button
                @click="showFindRelatedModal = true; findRelatedNodeId = selectedNodeData.id"
                class="px-3 py-1.5 text-xs font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors"
              >
                <i class="fas fa-search mr-1"></i>Find Related
              </button>
              <button
                @click="showNodeDiscoveryModal = true; discoveryNodeId = selectedNodeData.id"
                class="px-3 py-1.5 text-xs font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors"
              >
                <i class="fas fa-link mr-1"></i>Discover Edges
              </button>
            </div>
          </div>
        </div>

        <!-- Edge Details Panel -->
        <div
          v-if="selectedEdgeData"
          class="absolute top-4 right-4 bg-bg-elevated border border-border rounded-lg p-4 max-w-sm shadow-lg z-20"
        >
          <div class="flex items-center justify-between mb-3">
            <h3 class="font-semibold text-text-primary">Edge Details</h3>
            <button @click="selectedEdge = null" class="text-text-secondary hover:text-text-primary">
              <i class="fas fa-times"></i>
            </button>
          </div>
          <div class="space-y-2 text-sm">
            <div>
              <span class="font-medium text-text-secondary">ID:</span>
              <span class="text-text-primary ml-2">{{ selectedEdgeData.id }}</span>
            </div>
            <div>
              <span class="font-medium text-text-secondary">Type:</span>
              <span class="text-text-primary ml-2">{{ selectedEdgeData.relationship_type }}</span>
            </div>
            <div>
              <span class="font-medium text-text-secondary">Weight:</span>
              <span class="text-text-primary ml-2">{{ selectedEdgeData.weight?.toFixed(2) || 'N/A' }}</span>
            </div>
            <div>
              <span class="font-medium text-text-secondary">Source:</span>
              <span class="text-text-primary ml-2">{{ selectedEdgeData.source }}</span>
            </div>
            <div>
              <span class="font-medium text-text-secondary">Target:</span>
              <span class="text-text-primary ml-2">{{ selectedEdgeData.target }}</span>
            </div>
            <div class="flex gap-2 mt-3">
              <button
                @click="deleteEdgeConfirm(selectedEdgeData.id)"
                class="px-3 py-1.5 text-xs font-medium bg-error/20 text-error border border-error rounded hover:bg-error/30 transition-colors"
              >
                <i class="fas fa-trash mr-1"></i>Delete Edge
              </button>
            </div>
          </div>
        </div>

        <!-- Context Menu -->
        <div
          v-if="contextMenu.show"
          :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
          class="absolute bg-bg-elevated border border-border rounded-lg shadow-lg z-30 min-w-[200px]"
        >
          <div v-if="contextMenu.type === 'node'" class="py-1">
            <button
              @click="showNeighborsModal = true; neighborsNodeId = contextMenu.nodeId; contextMenu.show = false"
              class="w-full px-4 py-2 text-left text-sm text-text-primary hover:bg-bg-hover transition-colors"
            >
              <i class="fas fa-users mr-2"></i>View Neighbors
            </button>
            <button
              @click="showFindRelatedModal = true; findRelatedNodeId = contextMenu.nodeId; contextMenu.show = false"
              class="w-full px-4 py-2 text-left text-sm text-text-primary hover:bg-bg-hover transition-colors"
            >
              <i class="fas fa-search mr-2"></i>Find Related Nodes
            </button>
            <button
              @click="showNodeDiscoveryModal = true; discoveryNodeId = contextMenu.nodeId; contextMenu.show = false"
              class="w-full px-4 py-2 text-left text-sm text-text-primary hover:bg-bg-hover transition-colors"
            >
              <i class="fas fa-link mr-2"></i>Discover Edges for Node
            </button>
          </div>
          <div v-if="contextMenu.type === 'edge'" class="py-1">
            <button
              @click="deleteEdgeConfirm(contextMenu.edgeId); contextMenu.show = false"
              class="w-full px-4 py-2 text-left text-sm text-error hover:bg-error/20 transition-colors"
            >
              <i class="fas fa-trash mr-2"></i>Delete Edge
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Modals -->
    <!-- Create Edge Modal -->
    <Modal v-model="showCreateEdgeModal" title="Create Edge" icon="fas fa-plus" size="medium">
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-text-secondary mb-1">Source Node</label>
          <select
            v-model="createEdgeForm.source"
            class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary focus:outline-none focus:border-border-light transition-colors text-sm"
          >
            <option value="">Select source node</option>
            <option v-for="node in nodes" :key="node.id" :value="node.id">{{ node.id }}</option>
          </select>
        </div>
        <div>
          <label class="block text-sm font-medium text-text-secondary mb-1">Target Node</label>
          <select
            v-model="createEdgeForm.target"
            class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary focus:outline-none focus:border-border-light transition-colors text-sm"
          >
            <option value="">Select target node</option>
            <option v-for="node in nodes" :key="node.id" :value="node.id">{{ node.id }}</option>
          </select>
        </div>
        <div>
          <label class="block text-sm font-medium text-text-secondary mb-1">Relationship Type</label>
          <input
            v-model="createEdgeForm.relationshipType"
            type="text"
            class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm"
            placeholder="e.g., SIMILAR_TO, REFERENCES"
          />
        </div>
        <div>
          <label class="block text-sm font-medium text-text-secondary mb-1">Weight (0.0 - 1.0)</label>
          <input
            v-model.number="createEdgeForm.weight"
            type="number"
            min="0"
            max="1"
            step="0.01"
            class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm"
            placeholder="0.85"
          />
        </div>
      </div>
      <template #footer>
        <button @click="showCreateEdgeModal = false" class="px-4 py-2 text-sm font-medium bg-transparent text-text-secondary border border-border rounded hover:bg-bg-hover transition-colors">
          Cancel
        </button>
        <button
          @click="handleCreateEdge"
          :disabled="!createEdgeForm.source || !createEdgeForm.target || !createEdgeForm.relationshipType || creatingEdge"
          class="px-4 py-2 text-sm font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <i :class="['fas', creatingEdge ? 'fa-spinner fa-spin' : 'fa-check', 'mr-2']"></i>Create
        </button>
      </template>
    </Modal>

    <!-- Path Finder Modal -->
    <Modal v-model="showPathFinderModal" title="Find Path" icon="fas fa-route" size="medium">
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-text-secondary mb-1">Source Node</label>
          <select
            v-model="pathFinderForm.source"
            class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary focus:outline-none focus:border-border-light transition-colors text-sm"
          >
            <option value="">Select source node</option>
            <option v-for="node in nodes" :key="node.id" :value="node.id">{{ node.id }}</option>
          </select>
        </div>
        <div>
          <label class="block text-sm font-medium text-text-secondary mb-1">Target Node</label>
          <select
            v-model="pathFinderForm.target"
            class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary focus:outline-none focus:border-border-light transition-colors text-sm"
          >
            <option value="">Select target node</option>
            <option v-for="node in nodes" :key="node.id" :value="node.id">{{ node.id }}</option>
          </select>
        </div>
        <div v-if="pathResult" class="mt-4 p-4 bg-bg-tertiary rounded border border-border">
          <div v-if="pathResult.found" class="space-y-2">
            <p class="text-sm font-medium text-text-primary">Path Found!</p>
            <div class="text-sm text-text-secondary">
              <p>Path: {{ pathResult.path.map((n: any) => n.id).join(' → ') }}</p>
            </div>
          </div>
          <div v-else class="text-sm text-text-secondary">
            <p>No path found between these nodes</p>
          </div>
        </div>
      </div>
      <template #footer>
        <button @click="showPathFinderModal = false; pathResult = null; currentPath = []" class="px-4 py-2 text-sm font-medium bg-transparent text-text-secondary border border-border rounded hover:bg-bg-hover transition-colors">
          Close
        </button>
        <button
          @click="handleFindPath"
          :disabled="!pathFinderForm.source || !pathFinderForm.target || findingPath"
          class="px-4 py-2 text-sm font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <i :class="['fas', findingPath ? 'fa-spinner fa-spin' : 'fa-search', 'mr-2']"></i>Find Path
        </button>
      </template>
    </Modal>

    <!-- Neighbors Modal -->
    <Modal v-model="showNeighborsModal" title="Node Neighbors" icon="fas fa-users" size="large">
      <div v-if="loadingNeighbors" class="flex flex-col items-center justify-center py-12">
        <i class="fas fa-spinner fa-spin text-4xl text-text-secondary mb-4"></i>
        <p class="text-sm text-text-secondary">Loading neighbors...</p>
      </div>
      <div v-else-if="neighbors.length === 0" class="text-center py-12 text-text-secondary">
        <p class="text-sm">No neighbors found</p>
      </div>
      <div v-else class="space-y-3">
        <div v-for="neighbor in neighbors" :key="neighbor.node.id" class="bg-bg-tertiary border border-border rounded-lg p-4">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm font-medium text-text-primary">{{ neighbor.node.id }}</p>
              <p class="text-xs text-text-secondary mt-1">Type: {{ neighbor.node.node_type }} • Weight: {{ neighbor.edge.weight?.toFixed(2) || 'N/A' }}</p>
            </div>
            <button
              @click="selectNode(neighbor.node)"
              class="px-3 py-1.5 text-xs font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors"
            >
              View
            </button>
          </div>
        </div>
      </div>
      <template #footer>
        <button @click="showNeighborsModal = false" class="px-4 py-2 text-sm font-medium bg-transparent text-text-secondary border border-border rounded hover:bg-bg-hover transition-colors">
          Close
        </button>
      </template>
    </Modal>

    <!-- Find Related Modal -->
    <Modal v-model="showFindRelatedModal" title="Find Related Nodes" icon="fas fa-search" size="large">
      <div class="space-y-4 mb-4">
        <div>
          <label class="block text-sm font-medium text-text-secondary mb-1">Max Hops</label>
          <input
            v-model.number="findRelatedForm.maxHops"
            type="number"
            min="1"
            max="10"
            class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm"
            placeholder="2"
          />
        </div>
        <div>
          <label class="block text-sm font-medium text-text-secondary mb-1">Relationship Type (optional)</label>
          <select
            v-model="findRelatedForm.relationshipType"
            class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary focus:outline-none focus:border-border-light transition-colors text-sm"
          >
            <option value="">All Types</option>
            <option v-for="type in relationshipTypes" :key="type" :value="type">{{ type }}</option>
          </select>
        </div>
      </div>
      <div v-if="findingRelated" class="flex flex-col items-center justify-center py-12">
        <i class="fas fa-spinner fa-spin text-4xl text-text-secondary mb-4"></i>
        <p class="text-sm text-text-secondary">Finding related nodes...</p>
      </div>
      <div v-else-if="relatedNodes.length === 0 && findRelatedForm.maxHops" class="text-center py-12 text-text-secondary">
        <p class="text-sm">No related nodes found</p>
      </div>
      <div v-else class="space-y-3 max-h-96 overflow-y-auto">
        <div v-for="related in relatedNodes" :key="related.node.id" class="bg-bg-tertiary border border-border rounded-lg p-4">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm font-medium text-text-primary">{{ related.node.id }}</p>
              <p class="text-xs text-text-secondary mt-1">Distance: {{ related.distance?.toFixed(2) || 'N/A' }} • Weight: {{ related.weight?.toFixed(2) || 'N/A' }}</p>
            </div>
            <button
              @click="selectNode(related.node)"
              class="px-3 py-1.5 text-xs font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors"
            >
              View
            </button>
          </div>
        </div>
      </div>
      <template #footer>
        <button @click="showFindRelatedModal = false; relatedNodes = []" class="px-4 py-2 text-sm font-medium bg-transparent text-text-secondary border border-border rounded hover:bg-bg-hover transition-colors">
          Close
        </button>
        <button
          @click="handleFindRelated"
          :disabled="!findRelatedForm.maxHops || findingRelated"
          class="px-4 py-2 text-sm font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <i :class="['fas', findingRelated ? 'fa-spinner fa-spin' : 'fa-search', 'mr-2']"></i>Find
        </button>
      </template>
    </Modal>

    <!-- Discovery Modal (Collection) -->
    <Modal v-model="showDiscoveryModal" title="Discover Edges" icon="fas fa-search" size="medium">
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-text-secondary mb-1">Similarity Threshold (0.0 - 1.0)</label>
          <input
            v-model.number="discoveryForm.similarityThreshold"
            type="number"
            min="0"
            max="1"
            step="0.01"
            class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm"
            placeholder="0.7"
          />
        </div>
        <div>
          <label class="block text-sm font-medium text-text-secondary mb-1">Max Per Node</label>
          <input
            v-model.number="discoveryForm.maxPerNode"
            type="number"
            min="1"
            max="100"
            class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm"
            placeholder="10"
          />
        </div>
        <div v-if="discoveryStatus" class="mt-4 p-4 bg-bg-tertiary rounded border border-border">
          <div class="space-y-2 text-sm">
            <div class="flex justify-between">
              <span class="text-text-secondary">Total Nodes:</span>
              <span class="text-text-primary">{{ discoveryStatus.total_nodes }}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-text-secondary">Nodes with Edges:</span>
              <span class="text-text-primary">{{ discoveryStatus.nodes_with_edges }}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-text-secondary">Total Edges:</span>
              <span class="text-text-primary">{{ discoveryStatus.total_edges }}</span>
            </div>
            <div v-if="discoveryStatus.discovery_progress !== undefined" class="flex justify-between">
              <span class="text-text-secondary">Progress:</span>
              <span class="text-text-primary">{{ (discoveryStatus.discovery_progress * 100).toFixed(1) }}%</span>
            </div>
          </div>
        </div>
      </div>
      <template #footer>
        <button @click="showDiscoveryModal = false" class="px-4 py-2 text-sm font-medium bg-transparent text-text-secondary border border-border rounded hover:bg-bg-hover transition-colors">
          Close
        </button>
        <button
          @click="handleGetDiscoveryStatus"
          class="px-4 py-2 text-sm font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors"
        >
          <i class="fas fa-info-circle mr-2"></i>Status
        </button>
        <button
          @click="handleDiscoverEdges"
          :disabled="discoveringEdges"
          class="px-4 py-2 text-sm font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <i :class="['fas', discoveringEdges ? 'fa-spinner fa-spin' : 'fa-search', 'mr-2']"></i>Discover
        </button>
      </template>
    </Modal>

    <!-- Node Discovery Modal -->
    <Modal v-model="showNodeDiscoveryModal" title="Discover Edges for Node" icon="fas fa-link" size="medium">
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-text-secondary mb-1">Node ID</label>
          <input
            :value="discoveryNodeId"
            type="text"
            disabled
            class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary opacity-50 cursor-not-allowed text-sm"
          />
        </div>
        <div>
          <label class="block text-sm font-medium text-text-secondary mb-1">Similarity Threshold (0.0 - 1.0)</label>
          <input
            v-model.number="nodeDiscoveryForm.similarityThreshold"
            type="number"
            min="0"
            max="1"
            step="0.01"
            class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm"
            placeholder="0.7"
          />
        </div>
        <div>
          <label class="block text-sm font-medium text-text-secondary mb-1">Max Per Node</label>
          <input
            v-model.number="nodeDiscoveryForm.maxPerNode"
            type="number"
            min="1"
            max="100"
            class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm"
            placeholder="10"
          />
        </div>
      </div>
      <template #footer>
        <button @click="showNodeDiscoveryModal = false" class="px-4 py-2 text-sm font-medium bg-transparent text-text-secondary border border-border rounded hover:bg-bg-hover transition-colors">
          Cancel
        </button>
        <button
          @click="handleDiscoverEdgesForNode"
          :disabled="discoveringNodeEdges"
          class="px-4 py-2 text-sm font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <i :class="['fas', discoveringNodeEdges ? 'fa-spinner fa-spin' : 'fa-search', 'mr-2']"></i>Discover
        </button>
      </template>
    </Modal>

    <!-- Delete Edge Confirmation Dialog -->
    <Dialog
      v-model="showDeleteEdgeDialog"
      title="Delete Edge"
      message="Are you sure you want to delete this edge? This action cannot be undone."
      type="confirm"
      @confirm="handleDeleteEdge"
      @cancel="showDeleteEdgeDialog = false; edgeToDelete = null"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue';
import { storeToRefs } from 'pinia';
import { useVectorizerStore } from '../stores/vectorizer';
import { useToast } from '../composables/useToast';
import { useConfirm } from '../composables/useConfirm';
import Modal from '../components/Modal.vue';
import Dialog from '../components/Dialog.vue';
import { Network, Options } from 'vis-network';
import 'vis-network/styles/vis-network.css';

const vectorizerStore = useVectorizerStore();
const { collections, loading } = storeToRefs(vectorizerStore);
const toast = useToast();
const confirm = useConfirm();

// Graph data
const selectedCollection = ref<string>('');
const nodes = ref<any[]>([]);
const edges = ref<any[]>([]);
const searchQuery = ref('');
const relationshipFilter = ref('all');
const selectedNode = ref<string | null>(null);
const selectedEdge = ref<string | null>(null);
const currentPath = ref<string[]>([]);

// Graph visualization with vis-network
const networkContainer = ref<HTMLDivElement | null>(null);
const networkInstance = ref<Network | null>(null);
const rendering = ref(false);

// Modals
const showCreateEdgeModal = ref(false);
const showPathFinderModal = ref(false);
const showNeighborsModal = ref(false);
const showFindRelatedModal = ref(false);
const showDiscoveryModal = ref(false);
const showNodeDiscoveryModal = ref(false);
const showDeleteEdgeDialog = ref(false);

// Form data
const createEdgeForm = ref({
  source: '',
  target: '',
  relationshipType: '',
  weight: 0.85
});

const pathFinderForm = ref({
  source: '',
  target: ''
});

const findRelatedForm = ref({
  maxHops: 2,
  relationshipType: ''
});

const discoveryForm = ref({
  similarityThreshold: 0.7,
  maxPerNode: 10
});

const nodeDiscoveryForm = ref({
  similarityThreshold: 0.7,
  maxPerNode: 10
});

// Loading states
const creatingEdge = ref(false);
const findingPath = ref(false);
const loadingNeighbors = ref(false);
const findingRelated = ref(false);
const discoveringEdges = ref(false);
const discoveringNodeEdges = ref(false);

// Data
const neighbors = ref<any[]>([]);
const relatedNodes = ref<any[]>([]);
const pathResult = ref<any>(null);
const discoveryStatus = ref<any>(null);
const neighborsNodeId = ref<string>('');
const findRelatedNodeId = ref<string>('');
const discoveryNodeId = ref<string>('');
const edgeToDelete = ref<string | null>(null);

// Context menu
const contextMenu = ref({
  show: false,
  x: 0,
  y: 0,
  type: 'node' as 'node' | 'edge',
  nodeId: '',
  edgeId: ''
});

// Cache
const graphCache = new Map<string, { nodes: any[]; edges: any[]; timestamp: number }>();
const GRAPH_CACHE_TTL = 5 * 60 * 1000; // 5 minutes

// Computed
const filteredNodes = computed(() => {
  return nodes.value.filter((node) => {
    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase();
      if (
        !node.id.toLowerCase().includes(query) &&
        !node.node_type.toLowerCase().includes(query) &&
        !JSON.stringify(node.metadata || {}).toLowerCase().includes(query)
      ) {
        return false;
      }
    }
    return true;
  });
});

const filteredEdges = computed(() => {
  return edges.value.filter((edge) => {
    if (relationshipFilter.value !== 'all' && edge.relationship_type !== relationshipFilter.value) {
      return false;
    }
    return (
      filteredNodes.value.some((n) => n.id === edge.source) &&
      filteredNodes.value.some((n) => n.id === edge.target)
    );
  });
});

const relationshipTypes = computed(() => {
  return Array.from(new Set(edges.value.map((e) => e.relationship_type))).sort();
});

const selectedNodeData = computed(() => {
  if (!selectedNode.value) return null;
  return nodes.value.find((n) => n.id === selectedNode.value);
});

const selectedEdgeData = computed(() => {
  if (!selectedEdge.value) return null;
  return edges.value.find((e) => e.id === selectedEdge.value);
});

// Methods
function getNodeColor(node: any) {
  switch (node.node_type) {
    case 'document':
      return { background: '#a855f7', border: '#7c3aed', highlight: { background: '#c084fc', border: '#a855f7' } };
    case 'file':
      return { background: '#f59e0b', border: '#d97706', highlight: { background: '#fbbf24', border: '#f59e0b' } };
    case 'chunk':
      return { background: '#06b6d4', border: '#0891b2', highlight: { background: '#22d3ee', border: '#06b6d4' } };
    case 'vector':
      return { background: '#6366f1', border: '#4f46e5', highlight: { background: '#818cf8', border: '#6366f1' } };
    default:
      return { background: '#8b5cf6', border: '#7c3aed', highlight: { background: '#a78bfa', border: '#8b5cf6' } };
  }
}

function getEdgeColor(edge: any): string {
  if (selectedNode.value === edge.source || selectedNode.value === edge.target) {
    return '#60a5fa';
  }
  switch (edge.relationship_type) {
    case 'SIMILAR_TO':
      return '#34d399';
    case 'REFERENCES':
      return '#fbbf24';
    case 'CONTAINS':
      return '#a78bfa';
    case 'DERIVED_FROM':
      return '#22d3ee';
    default:
      return '#94a3b8';
  }
}

function initializeVisNetwork(): void {
  if (!networkContainer.value || filteredNodes.value.length === 0) {
    if (networkInstance.value) {
      networkInstance.value.destroy();
      networkInstance.value = null;
    }
    return;
  }

  // Convert to vis-network format
  const visNodes = filteredNodes.value.map((node) => {
    const colors = getNodeColor(node);
    if (selectedNode.value === node.id) {
      colors.background = '#3b82f6';
      colors.border = '#2563eb';
      colors.highlight = { background: '#60a5fa', border: '#3b82f6' };
    }

    return {
      id: node.id,
      label: node.id.length > 30 ? `${node.id.substring(0, 30)}...` : node.id,
      title: `${node.id}\nType: ${node.node_type}\nMetadata: ${JSON.stringify(node.metadata || {}, null, 2)}`,
      color: colors,
      shape: 'dot',
      size: selectedNode.value === node.id ? 25 : 20,
      font: {
        size: selectedNode.value === node.id ? 14 : 12,
        face: 'Inter, system-ui, sans-serif',
        color: selectedNode.value === node.id ? '#1e40af' : '#374151',
      },
      borderWidth: selectedNode.value === node.id ? 3 : 2,
      shadow: selectedNode.value === node.id,
    };
  });

  const visEdges = filteredEdges.value.map((edge) => {
    const baseWidth = 2;
    const weightMultiplier = Math.max(1, edge.weight || 1);
    const edgeWidth = Math.max(baseWidth, Math.min(5, baseWidth * weightMultiplier));

    return {
      id: edge.id,
      from: edge.source,
      to: edge.target,
      label: edge.relationship_type,
      color: {
        color: getEdgeColor(edge),
        highlight: '#60a5fa',
        hover: '#60a5fa',
        opacity: 0.8,
      },
      width: edgeWidth,
      arrows: {
        to: {
          enabled: true,
          scaleFactor: 1.5,
          type: 'arrow',
        },
      },
      smooth: {
        enabled: true,
        type: 'continuous',
        roundness: 0.5,
      },
      title: `${edge.relationship_type}\nWeight: ${edge.weight}`,
      selectionWidth: edgeWidth + 2,
      hoverWidth: edgeWidth + 1,
    };
  });

  const data = { nodes: visNodes, edges: visEdges };

  // Neo4j-style options (same as dashboard)
  const options: Options = {
    nodes: {
      shape: 'dot',
      font: {
        size: 12,
        face: 'Inter, system-ui, sans-serif',
      },
      borderWidth: 2,
      shadow: {
        enabled: true,
        color: 'rgba(0,0,0,0.2)',
        size: 5,
        x: 2,
        y: 2,
      },
    },
    edges: {
      width: 2,
      color: {
        color: '#94a3b8',
        highlight: '#60a5fa',
        hover: '#60a5fa',
        opacity: 0.8,
      },
      smooth: {
        enabled: true,
        type: 'continuous',
        roundness: 0.5,
      },
      arrows: {
        to: {
          enabled: true,
          scaleFactor: 1.5,
          type: 'arrow',
        },
      },
      font: {
        size: 11,
        align: 'middle',
        color: '#e2e8f0',
        strokeWidth: 2,
        strokeColor: '#1e293b',
      },
      selectionWidth: 4,
      hoverWidth: 3,
    },
    physics: {
      enabled: true,
      stabilization: {
        enabled: true,
        iterations: 200,
        fit: true,
      },
      barnesHut: {
        gravitationalConstant: -2000,
        centralGravity: 0.3,
        springLength: 200,
        springConstant: 0.04,
        damping: 0.09,
        avoidOverlap: 0.5,
      },
    },
    interaction: {
      hover: true,
      tooltipDelay: 100,
      zoomView: true,
      dragView: true,
      selectConnectedEdges: true,
    },
    layout: {
      improvedLayout: true,
      hierarchical: {
        enabled: false,
      },
    },
  };

  // Create or update network
  if (networkInstance.value) {
    rendering.value = true;
    networkInstance.value.setData(data);
    networkInstance.value.setOptions(options);

    const handleStabilization = () => {
      rendering.value = false;
    };

    networkInstance.value.once('stabilizationIterationsDone', handleStabilization);

    const fallbackTimeout = setTimeout(() => {
      rendering.value = false;
    }, 5000);

    networkInstance.value.once('stabilizationIterationsDone', () => {
      clearTimeout(fallbackTimeout);
    });
  } else {
    rendering.value = true;

    const network = new Network(networkContainer.value, data, options);
    networkInstance.value = network;

    network.once('stabilizationIterationsDone', () => {
      rendering.value = false;
    });

    const fallbackTimeout = setTimeout(() => {
      rendering.value = false;
    }, 5000);

    network.once('stabilizationIterationsDone', () => {
      clearTimeout(fallbackTimeout);
    });

    // Handle node selection
    network.on('selectNode', (params: any) => {
      if (params.nodes.length > 0) {
        selectedNode.value = params.nodes[0] as string;
      } else {
        selectedNode.value = null;
      }
    });

    // Handle edge selection
    network.on('selectEdge', (params: any) => {
      if (params.edges.length > 0) {
        selectedEdge.value = params.edges[0] as string;
      } else {
        selectedEdge.value = null;
      }
    });

    // Handle click on background
    network.on('click', (params: any) => {
      if (params.nodes.length === 0 && params.edges.length === 0) {
        selectedNode.value = null;
        selectedEdge.value = null;
      }
    });

    // Handle double click to focus
    network.on('doubleClick', (params: any) => {
      if (params.nodes.length > 0) {
        network.focus(params.nodes[0] as string, {
          scale: 1.2,
          animation: true,
        });
        showNeighborsModal.value = true;
        neighborsNodeId.value = params.nodes[0] as string;
        loadNeighbors(params.nodes[0] as string);
      }
    });
  }
}

async function loadGraphData(forceRefresh = false): Promise<void> {
  if (!selectedCollection.value) return;

  const cacheKey = selectedCollection.value;
  const cached = graphCache.get(cacheKey);
  const now = Date.now();

  if (!forceRefresh && cached && (now - cached.timestamp) < GRAPH_CACHE_TTL) {
    nodes.value = cached.nodes;
    edges.value = cached.edges;
    await nextTick();
    initializeVisNetwork();
    return;
  }

  try {
    const [nodesResponse, edgesResponse] = await Promise.all([
      vectorizerStore.listGraphNodes(selectedCollection.value),
      vectorizerStore.listGraphEdges(selectedCollection.value)
    ]);

    nodes.value = nodesResponse.nodes || [];
    edges.value = edgesResponse.edges || [];

    graphCache.set(cacheKey, {
      nodes: nodes.value,
      edges: edges.value,
      timestamp: now
    });

    await nextTick();
    initializeVisNetwork();
  } catch (error) {
    console.error('Error loading graph data:', error);
    toast.error(error instanceof Error ? error.message : 'Failed to load graph data');
    nodes.value = [];
    edges.value = [];
  }
}

async function refreshGraph(): Promise<void> {
  graphCache.delete(selectedCollection.value);
  await loadGraphData(true);
}

function selectNode(node: any): void {
  selectedNode.value = node.id;
  selectedEdge.value = null;
  currentPath.value = [];
  if (networkInstance.value) {
    networkInstance.value.selectNodes([node.id]);
  }
}

function selectEdge(edge: any): void {
  selectedEdge.value = edge.id;
  selectedNode.value = null;
  if (networkInstance.value) {
    networkInstance.value.selectEdges([edge.id]);
  }
}

function fitGraph(): void {
  if (networkInstance.value) {
    networkInstance.value.fit({
      animation: true,
    });
  }
}

function resetView(): void {
  selectedNode.value = null;
  selectedEdge.value = null;
  currentPath.value = [];
  searchQuery.value = '';
  relationshipFilter.value = 'all';
  if (networkInstance.value) {
    networkInstance.value.fit({
      animation: true,
    });
    networkInstance.value.moveTo({
      scale: 1,
      animation: true,
    });
  }
}


async function handleCreateEdge(): Promise<void> {
  if (!selectedCollection.value || !createEdgeForm.value.source || !createEdgeForm.value.target || !createEdgeForm.value.relationshipType) {
    return;
  }

  creatingEdge.value = true;
  try {
    await vectorizerStore.createGraphEdge(
      selectedCollection.value,
      createEdgeForm.value.source,
      createEdgeForm.value.target,
      createEdgeForm.value.relationshipType,
      createEdgeForm.value.weight
    );

    toast.success('Edge created successfully');
    showCreateEdgeModal.value = false;
    createEdgeForm.value = {
      source: '',
      target: '',
      relationshipType: '',
      weight: 0.85
    };
    await refreshGraph();
  } catch (error) {
    toast.error(error instanceof Error ? error.message : 'Failed to create edge');
  } finally {
    creatingEdge.value = false;
  }
}

async function handleFindPath(): Promise<void> {
  if (!selectedCollection.value || !pathFinderForm.value.source || !pathFinderForm.value.target) {
    return;
  }

  findingPath.value = true;
  try {
    const result = await vectorizerStore.findGraphPath(
      selectedCollection.value,
      pathFinderForm.value.source,
      pathFinderForm.value.target
    );

    pathResult.value = result;
    if (result.found && result.path) {
      currentPath.value = result.path.map((n: any) => n.id);
    } else {
      currentPath.value = [];
    }
  } catch (error) {
    toast.error(error instanceof Error ? error.message : 'Failed to find path');
    pathResult.value = null;
    currentPath.value = [];
  } finally {
    findingPath.value = false;
  }
}

async function loadNeighbors(nodeId: string): Promise<void> {
  if (!selectedCollection.value) return;

  loadingNeighbors.value = true;
  try {
    const response = await vectorizerStore.getGraphNeighbors(selectedCollection.value, nodeId);
    neighbors.value = response.neighbors || [];
  } catch (error) {
    toast.error(error instanceof Error ? error.message : 'Failed to load neighbors');
    neighbors.value = [];
  } finally {
    loadingNeighbors.value = false;
  }
}

async function handleFindRelated(): Promise<void> {
  if (!selectedCollection.value || !findRelatedNodeId.value) return;

  findingRelated.value = true;
  try {
    const response = await vectorizerStore.findRelatedNodes(
      selectedCollection.value,
      findRelatedNodeId.value,
      findRelatedForm.value.maxHops,
      findRelatedForm.value.relationshipType || undefined
    );
    relatedNodes.value = response.related || [];
  } catch (error) {
    toast.error(error instanceof Error ? error.message : 'Failed to find related nodes');
    relatedNodes.value = [];
  } finally {
    findingRelated.value = false;
  }
}

async function handleDiscoverEdges(): Promise<void> {
  if (!selectedCollection.value) return;

  discoveringEdges.value = true;
  try {
    const response = await vectorizerStore.discoverGraphEdges(
      selectedCollection.value,
      discoveryForm.value.similarityThreshold,
      discoveryForm.value.maxPerNode
    );

    toast.success(`Discovery completed: ${response.edges_created || 0} edges created`);
    await refreshGraph();
    await handleGetDiscoveryStatus();
  } catch (error) {
    toast.error(error instanceof Error ? error.message : 'Failed to discover edges');
  } finally {
    discoveringEdges.value = false;
  }
}

async function handleDiscoverEdgesForNode(): Promise<void> {
  if (!selectedCollection.value || !discoveryNodeId.value) return;

  discoveringNodeEdges.value = true;
  try {
    const response = await vectorizerStore.discoverGraphEdgesForNode(
      selectedCollection.value,
      discoveryNodeId.value,
      nodeDiscoveryForm.value.similarityThreshold,
      nodeDiscoveryForm.value.maxPerNode
    );

    toast.success(`Discovery completed: ${response.edges_created || 0} edges created`);
    showNodeDiscoveryModal.value = false;
    await refreshGraph();
  } catch (error) {
    toast.error(error instanceof Error ? error.message : 'Failed to discover edges for node');
  } finally {
    discoveringNodeEdges.value = false;
  }
}

async function handleGetDiscoveryStatus(): Promise<void> {
  if (!selectedCollection.value) return;

  try {
    const status = await vectorizerStore.getGraphDiscoveryStatus(selectedCollection.value);
    discoveryStatus.value = status;
  } catch (error) {
    toast.error(error instanceof Error ? error.message : 'Failed to get discovery status');
  }
}

function deleteEdgeConfirm(edgeId: string): void {
  edgeToDelete.value = edgeId;
  showDeleteEdgeDialog.value = true;
}

async function handleDeleteEdge(): Promise<void> {
  if (!edgeToDelete.value) return;

  try {
    await vectorizerStore.deleteGraphEdge(edgeToDelete.value);
    toast.success('Edge deleted successfully');
    showDeleteEdgeDialog.value = false;
    edgeToDelete.value = null;
    selectedEdge.value = null;
    await refreshGraph();
  } catch (error) {
    toast.error(error instanceof Error ? error.message : 'Failed to delete edge');
  }
}

// Watchers
watch(selectedCollection, () => {
  loadGraphData();
});

watch(showNeighborsModal, (show) => {
  if (show && neighborsNodeId.value) {
    loadNeighbors(neighborsNodeId.value);
  }
});

watch(showFindRelatedModal, (show) => {
  if (show && findRelatedNodeId.value) {
    findRelatedForm.value.maxHops = 2;
    findRelatedForm.value.relationshipType = '';
    relatedNodes.value = [];
  }
});

watch(showNodeDiscoveryModal, (show) => {
  if (show && discoveryNodeId.value) {
    nodeDiscoveryForm.value.similarityThreshold = 0.7;
    nodeDiscoveryForm.value.maxPerNode = 10;
  }
});

watch([filteredNodes, filteredEdges, selectedNode], () => {
  if (selectedCollection.value) {
    initializeVisNetwork();
  }
});

// Lifecycle
onMounted(() => {
  if (collections.value.length > 0 && !selectedCollection.value) {
    selectedCollection.value = collections.value[0].name;
  }

  // Close context menu on click outside
  document.addEventListener('click', () => {
    contextMenu.value.show = false;
  });
});

onUnmounted(() => {
  if (networkInstance.value) {
    networkInstance.value.destroy();
    networkInstance.value = null;
  }
  document.removeEventListener('click', () => {});
});
</script>

<style scoped>
/* Graph styles */
svg {
  background: var(--bg-primary);
}
</style>

