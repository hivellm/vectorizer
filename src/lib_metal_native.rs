//! # Vectorizer - Metal Native Library
//!
//! Simplified library that only supports Metal Native collections.


pub mod db {
    pub mod vector_store_metal_native_only;
}

pub mod gpu {
    pub mod metal_native;
    pub mod metal_native_legacy;
    pub mod metal_buffer_pool;
    pub mod vram_monitor;
}

pub mod models;
pub mod error;
pub mod embedding;
pub mod batch;
pub mod quantization;
pub mod persistence;
pub mod api;
pub mod mcp;

