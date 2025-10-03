//! Real CUDA HNSW bindings using cuhnsw C++ library
//! 
//! This module provides FFI bindings to the cuhnsw CUDA kernels

use std::os::raw::{c_int, c_float, c_char, c_void};
use std::ptr;
use std::ffi::CString;
use crate::error::{Result, VectorizerError};


// Declare FFI bindings conditionally
#[cfg_attr(feature = "cuda_library_available", link(name = "cuhnsw", kind = "static"))]
unsafe extern "C" {
    // CuHNSW constructor/destructor
    fn cuhnsw_create() -> *mut c_void;
    fn cuhnsw_destroy(handle: *mut c_void);

    // Initialize with config file
    fn cuhnsw_init(handle: *mut c_void, config_path: *const c_char) -> bool;

    // Set data
    fn cuhnsw_set_data(
        handle: *mut c_void,
        data: *const c_float,
        num_data: c_int,
        num_dims: c_int
    );

    // Set random levels
    fn cuhnsw_set_random_levels(
        handle: *mut c_void,
        levels: *const c_int
    );

    // Build graph
    fn cuhnsw_build_graph(handle: *mut c_void);

    // Save/Load index
    fn cuhnsw_save_index(handle: *mut c_void, file_path: *const c_char);
    fn cuhnsw_load_index(handle: *mut c_void, file_path: *const c_char);

    // Search KNN
    fn cuhnsw_search_knn(
        handle: *mut c_void,
        query_data: *const c_float,
        num_queries: c_int,
        topk: c_int,
        ef_search: c_int,
        nns: *mut c_int,
        distances: *mut c_float,
        found_cnt: *mut c_int
    );
}

// Safe wrapper functions that handle the conditional availability
#[cfg(feature = "cuda_library_available")]
mod real_ffi {
    use super::*;

    pub unsafe fn cuhnsw_create_safe() -> *mut c_void {
        cuhnsw_create()
    }

    pub unsafe fn cuhnsw_destroy_safe(handle: *mut c_void) {
        cuhnsw_destroy(handle)
    }

    pub unsafe fn cuhnsw_init_safe(handle: *mut c_void, config_path: *const c_char) -> bool {
        cuhnsw_init(handle, config_path)
    }

    pub unsafe fn cuhnsw_set_data_safe(
        handle: *mut c_void,
        data: *const c_float,
        num_data: c_int,
        num_dims: c_int
    ) {
        cuhnsw_set_data(handle, data, num_data, num_dims)
    }

    pub unsafe fn cuhnsw_set_random_levels_safe(
        handle: *mut c_void,
        levels: *const c_int
    ) {
        cuhnsw_set_random_levels(handle, levels)
    }

    pub unsafe fn cuhnsw_build_graph_safe(handle: *mut c_void) {
        cuhnsw_build_graph(handle)
    }

    pub unsafe fn cuhnsw_save_index_safe(handle: *mut c_void, file_path: *const c_char) {
        cuhnsw_save_index(handle, file_path)
    }

    pub unsafe fn cuhnsw_load_index_safe(handle: *mut c_void, file_path: *const c_char) {
        cuhnsw_load_index(handle, file_path)
    }

    pub unsafe fn cuhnsw_search_knn_safe(
        handle: *mut c_void,
        query_data: *const c_float,
        num_queries: c_int,
        topk: c_int,
        ef_search: c_int,
        nns: *mut c_int,
        distances: *mut c_float,
        found_cnt: *mut c_int
    ) {
        cuhnsw_search_knn(handle, query_data, num_queries, topk, ef_search, nns, distances, found_cnt)
    }
}

#[cfg(not(feature = "cuda_library_available"))]
mod stub_ffi {
    use super::*;

    pub unsafe fn cuhnsw_create_safe() -> *mut c_void {
        ptr::null_mut()
    }

    pub unsafe fn cuhnsw_destroy_safe(_handle: *mut c_void) {
        // Do nothing
    }

    pub unsafe fn cuhnsw_init_safe(_handle: *mut c_void, _config_path: *const c_char) -> bool {
        true
    }

    pub unsafe fn cuhnsw_set_data_safe(
        _handle: *mut c_void,
        _data: *const c_float,
        _num_data: c_int,
        _num_dims: c_int
    ) {
        // Do nothing
    }

    pub unsafe fn cuhnsw_set_random_levels_safe(
        _handle: *mut c_void,
        _levels: *const c_int
    ) {
        // Do nothing
    }

    pub unsafe fn cuhnsw_build_graph_safe(_handle: *mut c_void) {
        // Do nothing
    }

    pub unsafe fn cuhnsw_save_index_safe(_handle: *mut c_void, _file_path: *const c_char) {
        // Do nothing
    }

    pub unsafe fn cuhnsw_load_index_safe(_handle: *mut c_void, _file_path: *const c_char) {
        // Do nothing
    }

    pub unsafe fn cuhnsw_search_knn_safe(
        _handle: *mut c_void,
        _query_data: *const c_float,
        _num_queries: c_int,
        _topk: c_int,
        _ef_search: c_int,
        nns: *mut c_int,
        distances: *mut c_float,
        found_cnt: *mut c_int
    ) {
        // Fill with dummy results
        for i in 0..(_num_queries * _topk) as usize {
            unsafe {
                *nns.add(i) = i as c_int;
                *distances.add(i) = i as c_float * 0.1;
            }
        }
        for i in 0.._num_queries as usize {
            unsafe {
                *found_cnt.add(i) = _topk;
            }
        }
    }
}

/// Safe Rust wrapper for CuHNSW
pub struct CuHNSW {
    handle: *mut c_void,
}

impl CuHNSW {
    /// Create new CuHNSW instance
    pub fn new() -> Result<Self> {
        unsafe {
            #[cfg(feature = "cuda_library_available")]
            let handle = real_ffi::cuhnsw_create_safe();
            #[cfg(not(feature = "cuda_library_available"))]
            let handle = stub_ffi::cuhnsw_create_safe();

            if handle.is_null() {
                return Err(VectorizerError::InternalError(
                    "Failed to create CuHNSW instance".to_string()
                ));
            }
            Ok(Self { handle })
        }
    }
    
    /// Initialize with config
    pub fn init(&mut self, config: &CuHNSWConfig) -> Result<()> {
        // Create temporary config file
        let config_json = serde_json::to_string(config)?;
        let temp_file = std::env::temp_dir().join("cuhnsw_config.json");
        std::fs::write(&temp_file, config_json)?;

        let config_path = CString::new(temp_file.to_str().unwrap())
            .map_err(|_| VectorizerError::InternalError("Invalid path string".to_string()))?;

        unsafe {
            #[cfg(feature = "cuda_library_available")]
            let success = real_ffi::cuhnsw_init_safe(self.handle, config_path.as_ptr());
            #[cfg(not(feature = "cuda_library_available"))]
            let success = stub_ffi::cuhnsw_init_safe(self.handle, config_path.as_ptr());

            if !success {
                return Err(VectorizerError::InternalError(
                    "Failed to initialize CuHNSW".to_string()
                ));
            }
        }

        // Clean up temp file
        std::fs::remove_file(temp_file).ok();
        Ok(())
    }
    
    /// Set data for indexing
    pub fn set_data(&mut self, data: &[f32], num_dims: usize) -> Result<()> {
        let num_data = data.len() / num_dims;
        if data.len() % num_dims != 0 {
            return Err(VectorizerError::ConfigurationError(
                "Data length must be divisible by num_dims".to_string()
            ));
        }

        unsafe {
            #[cfg(feature = "cuda_library_available")]
            real_ffi::cuhnsw_set_data_safe(
                self.handle,
                data.as_ptr(),
                num_data as c_int,
                num_dims as c_int
            );
            #[cfg(not(feature = "cuda_library_available"))]
            stub_ffi::cuhnsw_set_data_safe(
                self.handle,
                data.as_ptr(),
                num_data as c_int,
                num_dims as c_int
            );
        }
        Ok(())
    }
    
    /// Build HNSW graph on GPU
    pub fn build_graph(&mut self, levels: &[i32]) -> Result<()> {
        unsafe {
            #[cfg(feature = "cuda_library_available")]
            {
                real_ffi::cuhnsw_set_random_levels_safe(self.handle, levels.as_ptr());
                real_ffi::cuhnsw_build_graph_safe(self.handle);
            }
            #[cfg(not(feature = "cuda_library_available"))]
            {
                stub_ffi::cuhnsw_set_random_levels_safe(self.handle, levels.as_ptr());
                stub_ffi::cuhnsw_build_graph_safe(self.handle);
            }
        }
        Ok(())
    }
    
    /// Search k nearest neighbors
    pub fn search_knn(
        &self,
        queries: &[f32],
        num_queries: usize,
        dimension: usize,
        k: usize,
        ef_search: usize,
    ) -> Result<(Vec<i32>, Vec<f32>)> {
        let mut nns = vec![0i32; num_queries * k];
        let mut distances = vec![0.0f32; num_queries * k];
        let mut found_cnt = vec![0i32; num_queries];

        unsafe {
            #[cfg(feature = "cuda_library_available")]
            real_ffi::cuhnsw_search_knn_safe(
                self.handle,
                queries.as_ptr(),
                num_queries as c_int,
                k as c_int,
                ef_search as c_int,
                nns.as_mut_ptr(),
                distances.as_mut_ptr(),
                found_cnt.as_mut_ptr()
            );
            #[cfg(not(feature = "cuda_library_available"))]
            stub_ffi::cuhnsw_search_knn_safe(
                self.handle,
                queries.as_ptr(),
                num_queries as c_int,
                k as c_int,
                ef_search as c_int,
                nns.as_mut_ptr(),
                distances.as_mut_ptr(),
                found_cnt.as_mut_ptr()
            );
        }

        Ok((nns, distances))
    }
    
    /// Save index to file
    pub fn save_index(&self, path: &str) -> Result<()> {
        let c_path = CString::new(path)
            .map_err(|_| VectorizerError::InternalError("Invalid path string".to_string()))?;
        unsafe {
            #[cfg(feature = "cuda_library_available")]
            real_ffi::cuhnsw_save_index_safe(self.handle, c_path.as_ptr());
            #[cfg(not(feature = "cuda_library_available"))]
            stub_ffi::cuhnsw_save_index_safe(self.handle, c_path.as_ptr());
        }
        Ok(())
    }

    /// Load index from file
    pub fn load_index(&mut self, path: &str) -> Result<()> {
        let c_path = CString::new(path)
            .map_err(|_| VectorizerError::InternalError("Invalid path string".to_string()))?;
        unsafe {
            #[cfg(feature = "cuda_library_available")]
            real_ffi::cuhnsw_load_index_safe(self.handle, c_path.as_ptr());
            #[cfg(not(feature = "cuda_library_available"))]
            stub_ffi::cuhnsw_load_index_safe(self.handle, c_path.as_ptr());
        }
        Ok(())
    }
}

impl Drop for CuHNSW {
    fn drop(&mut self) {
        unsafe {
            if !self.handle.is_null() {
                #[cfg(feature = "cuda_library_available")]
                real_ffi::cuhnsw_destroy_safe(self.handle);
                #[cfg(not(feature = "cuda_library_available"))]
                stub_ffi::cuhnsw_destroy_safe(self.handle);
            }
        }
    }
}

/// CuHNSW configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CuHNSWConfig {
    pub seed: i32,
    pub c_log_level: i32,
    pub py_log_level: i32,
    pub max_m: i32,
    pub max_m0: i32,
    pub ef_construction: i32,
    pub level_mult: Option<f64>,
    pub save_remains: bool,
    pub hyper_threads: f64,
    pub block_dim: i32,
    pub dist_type: String,
    pub visited_table_size: Option<i32>,
    pub visited_list_size: i32,
    pub nrz: bool,
    pub reverse_cand: bool,
    pub heuristic_coef: f64,
}

impl Default for CuHNSWConfig {
    fn default() -> Self {
        Self {
            seed: 777,
            c_log_level: 2,
            py_log_level: 2,
            max_m: 12,
            max_m0: 24,
            ef_construction: 150,
            level_mult: None,
            save_remains: false,
            hyper_threads: 10.0,
            block_dim: 32,
            dist_type: "dot".to_string(),
            visited_table_size: None,
            visited_list_size: 8192,
            nrz: false,
            reverse_cand: false,
            heuristic_coef: 0.25,
        }
    }
}

// Safe to send across threads
unsafe impl Send for CuHNSW {}
unsafe impl Sync for CuHNSW {}
