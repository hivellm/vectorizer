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

fn get_vector_at(offset: u32, dimension: u32) -> array<f32> {
    // This is a simplified version - in reality we'd need to handle
    // variable-length arrays differently in WGSL
    var vector: array<f32, 768>; // Max dimension
    for (var i: u32 = 0u; i < dimension; i++) {
        vector[i] = vector_buffer[offset + i];
    }
    return vector;
}

fn cosine_similarity(a: array<f32>, b: array<f32>, dimension: u32) -> f32 {
    var dot: f32 = 0.0;
    var norm_a: f32 = 0.0;
    var norm_b: f32 = 0.0;
    
    for (var i: u32 = 0u; i < dimension; i++) {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }
    
    norm_a = sqrt(norm_a);
    norm_b = sqrt(norm_b);
    
    if (norm_a == 0.0 || norm_b == 0.0) {
        return 0.0;
    }
    
    return dot / (norm_a * norm_b);
}

fn euclidean_distance(a: array<f32>, b: array<f32>, dimension: u32) -> f32 {
    var sum_squared_diff: f32 = 0.0;
    
    for (var i: u32 = 0u; i < dimension; i++) {
        let diff = a[i] - b[i];
        sum_squared_diff += diff * diff;
    }
    
    return sqrt(sum_squared_diff);
}

fn dot_product(a: array<f32>, b: array<f32>, dimension: u32) -> f32 {
    var dot: f32 = 0.0;
    
    for (var i: u32 = 0u; i < dimension; i++) {
        dot += a[i] * b[i];
    }
    
    return dot;
}

fn calculate_distance(query: array<f32>, vector: array<f32>, dimension: u32, metric_type: u32) -> f32 {
    switch (metric_type) {
        case 0u: { // Cosine similarity
            return cosine_similarity(query, vector, dimension);
        }
        case 1u: { // Euclidean distance
            return euclidean_distance(query, vector, dimension);
        }
        case 2u: { // Dot product
            return dot_product(query, vector, dimension);
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

fn greedy_search(query: array<f32>, entry_point: u32, level: u32) -> u32 {
    var current_node: u32 = entry_point;
    var improved: bool = true;
    
    while (improved) {
        improved = false;
        
        // Get current node
        let node = get_node_at(current_node);
        let current_vector = get_vector_at(node.vector_offset, params.dimension);
        let current_distance = calculate_distance(query, current_vector, params.dimension, params.metric_type);
        
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
            
            let neighbor_vector = get_vector_at(neighbor.vector_offset, params.dimension);
            let neighbor_distance = calculate_distance(query, neighbor_vector, params.dimension, params.metric_type);
            
            // For similarity metrics (cosine, dot), we want higher values
            // For distance metrics (euclidean), we want lower values
            let is_better = if (params.metric_type == 1u) { // Euclidean
                neighbor_distance < current_distance
            } else { // Cosine or Dot Product
                neighbor_distance > current_distance
            };
            
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
    // Initialize query vector from buffer
    var query: array<f32, 768>; // Max dimension
    for (var i: u32 = 0u; i < params.dimension; i++) {
        query[i] = query_buffer[i];
    }
    
    // Find entry point (highest level node)
    let entry_point = find_entry_point();
    
    // Start from the top level and work down
    var current_node = entry_point;
    let top_node = get_node_at(current_node);
    
    // Greedy search at each level from top to bottom
    for (var level: u32 = top_node.level; level > 0u; level--) {
        current_node = greedy_search(query, current_node, level);
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
