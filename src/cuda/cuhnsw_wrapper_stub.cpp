// Stub implementation for cuhnsw when CUDA compilation fails

#include <memory>
#include <string>
#include <cstring>
#include <iostream>

extern "C" {

// Opaque handle for CuHNSW instance (stub)
struct CuHNSWHandle {
    int dummy; // Just to make it non-empty
};

// Create new CuHNSW instance (stub)
void* cuhnsw_create() {
    std::cout << "[STUB] cuhnsw_create called - CUDA not available" << std::endl;
    try {
        auto handle = new CuHNSWHandle();
        handle->dummy = 42;
        return handle;
    } catch (...) {
        return nullptr;
    }
}

// Destroy CuHNSW instance (stub)
void cuhnsw_destroy(void* handle) {
    std::cout << "[STUB] cuhnsw_destroy called - CUDA not available" << std::endl;
    if (handle) {
        delete static_cast<CuHNSWHandle*>(handle);
    }
}

// Initialize with config file (stub)
bool cuhnsw_init(void* handle, const char* config_path) {
    std::cout << "[STUB] cuhnsw_init called - CUDA not available: " << (config_path ? config_path : "null") << std::endl;
    return handle != nullptr;
}

// Set data (stub)
void cuhnsw_set_data(void* handle, const float* data, int num_data, int num_dims) {
    std::cout << "[STUB] cuhnsw_set_data called - CUDA not available: " << num_data << "x" << num_dims << std::endl;
}

// Set random levels (stub)
void cuhnsw_set_random_levels(void* handle, const int* levels) {
    std::cout << "[STUB] cuhnsw_set_random_levels called - CUDA not available" << std::endl;
}

// Build graph (stub)
void cuhnsw_build_graph(void* handle) {
    std::cout << "[STUB] cuhnsw_build_graph called - CUDA not available" << std::endl;
}

// Save index (stub)
void cuhnsw_save_index(void* handle, const char* file_path) {
    std::cout << "[STUB] cuhnsw_save_index called - CUDA not available: " << (file_path ? file_path : "null") << std::endl;
}

// Load index (stub)
void cuhnsw_load_index(void* handle, const char* file_path) {
    std::cout << "[STUB] cuhnsw_load_index called - CUDA not available: " << (file_path ? file_path : "null") << std::endl;
}

// Search KNN (stub)
void cuhnsw_search_knn(
    void* handle,
    const float* query_data,
    int num_queries,
    int topk,
    int ef_search,
    int* nns,
    float* distances,
    int* found_cnt
) {
    std::cout << "[STUB] cuhnsw_search_knn called - CUDA not available: " << num_queries << " queries, topk=" << topk << std::endl;

    // Fill with dummy results
    if (nns && distances && found_cnt) {
        for (int i = 0; i < num_queries * topk; ++i) {
            nns[i] = i % 1000;  // Dummy neighbors
            distances[i] = 1.0f; // Dummy distances
        }
        for (int i = 0; i < num_queries; ++i) {
            found_cnt[i] = topk; // Found all requested neighbors
        }
    }
}

} // extern "C"
