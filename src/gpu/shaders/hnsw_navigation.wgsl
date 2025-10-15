//! GPU HNSW Navigation Shader
//!
//! This compute shader implements HNSW graph navigation entirely on GPU,
//! including hierarchical traversal and neighbor discovery.

// Navigation parameters
struct NavigationParams {
    dimension: u32,
    k: u32,
    ef_search: u32,
    max_connections: u32,
    node_count: u32,
    metric_type: u32, // 0=cosine, 1=euclidean, 2=dot_product
    _padding: u32,
}

// HNSW Node structure
struct HnswNode {
    id: u32,
    level: u32,
    connection_count: u32,
    vector_offset: u32,
}

// Bind group layout
@group(0) @binding(0)
var<uniform> params: NavigationParams;

@group(0) @binding(1)
var<storage, read> query_buffer: array<f32>;

@group(0) @binding(2)
var<storage, read> node_buffer: array<u32>; // Packed HnswNode data

@group(0) @binding(3)
var<storage, read> vector_buffer: array<f32>;

@group(0) @binding(4)
var<storage, read> connection_buffer: array<u32>;

@group(0) @binding(5)
var<storage, read_write> results_buffer: array<u32>;

@group(0) @binding(6)
var<storage, read_write> candidate_buffer: array<u32>;

// Helper functions
fn get_node_at(index: u32) -> HnswNode {
    let base = index * 4u; // 4 u32s per node
    return HnswNode(
        node_buffer[base + 0u],
        node_buffer[base + 1u], 
        node_buffer[base + 2u],
        node_buffer[base + 3u]
    );
}

fn get_vector_component(offset: u32, index: u32) -> f32 {
    return vector_buffer[offset + index];
}

fn cosine_similarity_with_offset(offset_a: u32, offset_b: u32, dimension: u32) -> f32 {
    var dot: f32 = 0.0;
    var norm_a: f32 = 0.0;
    var norm_b: f32 = 0.0;
    
    for (var i: u32 = 0u; i < dimension; i++) {
        let val_a = vector_buffer[offset_a + i];
        let val_b = vector_buffer[offset_b + i];
        dot += val_a * val_b;
        norm_a += val_a * val_a;
        norm_b += val_b * val_b;
    }
    
    norm_a = sqrt(norm_a);
    norm_b = sqrt(norm_b);
    
    if (norm_a == 0.0 || norm_b == 0.0) {
        return 0.0;
    }
    
    return dot / (norm_a * norm_b);
}

fn euclidean_distance_with_offset(offset_a: u32, offset_b: u32, dimension: u32) -> f32 {
    var sum_squared_diff: f32 = 0.0;
    
    for (var i: u32 = 0u; i < dimension; i++) {
        let diff = vector_buffer[offset_a + i] - vector_buffer[offset_b + i];
        sum_squared_diff += diff * diff;
    }
    
    return sqrt(sum_squared_diff);
}

fn dot_product_with_offset(offset_a: u32, offset_b: u32, dimension: u32) -> f32 {
    var dot: f32 = 0.0;
    
    for (var i: u32 = 0u; i < dimension; i++) {
        dot += vector_buffer[offset_a + i] * vector_buffer[offset_b + i];
    }
    
    return dot;
}

fn calculate_distance_with_offset(query_offset: u32, vector_offset: u32, dimension: u32, metric_type: u32) -> f32 {
    switch (metric_type) {
        case 0u: { // Cosine similarity
            return cosine_similarity_with_offset(query_offset, vector_offset, dimension);
        }
        case 1u: { // Euclidean distance
            return euclidean_distance_with_offset(query_offset, vector_offset, dimension);
        }
        case 2u: { // Dot product
            return dot_product_with_offset(query_offset, vector_offset, dimension);
        }
        default: {
            return 0.0;
        }
    }
}

fn find_entry_point() -> u32 {
    // Find the highest level node (entry point)
    var entry_point: u32 = 0u;
    var max_level: u32 = 0u;
    
    for (var i: u32 = 0u; i < params.node_count; i++) {
        let node = get_node_at(i);
        if (node.level > max_level) {
            max_level = node.level;
            entry_point = node.id;
        }
    }
    
    return entry_point;
}

fn greedy_search(query_offset: u32, entry_point: u32, level: u32) -> u32 {
    var current_node: u32 = entry_point;
    var improved: bool = true;
    
    while (improved) {
        improved = false;
        
        // Get current node
        let node = get_node_at(current_node);
        let current_distance = calculate_distance_with_offset(query_offset, node.vector_offset, params.dimension, params.metric_type);
        
        // Check all connections at this level
        for (var i: u32 = 0u; i < node.connection_count; i++) {
            let connection_base = current_node * params.max_connections + i;
            let neighbor_id = connection_buffer[connection_base];
            
            if (neighbor_id >= params.node_count) {
                continue; // Invalid connection
            }
            
            let neighbor = get_node_at(neighbor_id);
            if (neighbor.level < level) {
                continue; // Skip lower level connections
            }
            
            let neighbor_distance = calculate_distance_with_offset(query_offset, neighbor.vector_offset, params.dimension, params.metric_type);
            
            // For similarity metrics (cosine, dot), we want higher values
            // For distance metrics (euclidean), we want lower values
            var is_better: bool;
            if (params.metric_type == 1u) { // Euclidean
                is_better = neighbor_distance < current_distance;
            } else { // Cosine or Dot Product
                is_better = neighbor_distance > current_distance;
            }
            
            if (is_better) {
                current_node = neighbor_id;
                improved = true;
                break;
            }
        }
    }
    
    return current_node;
}

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // Query vector starts at offset 0 in query_buffer
    let query_offset: u32 = 0u;
    
    // Find entry point (highest level node)
    let entry_point = find_entry_point();
    
    // Start from the top level and work down
    var current_node = entry_point;
    let top_node = get_node_at(current_node);
    
    // Greedy search at each level from top to bottom
    for (var level: u32 = top_node.level; level > 0u; level--) {
        current_node = greedy_search(query_offset, current_node, level);
    }
    
    // Final search at level 0 with ef_search
    // This is a simplified implementation - in practice we'd need
    // a more sophisticated candidate management system
    
    // For now, just return the current node as the best result
    if (params.k > 0u) {
        results_buffer[0] = current_node;
    }
    
    // TODO: Implement proper k-NN search with ef_search parameter
    // This would involve:
    // 1. Maintaining a candidate list
    // 2. Exploring neighbors of candidates
    // 3. Selecting the best k results
}
