//! GPU Batch Construction Shader for HNSW
//!
//! This compute shader implements optimized batch processing for HNSW graph construction.
//! Processes multiple vectors simultaneously to maximize GPU parallelism.
//!
//! Key features:
//! - Parallel distance matrix computation (batch vs existing)
//! - Shared memory optimization for workgroup collaboration
//! - Efficient neighbor selection for entire batch
//! - Memory-efficient sparse matrix operations
//! - Support for large batches (1000-10000 vectors)

/// Batch construction parameters
struct BatchParams {
    batch_size: u32,           // Number of vectors in current batch
    batch_offset: u32,         // Offset in batch vector buffer
    total_nodes: u32,          // Total existing nodes in graph
    dimension: u32,            // Vector dimension
    m: u32,                    // Max connections per node
    m0: u32,                   // Max connections at level 0
    ef_construction: u32,      // Candidate list size
    metric_type: u32,          // 0=cosine, 1=euclidean, 2=dot_product
}

/// HNSW Node structure
struct HnswNode {
    id: u32,
    level: u32,
    connection_count: u32,
    vector_offset: u32,
}

/// Distance matrix entry (sparse storage)
struct DistanceEntry {
    batch_idx: u32,
    existing_idx: u32,
    distance: f32,
    valid: u32,
}

/// Bind group layout
@group(0) @binding(0) var<uniform> params: BatchParams;
@group(0) @binding(1) var<storage, read> batch_vectors: array<f32>;
@group(0) @binding(2) var<storage, read> existing_vectors: array<f32>;
@group(0) @binding(3) var<storage, read_write> nodes: array<u32>;
@group(0) @binding(4) var<storage, read_write> connections: array<u32>;
@group(0) @binding(5) var<storage, read_write> distance_matrix: array<f32>;
@group(0) @binding(6) var<storage, read_write> batch_candidates: array<DistanceEntry>;
@group(0) @binding(7) var<storage, read> levels: array<u32>;

// Shared memory for workgroup collaboration (Metal supports up to 32KB)
var<workgroup> shared_batch_vector: array<f32, 1024>;      // 4KB
var<workgroup> shared_existing_vector: array<f32, 1024>;   // 4KB
var<workgroup> shared_distances: array<f32, 256>;          // 1KB

// ============================================================================
// Helper Functions
// ============================================================================

fn get_node_at(index: u32) -> HnswNode {
    let base = index * 4u;
    return HnswNode(
        nodes[base + 0u],
        nodes[base + 1u],
        nodes[base + 2u],
        nodes[base + 3u]
    );
}

fn set_node_at(index: u32, node: HnswNode) {
    let base = index * 4u;
    nodes[base + 0u] = node.id;
    nodes[base + 1u] = node.level;
    nodes[base + 2u] = node.connection_count;
    nodes[base + 3u] = node.vector_offset;
}

// ============================================================================
// Distance Calculations (Optimized)
// ============================================================================

/// Cosine similarity using shared memory
fn cosine_similarity_shared(
    batch_offset: u32,
    existing_offset: u32,
    local_idx: u32,
    workgroup_size: u32
) -> f32 {
    var dot: f32 = 0.0;
    var norm_a: f32 = 0.0;
    var norm_b: f32 = 0.0;
    
    // Parallel reduction: cada thread processa dimension/workgroup_size elementos
    let items_per_thread = (params.dimension + workgroup_size - 1u) / workgroup_size;
    let start = local_idx * items_per_thread;
    let end = min(start + items_per_thread, params.dimension);
    
    for (var i = start; i < end; i++) {
        let val_a = batch_vectors[batch_offset + i];
        let val_b = existing_vectors[existing_offset + i];
        
        dot += val_a * val_b;
        norm_a += val_a * val_a;
        norm_b += val_b * val_b;
    }
    
    // Store partial results in shared memory
    shared_distances[local_idx * 3u + 0u] = dot;
    shared_distances[local_idx * 3u + 1u] = norm_a;
    shared_distances[local_idx * 3u + 2u] = norm_b;
    
    workgroupBarrier();
    
    // Reduction: combine partial results (only thread 0)
    if (local_idx == 0u) {
        var total_dot: f32 = 0.0;
        var total_norm_a: f32 = 0.0;
        var total_norm_b: f32 = 0.0;
        
        for (var i: u32 = 0u; i < workgroup_size; i++) {
            total_dot += shared_distances[i * 3u + 0u];
            total_norm_a += shared_distances[i * 3u + 1u];
            total_norm_b += shared_distances[i * 3u + 2u];
        }
        
        total_norm_a = sqrt(total_norm_a);
        total_norm_b = sqrt(total_norm_b);
        
        if (total_norm_a > 0.0 && total_norm_b > 0.0) {
            shared_distances[0] = 1.0 - (total_dot / (total_norm_a * total_norm_b));
        } else {
            shared_distances[0] = 1.0;
        }
    }
    
    workgroupBarrier();
    return shared_distances[0];
}

/// Euclidean distance using shared memory
fn euclidean_distance_shared(
    batch_offset: u32,
    existing_offset: u32,
    local_idx: u32,
    workgroup_size: u32
) -> f32 {
    var sum_squared: f32 = 0.0;
    
    let items_per_thread = (params.dimension + workgroup_size - 1u) / workgroup_size;
    let start = local_idx * items_per_thread;
    let end = min(start + items_per_thread, params.dimension);
    
    for (var i = start; i < end; i++) {
        let diff = batch_vectors[batch_offset + i] - existing_vectors[existing_offset + i];
        sum_squared += diff * diff;
    }
    
    shared_distances[local_idx] = sum_squared;
    workgroupBarrier();
    
    if (local_idx == 0u) {
        var total: f32 = 0.0;
        for (var i: u32 = 0u; i < workgroup_size; i++) {
            total += shared_distances[i];
        }
        shared_distances[0] = sqrt(total);
    }
    
    workgroupBarrier();
    return shared_distances[0];
}

/// Dot product using shared memory
fn dot_product_shared(
    batch_offset: u32,
    existing_offset: u32,
    local_idx: u32,
    workgroup_size: u32
) -> f32 {
    var dot: f32 = 0.0;
    
    let items_per_thread = (params.dimension + workgroup_size - 1u) / workgroup_size;
    let start = local_idx * items_per_thread;
    let end = min(start + items_per_thread, params.dimension);
    
    for (var i = start; i < end; i++) {
        dot += batch_vectors[batch_offset + i] * existing_vectors[existing_offset + i];
    }
    
    shared_distances[local_idx] = dot;
    workgroupBarrier();
    
    if (local_idx == 0u) {
        var total: f32 = 0.0;
        for (var i: u32 = 0u; i < workgroup_size; i++) {
            total += shared_distances[i];
        }
        shared_distances[0] = -total; // Negative for distance
    }
    
    workgroupBarrier();
    return shared_distances[0];
}

/// Calculate distance based on metric (non-shared fallback)
fn calculate_distance_direct(batch_offset: u32, existing_offset: u32) -> f32 {
    switch (params.metric_type) {
        case 0u: { // Cosine
            var dot: f32 = 0.0;
            var norm_a: f32 = 0.0;
            var norm_b: f32 = 0.0;
            
            for (var i: u32 = 0u; i < params.dimension; i++) {
                let val_a = batch_vectors[batch_offset + i];
                let val_b = existing_vectors[existing_offset + i];
                dot += val_a * val_b;
                norm_a += val_a * val_a;
                norm_b += val_b * val_b;
            }
            
            norm_a = sqrt(norm_a);
            norm_b = sqrt(norm_b);
            
            if (norm_a > 0.0 && norm_b > 0.0) {
                return 1.0 - (dot / (norm_a * norm_b));
            }
            return 1.0;
        }
        case 1u: { // Euclidean
            var sum_squared: f32 = 0.0;
            for (var i: u32 = 0u; i < params.dimension; i++) {
                let diff = batch_vectors[batch_offset + i] - existing_vectors[existing_offset + i];
                sum_squared += diff * diff;
            }
            return sqrt(sum_squared);
        }
        case 2u: { // Dot Product
            var dot: f32 = 0.0;
            for (var i: u32 = 0u; i < params.dimension; i++) {
                dot += batch_vectors[batch_offset + i] * existing_vectors[existing_offset + i];
            }
            return -dot;
        }
        default: { return 0.0; }
    }
}

// ============================================================================
// Distance Matrix Computation
// ============================================================================

/// Kernel 1: Compute distance matrix (batch vs existing)
/// Uses 2D dispatch: [batch_size, existing_nodes]
@compute @workgroup_size(16, 16)
fn compute_distance_matrix(@builtin(global_invocation_id) gid: vec3<u32>) {
    let batch_idx = gid.x;
    let existing_idx = gid.y;
    
    if (batch_idx >= params.batch_size || existing_idx >= params.total_nodes) {
        return;
    }
    
    // Calculate vector offsets
    let batch_vec_offset = (params.batch_offset + batch_idx) * params.dimension;
    let existing_vec_offset = existing_idx * params.dimension;
    
    // Compute distance
    let distance = calculate_distance_direct(batch_vec_offset, existing_vec_offset);
    
    // Store in matrix (row-major: batch_idx * total_nodes + existing_idx)
    let matrix_idx = batch_idx * params.total_nodes + existing_idx;
    distance_matrix[matrix_idx] = distance;
}

// ============================================================================
// Neighbor Selection
// ============================================================================

/// Kernel 2: Select K nearest neighbors for each batch vector
/// Uses 1D dispatch: [batch_size]
@compute @workgroup_size(256)
fn select_batch_neighbors(@builtin(global_invocation_id) gid: vec3<u32>) {
    let batch_idx = gid.x;
    
    if (batch_idx >= params.batch_size) {
        return;
    }
    
    // Find ef_construction nearest neighbors from distance matrix
    let matrix_row_start = batch_idx * params.total_nodes;
    
    // Build candidate list (using insertion sort for top-k)
    var candidates: array<DistanceEntry, 200>; // ef_construction max
    var candidate_count: u32 = 0u;
    
    // Initialize candidates
    for (var i: u32 = 0u; i < params.ef_construction; i++) {
        candidates[i] = DistanceEntry(0u, 0u, 1e10, 0u);
    }
    
    // Scan all existing nodes and keep top ef_construction
    for (var i: u32 = 0u; i < params.total_nodes; i++) {
        let distance = distance_matrix[matrix_row_start + i];
        
        // Find insertion point
        var insert_idx: u32 = params.ef_construction;
        for (var j: u32 = 0u; j < params.ef_construction; j++) {
            if (distance < candidates[j].distance) {
                insert_idx = j;
                break;
            }
        }
        
        if (insert_idx < params.ef_construction) {
            // Shift candidates
            for (var j = params.ef_construction - 1u; j > insert_idx; j--) {
                candidates[j] = candidates[j - 1u];
            }
            
            // Insert new candidate
            candidates[insert_idx] = DistanceEntry(batch_idx, i, distance, 1u);
            candidate_count = min(candidate_count + 1u, params.ef_construction);
        }
    }
    
    // Select M best using heuristic (simplified: just take M closest)
    let m = select(params.m, params.m0, true); // Assume level 0 for batch
    let final_count = min(m, candidate_count);
    
    // Write selected neighbors
    let candidate_base = batch_idx * params.ef_construction;
    for (var i: u32 = 0u; i < final_count; i++) {
        batch_candidates[candidate_base + i] = candidates[i];
    }
    
    // Mark rest as invalid
    for (var i = final_count; i < params.ef_construction; i++) {
        batch_candidates[candidate_base + i] = DistanceEntry(0u, 0u, 0.0, 0u);
    }
}

// ============================================================================
// Connection Writing
// ============================================================================

/// Kernel 3: Write connections from batch candidates to graph
@compute @workgroup_size(256)
fn write_batch_connections(@builtin(global_invocation_id) gid: vec3<u32>) {
    let batch_idx = gid.x;
    
    if (batch_idx >= params.batch_size) {
        return;
    }
    
    // Calculate actual node index in full graph
    let node_idx = params.batch_offset + batch_idx;
    
    if (node_idx >= params.total_nodes + params.batch_size) {
        return;
    }
    
    var node = get_node_at(node_idx);
    let candidate_base = batch_idx * params.ef_construction;
    
    // Count valid candidates
    var connection_count: u32 = 0u;
    for (var i: u32 = 0u; i < params.ef_construction; i++) {
        let candidate = batch_candidates[candidate_base + i];
        if (bool(candidate.valid)) {
            connection_count++;
        } else {
            break;
        }
    }
    
    // Write connections
    node.connection_count = connection_count;
    set_node_at(node_idx, node);
    
    let m0 = params.m0;
    for (var i: u32 = 0u; i < connection_count && i < m0; i++) {
        let candidate = batch_candidates[candidate_base + i];
        if (bool(candidate.valid)) {
            // Write connection (level 0)
            let connection_offset = node_idx * m0 + i;
            connections[connection_offset] = candidate.existing_idx;
        }
    }
}

// ============================================================================
// Bidirectional Connection Update
// ============================================================================

/// Kernel 4: Add reverse connections for batch
@compute @workgroup_size(256)
fn update_reverse_connections(@builtin(global_invocation_id) gid: vec3<u32>) {
    let batch_idx = gid.x;
    
    if (batch_idx >= params.batch_size) {
        return;
    }
    
    let node_idx = params.batch_offset + batch_idx;
    let candidate_base = batch_idx * params.ef_construction;
    
    // For each neighbor of this batch node
    for (var i: u32 = 0u; i < params.ef_construction; i++) {
        let candidate = batch_candidates[candidate_base + i];
        
        if (!bool(candidate.valid)) {
            break;
        }
        
        let neighbor_idx = candidate.existing_idx;
        
        if (neighbor_idx >= params.total_nodes) {
            continue;
        }
        
        var neighbor = get_node_at(neighbor_idx);
        
        // Check if reverse connection already exists
        var has_reverse = false;
        let neighbor_m0 = params.m0;
        
        for (var j: u32 = 0u; j < neighbor.connection_count && j < neighbor_m0; j++) {
            let connection_offset = neighbor_idx * neighbor_m0 + j;
            if (connections[connection_offset] == node_idx) {
                has_reverse = true;
                break;
            }
        }
        
        // Add reverse connection if space available
        if (!has_reverse && neighbor.connection_count < neighbor_m0) {
            let connection_offset = neighbor_idx * neighbor_m0 + neighbor.connection_count;
            connections[connection_offset] = node_idx;
            
            neighbor.connection_count++;
            set_node_at(neighbor_idx, neighbor);
        }
    }
}

// ============================================================================
// Utility Kernels
// ============================================================================

/// Clear distance matrix for next batch
@compute @workgroup_size(256)
fn clear_distance_matrix(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    let total_entries = params.batch_size * params.total_nodes;
    
    if (idx < total_entries) {
        distance_matrix[idx] = 0.0;
    }
}

/// Clear candidate buffer
@compute @workgroup_size(256)
fn clear_candidates(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    let total_entries = params.batch_size * params.ef_construction;
    
    if (idx < total_entries) {
        batch_candidates[idx] = DistanceEntry(0u, 0u, 0.0, 0u);
    }
}


