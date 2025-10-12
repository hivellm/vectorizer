#include <metal_stdlib>
using namespace metal;

// HNSW Node structure - represents a node in hierarchical graph
struct HnswNode {
    uint id;
    uint level;                // Maximum layer this node appears in
    uint base_connections_offset; // Offset to connections in base layer (layer 0)
    uint vector_offset;        // Offset into vectors buffer
};

// Layer-specific node data for GPU processing
struct HnswLayerNode {
    uint node_id;
    uint connections_offset;   // Offset to connections in this layer
    uint connection_count;     // Number of connections in this layer
};

// Search candidate with distance for priority queue
struct SearchCandidate {
    uint node_id;
    float distance;
};

// Search result structure
struct SearchResult {
    uint node_id;
    float distance;
};

// HNSW search state for layer navigation
struct HnswSearchState {
    uint current_layer;
    uint entry_point;
    uint ef;                   // Size of dynamic candidate list
    uint visited_count;        // Number of visited nodes
};

// Layer navigation result
struct LayerNavigationResult {
    uint best_node_id;
    float best_distance;
    bool found_better;
};

// Full GPU search structures
struct GpuSearchQuery {
    float data[512];           // Query vector (max 512 dimensions)
    uint dimension;            // Actual dimension
};

struct GpuSearchResult {
    uint vector_id;            // ID of the matched vector
    float distance;            // Distance to query
    uint vector_index;         // Index in vectors buffer
};

// Vector metadata for GPU processing
struct GpuVectorMetadata {
    uint vector_id;            // Original vector ID
    uint is_active;            // 1 if vector is active, 0 if removed
};

// Distance calculation function with robust numerical handling
float calculate_cosine_distance(device const float* vector_a, device const float* vector_b, uint dimension) {
    float dot_product = 0.0;
    float norm_a = 0.0;
    float norm_b = 0.0;
    
    for (uint i = 0; i < dimension; i++) {
        float a_val = vector_a[i];
        float b_val = vector_b[i];
        dot_product += a_val * b_val;
        norm_a += a_val * a_val;
        norm_b += b_val * b_val;
    }
    
    norm_a = sqrt(norm_a);
    norm_b = sqrt(norm_b);
    
    // Avoid division by zero and ensure numerical stability
    const float epsilon = 1e-8;
    float denom = max(norm_a * norm_b, epsilon);
    float similarity = dot_product / denom;
    
    // Clamp similarity to valid range [-1, 1]
    similarity = clamp(similarity, -1.0, 1.0);
    
    return 1.0 - similarity;
}

// HNSW Layer Navigation Kernel - Navigate within a single layer using greedy search
kernel void hnsw_navigate_layer(
    device const float* vectors [[buffer(0)]],
    device const HnswNode* nodes [[buffer(1)]],
    device const HnswLayerNode* layer_nodes [[buffer(2)]],
    device const uint* layer_connections [[buffer(3)]],
    device const float* query_vector [[buffer(4)]],
    device SearchCandidate* candidates [[buffer(5)]],
    device uint* visited_nodes [[buffer(6)]],
    constant uint& vector_dim [[buffer(7)]],
    constant uint& layer_node_count [[buffer(8)]],
    constant uint& max_candidates [[buffer(9)]],
    constant uint& entry_point [[buffer(10)]],
    uint tid [[thread_position_in_grid]]
) {
    if (tid >= layer_node_count) return;

    // This kernel processes one node from the candidates list
    // tid represents the index in the candidates array to explore

    // Get the candidate to explore
    SearchCandidate current_candidate = candidates[tid];
    uint current_node_id = current_candidate.node_id;

    // Find the layer node data for this node
    HnswLayerNode layer_node;
    bool found_node = false;

    for (uint i = 0; i < layer_node_count; i++) {
        if (layer_nodes[i].node_id == current_node_id) {
            layer_node = layer_nodes[i];
            found_node = true;
            break;
        }
    }

    if (!found_node || layer_node.connection_count == 0) {
        // No neighbors to explore from this node
        return;
    }

    // Explore all neighbors of this node
    for (uint i = 0; i < layer_node.connection_count; i++) {
        uint neighbor_id = layer_connections[layer_node.connections_offset + i];

        // Check if already visited (simple linear search for now)
        bool already_visited = false;
        for (uint j = 0; j < max_candidates; j++) {
            if (visited_nodes[j] == neighbor_id) {
                already_visited = true;
                break;
            }
        }

        if (already_visited) continue;

        // Calculate distance to neighbor
        HnswNode neighbor_node;
        bool found_neighbor = false;

        for (uint j = 0; j < layer_node_count; j++) {
            if (nodes[j].id == neighbor_id) {
                neighbor_node = nodes[j];
                found_neighbor = true;
                break;
            }
        }

        if (!found_neighbor) continue;

        device const float* neighbor_vector = &vectors[neighbor_node.vector_offset * vector_dim];
        float distance = calculate_cosine_distance(query_vector, neighbor_vector, vector_dim);

        // Try to add to candidates if better than current worst
        // This is a simplified version - in practice, we'd use a priority queue
        for (uint j = 0; j < max_candidates; j++) {
            if (candidates[j].distance > distance) {
                // Shift elements to make room
                for (uint k = max_candidates - 1; k > j; k--) {
                    candidates[k] = candidates[k - 1];
                }
                candidates[j] = SearchCandidate{.node_id = neighbor_id, .distance = distance};
                visited_nodes[j] = neighbor_id;
                break;
            }
        }
    }
}

// HNSW Complete Search Kernel - Performs full hierarchical search
kernel void hnsw_search_complete(
    device const float* vectors [[buffer(0)]],
    device const HnswNode* nodes [[buffer(1)]],
    device const HnswLayerNode* layer_nodes_base [[buffer(2)]],
    device const HnswLayerNode* layer_nodes_higher [[buffer(3)]],
    device const uint* layer_connections_base [[buffer(4)]],
    device const uint* layer_connections_higher [[buffer(5)]],
    device const float* query_vector [[buffer(6)]],
    device SearchResult* final_results [[buffer(7)]],
    constant uint& vector_dim [[buffer(8)]],
    constant uint& max_level [[buffer(9)]],
    constant uint& entry_point [[buffer(10)]],
    constant uint& ef_search [[buffer(11)]],
    constant uint& k [[buffer(12)]],
    uint search_id [[thread_position_in_grid]]
) {
    // This kernel performs a complete HNSW search for one query
    // Implements real hierarchical navigation with greedy search

    if (search_id >= 1) return; // Only handle one search at a time for now

    // Shared memory for candidates (max 256 candidates)
    threadgroup SearchCandidate candidates[256];
    threadgroup uint visited[256];
    threadgroup uint candidate_count[1];
    threadgroup uint visited_count[1];

    // Initialize
    if (search_id == 0) {
        candidate_count[0] = 0;
        visited_count[0] = 0;

        // Add entry point as first candidate
        if (entry_point < 10000) { // Safety check
            HnswNode entry_node = nodes[entry_point];
            device const float* entry_vector = &vectors[entry_node.vector_offset * vector_dim];
            float entry_distance = calculate_cosine_distance(query_vector, entry_vector, vector_dim);

            candidates[0] = SearchCandidate{.node_id = entry_point, .distance = entry_distance};
            visited[0] = entry_point;
            candidate_count[0] = 1;
            visited_count[0] = 1;
        }
    }
    threadgroup_barrier(mem_flags::mem_threadgroup);

    // Perform hierarchical search from top layer down to layer 0 using beam search
    uint current_entry = entry_point;

    for (uint current_level = max_level; current_level > 0; current_level--) {
        // Beam search in current layer - explore multiple candidates simultaneously
        uint beam_width = min(ef_search / 4 + 1, 8u); // Adaptive beam width
        bool found_improvement = true;
        uint iterations = 0;

        while (found_improvement && iterations < 10 && candidate_count[0] > 0) { // Limit iterations
            found_improvement = false;
            iterations++;

            // Sort current candidates by distance (beam selection)
            for (uint i = 0; i < candidate_count[0] - 1 && i < 256; i++) {
                for (uint j = i + 1; j < candidate_count[0] && j < 256; j++) {
                    if (candidates[j].distance < candidates[i].distance) {
                        SearchCandidate temp = candidates[i];
                        candidates[i] = candidates[j];
                        candidates[j] = temp;
                    }
                }
            }

            // Keep only top beam_width candidates
            if (candidate_count[0] > beam_width) {
                candidate_count[0] = beam_width;
            }

            // Explore neighbors of all beam candidates
            uint new_candidates_start = candidate_count[0];

            for (uint beam_idx = 0; beam_idx < beam_width && beam_idx < candidate_count[0]; beam_idx++) {
                uint current_node_id = candidates[beam_idx].node_id;

                // Explore neighbors of this beam candidate
                device const HnswLayerNode* layer_nodes = (current_level == 0) ?
                    layer_nodes_base : layer_nodes_higher;
                device const uint* layer_connections = (current_level == 0) ?
                    layer_connections_base : layer_connections_higher;

                // Find the layer node for current_node_id
                HnswLayerNode layer_node;
                bool found_layer_node = false;

                for (uint i = 0; i < 1000 && !found_layer_node; i++) { // Safety limit
                    if (layer_nodes[i].node_id == current_node_id) {
                        layer_node = layer_nodes[i];
                        found_layer_node = true;
                    }
                }

                if (found_layer_node && layer_node.connection_count > 0) {
                    for (uint i = 0; i < layer_node.connection_count && i < 32; i++) { // Max 32 neighbors
                        uint neighbor_id = layer_connections[layer_node.connections_offset + i];

                        // Check if already visited
                        bool already_visited = false;
                        for (uint j = 0; j < visited_count[0] && j < 256; j++) {
                            if (visited[j] == neighbor_id) {
                                already_visited = true;
                                break;
                            }
                        }

                        if (!already_visited && candidate_count[0] < 256) {
                            // Calculate distance to neighbor
                            HnswNode neighbor_node = nodes[neighbor_id];
                            device const float* neighbor_vector = &vectors[neighbor_node.vector_offset * vector_dim];
                            float distance = calculate_cosine_distance(query_vector, neighbor_vector, vector_dim);

                            // Add to candidates
                            candidates[candidate_count[0]] = SearchCandidate{
                                .node_id = neighbor_id,
                                .distance = distance
                            };
                            candidate_count[0]++;

                            // Add to visited
                            if (visited_count[0] < 256) {
                                visited[visited_count[0]] = neighbor_id;
                                visited_count[0]++;
                                found_improvement = true;
                            }
                        }
                    }
                }
            }

            // Remove duplicates and sort again
            if (candidate_count[0] > beam_width) {
                // Simple deduplication - keep unique nodes with best distances
                for (uint i = new_candidates_start; i < candidate_count[0] && i < 256; i++) {
                    for (uint j = 0; j < new_candidates_start && j < 256; j++) {
                        if (candidates[i].node_id == candidates[j].node_id) {
                            // Keep the better distance
                            if (candidates[i].distance < candidates[j].distance) {
                                candidates[j] = candidates[i];
                            }
                            // Mark for removal
                            candidates[i].distance = INFINITY;
                            break;
                        }
                    }
                }

                // Compact array (remove INFINITY entries)
                uint write_idx = 0;
                for (uint i = 0; i < candidate_count[0] && i < 256; i++) {
                    if (candidates[i].distance < INFINITY) {
                        if (write_idx != i) {
                            candidates[write_idx] = candidates[i];
                        }
                        write_idx++;
                    }
                }
                candidate_count[0] = write_idx;

                // Keep only top beam_width
                if (candidate_count[0] > beam_width) {
                    candidate_count[0] = beam_width;
                }
            }
        }

        // Prepare candidates for next level (keep only the best one)
        if (candidate_count[0] > 0) {
            // Find the best candidate to carry to next level
            uint best_idx = 0;
            float best_dist = INFINITY;

            for (uint i = 0; i < candidate_count[0] && i < 256; i++) {
                if (candidates[i].distance < best_dist) {
                    best_dist = candidates[i].distance;
                    best_idx = i;
                }
            }

            // Reset candidates for next level
            candidate_count[0] = 1;
            candidates[0] = candidates[best_idx];
            current_entry = candidates[0].node_id;
        }
    }

    // Now perform beam search in base layer (level 0) with ef_search candidates
    candidate_count[0] = 1;
    candidates[0] = SearchCandidate{.node_id = current_entry, .distance = INFINITY};

    // Calculate distance for entry point
    HnswNode entry_node = nodes[current_entry];
    device const float* entry_vector = &vectors[entry_node.vector_offset * vector_dim];
    candidates[0].distance = calculate_cosine_distance(query_vector, entry_vector, vector_dim);

    visited_count[0] = 1;
    visited[0] = current_entry;

    // Beam search in base layer with ef_search beam width
    uint base_beam_width = min(ef_search, 64u); // Limit beam width for base layer
    bool found_improvement = true;
    uint base_iterations = 0;

    while (found_improvement && base_iterations < 15 && candidate_count[0] > 0) { // More iterations for base layer
        found_improvement = false;
        base_iterations++;

        // Sort current candidates by distance and keep top beam_width
        for (uint i = 0; i < candidate_count[0] - 1 && i < 256; i++) {
            for (uint j = i + 1; j < candidate_count[0] && j < 256; j++) {
                if (candidates[j].distance < candidates[i].distance) {
                    SearchCandidate temp = candidates[i];
                    candidates[i] = candidates[j];
                    candidates[j] = temp;
                }
            }
        }

        // Keep only top base_beam_width candidates
        if (candidate_count[0] > base_beam_width) {
            candidate_count[0] = base_beam_width;
        }

        // Explore neighbors of all beam candidates in base layer
        uint new_candidates_start = candidate_count[0];

        for (uint beam_idx = 0; beam_idx < base_beam_width && beam_idx < candidate_count[0]; beam_idx++) {
            uint current_node_id = candidates[beam_idx].node_id;

            // Find the layer node for current_node_id in base layer
            HnswLayerNode layer_node;
            bool found_layer_node = false;

            for (uint i = 0; i < 1000 && !found_layer_node; i++) { // Safety limit
                if (layer_nodes_base[i].node_id == current_node_id) {
                    layer_node = layer_nodes_base[i];
                    found_layer_node = true;
                }
            }

            if (found_layer_node && layer_node.connection_count > 0) {
                for (uint i = 0; i < layer_node.connection_count && i < 32; i++) {
                    uint neighbor_id = layer_connections_base[layer_node.connections_offset + i];

                    // Check if already visited
                    bool already_visited = false;
                    for (uint j = 0; j < visited_count[0] && j < 256; j++) {
                        if (visited[j] == neighbor_id) {
                            already_visited = true;
                            break;
                        }
                    }

                    if (!already_visited && candidate_count[0] < ef_search && candidate_count[0] < 256) {
                        // Calculate distance to neighbor
                        HnswNode neighbor_node = nodes[neighbor_id];
                        device const float* neighbor_vector = &vectors[neighbor_node.vector_offset * vector_dim];
                        float distance = calculate_cosine_distance(query_vector, neighbor_vector, vector_dim);

                        // Add to candidates
                        candidates[candidate_count[0]] = SearchCandidate{
                            .node_id = neighbor_id,
        .distance = distance
    };
                        candidate_count[0]++;

                        // Add to visited
                        if (visited_count[0] < 256) {
                            visited[visited_count[0]] = neighbor_id;
                            visited_count[0]++;
                            found_improvement = true;
                        }
                    }
                }
            }
        }

        // Maintain ef_search candidates - replace worst if we have too many
        if (candidate_count[0] > ef_search) {
            // Sort all candidates and keep best ef_search
            for (uint i = 0; i < candidate_count[0] - 1 && i < 256; i++) {
                for (uint j = i + 1; j < candidate_count[0] && j < 256; j++) {
                    if (candidates[j].distance < candidates[i].distance) {
                        SearchCandidate temp = candidates[i];
                        candidates[i] = candidates[j];
                        candidates[j] = temp;
                    }
                }
            }
            candidate_count[0] = ef_search;
        }
    }

    // Select top k results from candidates
    for (uint i = 0; i < k && i < candidate_count[0] && i < 256; i++) {
        // Simple selection - find i-th best
        uint best_idx = 0;
        float best_dist = INFINITY;

        for (uint j = 0; j < candidate_count[0] && j < 256; j++) {
            bool already_selected = false;
            for (uint m = 0; m < i; m++) {
                if (final_results[m].node_id == candidates[j].node_id) {
                    already_selected = true;
                    break;
                }
            }

            if (!already_selected && candidates[j].distance < best_dist) {
                best_dist = candidates[j].distance;
                best_idx = j;
            }
        }

        if (best_dist < INFINITY) {
            final_results[i] = SearchResult{
                .node_id = candidates[best_idx].node_id,
                .distance = candidates[best_idx].distance
            };
        }
    }
}

// Top-K selection kernel with correct implementation
kernel void select_top_k(
    device const SearchCandidate* candidates [[buffer(0)]],
    device SearchResult* top_results [[buffer(1)]],
    constant uint& candidate_count [[buffer(2)]],
    constant uint& k [[buffer(3)]],
    uint tid [[thread_position_in_grid]]
) {
    if (tid >= k) return;
    
    // Sort candidates by distance (simplified selection sort)
    threadgroup SearchCandidate sorted_candidates[256]; // Shared memory for sorting

    // Copy candidates to threadgroup memory
    for (uint i = 0; i < candidate_count && i < 256; i++) {
        sorted_candidates[i] = candidates[i];
    }

    // Simple selection sort (not efficient but correct)
    for (uint i = 0; i < candidate_count - 1 && i < k; i++) {
        uint min_idx = i;
        for (uint j = i + 1; j < candidate_count && j < 256; j++) {
            if (sorted_candidates[j].distance < sorted_candidates[min_idx].distance) {
                min_idx = j;
            }
        }

        // Swap
        SearchCandidate temp = sorted_candidates[i];
        sorted_candidates[i] = sorted_candidates[min_idx];
        sorted_candidates[min_idx] = temp;
    }

    // Store the tid-th best result
    if (tid < candidate_count && tid < 256) {
        SearchCandidate result = sorted_candidates[tid];
        top_results[tid] = SearchResult{
            .node_id = result.node_id,
            .distance = result.distance
        };
    }
}

// HNSW Search Initialization Kernel - Sets up initial candidates for search
kernel void hnsw_search_init(
    device const float* vectors [[buffer(0)]],
    device const HnswNode* nodes [[buffer(1)]],
    device const float* query_vector [[buffer(2)]],
    device SearchCandidate* candidates [[buffer(3)]],
    device uint* visited_nodes [[buffer(4)]],
    constant uint& vector_dim [[buffer(5)]],
    constant uint& entry_point [[buffer(6)]],
    constant uint& max_candidates [[buffer(7)]],
    uint tid [[thread_position_in_grid]]
) {
    if (tid >= 1) return; // Only one thread initializes

    // Initialize with entry point
    HnswNode entry_node = nodes[entry_point];

    device const float* entry_vector = &vectors[entry_node.vector_offset * vector_dim];
    float entry_distance = calculate_cosine_distance(query_vector, entry_vector, vector_dim);

    // Set first candidate as entry point
    candidates[0] = SearchCandidate{
        .node_id = entry_point,
        .distance = entry_distance
    };

    // Initialize visited nodes
    visited_nodes[0] = entry_point;

    // Fill remaining candidates with invalid data (distance = INFINITY)
    for (uint i = 1; i < max_candidates; i++) {
        candidates[i] = SearchCandidate{
            .node_id = UINT_MAX,
            .distance = INFINITY
        };
        visited_nodes[i] = UINT_MAX;
    }
}

// HNSW Connection Building Kernel - Builds connections during index construction
kernel void hnsw_build_connections(
    device const float* vectors [[buffer(0)]],
    device const HnswNode* nodes [[buffer(1)]],
    device HnswLayerNode* layer_nodes [[buffer(2)]],
    device uint* layer_connections [[buffer(3)]],
    device const float* new_vector [[buffer(4)]],
    constant uint& vector_dim [[buffer(5)]],
    constant uint& layer [[buffer(6)]],
    constant uint& max_connections [[buffer(7)]],
    constant uint& new_node_id [[buffer(8)]],
    constant uint& layer_node_count [[buffer(9)]],
    uint tid [[thread_position_in_grid]]
) {
    // This kernel helps build connections for a new node in a specific layer
    // tid represents the index of existing nodes to consider for connection

    if (tid >= layer_node_count) return;

    HnswLayerNode existing_node = layer_nodes[tid];

    // Skip if this is the new node itself
    if (existing_node.node_id == new_node_id) return;

    // Calculate distance between new vector and existing node
    HnswNode existing_node_data;
    bool found = false;

    // Find the node data (simplified linear search)
    for (uint i = 0; i < 10000; i++) { // Safety limit
        if (nodes[i].id == existing_node.node_id) {
            existing_node_data = nodes[i];
            found = true;
            break;
        }
    }

    if (!found) return;

    device const float* existing_vector = &vectors[existing_node_data.vector_offset * vector_dim];
    // Calculate distance (stored for potential future use)
    // float distance = calculate_cosine_distance(new_vector, existing_vector, vector_dim);

    // This is a simplified version - in practice, we'd need atomic operations
    // and a more sophisticated selection algorithm
    // For now, this kernel is a placeholder for future connection building
}

// ============================================================================
// FULL GPU VECTOR SEARCH - Maintains all data in VRAM
// ============================================================================

// Full GPU search kernel - searches all vectors entirely on GPU
kernel void gpu_full_vector_search(
    device const float* vectors [[buffer(0)]],                    // All vector data (flattened)
    device const GpuVectorMetadata* metadata [[buffer(1)]],       // Vector metadata
    device const GpuSearchQuery* query [[buffer(2)]],             // Search query
    device GpuSearchResult* results [[buffer(3)]],                // Search results buffer
    constant uint& vector_count [[buffer(4)]],                   // Total number of vectors
    constant uint& k [[buffer(5)]],                              // Number of results to return
    constant uint& dimension [[buffer(6)]],                      // Vector dimension
    uint tid [[thread_position_in_grid]]                         // Thread ID
) {
    // Each thread processes one vector
    if (tid >= vector_count) return;

    // Check if vector is active (not removed)
    if (metadata[tid].is_active == 0) return;

    // Get vector data pointer
    device const float* vector_data = &vectors[tid * dimension];

    // Get query data
    device const float* query_data = query->data;

    // Calculate cosine distance
    float distance = calculate_cosine_distance(query_data, vector_data, dimension);

    // Store result for this vector
    results[tid].vector_id = metadata[tid].vector_id;
    results[tid].distance = distance;
    results[tid].vector_index = tid;
}

// Parallel reduction kernel to find top-k results
kernel void gpu_find_top_k_results(
    device const GpuSearchResult* all_results [[buffer(0)]],     // All search results
    device GpuSearchResult* final_results [[buffer(1)]],         // Final top-k results
    constant uint& total_vectors [[buffer(2)]],                  // Total number of vectors
    constant uint& k [[buffer(3)]],                              // Number of results to return
    uint tid [[thread_position_in_grid]]                         // Thread ID
) {
    // This is a simplified implementation
    // In practice, we'd use parallel reduction or prefix sum algorithms
    // For now, we'll do a simple bubble sort approach (not optimal)

    if (tid >= k) return;

    // Initialize with worst possible result
    GpuSearchResult best = { UINT_MAX, FLT_MAX, UINT_MAX };

    // Find the tid-th best result
    for (uint i = 0; i < total_vectors; i++) {
        GpuSearchResult current = all_results[i];

        // Skip invalid results
        if (current.vector_id == UINT_MAX) continue;

        // Check if this result is better than current best for this position
        bool better = false;
        if (current.distance < best.distance) {
            better = true;
        } else if (current.distance == best.distance && current.vector_id < best.vector_id) {
            better = true;
        }

        if (better) {
            // Count how many results are better than this one
            uint better_count = 0;
            for (uint j = 0; j < total_vectors; j++) {
                if (i == j) continue;
                GpuSearchResult other = all_results[j];
                if (other.vector_id == UINT_MAX) continue;

                if (other.distance < current.distance ||
                   (other.distance == current.distance && other.vector_id < current.vector_id)) {
                    better_count++;
                }
            }

            // If this is the right position for us, use it
            if (better_count == tid) {
                best = current;
            }
        }
    }

    final_results[tid] = best;
}