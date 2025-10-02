// C++ wrapper for cuhnsw to expose C API

#include <memory>
#include <string>
#include <cstring>
#include <iostream>

// Conditional include - only if CUDA compilation succeeded
#ifdef HAS_CUDA_HNSW
#include "cuhnsw.hpp"
#endif

extern "C" {

// Opaque handle for CuHNSW instance
struct CuHNSWHandle {
    std::unique_ptr<cuhnsw::CuHNSW> instance;
};

// Create new CuHNSW instance
void* cuhnsw_create() {
    try {
        auto handle = new CuHNSWHandle();
        handle->instance = std::make_unique<cuhnsw::CuHNSW>();
        return handle;
    } catch (...) {
        return nullptr;
    }
}

// Destroy CuHNSW instance
void cuhnsw_destroy(void* handle) {
    if (handle) {
        delete static_cast<CuHNSWHandle*>(handle);
    }
}

// Initialize with config file
bool cuhnsw_init(void* handle, const char* config_path) {
    if (!handle || !config_path) return false;
    
    try {
        auto h = static_cast<CuHNSWHandle*>(handle);
        return h->instance->Init(std::string(config_path));
    } catch (...) {
        return false;
    }
}

// Set data
void cuhnsw_set_data(void* handle, const float* data, int num_data, int num_dims) {
    if (!handle || !data) return;
    
    try {
        auto h = static_cast<CuHNSWHandle*>(handle);
        h->instance->SetData(data, num_data, num_dims);
    } catch (...) {
        // Log error
    }
}

// Set random levels
void cuhnsw_set_random_levels(void* handle, const int* levels) {
    if (!handle || !levels) return;
    
    try {
        auto h = static_cast<CuHNSWHandle*>(handle);
        h->instance->SetRandomLevels(levels);
    } catch (...) {
        // Log error
    }
}

// Build graph
void cuhnsw_build_graph(void* handle) {
    if (!handle) return;
    
    try {
        auto h = static_cast<CuHNSWHandle*>(handle);
        h->instance->BuildGraph();
    } catch (...) {
        // Log error
    }
}

// Save index
void cuhnsw_save_index(void* handle, const char* file_path) {
    if (!handle || !file_path) return;
    
    try {
        auto h = static_cast<CuHNSWHandle*>(handle);
        h->instance->SaveIndex(std::string(file_path));
    } catch (...) {
        // Log error
    }
}

// Load index
void cuhnsw_load_index(void* handle, const char* file_path) {
    if (!handle || !file_path) return;
    
    try {
        auto h = static_cast<CuHNSWHandle*>(handle);
        h->instance->LoadIndex(std::string(file_path));
    } catch (...) {
        // Log error
    }
}

// Search KNN
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
    if (!handle || !query_data || !nns || !distances || !found_cnt) return;
    
    try {
        auto h = static_cast<CuHNSWHandle*>(handle);
        h->instance->SearchGraph(query_data, num_queries, topk, ef_search, nns, distances, found_cnt);
    } catch (...) {
        // Log error
    }
}

} // extern "C"
