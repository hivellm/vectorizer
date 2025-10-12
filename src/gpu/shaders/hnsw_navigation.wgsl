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

// ✅ FIXED: Separate functions for query vs vector comparisons
fn cosine_similarity_query_to_vector(query_offset: u32, vector_offset: u32, dimension: u32) -> f32 {
    var dot: f32 = 0.0;
    var norm_query: f32 = 0.0;
    var norm_vector: f32 = 0.0;
    
    for (var i: u32 = 0u; i < dimension; i++) {
        let val_query = query_buffer[query_offset + i];  // ✅ From query_buffer
        let val_vector = vector_buffer[vector_offset + i];
        dot += val_query * val_vector;
        norm_query += val_query * val_query;
        norm_vector += val_vector * val_vector;
    }
    
    norm_query = sqrt(norm_query);
    norm_vector = sqrt(norm_vector);
    
    if (norm_query == 0.0 || norm_vector == 0.0) {
        return 0.0;
    }
    
    return dot / (norm_query * norm_vector);
}

fn euclidean_distance_query_to_vector(query_offset: u32, vector_offset: u32, dimension: u32) -> f32 {
    var sum_squared_diff: f32 = 0.0;
    
    for (var i: u32 = 0u; i < dimension; i++) {
        let val_query = query_buffer[query_offset + i];  // ✅ From query_buffer
        let val_vector = vector_buffer[vector_offset + i];
        let diff = val_query - val_vector;
        sum_squared_diff += diff * diff;
    }
    
    return sqrt(sum_squared_diff);
}

fn dot_product_query_to_vector(query_offset: u32, vector_offset: u32, dimension: u32) -> f32 {
    var dot: f32 = 0.0;
    
    for (var i: u32 = 0u; i < dimension; i++) {
        let val_query = query_buffer[query_offset + i];  // ✅ From query_buffer
        let val_vector = vector_buffer[vector_offset + i];
        dot += val_query * val_vector;
    }
    
    return dot;
}

// Legacy functions for vector-to-vector comparison (kept for compatibility)
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
            let sim = cosine_similarity_query_to_vector(query_offset, vector_offset, dimension);
            return 1.0 - sim;  // ✅ Convert similarity to distance
        }
        case 1u: { // Euclidean distance
            return euclidean_distance_query_to_vector(query_offset, vector_offset, dimension);
        }
        case 2u: { // Dot product (treat as similarity, convert to distance)
            let sim = dot_product_query_to_vector(query_offset, vector_offset, dimension);
            return -sim;  // ✅ Negative dot product as distance (higher is better)
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
            entry_point = i;  // ✅ FIXED: Use index, not node.id
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

// ============================================================================
// Score Calculation Functions
// ============================================================================

/// Calculate similarity score from distance (normalized to 0-1)
fn calculate_score(distance: f32, metric_type: u32) -> f32 {
    switch (metric_type) {
        case 0u: {  // Cosine similarity
            // distance = 1 - cosine_sim, so score = 1 - distance
            return 1.0 - distance;
        }
        case 1u: {  // Euclidean distance
            // Convert distance to similarity (0 to 1)
            // Using inverse function normalized
            return 1.0 / (1.0 + distance);
        }
        case 2u: {  // Dot product
            // distance = -dot_product, so score = -distance
            // Normalize to 0-1 assuming normalized vectors
            let dot_product = -distance;
            return (dot_product + 1.0) / 2.0;
        }
        default: {
            return 0.0;
        }
    }
}

// ============================================================================
// Candidate and Result Management
// ============================================================================

/// Candidate list entry
struct Candidate {
    node_id: u32,
    distance: f32,
    score: f32,
    visited: u32,
}

/// Result set entry
struct ScoredResult {
    node_id: u32,
    score: f32,
    distance: f32,
    vector_offset: u32,
}

/// Visited set using bitfield (32 bits per u32) with atomic operations
var<workgroup> visited_bits: array<atomic<u32>, 1024>;  // Supports up to 32,768 nodes

fn mark_visited(node_id: u32) {
    let word_idx = node_id / 32u;
    let bit_idx = node_id % 32u;
    
    if (word_idx < 1024u) {
        atomicOr(&visited_bits[word_idx], 1u << bit_idx);
    }
}

fn is_visited(node_id: u32) -> bool {
    let word_idx = node_id / 32u;
    let bit_idx = node_id % 32u;
    
    if (word_idx >= 1024u) {
        return false;
    }
    
    let word = atomicLoad(&visited_bits[word_idx]);
    return bool((word >> bit_idx) & 1u);
}

fn clear_visited() {
    for (var i: u32 = 0u; i < 1024u; i++) {
        atomicStore(&visited_bits[i], 0u);
    }
}

/// Insert candidate into sorted list (min-heap style)
fn insert_candidate(
    candidates: ptr<function, array<Candidate, 200>>,
    count: ptr<function, u32>,
    node_id: u32,
    distance: f32,
    score: f32
) {
    let capacity = 200u; // ef_search max
    
    // If list is not full, add to end
    if (*count) < capacity {
        (*candidates)[*count] = Candidate(node_id, distance, score, 0u);
        (*count)++;
        return;
    }
    
    // If list is full, replace worst if new candidate is better
    var worst_idx: u32 = 0u;
    var worst_distance: f32 = (*candidates)[0].distance;
    
    for (var i: u32 = 1u; i < capacity; i++) {
        if ((*candidates)[i].distance > worst_distance) {
            worst_distance = (*candidates)[i].distance;
            worst_idx = i;
        }
    }
    
    if (distance < worst_distance) {
        (*candidates)[worst_idx] = Candidate(node_id, distance, score, 0u);
    }
}

/// Get closest unvisited candidate
fn get_closest_unvisited(
    candidates: ptr<function, array<Candidate, 200>>,
    count: u32
) -> Candidate {
    var best_idx: u32 = 0u;
    var best_distance: f32 = 1e10;
    var found = false;
    
    for (var i: u32 = 0u; i < count; i++) {
        let candidate = (*candidates)[i];
        
        if (!bool(candidate.visited) && candidate.distance < best_distance) {
            best_distance = candidate.distance;
            best_idx = i;
            found = true;
        }
    }
    
    if (found) {
        (*candidates)[best_idx].visited = 1u;
        return (*candidates)[best_idx];
    }
    
    // Return invalid candidate
    return Candidate(0xFFFFFFFFu, 1e10, 0.0, 1u);
}

/// Insert into result set (keep best k)
fn insert_result(
    results: ptr<function, array<ScoredResult, 100>>,
    count: ptr<function, u32>,
    node_id: u32,
    distance: f32,
    score: f32,
    vector_offset: u32
) {
    let k = params.k;
    
    // If list is not full, add to end and sort
    if (*count) < k {
        (*results)[*count] = ScoredResult(node_id, score, distance, vector_offset);
        (*count)++;
        
        // Insertion sort
        var i = (*count) - 1u;
        while (i > 0u && (*results)[i].score > (*results)[i - 1u].score) {
            let temp = (*results)[i];
            (*results)[i] = (*results)[i - 1u];
            (*results)[i - 1u] = temp;
            i--;
        }
        return;
    }
    
    // If list is full, replace worst if new result is better
    if (score > (*results)[k - 1u].score) {
        (*results)[k - 1u] = ScoredResult(node_id, score, distance, vector_offset);
        
        // Bubble up
        var i = k - 1u;
        while (i > 0u && (*results)[i].score > (*results)[i - 1u].score) {
            let temp = (*results)[i];
            (*results)[i] = (*results)[i - 1u];
            (*results)[i - 1u] = temp;
            i--;
        }
    }
}

// ============================================================================
// Complete k-NN Search Implementation
// ============================================================================

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // Clear visited set
    clear_visited();
    
    // Query vector starts at offset 0 in query_buffer
    let query_offset: u32 = 0u;
    
    // Find entry point (highest level node)
    let entry_point = find_entry_point();
    
    if (entry_point >= params.node_count) {
        return; // No nodes in graph
    }
    
    // Start from the top level and work down
    var current_node = entry_point;
    let top_node = get_node_at(current_node);
    
    // Greedy search at each level from top to bottom (zoom in)
    for (var level: u32 = top_node.level; level > 0u; level--) {
        current_node = greedy_search(query_offset, current_node, level);
    }
    
    // ========================================================================
    // k-NN Search at Level 0 with ef_search
    // ========================================================================
    
    // Initialize candidate list and result set
    var candidates: array<Candidate, 200>; // ef_search max
    var candidate_count: u32 = 0u;
    
    var results: array<ScoredResult, 100>; // k max
    var result_count: u32 = 0u;
    
    // Add entry point to candidates
    let entry_node = get_node_at(current_node);
    let entry_distance = calculate_distance_with_offset(
        query_offset,
        entry_node.vector_offset,
        params.dimension,
        params.metric_type
    );
    let entry_score = calculate_score(entry_distance, params.metric_type);
    
    insert_candidate(&candidates, &candidate_count, current_node, entry_distance, entry_score);
    insert_result(&results, &result_count, current_node, entry_distance, entry_score, entry_node.vector_offset);
    mark_visited(current_node);
    
    // Main search loop
    var iterations: u32 = 0u;
    let max_iterations = params.ef_search * 2u; // Safety limit
    
    while (iterations < max_iterations) {
        iterations++;
        
        // Get closest unvisited candidate
        let candidate = get_closest_unvisited(&candidates, candidate_count);
        
        if (candidate.node_id == 0xFFFFFFFFu) {
            break; // No more unvisited candidates
        }
        
        // Check stopping condition: 
        // If candidate is farther than worst result and we have k results, stop
        if (result_count >= params.k) {
            let worst_score = results[result_count - 1u].score;
            if (candidate.score < worst_score) {
                break;
            }
        }
        
        // Explore neighbors of candidate
        let candidate_node = get_node_at(candidate.node_id);
        
        for (var i: u32 = 0u; i < candidate_node.connection_count && i < params.max_connections; i++) {
            let connection_base = candidate.node_id * params.max_connections + i;
            let neighbor_id = connection_buffer[connection_base];
            
            if (neighbor_id >= params.node_count) {
                continue; // Invalid connection
            }
            
            if (is_visited(neighbor_id)) {
                continue; // Already visited
            }
            
            mark_visited(neighbor_id);
            
            // Calculate distance to neighbor
            let neighbor = get_node_at(neighbor_id);
            let neighbor_distance = calculate_distance_with_offset(
                query_offset,
                neighbor.vector_offset,
                params.dimension,
                params.metric_type
            );
            let neighbor_score = calculate_score(neighbor_distance, params.metric_type);
            
            // Add to candidates if promising
            let worst_result_score = select(0.0, results[result_count - 1u].score, result_count >= params.k);
            
            if (result_count < params.k || neighbor_score > worst_result_score) {
                insert_candidate(&candidates, &candidate_count, neighbor_id, neighbor_distance, neighbor_score);
                insert_result(&results, &result_count, neighbor_id, neighbor_distance, neighbor_score, neighbor.vector_offset);
            }
        }
    }
    
    // ========================================================================
    // Write Results
    // ========================================================================
    
    // Results buffer format: [ScoredResult, ScoredResult, ...]
    // Each ScoredResult = [node_id, score, distance, vector_offset] = 4 u32
    for (var i: u32 = 0u; i < result_count && i < params.k; i++) {
        let result = results[i];
        let base = i * 4u;
        
        results_buffer[base + 0u] = result.node_id;
        // Cast score and distance as u32 (reinterpret bits)
        results_buffer[base + 1u] = bitcast<u32>(result.score);
        results_buffer[base + 2u] = bitcast<u32>(result.distance);
        results_buffer[base + 3u] = result.vector_offset;
    }
    
    // Mark end of results with sentinel
    if (result_count < params.k) {
        results_buffer[result_count * 4u] = 0xFFFFFFFFu;
    }
}
