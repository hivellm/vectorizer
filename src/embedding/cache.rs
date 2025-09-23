//! High-performance embedding cache with memory-mapped persistence
//!
//! This module provides efficient caching for embeddings with:
//! - Memory-mapped files for zero-copy loading
//! - Content hashing for incremental builds
//! - Parallel cache population
//! - Arrow/Parquet support for analytics

use anyhow::Result;
use memmap2::Mmap;
use parking_lot::RwLock;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, info};
use xxhash_rust::xxh3::xxh3_64;

#[cfg(feature = "arrow")]
use arrow::array::{Float32Array, StringArray};
#[cfg(feature = "arrow")]
use arrow::datatypes::{DataType, Field, Schema};
#[cfg(feature = "arrow")]
use arrow::record_batch::RecordBatch;

#[cfg(feature = "parquet")]
use parquet::arrow::ArrowWriter;
#[cfg(feature = "parquet")]
use parquet::basic::Compression;
#[cfg(feature = "parquet")]
use parquet::file::properties::WriterProperties;

/// Cache entry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Content hash
    pub content_hash: u64,
    /// File path
    pub file_path: String,
    /// Embedding offset in cache
    pub offset: usize,
    /// Embedding dimension
    pub dimension: usize,
    /// Timestamp
    pub timestamp: u64,
}

/// Embedding cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Cache directory
    pub cache_dir: PathBuf,
    /// Maximum cache size in bytes
    pub max_size: usize,
    /// Enable memory mapping
    pub use_mmap: bool,
    /// Cache file prefix
    pub prefix: String,
    /// Shard count for parallel access
    pub num_shards: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            cache_dir: PathBuf::from("./cache/embeddings"),
            max_size: 10 * 1024 * 1024 * 1024, // 10GB
            use_mmap: true,
            prefix: "embed".to_string(),
            num_shards: 16,
        }
    }
}

/// High-performance embedding cache
pub struct EmbeddingCache {
    config: CacheConfig,
    shards: Vec<Arc<RwLock<CacheShard>>>,
    metadata: Arc<RwLock<CacheMetadata>>,
}

/// Cache shard for parallel access
struct CacheShard {
    #[allow(dead_code)]
    id: usize,
    entries: HashMap<u64, CacheEntry>,
    data_file: PathBuf,
    mmap: Option<Mmap>,
    current_offset: usize,
}

/// Global cache metadata
#[derive(Debug, Serialize, Deserialize)]
struct CacheMetadata {
    version: u32,
    total_entries: usize,
    total_size: usize,
    created_at: u64,
    last_updated: u64,
}

impl EmbeddingCache {
    /// Create a new embedding cache
    pub fn new(config: CacheConfig) -> Result<Self> {
        std::fs::create_dir_all(&config.cache_dir)?;

        let metadata_path = config.cache_dir.join("metadata.json");
        let metadata = if metadata_path.exists() {
            let data = std::fs::read_to_string(&metadata_path)?;
            serde_json::from_str(&data)?
        } else {
            CacheMetadata {
                version: 1,
                total_entries: 0,
                total_size: 0,
                created_at: chrono::Utc::now().timestamp() as u64,
                last_updated: chrono::Utc::now().timestamp() as u64,
            }
        };

        let mut shards = Vec::with_capacity(config.num_shards);
        for i in 0..config.num_shards {
            shards.push(Arc::new(RwLock::new(CacheShard::new(i, &config)?)));
        }

        Ok(Self {
            config,
            shards,
            metadata: Arc::new(RwLock::new(metadata)),
        })
    }

    /// Get embedding from cache
    pub fn get(&self, content: &str) -> Option<Vec<f32>> {
        let hash = xxh3_64(content.as_bytes());
        let shard_id = (hash as usize) % self.config.num_shards;

        let shard = self.shards[shard_id].read();
        if let Some(entry) = shard.entries.get(&hash) {
            return shard.read_embedding(entry);
        }

        None
    }

    /// Put embedding into cache
    pub fn put(&self, content: &str, embedding: &[f32]) -> Result<()> {
        let hash = xxh3_64(content.as_bytes());
        let shard_id = (hash as usize) % self.config.num_shards;

        let mut shard = self.shards[shard_id].write();
        shard.write_embedding(hash, content, embedding)?;

        // Update metadata
        let mut meta = self.metadata.write();
        meta.total_entries += 1;
        meta.total_size += embedding.len() * std::mem::size_of::<f32>();
        meta.last_updated = chrono::Utc::now().timestamp() as u64;

        Ok(())
    }

    /// Batch get embeddings
    pub fn get_batch(&self, contents: &[&str]) -> Vec<Option<Vec<f32>>> {
        contents
            .par_iter()
            .map(|content| self.get(content))
            .collect()
    }

    /// Batch put embeddings
    pub fn put_batch(&self, contents: &[&str], embeddings: &[Vec<f32>]) -> Result<()> {
        contents
            .par_iter()
            .zip(embeddings.par_iter())
            .try_for_each(|(content, embedding)| self.put(content, embedding))
    }

    /// Check if content exists in cache
    pub fn contains(&self, content: &str) -> bool {
        let hash = xxh3_64(content.as_bytes());
        let shard_id = (hash as usize) % self.config.num_shards;

        self.shards[shard_id].read().entries.contains_key(&hash)
    }

    /// Build cache from directory of files
    pub fn build_from_directory<F>(&self, dir: &Path, embed_fn: F) -> Result<usize>
    where
        F: Fn(&str) -> Result<Vec<f32>> + Send + Sync,
    {
        let files = Self::collect_files(dir)?;
        let mut new_count = 0;

        // Process files in parallel
        let results: Vec<_> = files
            .par_iter()
            .filter_map(|path| {
                let content = std::fs::read_to_string(path).ok()?;
                let hash = xxh3_64(content.as_bytes());

                // Check if already cached
                let shard_id = (hash as usize) % self.config.num_shards;
                if self.shards[shard_id].read().entries.contains_key(&hash) {
                    return None;
                }

                // Compute embedding
                match embed_fn(&content) {
                    Ok(embedding) => Some((content, embedding)),
                    Err(e) => {
                        debug!("Failed to embed {}: {}", path.display(), e);
                        None
                    }
                }
            })
            .collect();

        // Store results
        for (content, embedding) in results {
            self.put(&content, &embedding)?;
            new_count += 1;
        }

        info!("Added {} new embeddings to cache", new_count);
        Ok(new_count)
    }

    /// Export cache to Arrow/Parquet format
    #[cfg(feature = "parquet")]
    pub fn export_to_parquet(&self, output_path: &Path) -> Result<()> {
        // use arrow::array::Array;

        // Collect all entries
        let mut contents = Vec::new();
        let mut embeddings = Vec::new();
        let mut hashes = Vec::new();

        for shard in &self.shards {
            let shard = shard.read();
            for (hash, entry) in &shard.entries {
                if let Some(embedding) = shard.read_embedding(entry) {
                    contents.push(entry.file_path.clone());
                    embeddings.extend(embedding);
                    hashes.push(*hash as i64);
                }
            }
        }

        // Create Arrow schema
        let schema = Schema::new(vec![
            Field::new("content_hash", DataType::Int64, false),
            Field::new("file_path", DataType::Utf8, false),
            Field::new("embedding", DataType::Float32, false),
        ]);

        // Create record batch
        let batch = RecordBatch::try_new(
            Arc::new(schema.clone()),
            vec![
                Arc::new(arrow::array::Int64Array::from(hashes)),
                Arc::new(StringArray::from(contents)),
                Arc::new(Float32Array::from(embeddings)),
            ],
        )?;

        // Write to Parquet
        let file = File::create(output_path)?;
        let props = WriterProperties::builder()
            .set_compression(Compression::SNAPPY)
            .build();

        let mut writer = ArrowWriter::try_new(file, Arc::new(schema), Some(props))?;
        writer.write(&batch)?;
        writer.close()?;

        info!("Exported cache to {}", output_path.display());
        Ok(())
    }

    /// Collect files recursively
    fn collect_files(dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        Self::collect_files_recursive(dir, &mut files)?;
        Ok(files)
    }

    fn collect_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                Self::collect_files_recursive(&path, files)?;
            } else if path.is_file() {
                files.push(path);
            }
        }
        Ok(())
    }

    /// Save metadata
    pub fn save_metadata(&self) -> Result<()> {
        let metadata_path = self.config.cache_dir.join("metadata.json");
        let data = serde_json::to_string_pretty(&*self.metadata.read())?;
        std::fs::write(&metadata_path, data)?;
        Ok(())
    }
}

impl CacheShard {
    fn new(id: usize, config: &CacheConfig) -> Result<Self> {
        let data_file = config
            .cache_dir
            .join(format!("{}_{}.bin", config.prefix, id));
        let index_file = config
            .cache_dir
            .join(format!("{}_{}.idx", config.prefix, id));

        // Load existing index
        let entries = if index_file.exists() {
            let data = std::fs::read(&index_file)?;
            bincode::deserialize(&data)?
        } else {
            HashMap::new()
        };

        // Memory map data file if it exists
        let (mmap, current_offset) = if data_file.exists() && config.use_mmap {
            let file = File::open(&data_file)?;
            let metadata = file.metadata()?;
            let mmap = unsafe { Mmap::map(&file)? };
            (Some(mmap), metadata.len() as usize)
        } else {
            (None, 0)
        };

        Ok(Self {
            id,
            entries,
            data_file,
            mmap,
            current_offset,
        })
    }

    fn read_embedding(&self, entry: &CacheEntry) -> Option<Vec<f32>> {
        let byte_len = entry.dimension * std::mem::size_of::<f32>();

        // Fast path: read from mmap if available and in-bounds
        if let Some(ref mmap) = self.mmap {
            let start = entry.offset;
            let end = start + byte_len;
            if end <= mmap.len() {
                let bytes = &mmap[start..end];
                let embedding: Vec<f32> = bytes
                    .chunks_exact(std::mem::size_of::<f32>())
                    .map(|chunk| {
                        let array: [u8; 4] = chunk.try_into().unwrap();
                        f32::from_le_bytes(array)
                    })
                    .collect();
                return Some(embedding);
            }
        }

        // Fallback: read directly from file when mmap is not available or out-of-bounds
        if let Ok(mut file) = File::open(&self.data_file) {
            if file.seek(SeekFrom::Start(entry.offset as u64)).is_ok() {
                let mut buf = vec![0u8; byte_len];
                if file.read_exact(&mut buf).is_ok() {
                    let embedding: Vec<f32> = buf
                        .chunks_exact(std::mem::size_of::<f32>())
                        .map(|chunk| {
                            let array: [u8; 4] = chunk.try_into().unwrap();
                            f32::from_le_bytes(array)
                        })
                        .collect();
                    return Some(embedding);
                }
            }
        }

        None
    }

    fn write_embedding(&mut self, hash: u64, content: &str, embedding: &[f32]) -> Result<()> {
        // Write to data file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.data_file)?;

        let offset = self.current_offset;
        let dimension = embedding.len();

        // Write embedding data
        for &value in embedding {
            file.write_all(&value.to_le_bytes())?;
        }

        // Update index
        let entry = CacheEntry {
            content_hash: hash,
            file_path: content.to_string(),
            offset,
            dimension,
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        self.entries.insert(hash, entry);
        self.current_offset += dimension * std::mem::size_of::<f32>();

        // Save index periodically
        if self.entries.len() % 1000 == 0 {
            self.save_index()?;
        }

        Ok(())
    }

    fn save_index(&self) -> Result<()> {
        let index_file = self.data_file.with_extension("idx");
        let data = bincode::serialize(&self.entries)?;
        std::fs::write(&index_file, data)?;
        Ok(())
    }
}

impl Drop for CacheShard {
    fn drop(&mut self) {
        // Save index on drop
        if let Err(e) = self.save_index() {
            debug!("Failed to save cache index: {}", e);
        }
    }
}

impl Drop for EmbeddingCache {
    fn drop(&mut self) {
        // Save metadata on drop
        if let Err(e) = self.save_metadata() {
            debug!("Failed to save cache metadata: {}", e);
        }
    }
}
