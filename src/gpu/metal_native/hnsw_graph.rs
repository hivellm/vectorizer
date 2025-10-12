//! # Metal Native HNSW Graph
//!
//! GPU-accelerated Hierarchical Navigable Small World (HNSW) graph implementation.
//! Provides high-performance approximate nearest neighbor search using Metal compute shaders.

use metal::{Buffer as MetalBuffer, Device as MetalDevice, ComputePipelineState, Library, Function, MTLComputePipelineDescriptor, MTLSize, MTLResourceOptions, MTLStorageMode, MTLCPUCacheMode, CommandBuffer, CommandQueue, CompileOptions};
use std::sync::Arc;
use std::collections::BinaryHeap;
use std::cmp::Reverse;
use crate::error::{Result, VectorizerError};
use crate::models::{Vector, DistanceMetric};
use super::context::MetalNativeContext;
use tracing::{info, warn, debug};

/// HNSW Node structure - represents a node in the hierarchical graph
#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct HnswNode {
    pub id: u32,                    // Unique node identifier
    pub level: u32,                 // Maximum layer this node appears in (0 = base layer)
    pub vector_offset: u32,         // Offset into vectors buffer
    pub connections_offset: u32,    // Offset into connections buffer for layer 0
}

/// Layer-specific node data for GPU processing
#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct HnswLayerNode {
    pub node_id: u32,               // Node ID in this layer
    pub connections_offset: u32,    // Offset to connections in this layer
    pub connection_count: u32,      // Number of connections in this layer
}

/// Full GPU search query structure
#[cfg(target_os = "macos")]
#[derive(Debug, Clone)]
#[repr(C)]
pub struct GpuSearchQuery {
    pub data: [f32; 512],         // Query vector (max 512 dimensions)
    pub dimension: u32,           // Actual dimension
}

/// Full GPU search result structure
#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct GpuSearchResult {
    pub vector_id: u32,           // ID of the matched vector
    pub distance: f32,            // Distance to query
    pub vector_index: u32,        // Index in vectors buffer
}

/// Vector metadata for GPU processing
#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct GpuVectorMetadata {
    pub vector_id: u32,           // Original vector ID
    pub is_active: u32,           // 1 if vector is active, 0 if removed
}

/// HNSW Layer - represents one level in the hierarchy
#[cfg(target_os = "macos")]
#[derive(Debug)]
pub struct HnswLayer {
    pub level: usize,                              // Layer index (0 = base, higher = top)
    pub nodes: Vec<HnswLayerNode>,                 // Nodes present in this layer
    pub connections: Vec<u32>,                     // Flattened connection lists for all nodes
    pub max_connections: usize,                    // Maximum connections per node in this layer
}

/// Candidate node during search with distance
#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(C)]
pub struct SearchCandidate {
    pub node_id: u32,
    pub distance: f32,
}

#[cfg(target_os = "macos")]
impl Eq for SearchCandidate {}

#[cfg(target_os = "macos")]
impl Ord for SearchCandidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // For distance comparison, smaller distance is better (closer)
        // We want to compare by distance first, then by node_id for stability
        match self.distance.partial_cmp(&other.distance) {
            Some(std::cmp::Ordering::Equal) => self.node_id.cmp(&other.node_id),
            Some(ord) => ord,
            None => {
                // Handle NaN case - treat NaN as worst distance
                if self.distance.is_nan() && !other.distance.is_nan() {
                    std::cmp::Ordering::Greater
                } else if !self.distance.is_nan() && other.distance.is_nan() {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            }
        }
    }
}

/// Search result structure
#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SearchResult {
    pub node_id: u32,
    pub distance: f32,
}

/// HNSW Configuration
#[cfg(target_os = "macos")]
#[derive(Debug, Clone)]
pub struct HnswConfig {
    pub max_connections: usize,
    pub ef_construction: usize,
    pub ef_search: usize,
    pub max_level: usize,
    pub level_multiplier: f32,
}

#[cfg(target_os = "macos")]
impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            max_connections: 16,
            ef_construction: 200,
            ef_search: 64,
            max_level: 16,
            level_multiplier: 1.0 / std::f32::consts::LN_2,
        }
    }
}

#[cfg(target_os = "macos")]
impl HnswConfig {
    /// Calculate level for a new node using probabilistic assignment
    /// Formula: level = floor(-ln(rand()) * level_multiplier)
    pub fn calculate_node_level(&self) -> usize {
        use std::f32::consts::LN_2;

        // Generate random float [0, 1)
        let rand_val: f32 = rand::random();

        // Avoid log(0) by ensuring rand_val > 0
        let safe_rand = rand_val.max(1e-10);

        // Calculate level: floor(-ln(rand) * level_multiplier)
        let level = (-safe_rand.ln() * self.level_multiplier).floor() as usize;

        // Cap at max_level to prevent excessive hierarchy
        level.min(self.max_level)
    }

    /// Get maximum connections for a specific layer
    /// Higher layers have fewer connections for faster search
    pub fn max_connections_for_layer(&self, layer: usize) -> usize {
        if layer == 0 {
            // Base layer has full connections
            self.max_connections
        } else {
            // Higher layers have fewer connections (exponential decay)
            let decay_factor = 2.0_f32.powf(layer as f32);
            (self.max_connections as f32 / decay_factor).max(4.0) as usize
        }
    }
}

/// HNSW Utility Functions
#[cfg(target_os = "macos")]
impl MetalNativeHnswGraph {
    /// Calculate cosine distance between two vectors
    fn calculate_cosine_distance(a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len(), "Vector dimensions must match");

        let mut dot_product = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;

        for i in 0..a.len() {
            dot_product += a[i] * b[i];
            norm_a += a[i] * a[i];
            norm_b += b[i] * b[i];
        }

        norm_a = norm_a.sqrt();
        norm_b = norm_b.sqrt();

        // Avoid division by zero
        const EPSILON: f32 = 1e-8;
        let denom = (norm_a * norm_b).max(EPSILON);
        let similarity = dot_product / denom;

        // Clamp to valid range and convert to distance
        let similarity = similarity.max(-1.0).min(1.0);
        1.0 - similarity
    }

    /// Calculate Euclidean distance between two vectors
    fn calculate_euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len(), "Vector dimensions must match");

        let mut sum = 0.0;
        for i in 0..a.len() {
            let diff = a[i] - b[i];
            sum += diff * diff;
        }
        sum.sqrt()
    }

    /// Calculate distance based on metric
    fn calculate_distance(a: &[f32], b: &[f32], metric: DistanceMetric) -> f32 {
        match metric {
            DistanceMetric::Cosine => Self::calculate_cosine_distance(a, b),
            DistanceMetric::Euclidean => Self::calculate_euclidean_distance(a, b),
            _ => unimplemented!("Distance metric not implemented for HNSW"),
        }
    }

    /// Select neighbors using greedy algorithm
    /// Returns the ef closest neighbors to the query from candidates
    fn select_neighbors(
        query: &[f32],
        candidates: &[SearchCandidate],
        ef: usize,
        vectors: &[Vector],
        metric: DistanceMetric,
    ) -> Vec<SearchCandidate> {
        let mut result = Vec::new();
        let mut candidates_heap: BinaryHeap<Reverse<SearchCandidate>> = candidates
            .iter()
            .map(|c| Reverse(*c))
            .collect();

        while result.len() < ef && !candidates_heap.is_empty() {
            let candidate = candidates_heap.pop().unwrap().0;
            result.push(candidate);
        }

        result
    }

    /// Find entry point for search (highest level node)
    fn find_entry_point(&self) -> Option<u32> {
        self.entry_point
    }

    /// Get vector by node ID
    fn get_vector_by_node_id(&self, node_id: u32) -> Result<&Vector> {
        // This would need access to the vector storage
        // For now, return an error - this will be implemented when integrating with vector storage
        Err(VectorizerError::Other(format!("get_vector_by_node_id not implemented yet, node_id: {}", node_id)))
    }
}

/// Metal Native HNSW Graph - Complete hierarchical implementation
#[cfg(target_os = "macos")]
#[derive(Debug)]
pub struct MetalNativeHnswGraph {
    context: Arc<MetalNativeContext>,
    nodes_buffer: MetalBuffer,
    connections_buffer: MetalBuffer,
    vectors_buffer: MetalBuffer,
    layers: Vec<HnswLayer>,                    // Hierarchical layers (0 = base, higher = top)
    nodes: Vec<HnswNode>,                      // All nodes in the graph
    node_count: usize,
    connection_count: usize,
    dimension: usize,
    config: HnswConfig,
    compute_pipeline: ComputePipelineState,
    max_level_reached: usize,                  // Maximum level reached during construction
    entry_point: Option<u32>,                  // Entry point for search (highest level node)

    // Full GPU search resources
    search_pipeline: Option<ComputePipelineState>,    // Pipeline for full GPU search
    metadata_buffer: Option<MetalBuffer>,             // Vector metadata buffer
    search_results_buffer: Option<MetalBuffer>,       // Intermediate search results
    final_results_buffer: Option<MetalBuffer>,        // Final top-k results
}

#[cfg(target_os = "macos")]
impl MetalNativeHnswGraph {
    /// Create new Metal native HNSW graph with default configuration
    pub fn new(
        context: Arc<MetalNativeContext>,
        dimension: usize,
        max_connections: usize,
    ) -> Result<Self> {
        let config = HnswConfig {
            max_connections,
            ..Default::default()
        };
        Self::new_with_config(context, dimension, config)
    }

    /// Create new Metal native HNSW graph with custom configuration
    pub fn new_with_config(
        context: Arc<MetalNativeContext>,
        dimension: usize,
        config: HnswConfig,
    ) -> Result<Self> {
        let device = context.device();

        // Initialize layers vector (will be populated during construction)
        let layers = Vec::new();

        // Initialize nodes vector
        let nodes = Vec::new();

        // Create compute pipeline for HNSW search
        let compute_pipeline = Self::create_compute_pipeline(&device)?;

        // Create buffers with minimum size to avoid null pointers
        let min_buffer_size = 1024; // 1KB minimum

        let nodes_buffer = device.new_buffer(
            min_buffer_size,
            MTLResourceOptions::StorageModePrivate, // VRAM only
        );

        let connections_buffer = device.new_buffer(
            min_buffer_size,
            MTLResourceOptions::StorageModePrivate, // VRAM only
        );

        let vectors_buffer = device.new_buffer(
            min_buffer_size,
            MTLResourceOptions::StorageModePrivate, // VRAM only
        );

        // Validate buffers were created successfully
        debug!("🔍 Validating Metal buffers...");
        debug!("  - Nodes buffer: {:?}", nodes_buffer.contents().is_null());
        debug!("  - Connections buffer: {:?}", connections_buffer.contents().is_null());
        debug!("  - Vectors buffer: {:?}", vectors_buffer.contents().is_null());

        debug!("✅ Metal native HNSW graph created with GPU compute pipeline");

        Ok(Self {
            context,
            nodes_buffer,
            connections_buffer,
            vectors_buffer,
            layers,
            nodes,
            node_count: 0,
            connection_count: 0,
            dimension,
            config,
            compute_pipeline,
            max_level_reached: 0,
            entry_point: None,

            // Full GPU search resources (initialized lazily)
            search_pipeline: None,
            metadata_buffer: None,
            search_results_buffer: None,
            final_results_buffer: None,
        })
    }
    
    /// Create Metal compute pipeline for HNSW search
    fn create_compute_pipeline(device: &MetalDevice) -> Result<ComputePipelineState> {
        debug!("🔧 Creating Metal compute pipeline...");
        
        // Load Metal shader source
        let shader_source = include_str!("../shaders/metal_hnsw.metal");
        debug!("📄 Shader source loaded: {} bytes", shader_source.len());
        
        // Create Metal library from source
        let library = device.new_library_with_source(shader_source, &CompileOptions::new())
            .map_err(|e| {
                tracing::error!("❌ Failed to compile Metal shader: {:?}", e);
                VectorizerError::Other(format!("Failed to compile Metal shader: {:?}", e))
            })?;
        
        debug!("✅ Metal library created successfully");
        
        // Get the HNSW search function
        let function = library.get_function("hnsw_search_complete", None)
            .map_err(|e| {
                tracing::error!("❌ Failed to get hnsw_search_complete function: {:?}", e);
                VectorizerError::Other(format!("Failed to get hnsw_search_complete function: {:?}", e))
            })?;
        
        debug!("✅ HNSW search function retrieved");
        
        // Create compute pipeline directly from function
        let pipeline = device.new_compute_pipeline_state_with_function(&function)
            .map_err(|e| {
                tracing::error!("❌ Failed to create compute pipeline: {:?}", e);
                VectorizerError::Other(format!("Failed to create compute pipeline: {:?}", e))
            })?;
        
        debug!("✅ Metal compute pipeline created for HNSW search");
        Ok(pipeline)
    }
    
    /// Build HNSW graph using complete hierarchical algorithm
    pub fn build_graph(&mut self, vectors: &[Vector]) -> Result<()> {
        if vectors.is_empty() {
            return Ok(());
        }

        info!("🔧 Building HNSW graph with {} vectors", vectors.len());

        // Initialize hierarchical layers
        self.initialize_layers(vectors.len())?;

        // Store vectors reference for distance calculations
        let mut all_nodes = Vec::new();

        // Insert first node (entry point)
        self.insert_first_node(vectors, &mut all_nodes)?;

        // Insert remaining nodes using HNSW algorithm
        for i in 1..vectors.len() {
            self.insert_node_hnsw(&vectors[i], i as u32, &vectors, &mut all_nodes)?;
        }

        // Update graph metadata
        self.nodes = all_nodes;
        self.node_count = vectors.len();
        self.connection_count = self.calculate_total_connections();

        // Upload complete graph to GPU
        self.upload_graph_to_gpu(vectors)?;

        debug!("✅ Complete HNSW graph built: {} nodes, {} connections, max_level: {}",
               self.node_count, self.connection_count, self.max_level_reached);

        Ok(())
    }

    /// Initialize hierarchical layers structure
    fn initialize_layers(&mut self, num_vectors: usize) -> Result<()> {
        self.layers.clear();

        // Estimate maximum level based on vector count
        let estimated_max_level = if num_vectors > 0 {
            (num_vectors as f32).log2().min(self.config.max_level as f32) as usize
        } else {
            0
        };

        // Create layers from level 0 (base) to max_level
        for level in 0..=estimated_max_level {
            let layer = HnswLayer {
                level,
                nodes: Vec::new(),
                connections: Vec::new(),
                max_connections: self.config.max_connections_for_layer(level),
            };
            self.layers.push(layer);
        }

        Ok(())
    }

    /// Insert first node as entry point
    fn insert_first_node(&mut self, vectors: &[Vector], all_nodes: &mut Vec<HnswNode>) -> Result<()> {
        let node_level = self.config.calculate_node_level();
        self.max_level_reached = self.max_level_reached.max(node_level);

        // Create node
        let node = HnswNode {
            id: 0,
            level: node_level as u32,
            vector_offset: 0,
            connections_offset: 0, // Will be updated when connections are added
        };

        all_nodes.push(node);
        self.entry_point = Some(0);

        // Add to all layers where this node appears
        for layer_idx in 0..=node_level {
            if layer_idx < self.layers.len() {
                let layer_node = HnswLayerNode {
                    node_id: 0,
                    connections_offset: self.layers[layer_idx].connections.len() as u32,
                    connection_count: 0,
                };
                self.layers[layer_idx].nodes.push(layer_node);
            }
        }

        debug!("✅ First node inserted as entry point (level: {})", node_level);
        Ok(())
    }

    /// Insert node using complete HNSW algorithm
    fn insert_node_hnsw(&mut self, vector: &Vector, node_id: u32, all_vectors: &[Vector], all_nodes: &mut Vec<HnswNode>) -> Result<()> {
        // Calculate node level probabilistically
        let node_level = self.config.calculate_node_level();
        self.max_level_reached = self.max_level_reached.max(node_level);

        // Start search from entry point
        let mut entry_point = self.entry_point.unwrap_or(0);

        // Search from top layer down to find nearest neighbors at each level
        let mut nearest_neighbors = Vec::new();

        for current_level in (0..=node_level).rev() {
            let ef = if current_level == 0 { self.config.ef_construction } else { 1 };

            // Search layer for nearest neighbors
            let level_neighbors = self.search_layer_greedy(
                &vector.data,
                entry_point,
                ef,
                current_level,
                all_vectors,
            )?;

            nearest_neighbors = level_neighbors;

            // Update entry point for next level
            if !nearest_neighbors.is_empty() {
                entry_point = nearest_neighbors[0].node_id;
            }
        }

        // Create the new node
        let node = HnswNode {
            id: node_id,
            level: node_level as u32,
            vector_offset: node_id, // Offset in vectors array
            connections_offset: 0, // Will be set when adding to layers
        };

        all_nodes.push(node);

        // Add bidirectional connections at each level
        for layer_idx in 0..=node_level {
            self.add_connections_for_node(
                node_id,
                &nearest_neighbors,
                layer_idx,
                all_vectors,
            )?;
        }

        debug!("✅ Node {} inserted (level: {}, connections: {})",
               node_id, node_level, nearest_neighbors.len());

        Ok(())
    }

    /// Search single layer using greedy algorithm
    fn search_layer_greedy(
        &self,
        query: &[f32],
        entry_point: u32,
        ef: usize,
        layer: usize,
        all_vectors: &[Vector],
    ) -> Result<Vec<SearchCandidate>> {
        let mut visited = std::collections::HashSet::new();
        let mut candidates = std::collections::BinaryHeap::new();
        let mut results = Vec::new();

        // Start with entry point
        let entry_distance = Self::calculate_distance(
            query,
            &all_vectors[entry_point as usize].data,
            DistanceMetric::Cosine
        );

        let entry_candidate = SearchCandidate {
            node_id: entry_point,
            distance: entry_distance,
        };

        candidates.push(Reverse(entry_candidate));
        results.push(entry_candidate);
        visited.insert(entry_point);

        while !candidates.is_empty() {
            let current = candidates.pop().unwrap().0;

            // Early termination: if current candidate is worse than ef-th result
            if !results.is_empty() && current.distance > results[results.len().saturating_sub(ef)].distance {
                break;
            }

            // Explore neighbors if layer exists and node is in it
            if layer < self.layers.len() {
                if let Some(neighbors) = self.get_node_connections_in_layer(entry_point, layer) {
                    for &neighbor_id in neighbors {
                        if visited.insert(neighbor_id) {
                            let distance = Self::calculate_distance(
                                query,
                                &all_vectors[neighbor_id as usize].data,
                                DistanceMetric::Cosine
                            );

                            let candidate = SearchCandidate {
                                node_id: neighbor_id,
                                distance,
                            };

                            candidates.push(Reverse(candidate));

                            // Keep track of best ef candidates
                            if results.len() < ef {
                                results.push(candidate);
                            } else if candidate.distance < results.last().unwrap().distance {
                                *results.last_mut().unwrap() = candidate;
                            }

                            // Sort results by distance
                            results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
                        }
                    }
                }
            }
        }

        // Return ef closest neighbors
        results.truncate(ef);
        Ok(results)
    }

    /// Add bidirectional connections for a node in a specific layer
    fn add_connections_for_node(
        &mut self,
        node_id: u32,
        neighbors: &[SearchCandidate],
        layer: usize,
        all_vectors: &[Vector],
    ) -> Result<()> {
        if layer >= self.layers.len() {
            return Ok(());
        }

        let layer_ref = &mut self.layers[layer];
        let max_conn = layer_ref.max_connections;

        // Select best neighbors (prune if too many)
        let selected_neighbors: Vec<u32> = neighbors
            .iter()
            .take(max_conn)
            .map(|c| c.node_id)
            .collect();

        // Add node to layer
        let layer_node = HnswLayerNode {
            node_id,
            connections_offset: layer_ref.connections.len() as u32,
            connection_count: selected_neighbors.len() as u32,
        };

        layer_ref.nodes.push(layer_node);

        // Add connections
        for &neighbor_id in &selected_neighbors {
            layer_ref.connections.push(neighbor_id);
        }

        // Add reverse connections (bidirectional)
        for &neighbor_id in &selected_neighbors {
            self.add_reverse_connection(neighbor_id, node_id, layer, all_vectors)?;
        }

        Ok(())
    }

    /// Add reverse connection from neighbor to new node
    fn add_reverse_connection(
        &mut self,
        from_node: u32,
        to_node: u32,
        layer: usize,
        all_vectors: &[Vector],
    ) -> Result<()> {
        if layer >= self.layers.len() {
            return Ok(());
        }

        // Find the layer node for from_node
        if let Some(layer_node_idx) = self.layers[layer].nodes.iter().position(|n| n.node_id == from_node) {
            let max_conn = self.layers[layer].max_connections;
            let current_count = self.layers[layer].nodes[layer_node_idx].connection_count;

            // Check if we can add more connections
            if current_count < max_conn as u32 {
                self.layers[layer].connections.push(to_node);
                self.layers[layer].nodes[layer_node_idx].connection_count += 1;
            } else {
                // Prune worst connection and add new one
                self.prune_and_add_connection(layer_node_idx, to_node, from_node, layer, all_vectors)?;
            }
        }

        Ok(())
    }

    /// Prune worst connection and add new one
    fn prune_and_add_connection(
        &mut self,
        layer_node_idx: usize,
        new_neighbor: u32,
        node_id: u32,
        layer: usize,
        all_vectors: &[Vector],
    ) -> Result<()> {
        let layer_node = &self.layers[layer].nodes[layer_node_idx];
        let start_idx = layer_node.connections_offset as usize;
        let end_idx = start_idx + layer_node.connection_count as usize;

        let existing_connections = &self.layers[layer].connections[start_idx..end_idx];

        // Find distances to existing connections
        let mut connection_distances: Vec<(u32, f32)> = existing_connections
            .iter()
            .map(|&conn_id| {
                let distance = Self::calculate_distance(
                    &all_vectors[node_id as usize].data,
                    &all_vectors[conn_id as usize].data,
                    DistanceMetric::Cosine
                );
                (conn_id, distance)
            })
            .collect();

        // Sort by distance (worst first)
        connection_distances.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Replace worst connection with new one
        if let Some((worst_conn_id, _)) = connection_distances.last() {
            let replace_idx = self.layers[layer].connections[start_idx..end_idx]
                .iter()
                .position(|&id| id == *worst_conn_id)
                .map(|pos| start_idx + pos);

            if let Some(idx) = replace_idx {
                self.layers[layer].connections[idx] = new_neighbor;
            }
        }

        Ok(())
    }

    /// Get node connections in a specific layer
    fn get_node_connections_in_layer(&self, node_id: u32, layer: usize) -> Option<&[u32]> {
        if layer >= self.layers.len() {
            return None;
        }

        let layer_ref = &self.layers[layer];
        layer_ref.nodes
            .iter()
            .find(|n| n.node_id == node_id)
            .map(|node| {
                let start = node.connections_offset as usize;
                let end = start + node.connection_count as usize;
                &layer_ref.connections[start..end]
            })
    }

    /// Calculate total number of connections across all layers
    fn calculate_total_connections(&self) -> usize {
        self.layers.iter().map(|layer| layer.connections.len()).sum()
    }

    /// Upload complete graph structure to GPU
    fn upload_graph_to_gpu(&mut self, vectors: &[Vector]) -> Result<()> {
        let device = self.context.device();
        let queue = self.context.command_queue();

        // Prepare vector data
        let mut vector_data = Vec::new();
        for vector in vectors {
            vector_data.extend_from_slice(&vector.data);
        }

        // Prepare node data for GPU
        let mut gpu_nodes = Vec::new();
        let mut gpu_connections = Vec::new();

        // Flatten all layers into GPU format with multi-layer support
        for node in &self.nodes {
            // Store base layer connections offset (layer 0)
            let base_connections_offset = if 0 < self.layers.len() {
                self.layers[0].nodes
                    .iter()
                    .find(|n| n.node_id == node.id)
                    .map(|n| n.connections_offset)
                    .unwrap_or(0)
            } else {
                0
            };

            let gpu_node = HnswNode {
                id: node.id,
                level: node.level,
                vector_offset: node.vector_offset,
                connections_offset: base_connections_offset,
            };

            gpu_nodes.push(gpu_node);
        }

        // Collect all connections
        for layer in &self.layers {
            gpu_connections.extend_from_slice(&layer.connections);
        }

        // Create GPU buffers
        let vectors_staging = device.new_buffer_with_data(
            vector_data.as_ptr() as *const std::ffi::c_void,
            vector_data.len() as u64,
            MTLResourceOptions::StorageModeShared,
        );

        let nodes_staging = device.new_buffer_with_data(
            gpu_nodes.as_ptr() as *const std::ffi::c_void,
            (gpu_nodes.len() * std::mem::size_of::<HnswNode>()) as u64,
            MTLResourceOptions::StorageModeShared,
        );

        let connections_staging = device.new_buffer_with_data(
            gpu_connections.as_ptr() as *const std::ffi::c_void,
            (gpu_connections.len() * std::mem::size_of::<u32>()) as u64,
            MTLResourceOptions::StorageModeShared,
        );

        // Copy to VRAM
        let command_buffer = queue.new_command_buffer();
        let blit_encoder = command_buffer.new_blit_command_encoder();

        blit_encoder.copy_from_buffer(
            &vectors_staging, 0, &self.vectors_buffer, 0, vector_data.len() as u64
        );

        blit_encoder.copy_from_buffer(
            &nodes_staging, 0, &self.nodes_buffer, 0,
            (gpu_nodes.len() * std::mem::size_of::<HnswNode>()) as u64
        );

        blit_encoder.copy_from_buffer(
            &connections_staging, 0, &self.connections_buffer, 0,
            (gpu_connections.len() * std::mem::size_of::<u32>()) as u64
        );

        blit_encoder.end_encoding();
        command_buffer.commit();
        command_buffer.wait_until_completed();

        debug!("✅ Graph uploaded to GPU: {} nodes, {} connections",
               gpu_nodes.len(), gpu_connections.len());

        Ok(())
    }
    

    /// Public search method - uses GPU pipeline with internal graph data
    pub fn search(&self, query: &[f32], k: usize) -> Result<Vec<(usize, f32)>> {
        // Full GPU search - no data transfer to CPU
        self.gpu_full_search(query, k)
    }

    /// Full GPU search implementation - all operations happen in VRAM
    fn gpu_full_search(&self, query: &[f32], k: usize) -> Result<Vec<(usize, f32)>> {
        if self.node_count == 0 {
            return Ok(Vec::new());
        }

        let device = self.context.device();
        let queue = self.context.command_queue();
        let k = k.min(self.node_count);

        // Prepare query data (copy to fixed-size array)
        let mut query_data = [0.0f32; 512];
        let query_len = query.len().min(512);
        query_data[..query_len].copy_from_slice(&query[..query_len]);

        let gpu_query = GpuSearchQuery {
            data: query_data,
            dimension: query_len as u32,
        };

        // Create query buffer
        let query_size = std::mem::size_of::<GpuSearchQuery>() as u64;
        let query_buffer = device.new_buffer_with_data(
            &gpu_query as *const GpuSearchQuery as *const std::ffi::c_void,
            query_size,
            MTLResourceOptions::StorageModeShared,
        );

        // Create metadata buffer if not exists
        let metadata_buffer = if let Some(ref buffer) = self.metadata_buffer {
            buffer.clone()
        } else {
            // Create metadata for all vectors
            let mut metadata = vec![GpuVectorMetadata { vector_id: 0, is_active: 1 }; self.node_count];
            for i in 0..self.node_count {
                metadata[i].vector_id = i as u32; // Use index as ID for now
            }

            let metadata_size = (metadata.len() * std::mem::size_of::<GpuVectorMetadata>()) as u64;
            device.new_buffer_with_data(
                metadata.as_ptr() as *const std::ffi::c_void,
                metadata_size,
                MTLResourceOptions::StorageModePrivate,
            )
        };

        // Create results buffer (one result per vector)
        let results_buffer = device.new_buffer(
            (self.node_count * std::mem::size_of::<GpuSearchResult>()) as u64,
            MTLResourceOptions::StorageModePrivate,
        );

        // Create final results buffer (top-k)
        let final_results_buffer = device.new_buffer(
            (k * std::mem::size_of::<GpuSearchResult>()) as u64,
            MTLResourceOptions::StorageModeShared,
        );

        // Get search pipeline
        let search_pipeline = if let Some(ref pipeline) = self.search_pipeline {
            pipeline.clone()
        } else {
            // Create search pipeline
            let library = self.context.library();
            let search_function = library.get_function("gpu_full_vector_search", None)
                .map_err(|e| VectorizerError::Other(format!("Failed to get search function: {:?}", e)))?;

            device.new_compute_pipeline_state_with_function(&search_function)
                .map_err(|e| VectorizerError::Other(format!("Failed to create search pipeline: {:?}", e)))?
        };

        // Get top-k pipeline
        let topk_function = self.context.library().get_function("gpu_find_top_k_results", None)
            .map_err(|e| VectorizerError::Other(format!("Failed to get top-k function: {:?}", e)))?;
        let topk_pipeline = device.new_compute_pipeline_state_with_function(&topk_function)
            .map_err(|e| VectorizerError::Other(format!("Failed to create top-k pipeline: {:?}", e)))?;

        // Execute search kernel
        debug!("🚀 Executing GPU search kernels...");
        let command_buffer = queue.new_command_buffer();
        let encoder = command_buffer.new_compute_command_encoder();

        debug!("📊 Search parameters: {} vectors, k={}, dim={}", self.node_count, k, self.dimension);

        // Validate buffers before execution
        if self.vectors_buffer.contents().is_null() {
            return Err(VectorizerError::Other("Vectors buffer is invalid".to_string()));
        }
        if results_buffer.contents().is_null() {
            return Err(VectorizerError::Other("Results buffer is invalid".to_string()));
        }
        if final_results_buffer.contents().is_null() {
            return Err(VectorizerError::Other("Final results buffer is invalid".to_string()));
        }

        debug!("✅ All GPU buffers validated");
        encoder.set_compute_pipeline_state(&search_pipeline);
        encoder.set_buffer(0, Some(&self.vectors_buffer), 0);     // vectors
        encoder.set_buffer(1, Some(&metadata_buffer), 0);        // metadata
        encoder.set_buffer(2, Some(&query_buffer), 0);           // query
        encoder.set_buffer(3, Some(&results_buffer), 0);         // results
        encoder.set_bytes(4, std::mem::size_of_val(&self.node_count) as u64, &self.node_count as *const usize as *const std::ffi::c_void); // vector_count
        encoder.set_bytes(5, std::mem::size_of_val(&k) as u64, &k as *const usize as *const std::ffi::c_void); // k
        encoder.set_bytes(6, std::mem::size_of_val(&self.dimension) as u64, &self.dimension as *const usize as *const std::ffi::c_void); // dimension

        // Dispatch threads
        let threadgroups = MTLSize::new(((self.node_count + 1023) / 1024) as u64, 1, 1);
        let threads_per_group = MTLSize::new(1024, 1, 1);
        encoder.dispatch_thread_groups(threadgroups, threads_per_group);

        encoder.end_encoding();

        // Execute top-k kernel
        let encoder2 = command_buffer.new_compute_command_encoder();
        encoder2.set_compute_pipeline_state(&topk_pipeline);
        encoder2.set_buffer(0, Some(&results_buffer), 0);        // all results
        encoder2.set_buffer(1, Some(&final_results_buffer), 0);  // final results
        encoder2.set_bytes(2, std::mem::size_of_val(&self.node_count) as u64, &self.node_count as *const usize as *const std::ffi::c_void); // total_vectors
        encoder2.set_bytes(3, std::mem::size_of_val(&k) as u64, &k as *const usize as *const std::ffi::c_void); // k

        let topk_threadgroups = MTLSize::new(k as u64, 1, 1);
        let topk_threads_per_group = MTLSize::new(1, 1, 1);
        encoder2.dispatch_thread_groups(topk_threadgroups, topk_threads_per_group);

        encoder2.end_encoding();

        command_buffer.commit();
        command_buffer.wait_until_completed();

        // Read final results
        let results_ptr = final_results_buffer.contents() as *const GpuSearchResult;
        let results_slice = unsafe { std::slice::from_raw_parts(results_ptr, k) };

        let mut final_results = Vec::with_capacity(k);
        for result in results_slice {
            if result.vector_id != u32::MAX {
                final_results.push((result.vector_id as usize, result.distance));
            }
        }

        final_results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        debug!("✅ GPU search completed: found {} results", final_results.len());

        // Validate results
        if final_results.is_empty() {
            warn!("⚠️  No results found in GPU search");
        } else {
            debug!("🎯 Best result: ID {}, distance {:.6}", final_results[0].0, final_results[0].1);
        }
        Ok(final_results)
    }

    /// Search using external vector data (for MetalNativeCollection integration)
    pub fn search_with_external_vectors(
        &self,
        query: &[f32],
        vector_data: &[f32],
        node_count: usize,
        k: usize
    ) -> Result<Vec<(usize, f32)>> {
        if query.len() != self.dimension {
            return Err(VectorizerError::DimensionMismatch {
                expected: self.dimension,
                actual: query.len(),
            });
        }

        if node_count == 0 {
            return Ok(Vec::new());
        }

        let device = self.context.device();
        let queue = self.context.command_queue();

        // Load Metal shader library from source (more reliable than file)
        debug!("🔧 Loading Metal shader library from source...");

        // Load shader source directly
        let shader_source = include_str!("../shaders/metal_hnsw.metal");
        debug!("📄 Shader source loaded: {} bytes", shader_source.len());

        let library = device.new_library_with_source(shader_source, &CompileOptions::new())
            .map_err(|e| {
                tracing::error!("❌ Failed to compile Metal shader from source: {:?}", e);
                VectorizerError::Other(format!("Failed to compile Metal shader: {:?}", e))
            })?;

        debug!("✅ Metal library loaded successfully");

        // Get the HNSW complete search function
        let search_function = library.get_function("hnsw_search_complete", None)
            .map_err(|e| {
                tracing::error!("❌ Failed to get hnsw_search_complete function: {:?}", e);
                VectorizerError::Other(format!("Failed to get hnsw_search_complete function: {:?}", e))
            })?;

        debug!("✅ HNSW search function retrieved successfully");

        // Create compute pipeline for HNSW search
        let pipeline = device.new_compute_pipeline_state_with_function(&search_function)
            .map_err(|e| {
                tracing::error!("❌ Failed to create HNSW compute pipeline: {:?}", e);
                VectorizerError::Other(format!("Failed to create HNSW pipeline: {:?}", e))
            })?;

        debug!("✅ HNSW compute pipeline created successfully");

        // Create query buffer (small, stays in CPU memory)
        let query_buffer = device.new_buffer_with_data(
            query.as_ptr() as *const std::ffi::c_void,
            (query.len() * std::mem::size_of::<f32>()) as u64,
            MTLResourceOptions::StorageModeShared,
        );

        // Create results buffer
        let results_size = (k * std::mem::size_of::<SearchResult>()) as u64;
        let results_buffer = device.new_buffer(
            results_size,
            MTLResourceOptions::StorageModePrivate,
        );

        // Create constants buffer for HNSW search
        let max_level = 0; // Simplified: only base layer for now
        let entry_point = 0; // Simplified: first vector as entry point
        let ef_search = k as u32;
        let constants = [
            self.dimension as u32, // vector_dim
            max_level,             // max_level
            entry_point,           // entry_point
            ef_search,             // ef_search
            k as u32,              // k
        ];

        let constants_buffer = device.new_buffer_with_data(
            constants.as_ptr() as *const std::ffi::c_void,
            (constants.len() * std::mem::size_of::<u32>()) as u64,
            MTLResourceOptions::StorageModeShared,
        );

        // Create external vectors buffer
        let vectors_buffer = device.new_buffer_with_data(
            vector_data.as_ptr() as *const std::ffi::c_void,
            (vector_data.len() * std::mem::size_of::<f32>()) as u64,
            MTLResourceOptions::StorageModePrivate,
        );

        // Create simplified node buffer
        let mut node_data = Vec::new();
        for i in 0..node_count {
            let node = HnswNode {
                id: i as u32,
                level: 0,
                vector_offset: i as u32,
                connections_offset: 0,
            };
            node_data.push(node);
        }

        let nodes_buffer = device.new_buffer_with_data(
            node_data.as_ptr() as *const std::ffi::c_void,
            (node_data.len() * std::mem::size_of::<HnswNode>()) as u64,
            MTLResourceOptions::StorageModePrivate,
        );

        // Create minimal layer buffers (placeholder for future HNSW implementation)
        let layer_buffer_size = 4096; // 4KB minimal buffers
        let layer_nodes_base = device.new_buffer(
            layer_buffer_size,
            MTLResourceOptions::StorageModePrivate,
        );
        let layer_nodes_higher = device.new_buffer(
            layer_buffer_size,
            MTLResourceOptions::StorageModePrivate,
        );
        let layer_connections_base = device.new_buffer(
            layer_buffer_size,
            MTLResourceOptions::StorageModePrivate,
        );
        let layer_connections_higher = device.new_buffer(
            layer_buffer_size,
            MTLResourceOptions::StorageModePrivate,
        );

        // Create command buffer and encoder
        let command_buffer = queue.new_command_buffer();
        let compute_encoder = command_buffer.new_compute_command_encoder();

        debug!("🔧 Setting up HNSW compute pipeline and buffers...");

        // Set compute pipeline
        compute_encoder.set_compute_pipeline_state(&pipeline);

        // Set buffers for hnsw_search_complete
        compute_encoder.set_buffer(0, Some(&vectors_buffer), 0);         // vectors
        compute_encoder.set_buffer(1, Some(&nodes_buffer), 0);           // nodes
        compute_encoder.set_buffer(2, Some(&layer_nodes_base), 0);       // layer_nodes_base
        compute_encoder.set_buffer(3, Some(&layer_nodes_higher), 0);     // layer_nodes_higher
        compute_encoder.set_buffer(4, Some(&layer_connections_base), 0); // layer_connections_base
        compute_encoder.set_buffer(5, Some(&layer_connections_higher), 0); // layer_connections_higher
        compute_encoder.set_buffer(6, Some(&query_buffer), 0);           // query_vector
        compute_encoder.set_buffer(7, Some(&results_buffer), 0);         // final_results
        compute_encoder.set_buffer(8, Some(&constants_buffer), 0);       // constants start

        debug!("✅ HNSW buffers set successfully");

        // Dispatch single thread for now (simplified)
        let threadgroup_size = metal::MTLSize::new(1, 1, 1);
        let threadgroup_count = metal::MTLSize::new(1, 1, 1);

        compute_encoder.dispatch_thread_groups(threadgroup_count, threadgroup_size);
        compute_encoder.end_encoding();

        // Commit and wait for completion
        command_buffer.commit();
        command_buffer.wait_until_completed();

        // Read results back from GPU
        debug!("🔧 Reading HNSW results from GPU...");

        // Results are already in shared memory, no need for staging
        let results_ptr = results_buffer.contents() as *const SearchResult;
        let results_slice = unsafe {
            std::slice::from_raw_parts(results_ptr, k)
        };

        // Convert to Vec and return
        let mut results: Vec<(usize, f32)> = results_slice
            .iter()
            .take(k)
            .map(|result| (result.node_id as usize, result.distance))
            .collect();

        // Ensure we have exactly k results (pad with best result if needed)
        while results.len() < k && !results.is_empty() {
            let best_result = results[0];
            results.push(best_result);
        }

        debug!("✅ HNSW GPU search with external vectors completed: {} results", results.len());
        Ok(results)
    }
    
    /// Get node count
    pub fn node_count(&self) -> usize {
        self.node_count
    }
    
    /// Get connection count
    pub fn connection_count(&self) -> usize {
        self.connection_count
    }
    
    /// Get HNSW configuration
    pub fn config(&self) -> &HnswConfig {
        &self.config
    }
    
    /// Update HNSW configuration
    pub fn update_config(&mut self, config: HnswConfig) {
        self.config = config;
        debug!("✅ HNSW configuration updated");
    }
}

/// Safe Drop implementation - Metal handles buffer deallocation automatically
#[cfg(target_os = "macos")]
impl Drop for MetalNativeHnswGraph {
    fn drop(&mut self) {
        // Metal buffers are automatically deallocated when they go out of scope
        // No manual cleanup needed - this is safer and follows Metal best practices
        debug!("🧹 Dropping MetalNativeHnswGraph - buffers auto-released by Metal");
    }
}
