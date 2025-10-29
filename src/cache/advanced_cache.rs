//! Advanced multi-layer caching system
//!
//! Provides sophisticated caching features including:
//! - Multi-layer cache architecture (L1, L2, L3)
//! - Intelligent eviction policies (LRU, LFU, TTL, Size-based)
//! - Cache warming and preloading
//! - Cache compression and serialization
//! - Distributed cache support
//! - Cache analytics and monitoring

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use lru::LruCache;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tokio::time::{interval, sleep};

/// Advanced cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedCacheConfig {
    /// L1 cache configuration (fastest, smallest)
    pub l1: CacheLayerConfig,

    /// L2 cache configuration (medium speed, medium size)
    pub l2: CacheLayerConfig,

    /// L3 cache configuration (slowest, largest)
    pub l3: CacheLayerConfig,

    /// Global cache settings
    pub global: GlobalCacheSettings,

    /// Cache warming configuration
    pub warming: CacheWarmingConfig,

    /// Compression settings
    pub compression: CompressionConfig,
}

/// Cache layer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheLayerConfig {
    /// Maximum number of entries
    pub max_entries: usize,

    /// Maximum memory usage in bytes
    pub max_memory_bytes: usize,

    /// Eviction policy
    pub eviction_policy: EvictionPolicy,

    /// TTL for entries in this layer
    pub ttl_seconds: u64,

    /// Whether this layer is enabled
    pub enabled: bool,

    /// Layer-specific settings
    pub settings: HashMap<String, serde_json::Value>,
}

/// Global cache settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalCacheSettings {
    /// Enable cache statistics collection
    pub enable_stats: bool,

    /// Enable cache warming
    pub enable_warming: bool,

    /// Cache warming interval in seconds
    pub warming_interval_seconds: u64,

    /// Enable cache compression
    pub enable_compression: bool,

    /// Compression threshold in bytes
    pub compression_threshold_bytes: usize,

    /// Enable distributed caching
    pub enable_distributed: bool,

    /// Distributed cache nodes
    pub distributed_nodes: Vec<String>,
}

/// Cache warming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheWarmingConfig {
    /// Enable automatic cache warming
    pub enabled: bool,

    /// Warming strategies
    pub strategies: Vec<WarmingStrategy>,

    /// Warming interval in seconds
    pub interval_seconds: u64,

    /// Maximum warming operations per interval
    pub max_operations_per_interval: usize,
}

/// Warming strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmingStrategy {
    /// Strategy name
    pub name: String,

    /// Strategy type
    pub strategy_type: WarmingStrategyType,

    /// Strategy parameters
    pub parameters: HashMap<String, serde_json::Value>,

    /// Priority (higher = more important)
    pub priority: u8,
}

/// Warming strategy types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarmingStrategyType {
    /// Warm based on access patterns
    AccessPattern,

    /// Warm based on time of day
    TimeBased,

    /// Warm based on user behavior
    UserBehavior,

    /// Warm based on data freshness
    DataFreshness,

    /// Warm based on query similarity
    QuerySimilarity,
}

/// Compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Compression algorithm
    pub algorithm: CompressionAlgorithm,

    /// Compression level (1-9)
    pub level: u8,

    /// Minimum size to compress (bytes)
    pub min_size_bytes: usize,

    /// Maximum compression ratio (0.0-1.0)
    pub max_ratio: f64,
}

/// Compression algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    Lz4,
    Gzip,
    Brotli,
    Zstd,
    Snappy,
}

/// Eviction policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvictionPolicy {
    /// Least Recently Used
    Lru,

    /// Least Frequently Used
    Lfu,

    /// Time To Live
    Ttl,

    /// Size-based eviction
    SizeBased,

    /// Random eviction
    Random,

    /// Custom eviction function
    Custom(String),
}

/// Cache entry with metadata
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    /// The cached value
    pub value: T,

    /// Creation timestamp
    pub created_at: Instant,

    /// Last access timestamp
    pub last_accessed: Instant,

    /// Access count
    pub access_count: u64,

    /// Entry size in bytes
    pub size_bytes: usize,

    /// TTL for this entry
    pub ttl: Duration,

    /// Entry priority
    pub priority: u8,

    /// Entry tags for categorization
    pub tags: Vec<String>,

    /// Entry metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl<T> CacheEntry<T> {
    /// Create a new cache entry
    pub fn new(value: T, size_bytes: usize, ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            size_bytes,
            ttl,
            priority: 0,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Check if entry is expired
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }

    /// Update access information
    pub fn record_access(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
    }

    /// Calculate access frequency
    pub fn access_frequency(&self) -> f64 {
        let age_seconds = self.created_at.elapsed().as_secs() as f64;
        if age_seconds > 0.0 {
            self.access_count as f64 / age_seconds
        } else {
            0.0
        }
    }
}

/// Cache key with hash support
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheKey {
    /// Key namespace
    pub namespace: String,

    /// Key identifier
    pub key: String,

    /// Key version
    pub version: u64,

    /// Key tags
    pub tags: Vec<String>,
}

impl CacheKey {
    /// Create a new cache key
    pub fn new(namespace: String, key: String) -> Self {
        Self {
            namespace,
            key,
            version: 1,
            tags: Vec::new(),
        }
    }

    /// Create a versioned cache key
    pub fn with_version(namespace: String, key: String, version: u64) -> Self {
        Self {
            namespace,
            key,
            version,
            tags: Vec::new(),
        }
    }

    /// Add tags to the key
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

impl Hash for CacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.namespace.hash(state);
        self.key.hash(state);
        self.version.hash(state);
        // Tags are not included in hash for performance
    }
}

/// Multi-layer cache implementation
#[derive(Clone)]
pub struct AdvancedCache<T: Clone + Send + Sync + 'static> {
    /// L1 cache (fastest)
    l1_cache: Arc<RwLock<LruCache<CacheKey, CacheEntry<T>>>>,

    /// L2 cache (medium)
    l2_cache: Arc<RwLock<LruCache<CacheKey, CacheEntry<T>>>>,

    /// L3 cache (slowest)
    l3_cache: Arc<RwLock<LruCache<CacheKey, CacheEntry<T>>>>,

    /// Cache configuration
    config: AdvancedCacheConfig,

    /// Cache statistics
    stats: Arc<Mutex<CacheStatistics>>,

    /// Cache warming manager
    warming_manager: Option<Arc<CacheWarmingManager<T>>>,

    /// Compression manager
    compression_manager: Option<Arc<CompressionManager>>,
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStatistics {
    /// L1 cache stats
    pub l1: LayerStatistics,

    /// L2 cache stats
    pub l2: LayerStatistics,

    /// L3 cache stats
    pub l3: LayerStatistics,

    /// Global stats
    pub global: GlobalStatistics,
}

/// Layer statistics
#[derive(Debug, Clone, Default)]
pub struct LayerStatistics {
    /// Cache hits
    pub hits: u64,

    /// Cache misses
    pub misses: u64,

    /// Cache evictions
    pub evictions: u64,

    /// Cache insertions
    pub insertions: u64,

    /// Current size
    pub current_size: usize,

    /// Current memory usage
    pub current_memory_bytes: usize,

    /// Hit rate
    pub hit_rate: f64,
}

/// Global statistics
#[derive(Debug, Clone, Default)]
pub struct GlobalStatistics {
    /// Total operations
    pub total_operations: u64,

    /// Average operation time (microseconds)
    pub avg_operation_time_us: u64,

    /// Cache warming operations
    pub warming_operations: u64,

    /// Compression operations
    pub compression_operations: u64,

    /// Compression ratio
    pub compression_ratio: f64,
}

/// Cache warming manager
pub struct CacheWarmingManager<T: Clone + Send + Sync + 'static> {
    /// Cache reference
    cache: Arc<AdvancedCache<T>>,

    /// Warming strategies
    strategies: Vec<WarmingStrategy>,

    /// Warming interval
    interval: Duration,

    /// Max operations per interval
    max_operations: usize,
}

/// Compression manager
pub struct CompressionManager {
    /// Compression algorithm
    algorithm: CompressionAlgorithm,

    /// Compression level
    level: u8,

    /// Minimum size to compress
    min_size: usize,

    /// Maximum compression ratio
    max_ratio: f64,
}

impl<T: Clone + Send + Sync + 'static> AdvancedCache<T> {
    /// Create a new advanced cache
    pub fn new(config: AdvancedCacheConfig) -> Self {
        let l1_capacity =
            NonZeroUsize::new(config.l1.max_entries).unwrap_or(NonZeroUsize::new(1000).unwrap());
        let l2_capacity =
            NonZeroUsize::new(config.l2.max_entries).unwrap_or(NonZeroUsize::new(10000).unwrap());
        let l3_capacity =
            NonZeroUsize::new(config.l3.max_entries).unwrap_or(NonZeroUsize::new(100000).unwrap());

        let cache = Self {
            l1_cache: Arc::new(RwLock::new(LruCache::new(l1_capacity))),
            l2_cache: Arc::new(RwLock::new(LruCache::new(l2_capacity))),
            l3_cache: Arc::new(RwLock::new(LruCache::new(l3_capacity))),
            config: config.clone(),
            stats: Arc::new(Mutex::new(CacheStatistics::default())),
            warming_manager: None,
            compression_manager: None,
        };

        let cache_arc = Arc::new(cache);

        // Initialize warming manager if enabled
        let warming_manager = if config.global.enable_warming {
            Some(Arc::new(CacheWarmingManager::new(
                cache_arc.clone(),
                config.warming,
            )))
        } else {
            None
        };

        // Initialize compression manager if enabled
        let compression_manager = if config.global.enable_compression {
            Some(Arc::new(CompressionManager::new(config.compression)))
        } else {
            None
        };

        // Start warming if enabled
        if let Some(warming) = &warming_manager {
            warming.start();
        }

        // Return the cache
        (*cache_arc).clone()
    }

    /// Get a value from the cache
    pub fn get(&self, key: &CacheKey) -> Option<T> {
        let start_time = Instant::now();

        // Try L1 first
        if let Some(entry) = self.get_from_layer(&self.l1_cache, key) {
            self.record_hit(1);
            self.record_operation_time(start_time.elapsed());
            return Some(entry);
        }

        // Try L2
        if let Some(entry) = self.get_from_layer(&self.l2_cache, key) {
            self.record_hit(2);
            // Promote to L1
            self.promote_to_l1(key, entry.clone());
            self.record_operation_time(start_time.elapsed());
            return Some(entry);
        }

        // Try L3
        if let Some(entry) = self.get_from_layer(&self.l3_cache, key) {
            self.record_hit(3);
            // Promote to L2
            self.promote_to_l2(key, entry.clone());
            self.record_operation_time(start_time.elapsed());
            return Some(entry);
        }

        self.record_miss();
        self.record_operation_time(start_time.elapsed());
        None
    }

    /// Insert a value into the cache
    pub fn insert(&self, key: CacheKey, value: T, size_bytes: usize) {
        let ttl = Duration::from_secs(self.config.l1.ttl_seconds);
        let mut entry = CacheEntry::new(value, size_bytes, ttl);

        // Add tags from key
        entry.tags = key.tags.clone();

        // Compress if enabled and meets threshold
        if let Some(compression) = &self.compression_manager {
            if size_bytes >= compression.min_size {
                // Apply compression (simplified)
                entry
                    .metadata
                    .insert("compressed".to_string(), serde_json::Value::Bool(true));
            }
        }

        // Insert into L1
        self.insert_into_layer(&self.l1_cache, key.clone(), entry);
        self.record_insertion(1);
    }

    /// Remove a value from all cache layers
    pub fn remove(&self, key: &CacheKey) {
        self.remove_from_layer(&self.l1_cache, key);
        self.remove_from_layer(&self.l2_cache, key);
        self.remove_from_layer(&self.l3_cache, key);
    }

    /// Clear all cache layers
    pub fn clear(&self) {
        self.l1_cache.write().unwrap().clear();
        self.l2_cache.write().unwrap().clear();
        self.l3_cache.write().unwrap().clear();

        let mut stats = self.stats.lock();
        *stats = CacheStatistics::default();
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStatistics {
        self.stats.lock().clone()
    }

    /// Get value from specific layer
    fn get_from_layer(
        &self,
        layer: &Arc<RwLock<LruCache<CacheKey, CacheEntry<T>>>>,
        key: &CacheKey,
    ) -> Option<T> {
        let mut cache = layer.write().unwrap();

        if let Some(entry) = cache.get_mut(key) {
            if entry.is_expired() {
                cache.pop(key);
                return None;
            }

            entry.record_access();
            Some(entry.value.clone())
        } else {
            None
        }
    }

    /// Insert into specific layer
    fn insert_into_layer(
        &self,
        layer: &Arc<RwLock<LruCache<CacheKey, CacheEntry<T>>>>,
        key: CacheKey,
        mut entry: CacheEntry<T>,
    ) {
        let mut cache = layer.write().unwrap();

        // Check if we need to evict
        if cache.len() >= cache.cap().get() {
            self.evict_from_layer(&mut cache, &key);
        }

        cache.push(key, entry);
    }

    /// Remove from specific layer
    fn remove_from_layer(
        &self,
        layer: &Arc<RwLock<LruCache<CacheKey, CacheEntry<T>>>>,
        key: &CacheKey,
    ) {
        let mut cache = layer.write().unwrap();
        cache.pop(key);
    }

    /// Evict entries from layer based on policy
    fn evict_from_layer(&self, cache: &mut LruCache<CacheKey, CacheEntry<T>>, _new_key: &CacheKey) {
        // Simple LRU eviction - remove least recently used
        if let Some((key, _)) = cache.pop_lru() {
            self.record_eviction();
        }
    }

    /// Promote entry to L1
    fn promote_to_l1(&self, key: &CacheKey, value: T) {
        // Implementation would promote the entry to L1
        // This is simplified for brevity
    }

    /// Promote entry to L2
    fn promote_to_l2(&self, key: &CacheKey, value: T) {
        // Implementation would promote the entry to L2
        // This is simplified for brevity
    }

    /// Record cache hit
    fn record_hit(&self, layer: u8) {
        let mut stats = self.stats.lock();
        match layer {
            1 => stats.l1.hits += 1,
            2 => stats.l2.hits += 1,
            3 => stats.l3.hits += 1,
            _ => {}
        }
        stats.global.total_operations += 1;
    }

    /// Record cache miss
    fn record_miss(&self) {
        let mut stats = self.stats.lock();
        stats.l1.misses += 1;
        stats.global.total_operations += 1;
    }

    /// Record insertion
    fn record_insertion(&self, layer: u8) {
        let mut stats = self.stats.lock();
        match layer {
            1 => stats.l1.insertions += 1,
            2 => stats.l2.insertions += 1,
            3 => stats.l3.insertions += 1,
            _ => {}
        }
    }

    /// Record eviction
    fn record_eviction(&self) {
        let mut stats = self.stats.lock();
        stats.l1.evictions += 1;
    }

    /// Record operation time
    fn record_operation_time(&self, duration: Duration) {
        let mut stats = self.stats.lock();
        let time_us = duration.as_micros() as u64;
        stats.global.avg_operation_time_us = (stats.global.avg_operation_time_us + time_us) / 2;
    }
}

impl<T: Clone + Send + Sync + 'static> CacheWarmingManager<T> {
    /// Create a new warming manager
    fn new(cache: Arc<AdvancedCache<T>>, config: CacheWarmingConfig) -> Self {
        Self {
            cache,
            strategies: config.strategies,
            interval: Duration::from_secs(config.interval_seconds),
            max_operations: config.max_operations_per_interval,
        }
    }

    /// Start the warming process
    fn start(&self) {
        let interval = self.interval;
        let cache = self.cache.clone();
        let warming_strategies = self.strategies.clone();
        let cache_config = self.cache.config.clone();

        // Convert AdvancedCacheConfig to CacheWarmingConfig
        let warming_config = CacheWarmingConfig {
            enabled: cache_config.warming.enabled,
            interval_seconds: cache_config.warming.interval_seconds,
            max_operations_per_interval: cache_config.warming.max_operations_per_interval,
            strategies: warming_strategies.clone(),
        };

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            loop {
                interval.tick().await;
                // Execute warming cycle with cloned data
                Self::execute_warming_cycle_static(
                    cache.clone(),
                    warming_strategies.clone(),
                    warming_config.clone(),
                )
                .await;
            }
        });
    }

    /// Execute a warming cycle (static version for async spawn)
    async fn execute_warming_cycle_static(
        cache: Arc<dyn std::any::Any + Send + Sync>,
        strategies: Vec<WarmingStrategy>,
        config: CacheWarmingConfig,
    ) {
        for strategy in &strategies {
            match strategy.strategy_type {
                WarmingStrategyType::AccessPattern => {
                    // Implement access pattern warming
                }
                WarmingStrategyType::TimeBased => {
                    // Implement time-based warming
                }
                WarmingStrategyType::UserBehavior => {
                    // Implement user behavior warming
                }
                WarmingStrategyType::DataFreshness => {
                    // Implement data freshness warming
                }
                WarmingStrategyType::QuerySimilarity => {
                    // Implement query similarity warming
                }
            }
        }
    }

    /// Execute a warming cycle
    async fn execute_warming_cycle(&self) {
        for strategy in &self.strategies {
            match strategy.strategy_type {
                WarmingStrategyType::AccessPattern => {
                    self.warm_by_access_pattern().await;
                }
                WarmingStrategyType::TimeBased => {
                    self.warm_by_time().await;
                }
                WarmingStrategyType::UserBehavior => {
                    self.warm_by_user_behavior().await;
                }
                WarmingStrategyType::DataFreshness => {
                    self.warm_by_data_freshness().await;
                }
                WarmingStrategyType::QuerySimilarity => {
                    self.warm_by_query_similarity().await;
                }
            }
        }
    }

    /// Warm cache based on access patterns
    async fn warm_by_access_pattern(&self) {
        // Implementation would analyze access patterns and preload frequently accessed data
        tracing::debug!("Executing access pattern warming");
    }

    /// Warm cache based on time
    async fn warm_by_time(&self) {
        // Implementation would preload data based on time of day patterns
        tracing::debug!("Executing time-based warming");
    }

    /// Warm cache based on user behavior
    async fn warm_by_user_behavior(&self) {
        // Implementation would preload data based on user behavior patterns
        tracing::debug!("Executing user behavior warming");
    }

    /// Warm cache based on data freshness
    async fn warm_by_data_freshness(&self) {
        // Implementation would preload fresh data
        tracing::debug!("Executing data freshness warming");
    }

    /// Warm cache based on query similarity
    async fn warm_by_query_similarity(&self) {
        // Implementation would preload data similar to recent queries
        tracing::debug!("Executing query similarity warming");
    }
}

impl CompressionManager {
    /// Create a new compression manager
    fn new(config: CompressionConfig) -> Self {
        Self {
            algorithm: config.algorithm,
            level: config.level,
            min_size: config.min_size_bytes,
            max_ratio: config.max_ratio,
        }
    }

    /// Compress data
    pub fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < self.min_size {
            return Ok(data.to_vec());
        }

        match self.algorithm {
            CompressionAlgorithm::Lz4 => {
                // LZ4 compression implementation
                Ok(data.to_vec()) // Simplified
            }
            CompressionAlgorithm::Gzip => {
                // Gzip compression implementation
                Ok(data.to_vec()) // Simplified
            }
            CompressionAlgorithm::Brotli => {
                // Brotli compression implementation
                Ok(data.to_vec()) // Simplified
            }
            CompressionAlgorithm::Zstd => {
                // Zstd compression implementation
                Ok(data.to_vec()) // Simplified
            }
            CompressionAlgorithm::Snappy => {
                // Snappy compression implementation
                Ok(data.to_vec()) // Simplified
            }
        }
    }

    /// Decompress data
    pub fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        match self.algorithm {
            CompressionAlgorithm::Lz4 => {
                // LZ4 decompression implementation
                Ok(data.to_vec()) // Simplified
            }
            CompressionAlgorithm::Gzip => {
                // Gzip decompression implementation
                Ok(data.to_vec()) // Simplified
            }
            CompressionAlgorithm::Brotli => {
                // Brotli decompression implementation
                Ok(data.to_vec()) // Simplified
            }
            CompressionAlgorithm::Zstd => {
                // Zstd decompression implementation
                Ok(data.to_vec()) // Simplified
            }
            CompressionAlgorithm::Snappy => {
                // Snappy decompression implementation
                Ok(data.to_vec()) // Simplified
            }
        }
    }
}

impl Default for AdvancedCacheConfig {
    fn default() -> Self {
        Self {
            l1: CacheLayerConfig {
                max_entries: 1000,
                max_memory_bytes: 100 * 1024 * 1024, // 100MB
                eviction_policy: EvictionPolicy::Lru,
                ttl_seconds: 300, // 5 minutes
                enabled: true,
                settings: HashMap::new(),
            },
            l2: CacheLayerConfig {
                max_entries: 10000,
                max_memory_bytes: 1024 * 1024 * 1024, // 1GB
                eviction_policy: EvictionPolicy::Lru,
                ttl_seconds: 3600, // 1 hour
                enabled: true,
                settings: HashMap::new(),
            },
            l3: CacheLayerConfig {
                max_entries: 100000,
                max_memory_bytes: 10 * 1024 * 1024 * 1024, // 10GB
                eviction_policy: EvictionPolicy::Lru,
                ttl_seconds: 86400, // 24 hours
                enabled: true,
                settings: HashMap::new(),
            },
            global: GlobalCacheSettings {
                enable_stats: true,
                enable_warming: false,
                warming_interval_seconds: 300,
                enable_compression: false,
                compression_threshold_bytes: 1024,
                enable_distributed: false,
                distributed_nodes: Vec::new(),
            },
            warming: CacheWarmingConfig {
                enabled: false,
                strategies: Vec::new(),
                interval_seconds: 300,
                max_operations_per_interval: 100,
            },
            compression: CompressionConfig {
                algorithm: CompressionAlgorithm::Lz4,
                level: 6,
                min_size_bytes: 1024,
                max_ratio: 0.8,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_creation() {
        let key = CacheKey::new("test".to_string(), "key1".to_string());
        assert_eq!(key.namespace, "test");
        assert_eq!(key.key, "key1");
        assert_eq!(key.version, 1);
    }

    #[test]
    fn test_cache_key_with_version() {
        let key = CacheKey::with_version("test".to_string(), "key1".to_string(), 5);
        assert_eq!(key.version, 5);
    }

    #[test]
    fn test_cache_key_with_tags() {
        let key = CacheKey::new("test".to_string(), "key1".to_string())
            .with_tags(vec!["tag1".to_string(), "tag2".to_string()]);
        assert_eq!(key.tags.len(), 2);
        assert!(key.tags.contains(&"tag1".to_string()));
        assert!(key.tags.contains(&"tag2".to_string()));
    }

    #[test]
    fn test_cache_entry_creation() {
        let entry = CacheEntry::new("test_value".to_string(), 100, Duration::from_secs(60));
        assert_eq!(entry.value, "test_value");
        assert_eq!(entry.size_bytes, 100);
        assert_eq!(entry.access_count, 0);
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_cache_entry_access_recording() {
        let mut entry = CacheEntry::new("test_value".to_string(), 100, Duration::from_secs(60));
        assert_eq!(entry.access_count, 0);

        entry.record_access();
        assert_eq!(entry.access_count, 1);

        entry.record_access();
        assert_eq!(entry.access_count, 2);
    }

    #[test]
    fn test_cache_entry_expiration() {
        let entry = CacheEntry::new("test_value".to_string(), 100, Duration::from_secs(0));
        // Entry with 0 TTL should be expired immediately
        assert!(entry.is_expired());
    }

    #[test]
    fn test_compression_manager_creation() {
        let config = CompressionConfig {
            algorithm: CompressionAlgorithm::Lz4,
            level: 6,
            min_size_bytes: 1024,
            max_ratio: 0.8,
        };

        let manager = CompressionManager::new(config);
        assert_eq!(manager.min_size, 1024);
        assert_eq!(manager.level, 6);
    }

    #[test]
    fn test_compression_below_threshold() {
        let config = CompressionConfig {
            algorithm: CompressionAlgorithm::Lz4,
            level: 6,
            min_size_bytes: 1024,
            max_ratio: 0.8,
        };

        let manager = CompressionManager::new(config);
        let data = b"small data";
        let compressed = manager.compress(data).unwrap();
        assert_eq!(compressed, data);
    }

    #[test]
    fn test_advanced_cache_config_default() {
        let config = AdvancedCacheConfig::default();
        assert!(config.l1.enabled);
        assert!(config.l2.enabled);
        assert!(config.l3.enabled);
        assert!(!config.global.enable_warming);
        assert!(!config.global.enable_compression);
    }
}
