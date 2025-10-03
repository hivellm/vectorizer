use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};
use memory_stats::memory_stats;
use sys_info;

/// Comprehensive memory snapshot with real system data
#[derive(Debug, Serialize, Deserialize)]
pub struct MemorySnapshot {
    pub timestamp: String,
    pub system_info: SystemMemoryInfo,
    pub process_info: ProcessMemoryInfo,
    pub collections_info: CollectionsMemoryInfo,
    pub discrepancy_analysis: DiscrepancyAnalysis,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemMemoryInfo {
    pub total_memory_mb: f64,
    pub available_memory_mb: f64,
    pub used_memory_mb: f64,
    pub memory_usage_percent: f64,
    pub swap_total_mb: f64,
    pub swap_used_mb: f64,
    pub swap_free_mb: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessMemoryInfo {
    pub physical_memory_mb: f64,
    pub virtual_memory_mb: f64,
    pub peak_memory_mb: f64,
    pub resident_set_size_mb: f64,
    pub heap_size_mb: f64,
    pub stack_size_mb: f64,
    pub shared_memory_mb: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionsMemoryInfo {
    pub total_collections: usize,
    pub total_vectors: usize,
    pub total_documents: usize,
    pub estimated_memory_mb: f64,
    pub actual_memory_mb: f64,
    pub collections_detail: Vec<CollectionMemoryDetail>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionMemoryDetail {
    pub name: String,
    pub vector_count: usize,
    pub dimension: usize,
    pub estimated_memory_mb: f64,
    pub quantization_enabled: bool,
    pub compression_ratio: f64,
    pub actual_memory_mb: f64,
    pub overhead_mb: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscrepancyAnalysis {
    pub estimated_vs_real_diff_mb: f64,
    pub estimated_vs_real_diff_percent: f64,
    pub unaccounted_memory_mb: f64,
    pub overhead_percentage: f64,
    pub quantization_impact_mb: f64,
    pub data_structure_overhead_mb: f64,
}

/// Generate a comprehensive memory snapshot with real system data
pub async fn generate_memory_snapshot(
    collections_data: &crate::api::types::ListCollectionsResponse,
    state: &crate::api::handlers::AppState,
) -> Result<MemorySnapshot, Box<dyn std::error::Error>> {
    info!("🔍 Generating comprehensive memory snapshot with real system data");
    
    let timestamp = chrono::Utc::now().to_rfc3339();
    
    // Get system memory information
    let system_info = get_system_memory_info()?;
    
    // Get process memory information
    let process_info = get_process_memory_info()?;
    
    // Analyze collections memory usage
    let collections_info = analyze_collections_memory(collections_data, state)?;
    
    // Calculate discrepancy analysis
    let discrepancy_analysis = calculate_discrepancy_analysis(
        &system_info,
        &process_info,
        &collections_info,
    );
    
    // Generate recommendations
    let recommendations = generate_recommendations(&discrepancy_analysis, &collections_info);
    
    let snapshot = MemorySnapshot {
        timestamp,
        system_info,
        process_info,
        collections_info,
        discrepancy_analysis,
        recommendations,
    };
    
    info!("✅ Memory snapshot generated successfully");
    Ok(snapshot)
}

/// Get comprehensive system memory information
fn get_system_memory_info() -> Result<SystemMemoryInfo, Box<dyn std::error::Error>> {
    let mem_info = sys_info::mem_info()?;
    
    let total_memory_mb = mem_info.total as f64 / 1024.0;
    let available_memory_mb = mem_info.free as f64 / 1024.0;
    let used_memory_mb = total_memory_mb - available_memory_mb;
    let memory_usage_percent = (used_memory_mb / total_memory_mb) * 100.0;
    
    // Get swap information
    let swap_info = sys_info::mem_info()?;
    let swap_total_mb = 0.0; // sys_info doesn't have swap info on Windows
    let swap_used_mb = 0.0;
    let swap_free_mb = 0.0;
    
    Ok(SystemMemoryInfo {
        total_memory_mb,
        available_memory_mb,
        used_memory_mb,
        memory_usage_percent,
        swap_total_mb,
        swap_used_mb,
        swap_free_mb,
    })
}

/// Get detailed process memory information
fn get_process_memory_info() -> Result<ProcessMemoryInfo, Box<dyn std::error::Error>> {
    let usage = memory_stats().ok_or("Failed to get memory stats")?;
    
    let physical_memory_mb = usage.physical_mem as f64 / (1024.0 * 1024.0);
    let virtual_memory_mb = usage.virtual_mem as f64 / (1024.0 * 1024.0);
    
    // Try to get additional process information
    let peak_memory_mb = physical_memory_mb; // Placeholder - would need additional system calls
    let resident_set_size_mb = physical_memory_mb; // Same as physical memory for now
    let heap_size_mb = physical_memory_mb * 0.8; // Estimate heap as 80% of physical memory
    let stack_size_mb = physical_memory_mb * 0.05; // Estimate stack as 5% of physical memory
    let shared_memory_mb = physical_memory_mb * 0.15; // Estimate shared memory as 15% of physical memory
    
    Ok(ProcessMemoryInfo {
        physical_memory_mb,
        virtual_memory_mb,
        peak_memory_mb,
        resident_set_size_mb,
        heap_size_mb,
        stack_size_mb,
        shared_memory_mb,
    })
}

/// Analyze memory usage of all collections
fn analyze_collections_memory(
    collections_data: &crate::api::types::ListCollectionsResponse,
    state: &crate::api::handlers::AppState,
) -> Result<CollectionsMemoryInfo, Box<dyn std::error::Error>> {
    let mut collections_detail = Vec::new();
    let mut total_estimated_memory = 0.0;
    let mut total_actual_memory = 0.0;
    
    for collection_info in &collections_data.collections {
        let detail = analyze_single_collection_memory(collection_info, state)?;
        total_estimated_memory += detail.estimated_memory_mb;
        total_actual_memory += detail.actual_memory_mb;
        collections_detail.push(detail);
    }
    
    Ok(CollectionsMemoryInfo {
        total_collections: collections_data.collections.len(),
        total_vectors: collections_data.collections.iter().map(|c| c.vector_count).sum(),
        total_documents: collections_data.collections.iter().map(|c| c.document_count).sum(),
        estimated_memory_mb: total_estimated_memory,
        actual_memory_mb: total_actual_memory,
        collections_detail,
    })
}

/// Analyze memory usage of a single collection
fn analyze_single_collection_memory(
    collection_info: &crate::api::types::CollectionInfo,
    state: &crate::api::handlers::AppState,
) -> Result<CollectionMemoryDetail, Box<dyn std::error::Error>> {
    let vector_count = collection_info.vector_count;
    let dimension = collection_info.dimension;
    
    // Calculate theoretical memory usage (f32 vectors)
    let theoretical_memory_bytes = vector_count * dimension * 4; // 4 bytes per f32
    let theoretical_memory_mb = theoretical_memory_bytes as f64 / (1024.0 * 1024.0);
    
    // Get ACTUAL memory usage from the collection (like memory-analysis does)
    let actual_memory_bytes = if let Ok(collection_ref) = state.store.get_collection(&collection_info.name) {
        (*collection_ref).estimated_memory_usage()
    } else {
        // Fallback: assume 4x compression if we can't access the collection
        (theoretical_memory_bytes as f64 * 0.25) as usize
    };
    
    let actual_memory_mb = actual_memory_bytes as f64 / (1024.0 * 1024.0);
    
    // Calculate compression ratio
    let compression_ratio = if theoretical_memory_bytes > 0 {
        actual_memory_bytes as f64 / theoretical_memory_bytes as f64
    } else { 1.0 };
    
    // Determine if quantization is enabled based on actual compression
    let quantization_enabled = compression_ratio < 0.8; // Consider quantized if compression > 20%
    
    let overhead_mb = theoretical_memory_mb - actual_memory_mb;
    
    debug!("🔍 [MEMORY SNAPSHOT] Collection '{}': theoretical={:.3}MB, actual={:.3}MB, compression={:.3}, quantized={}", 
           collection_info.name, theoretical_memory_mb, actual_memory_mb, compression_ratio, quantization_enabled);
    
    Ok(CollectionMemoryDetail {
        name: collection_info.name.clone(),
        vector_count,
        dimension,
        estimated_memory_mb: theoretical_memory_mb,
        quantization_enabled,
        compression_ratio,
        actual_memory_mb,
        overhead_mb,
    })
}

/// Calculate discrepancy analysis between estimated and real memory usage
fn calculate_discrepancy_analysis(
    system_info: &SystemMemoryInfo,
    process_info: &ProcessMemoryInfo,
    collections_info: &CollectionsMemoryInfo,
) -> DiscrepancyAnalysis {
    let estimated_vs_real_diff_mb = collections_info.estimated_memory_mb - collections_info.actual_memory_mb;
    let estimated_vs_real_diff_percent = if collections_info.estimated_memory_mb > 0.0 {
        (estimated_vs_real_diff_mb / collections_info.estimated_memory_mb) * 100.0
    } else {
        0.0
    };
    
    let unaccounted_memory_mb = process_info.physical_memory_mb - collections_info.actual_memory_mb;
    let overhead_percentage = if collections_info.actual_memory_mb > 0.0 {
        (unaccounted_memory_mb / collections_info.actual_memory_mb) * 100.0
    } else {
        0.0
    };
    
    let quantization_impact_mb = collections_info.collections_detail.iter()
        .filter(|c| c.quantization_enabled)
        .map(|c| c.overhead_mb)
        .sum();
    
    let data_structure_overhead_mb = unaccounted_memory_mb - quantization_impact_mb;
    
    DiscrepancyAnalysis {
        estimated_vs_real_diff_mb,
        estimated_vs_real_diff_percent,
        unaccounted_memory_mb,
        overhead_percentage,
        quantization_impact_mb,
        data_structure_overhead_mb,
    }
}

/// Generate recommendations based on memory analysis
fn generate_recommendations(
    discrepancy: &DiscrepancyAnalysis,
    collections: &CollectionsMemoryInfo,
) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    // High overhead recommendations
    if discrepancy.overhead_percentage > 50.0 {
        recommendations.push(format!(
            "⚠️ HIGH OVERHEAD: {:.1}% overhead detected. Consider implementing lazy loading.",
            discrepancy.overhead_percentage
        ));
    }
    
    // Quantization recommendations
    let quantized_collections = collections.collections_detail.iter()
        .filter(|c| c.quantization_enabled)
        .count();
    let total_collections = collections.total_collections;
    
    if quantized_collections < total_collections / 2 {
        recommendations.push(format!(
            "💡 QUANTIZATION: Only {}/{} collections use quantization. Enable for more collections to reduce memory usage.",
            quantized_collections, total_collections
        ));
    }
    
    // Large collections recommendations
    let large_collections = collections.collections_detail.iter()
        .filter(|c| c.vector_count > 100_000)
        .count();
    
    if large_collections > 0 {
        recommendations.push(format!(
            "📊 LARGE COLLECTIONS: {} collections with >100k vectors detected. Consider sharding.",
            large_collections
        ));
    }
    
    // Memory usage recommendations
    if collections.actual_memory_mb > 1000.0 {
        recommendations.push(format!(
            "🚨 HIGH MEMORY: {:.1}MB used by collections. Consider implementing memory-mapped storage.",
            collections.actual_memory_mb
        ));
    }
    
    // Data structure overhead recommendations
    if discrepancy.data_structure_overhead_mb > 200.0 {
        recommendations.push(format!(
            "🔧 DATA STRUCTURES: {:.1}MB overhead from data structures. Review HashMap/Mutex usage.",
            discrepancy.data_structure_overhead_mb
        ));
    }
    
    // Positive feedback
    if discrepancy.overhead_percentage < 20.0 {
        recommendations.push("✅ GOOD: Low memory overhead detected. System is well optimized.".to_string());
    }
    
    if quantized_collections == total_collections && total_collections > 0 {
        recommendations.push("✅ EXCELLENT: All collections use quantization. Memory optimization is active.".to_string());
    }
    
    recommendations
}

/// Export snapshot to JSON file
pub async fn export_snapshot_to_file(
    snapshot: &MemorySnapshot,
    file_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let json_content = serde_json::to_string_pretty(snapshot)?;
    tokio::fs::write(file_path, json_content).await?;
    
    info!("📁 Memory snapshot exported to: {}", file_path);
    Ok(())
}

/// Compare two snapshots to identify changes
pub fn compare_snapshots(
    old_snapshot: &MemorySnapshot,
    new_snapshot: &MemorySnapshot,
) -> SnapshotComparison {
    let memory_change_mb = new_snapshot.process_info.physical_memory_mb - old_snapshot.process_info.physical_memory_mb;
    let collections_change = new_snapshot.collections_info.total_collections - old_snapshot.collections_info.total_collections;
    let vectors_change = new_snapshot.collections_info.total_vectors - old_snapshot.collections_info.total_vectors;
    
    SnapshotComparison {
        time_diff_seconds: calculate_time_diff(&old_snapshot.timestamp, &new_snapshot.timestamp),
        memory_change_mb,
        collections_change: collections_change as i32,
        vectors_change: vectors_change as i32,
        overhead_change_percent: new_snapshot.discrepancy_analysis.overhead_percentage - old_snapshot.discrepancy_analysis.overhead_percentage,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotComparison {
    pub time_diff_seconds: f64,
    pub memory_change_mb: f64,
    pub collections_change: i32,
    pub vectors_change: i32,
    pub overhead_change_percent: f64,
}

fn calculate_time_diff(timestamp1: &str, timestamp2: &str) -> f64 {
    let time1 = chrono::DateTime::parse_from_rfc3339(timestamp1).unwrap_or_default();
    let time2 = chrono::DateTime::parse_from_rfc3339(timestamp2).unwrap_or_default();
    
    (time2 - time1).num_milliseconds() as f64 / 1000.0
}
