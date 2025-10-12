//! GPU HNSW Graph Construction Shader (v2 - Refactored)
//!
//! Refactored based on WGSL spec from Context7:
//! - No ptr<storage> in function parameters (not allowed in WGSL)
//! - Uses var<workgroup> for temporary data
//! - Inline logic for array operations
//!
//! Key changes from v1:
//! - Removed insert_candidate() function -> inlined
//! - Removed select_neighbors_heuristic() function -> simplified inline version
//! - Uses workgroup memory for candidate lists
//! - Direct storage buffer access without pointer passing

/// Construction parameters
struct ConstructionParams {
    dimension: u32,            // Vector dimension
    m: u32,                    // Max connections per node
    m0: u32,                   // Max connections at level 0
    ml: f32,                   // Level multiplier
    ef_construction: u32,      // Size of dynamic candidate list
    node_count: u32,           // Total number of nodes
    current_level: u32,        // Current level being constructed
    metric_type: u32,          // 0=cosine, 1=euclidean, 2=dot_product
}

/// HNSW Node structure
struct HnswNode {
    id: u32,
    level: u32,
    connection_count: u32,
    vector_offset: u32,
}

/// Candidate for neighbor selection
struct Candidate {
    node_id: u32,
    distance: f32,
    valid: u32,
    _padding: u32,
}

/// Bind group layout
@group(0) @binding(0) var<uniform> params: ConstructionParams;
@group(0) @binding(1) var<storage, read> vectors: array<f32>;
@group(0) @binding(2) var<storage, read_write> nodes: array<u32>;
@group(0) @binding(3) var<storage, read_write> connections: array<u32>;
@group(0) @binding(4) var<storage, read> levels: array<u32>;

// ✅ WORKGROUP MEMORY - No pointer passing needed
// Each workgroup has its own candidate list
const MAX_EF: u32 = 200u;
const MAX_M: u32 = 64u;
var<workgroup> local_candidates: array<Candidate, 200>;
var<workgroup> local_selected: array<u32, 64>;

// ============================================================================
// Helper Functions
// ============================================================================

/// Get node at index
fn get_node_at(index: u32) -> HnswNode {
    let base = index * 4u;
    return HnswNode(
        nodes[base + 0u],
        nodes[base + 1u],
        nodes[base + 2u],
        nodes[base + 3u]
    );
}

/// Set node at index
fn set_node_at(index: u32, node: HnswNode) {
    let base = index * 4u;
    nodes[base + 0u] = node.id;
    nodes[base + 1u] = node.level;
    nodes[base + 2u] = node.connection_count;
    nodes[base + 3u] = node.vector_offset;
}

/// Get connection at specific index for node
fn get_connection(node_id: u32, connection_idx: u32, level: u32) -> u32 {
    let level_0_size = params.m0;
    let level_n_size = params.m;
    
    var base_offset: u32;
    if (level == 0u) {
        base_offset = node_id * level_0_size;
    } else {
        base_offset = (node_id * level_0_size) + 
                     (level * params.node_count * level_n_size) + 
                     (node_id * level_n_size);
    }
    
    return connections[base_offset + connection_idx];
}

/// Set connection for node at level
fn set_connection(node_id: u32, connection_idx: u32, level: u32, neighbor_id: u32) {
    var base_offset: u32;
    if (level == 0u) {
        base_offset = node_id * params.m0;
    } else {
        base_offset = (node_id * params.m0) + 
                     (level * params.node_count * params.m) + 
                     (node_id * params.m);
    }
    
    connections[base_offset + connection_idx] = neighbor_id;
}

// ============================================================================
// Distance Calculations
// ============================================================================

/// Calculate distance between two vectors
fn calculate_distance(offset_a: u32, offset_b: u32) -> f32 {
    var sum: f32 = 0.0;
    var dot: f32 = 0.0;
    var norm_a: f32 = 0.0;
    var norm_b: f32 = 0.0;
    
    let base_a = offset_a * params.dimension;
    let base_b = offset_b * params.dimension;
    
    for (var i: u32 = 0u; i < params.dimension; i++) {
        let a = vectors[base_a + i];
        let b = vectors[base_b + i];
        
        if (params.metric_type == 0u) { // Cosine
            dot += a * b;
            norm_a += a * a;
            norm_b += b * b;
        } else if (params.metric_type == 1u) { // Euclidean
            let diff = a - b;
            sum += diff * diff;
        } else { // Dot Product
            dot += a * b;
        }
    }
    
    if (params.metric_type == 0u) {
        return 1.0 - (dot / sqrt(norm_a * norm_b));
    } else if (params.metric_type == 1u) {
        return sqrt(sum);
    } else {
        return -dot;
    }
}

// ============================================================================
// INLINED OPERATIONS (No ptr<storage> parameters)
// ============================================================================

/// Initialize candidate list in workgroup memory
fn init_local_candidates(count: u32) {
    for (var i: u32 = 0u; i < count && i < MAX_EF; i++) {
        local_candidates[i] = Candidate(0xFFFFFFFFu, 1e10, 0u, 0u);
    }
}

/// Insert candidate into sorted list - INLINED VERSION
/// This replaces the old insert_candidate() function that couldn't use ptr<storage>
fn insert_into_local_candidates(node_id: u32, distance: f32, capacity: u32) {
    // Find insertion point
    var insert_pos: u32 = capacity;
    for (var i: u32 = 0u; i < capacity && i < MAX_EF; i++) {
        if (local_candidates[i].valid == 0u || distance < local_candidates[i].distance) {
            insert_pos = i;
            break;
        }
    }
    
    if (insert_pos >= capacity || insert_pos >= MAX_EF) {
        return; // List full or invalid position
    }
    
    // Shift elements to make room
    for (var i: u32 = min(capacity - 1u, MAX_EF - 1u); i > insert_pos; i--) {
        local_candidates[i] = local_candidates[i - 1u];
    }
    
    // Insert new candidate
    local_candidates[insert_pos] = Candidate(node_id, distance, 1u, 0u);
}

/// Select M best neighbors - SIMPLIFIED INLINED VERSION
/// This replaces the complex select_neighbors_heuristic() function
fn select_best_neighbors(candidate_count: u32, m: u32) -> u32 {
    var selected_count: u32 = 0u;
    
    // Simple greedy selection: take M closest candidates
    for (var i: u32 = 0u; i < candidate_count && i < MAX_EF && selected_count < m && selected_count < MAX_M; i++) {
        if (local_candidates[i].valid != 0u && local_candidates[i].node_id != 0xFFFFFFFFu) {
            local_selected[selected_count] = local_candidates[i].node_id;
            selected_count++;
        }
    }
    
    return selected_count;
}

// ============================================================================
// COMPUTE KERNELS
// ============================================================================

/// Find neighbors for each node at current level
@compute @workgroup_size(256)
fn find_neighbors(@builtin(global_invocation_id) gid: vec3<u32>) {
    let node_idx = gid.x;
    
    if (node_idx >= params.node_count) {
        return;
    }
    
    let node = get_node_at(node_idx);
    let node_level = levels[node_idx];
    
    // Only process nodes at or above current level
    if (node_level < params.current_level) {
        return;
    }
    
    // Initialize local candidate list
    init_local_candidates(params.ef_construction);
    var candidate_count: u32 = 0u;
    
    // Find candidates: scan all other nodes at this level
    for (var i: u32 = 0u; i < params.node_count; i++) {
        if (i == node_idx) {
            continue;
        }
        
        let other_node = get_node_at(i);
        let other_level = levels[i];
        
        // Skip if other node doesn't exist at this level
        if (other_level < params.current_level) {
            continue;
        }
        
        // Calculate distance
        let dist = calculate_distance(node.vector_offset, other_node.vector_offset);
        
        // ✅ INLINE insertion - no ptr<storage> needed
        insert_into_local_candidates(i, dist, params.ef_construction);
        candidate_count = min(candidate_count + 1u, params.ef_construction);
    }
    
    // Select M best neighbors using simplified greedy selection
    let m = select(params.m, params.m0, params.current_level == 0u);
    let selected_count = select_best_neighbors(candidate_count, m);
    
    // Write connections back to storage
    var node_updated = node;
    node_updated.connection_count = selected_count;
    set_node_at(node_idx, node_updated);
    
    for (var i: u32 = 0u; i < selected_count; i++) {
        set_connection(node_idx, i, params.current_level, local_selected[i]);
    }
}

/// Make connections bidirectional
@compute @workgroup_size(256)
fn make_bidirectional(@builtin(global_invocation_id) gid: vec3<u32>) {
    let node_idx = gid.x;
    
    if (node_idx >= params.node_count) {
        return;
    }
    
    let node = get_node_at(node_idx);
    let node_level = levels[node_idx];
    
    if (node_level < params.current_level) {
        return;
    }
    
    // For each connection of this node, add reverse connection
    for (var i: u32 = 0u; i < node.connection_count; i++) {
        let neighbor_id = get_connection(node_idx, i, params.current_level);
        
        if (neighbor_id < params.node_count) {
            var neighbor = get_node_at(neighbor_id);
            
            // Check if reverse connection already exists
            var has_reverse = false;
            for (var j: u32 = 0u; j < neighbor.connection_count; j++) {
                if (get_connection(neighbor_id, j, params.current_level) == node_idx) {
                    has_reverse = true;
                    break;
                }
            }
            
            // Add reverse connection if not exists and space available
            let max_connections = select(params.m, params.m0, params.current_level == 0u);
            if (!has_reverse && neighbor.connection_count < max_connections) {
                set_connection(neighbor_id, neighbor.connection_count, params.current_level, node_idx);
                neighbor.connection_count++;
                set_node_at(neighbor_id, neighbor);
            }
        }
    }
}

/// Prune connections to maintain M limit
@compute @workgroup_size(256)
fn prune_connections(@builtin(global_invocation_id) gid: vec3<u32>) {
    let node_idx = gid.x;
    
    if (node_idx >= params.node_count) {
        return;
    }
    
    var node = get_node_at(node_idx);
    let node_level = levels[node_idx];
    
    if (node_level < params.current_level) {
        return;
    }
    
    let max_connections = select(params.m, params.m0, params.current_level == 0u);
    
    // If we have too many connections, prune to M
    if (node.connection_count > max_connections) {
        // Initialize local candidates with existing neighbors
        init_local_candidates(node.connection_count);
        
        for (var i: u32 = 0u; i < node.connection_count && i < MAX_EF; i++) {
            let neighbor_id = get_connection(node_idx, i, params.current_level);
            
            if (neighbor_id < params.node_count) {
                let neighbor = get_node_at(neighbor_id);
                let dist = calculate_distance(node.vector_offset, neighbor.vector_offset);
                
                // Store in local candidates
                local_candidates[i] = Candidate(neighbor_id, dist, 1u, 0u);
            }
        }
        
        // Select best M
        let selected_count = select_best_neighbors(node.connection_count, max_connections);
        
        // Update connections
        node.connection_count = selected_count;
        set_node_at(node_idx, node);
        
        for (var i: u32 = 0u; i < selected_count; i++) {
            set_connection(node_idx, i, params.current_level, local_selected[i]);
        }
    }
}

