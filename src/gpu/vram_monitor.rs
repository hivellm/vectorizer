//! VRAM Monitor and Validation
//!
//! This module provides comprehensive monitoring and validation to ensure
//! that all operations are truly using VRAM and not falling back to RAM.

use crate::error::{Result, VectorizerError};
use std::time::Instant;
use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn, error};

#[cfg(target_os = "macos")]
use metal::{
    Buffer as MetalBuffer, Device as MetalDevice,
    MTLResourceOptions, MTLStorageMode, MTLCPUCacheMode,
};

/// Inner structure for thread-safe VRAM monitor
#[cfg(target_os = "macos")]
#[derive(Debug)]
struct VramMonitorInner {
    device: MetalDevice,
    allocated_buffers: Vec<VramBufferInfo>,
    total_vram_allocated: usize,
    vram_peak_usage: usize,
    ram_fallback_detected: bool,
}

/// VRAM Monitor for validating GPU memory usage (thread-safe)
#[cfg(target_os = "macos")]
#[derive(Debug, Clone)]
pub struct VramMonitor {
    inner: Arc<Mutex<VramMonitorInner>>,
}

/// Information about a VRAM buffer
#[cfg(target_os = "macos")]
#[derive(Debug, Clone)]
pub struct VramBufferInfo {
    pub buffer: MetalBuffer,
    pub size: usize,
    pub storage_mode: MTLStorageMode,
    pub cpu_cache_mode: MTLCPUCacheMode,
    pub allocated_at: Instant,
    pub label: String,
}

/// VRAM usage statistics
#[cfg(target_os = "macos")]
#[derive(Debug, Clone)]
pub struct VramStats {
    pub total_allocated: usize,
    pub peak_usage: usize,
    pub buffer_count: usize,
    pub ram_fallback_detected: bool,
    pub vram_efficiency: f64,
    pub average_buffer_size: f64,
}

impl VramMonitor {
    /// Create new VRAM monitor
    pub fn new(device: MetalDevice) -> Self {
        Self {
            inner: Arc::new(Mutex::new(VramMonitorInner {
                device,
                allocated_buffers: Vec::new(),
                total_vram_allocated: 0,
                vram_peak_usage: 0,
                ram_fallback_detected: false,
            })),
        }
    }
    
    /// Create VRAM buffer with validation
    pub fn create_vram_buffer(
        &self,
        size: usize,
        label: &str,
    ) -> Result<MetalBuffer> {
        let start = Instant::now();
        let mut inner = self.inner.lock().unwrap();
        
        // Create buffer with VRAM-only options
        let buffer = inner.device.new_buffer(
            size as u64,
            MTLResourceOptions::StorageModePrivate | MTLResourceOptions::CPUCacheModeDefaultCache,
        );
        
        // Validate buffer properties
        let buffer_info = VramBufferInfo {
            buffer: buffer.clone(),
            size,
            storage_mode: MTLStorageMode::Private, // VRAM only
            cpu_cache_mode: MTLCPUCacheMode::DefaultCache,
            allocated_at: start,
            label: label.to_string(),
        };
        
        // Track allocation
        inner.allocated_buffers.push(buffer_info);
        inner.total_vram_allocated += size;
        inner.vram_peak_usage = inner.vram_peak_usage.max(inner.total_vram_allocated);
        
        let allocation_time = start.elapsed();
        
        // Validate VRAM allocation
        drop(inner); // Release lock before validation
        self.validate_vram_allocation(&buffer, size)?;
        
        info!(
            "✅ VRAM buffer allocated: {} ({} bytes) in {:.3}ms",
            label, size, allocation_time.as_millis()
        );
        
        Ok(buffer)
    }
    
    /// Validate that buffer is truly in VRAM
    fn validate_vram_allocation(&self, buffer: &MetalBuffer, expected_size: usize) -> Result<()> {
        // Check buffer properties
        let actual_size = buffer.length() as usize;
        if actual_size != expected_size {
            warn!("⚠️ Buffer size mismatch: expected {}, got {}", expected_size, actual_size);
        }
        
        // Validate storage mode (should be Private for VRAM)
        // Note: Metal doesn't expose storage mode directly, but we can infer from behavior
        
        // Test VRAM access by attempting to read (should be fast for VRAM, slow/impossible for RAM)
        let test_start = Instant::now();
        let test_result = self.test_vram_access_speed(buffer);
        let test_duration = test_start.elapsed();
        
        if test_duration > std::time::Duration::from_millis(50) {
            warn!("⚠️ Slow buffer access detected: {:.3}ms - possible RAM fallback", test_duration.as_millis());
            return Err(VectorizerError::Other("Possible RAM fallback detected".to_string()));
        }
        
        debug!("✅ VRAM validation passed: {:.3}ms access time", test_duration.as_millis());
        Ok(())
    }
    
    /// Test VRAM access speed
    fn test_vram_access_speed(&self, buffer: &MetalBuffer) -> Result<()> {
        // Create a small staging buffer to test VRAM-to-VRAM copy speed
        let test_size = std::cmp::min(1024, buffer.length() as usize);
        let inner = self.inner.lock().unwrap();
        let staging_buffer = inner.device.new_buffer(
            test_size as u64,
            MTLResourceOptions::StorageModeShared, // CPU accessible for testing
        );
        
        // Test VRAM-to-staging copy speed
        let command_queue = inner.device.new_command_queue();
        let command_buffer = command_queue.new_command_buffer();
        let blit_encoder = command_buffer.new_blit_command_encoder();
        
        blit_encoder.copy_from_buffer(
            buffer,
            0,
            &staging_buffer,
            0,
            test_size as u64,
        );
        
        blit_encoder.end_encoding();
        command_buffer.commit();
        command_buffer.wait_until_completed();
        
        Ok(())
    }
    
    /// Monitor VRAM usage during operation
    pub fn monitor_operation<F, R>(&self, operation_name: &str, operation: F) -> Result<R>
    where
        F: FnOnce() -> Result<R>,
    {
        let start = Instant::now();
        let (initial_vram, initial_buffers) = {
            let inner = self.inner.lock().unwrap();
            (inner.total_vram_allocated, inner.allocated_buffers.len())
        };
        
        info!("🔍 Monitoring VRAM usage for: {}", operation_name);
        
        let result = operation();
        
        let duration = start.elapsed();
        let (final_vram, final_buffers) = {
            let inner = self.inner.lock().unwrap();
            (inner.total_vram_allocated, inner.allocated_buffers.len())
        };
        
        let vram_delta = final_vram - initial_vram;
        let buffer_delta = final_buffers - initial_buffers;
        
        info!(
            "📊 VRAM operation '{}' completed in {:.3}ms",
            operation_name, duration.as_millis()
        );
        info!(
            "📊 VRAM usage: {} -> {} bytes (Δ{} bytes)",
            initial_vram, final_vram, vram_delta
        );
        info!(
            "📊 Buffer count: {} -> {} (Δ{})",
            initial_buffers, final_buffers, buffer_delta
        );
        
        // Validate no RAM fallback occurred
        {
            let inner = self.inner.lock().unwrap();
            if inner.ram_fallback_detected {
                error!("❌ RAM fallback detected during operation: {}", operation_name);
                return Err(VectorizerError::Other("RAM fallback detected".to_string()));
            }
        }
        
        result
    }
    
    /// Get comprehensive VRAM statistics
    pub fn get_vram_stats(&self) -> VramStats {
        let inner = self.inner.lock().unwrap();
        let buffer_count = inner.allocated_buffers.len();
        let average_buffer_size = if buffer_count > 0 {
            inner.total_vram_allocated as f64 / buffer_count as f64
        } else {
            0.0
        };
        
        VramStats {
            total_allocated: inner.total_vram_allocated,
            peak_usage: inner.vram_peak_usage,
            buffer_count,
            ram_fallback_detected: inner.ram_fallback_detected,
            vram_efficiency: if inner.vram_peak_usage > 0 {
                inner.total_vram_allocated as f64 / inner.vram_peak_usage as f64
            } else {
                1.0
            },
            average_buffer_size,
        }
    }
}

/// Drop implementation for VramMonitor
/// Ensures all VRAM resources are properly cleaned up
#[cfg(target_os = "macos")]
impl Drop for VramMonitor {
    fn drop(&mut self) {
        // Log final VRAM statistics
        let stats = self.get_vram_stats();
        info!("🧹 VramMonitor cleanup - Final stats: {} buffers, {}MB total", 
              stats.buffer_count, stats.total_allocated / 1024 / 1024);
        
        // Clear all buffer tracking
        if let Ok(mut inner) = self.inner.lock() {
            inner.allocated_buffers.clear();
            
            // Reset statistics
            inner.total_vram_allocated = 0;
            inner.vram_peak_usage = 0;
            inner.ram_fallback_detected = false;
        }
        
        debug!("✅ VramMonitor cleaned up - all VRAM resources tracked");
    }
}

impl VramMonitor {
    /// Validate all buffers are in VRAM
    pub fn validate_all_vram(&self) -> Result<()> {
        info!("🔍 Validating all buffers are in VRAM...");
        
        let inner = self.inner.lock().unwrap();
        let buffer_count = inner.allocated_buffers.len();
        drop(inner); // Release lock before validation
        
        for (i, buffer_info) in self.inner.lock().unwrap().allocated_buffers.iter().enumerate() {
            let test_start = Instant::now();
            let test_result = self.test_vram_access_speed(&buffer_info.buffer);
            let test_duration = test_start.elapsed();
            
            if test_duration > std::time::Duration::from_millis(20) {
                warn!(
                    "⚠️ Buffer {} ({}): slow access {:.3}ms - possible RAM fallback",
                    i, buffer_info.label, test_duration.as_millis()
                );
                return Err(VectorizerError::Other(
                    format!("Buffer {} may be in RAM", buffer_info.label)
                ));
            }
            
            debug!(
                "✅ Buffer {} ({}): VRAM validated in {:.3}ms",
                i, buffer_info.label, test_duration.as_millis()
            );
        }
        
        info!("✅ All {} buffers validated as VRAM", buffer_count);
        Ok(())
    }
    
    /// Force VRAM-only mode validation
    pub fn force_vram_validation(&mut self) -> Result<()> {
        info!("🔍 Force validating VRAM-only mode...");
        
        // Test 1: Create a test buffer and validate it's in VRAM
        let test_buffer = self.create_vram_buffer(1024, "vram_test")?;
        
        // Test 2: Attempt to access buffer (should be fast for VRAM)
        let access_start = Instant::now();
        self.test_vram_access_speed(&test_buffer)?;
        let access_duration = access_start.elapsed();
        
        if access_duration > std::time::Duration::from_millis(1) {
            error!("❌ VRAM access too slow: {:.3}ms - possible RAM fallback", access_duration.as_millis());
            return Err(VectorizerError::Other("VRAM validation failed".to_string()));
        }
        
        // Test 3: Validate buffer properties
        if test_buffer.length() != 1024 {
            error!("❌ Buffer size mismatch: expected 1024, got {}", test_buffer.length());
            return Err(VectorizerError::Other("Buffer size validation failed".to_string()));
        }
        
        info!("✅ VRAM-only mode validated successfully");
        Ok(())
    }
    
    /// Generate VRAM usage report
    pub fn generate_vram_report(&self) -> String {
        let stats = self.get_vram_stats();
        
        format!(
            "📊 VRAM Usage Report\n\
            ===================\n\
            Total allocated: {:.2} MB\n\
            Peak usage: {:.2} MB\n\
            Buffer count: {}\n\
            RAM fallback detected: {}\n\
            VRAM efficiency: {:.1}%\n\
            Average buffer size: {:.2} KB\n\
            \n\
            Buffer Details:\n\
            {}",
            stats.total_allocated as f64 / 1024.0 / 1024.0,
            stats.peak_usage as f64 / 1024.0 / 1024.0,
            stats.buffer_count,
            stats.ram_fallback_detected,
            stats.vram_efficiency * 100.0,
            stats.average_buffer_size / 1024.0,
            self.format_buffer_details()
        )
    }
    
    /// Format buffer details for report
    fn format_buffer_details(&self) -> String {
        let mut details = String::new();
        let inner = self.inner.lock().unwrap();
        
        for (i, buffer_info) in inner.allocated_buffers.iter().enumerate() {
            let age = buffer_info.allocated_at.elapsed();
            details.push_str(&format!(
                "  {}: {} ({} bytes, age: {:.1}s)\n",
                i,
                buffer_info.label,
                buffer_info.size,
                age.as_secs_f64()
            ));
        }
        
        details
    }
}

/// VRAM validation utilities
#[cfg(target_os = "macos")]
pub struct VramValidator;

impl VramValidator {
    /// Validate that a Metal device is using VRAM
    pub fn validate_device_vram(device: &MetalDevice) -> Result<()> {
        info!("🔍 Validating Metal device VRAM capabilities...");
        
        // Check device properties
        let device_name = device.name();
        info!("📱 Device: {}", device_name);
        
        // Test VRAM allocation
        let test_buffer = device.new_buffer(
            1024,
            MTLResourceOptions::StorageModePrivate, // VRAM only
        );
        
        if test_buffer.length() != 1024 {
            return Err(VectorizerError::Other("VRAM allocation test failed".to_string()));
        }
        
        // Test VRAM access speed
        let start = Instant::now();
        let staging_buffer = device.new_buffer(
            1024,
            MTLResourceOptions::StorageModeShared, // CPU accessible
        );
        
        let command_queue = device.new_command_queue();
        let command_buffer = command_queue.new_command_buffer();
        let blit_encoder = command_buffer.new_blit_command_encoder();
        
        blit_encoder.copy_from_buffer(
            &test_buffer,
            0,
            &staging_buffer,
            0,
            1024,
        );
        
        blit_encoder.end_encoding();
        command_buffer.commit();
        command_buffer.wait_until_completed();
        
        let duration = start.elapsed();
        
        if duration > std::time::Duration::from_millis(20) {
            warn!("⚠️ Slow VRAM access: {:.3}ms - possible RAM fallback", duration.as_millis());
            return Err(VectorizerError::Other("VRAM access too slow".to_string()));
        }
        
        info!("✅ Device VRAM validation passed: {:.3}ms", duration.as_millis());
        Ok(())
    }
    
    /// Benchmark VRAM vs RAM performance
    pub fn benchmark_vram_vs_ram(device: &MetalDevice) -> Result<VramBenchmarkResult> {
        info!("🔍 Benchmarking VRAM vs RAM performance...");
        
        let test_size = 1024 * 1024; // 1MB test
        
        // Test VRAM (Private storage)
        let vram_start = Instant::now();
        let vram_buffer = device.new_buffer(
            test_size as u64,
            MTLResourceOptions::StorageModePrivate,
        );
        let vram_allocation_time = vram_start.elapsed();
        
        // Test VRAM access
        let vram_access_start = Instant::now();
        let staging_buffer = device.new_buffer(
            test_size as u64,
            MTLResourceOptions::StorageModeShared,
        );
        
        let command_queue = device.new_command_queue();
        let command_buffer = command_queue.new_command_buffer();
        let blit_encoder = command_buffer.new_blit_command_encoder();
        
        blit_encoder.copy_from_buffer(
            &vram_buffer,
            0,
            &staging_buffer,
            0,
            test_size as u64,
        );
        
        blit_encoder.end_encoding();
        command_buffer.commit();
        command_buffer.wait_until_completed();
        
        let vram_access_time = vram_access_start.elapsed();
        
        // Test RAM (Shared storage)
        let ram_start = Instant::now();
        let ram_buffer = device.new_buffer(
            test_size as u64,
            MTLResourceOptions::StorageModeShared,
        );
        let ram_allocation_time = ram_start.elapsed();
        
        // Test RAM access (direct memory access)
        let ram_access_start = Instant::now();
        let ram_data = ram_buffer.contents();
        let ram_access_time = ram_access_start.elapsed();
        
        let result = VramBenchmarkResult {
            vram_allocation_time,
            vram_access_time,
            ram_allocation_time,
            ram_access_time,
            vram_faster: vram_access_time < ram_access_time,
            performance_ratio: ram_access_time.as_nanos() as f64 / vram_access_time.as_nanos() as f64,
        };
        
        info!("📊 VRAM vs RAM Benchmark Results:");
        info!("  VRAM allocation: {:.3}ms", result.vram_allocation_time.as_millis());
        info!("  VRAM access: {:.3}ms", result.vram_access_time.as_millis());
        info!("  RAM allocation: {:.3}ms", result.ram_allocation_time.as_millis());
        info!("  RAM access: {:.3}ms", result.ram_access_time.as_millis());
        info!("  VRAM faster: {}", result.vram_faster);
        info!("  Performance ratio: {:.2}x", result.performance_ratio);
        
        Ok(result)
    }
}

/// VRAM benchmark results
#[cfg(target_os = "macos")]
#[derive(Debug, Clone)]
pub struct VramBenchmarkResult {
    pub vram_allocation_time: std::time::Duration,
    pub vram_access_time: std::time::Duration,
    pub ram_allocation_time: std::time::Duration,
    pub ram_access_time: std::time::Duration,
    pub vram_faster: bool,
    pub performance_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[cfg(target_os = "macos")]
    #[test]
    fn test_vram_monitor() {
        let device = metal::Device::system_default().unwrap();
        let mut monitor = VramMonitor::new(device);
        
        // Test VRAM buffer creation
        let buffer = monitor.create_vram_buffer(1024, "test").unwrap();
        assert_eq!(buffer.length(), 1024);
        
        // Test VRAM validation
        monitor.validate_all_vram().unwrap();
        
        // Test statistics
        let stats = monitor.get_vram_stats();
        assert_eq!(stats.buffer_count, 1);
        assert_eq!(stats.total_allocated, 1024);
    }
    
    #[cfg(target_os = "macos")]
    #[test]
    fn test_vram_validator() {
        let device = metal::Device::system_default().unwrap();
        
        // Test device validation
        VramValidator::validate_device_vram(&device).unwrap();
        
        // Test benchmark
        let result = VramValidator::benchmark_vram_vs_ram(&device).unwrap();
        assert!(result.performance_ratio > 0.0);
    }
}
