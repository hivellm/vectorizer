//! Parallel processing optimizations for embeddings and indexing
//!
//! This module provides controlled parallelism with:
//! - Thread pool management to prevent oversubscription
//! - BLAS thread control for numerical operations
//! - Batch processing pipelines
//! - Work stealing for load balancing

use anyhow::Result;
use crossbeam::channel::{Receiver, Sender, bounded};
use parking_lot::Mutex;
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::env;
use std::sync::Arc;
use std::thread;
use tracing::{debug, info};

/// Global thread pool for embedding operations
static EMBEDDING_POOL: once_cell::sync::OnceCell<ThreadPool> = once_cell::sync::OnceCell::new();

/// Global thread pool for indexing operations
static INDEXING_POOL: once_cell::sync::OnceCell<ThreadPool> = once_cell::sync::OnceCell::new();

/// Parallel processing configuration
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Number of embedding threads
    pub embedding_threads: usize,
    /// Number of indexing threads
    pub indexing_threads: usize,
    /// BLAS threads per worker
    pub blas_threads: usize,
    /// Channel buffer size
    pub channel_size: usize,
    /// Batch size for processing
    pub batch_size: usize,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        let num_cores = num_cpus::get();

        Self {
            // Use half cores for embedding to leave room for other operations
            embedding_threads: (num_cores / 2).max(1),
            // Use quarter cores for indexing
            indexing_threads: (num_cores / 4).max(1),
            // Single thread for BLAS to prevent oversubscription
            blas_threads: 1,
            // Large channel for buffering
            channel_size: 10_000,
            // Optimal batch size
            batch_size: 128,
        }
    }
}

/// Initialize parallel processing environment
pub fn init_parallel_env(config: &ParallelConfig) -> Result<()> {
    // Set BLAS thread count
    unsafe {
        env::set_var("OMP_NUM_THREADS", config.blas_threads.to_string());
        env::set_var("MKL_NUM_THREADS", config.blas_threads.to_string());
        env::set_var("OPENBLAS_NUM_THREADS", config.blas_threads.to_string());
        env::set_var("BLIS_NUM_THREADS", config.blas_threads.to_string());
    }

    // Initialize embedding thread pool
    EMBEDDING_POOL.get_or_init(|| {
        ThreadPoolBuilder::new()
            .num_threads(config.embedding_threads)
            .thread_name(|i| format!("embed-{}", i))
            .build()
            .expect("Failed to create embedding thread pool")
    });

    // Initialize indexing thread pool
    INDEXING_POOL.get_or_init(|| {
        ThreadPoolBuilder::new()
            .num_threads(config.indexing_threads)
            .thread_name(|i| format!("index-{}", i))
            .build()
            .expect("Failed to create indexing thread pool")
    });

    info!(
        "Initialized parallel environment: {} embedding threads, {} indexing threads, {} BLAS threads",
        config.embedding_threads, config.indexing_threads, config.blas_threads
    );

    Ok(())
}

/// Get embedding thread pool
pub fn embedding_pool() -> &'static ThreadPool {
    EMBEDDING_POOL
        .get()
        .expect("Embedding pool not initialized. Call init_parallel_env first.")
}

/// Get indexing thread pool
pub fn indexing_pool() -> &'static ThreadPool {
    INDEXING_POOL
        .get()
        .expect("Indexing pool not initialized. Call init_parallel_env first.")
}

/// Parallel document processing pipeline
pub struct ProcessingPipeline<T, E, I> {
    /// Input receiver
    input_rx: Receiver<T>,
    /// Embedding sender
    embed_tx: Sender<E>,
    /// Index sender
    index_tx: Sender<I>,
    /// Configuration
    config: ParallelConfig,
}

impl<T, E, I> ProcessingPipeline<T, E, I>
where
    T: Send + Clone + 'static,
    E: Send + Clone + 'static,
    I: Send + 'static,
{
    /// Create a new processing pipeline
    pub fn new(config: ParallelConfig) -> (Self, Sender<T>, Receiver<E>, Receiver<I>) {
        let (input_tx, input_rx) = bounded(config.channel_size);
        let (embed_tx, embed_rx) = bounded(config.channel_size);
        let (index_tx, index_rx) = bounded(config.channel_size);

        let pipeline = Self {
            input_rx,
            embed_tx,
            index_tx,
            config,
        };

        (pipeline, input_tx, embed_rx, index_rx)
    }

    /// Start the pipeline with custom processing functions
    pub fn start<FE, FI>(self, embed_fn: FE, index_fn: FI) -> Result<()>
    where
        FE: Fn(Vec<T>) -> Result<Vec<E>> + Send + Sync + 'static,
        FI: Fn(Vec<E>) -> Result<Vec<I>> + Send + Sync + 'static,
    {
        let embed_fn = Arc::new(embed_fn);
        let index_fn = Arc::new(index_fn);

        // For forwarding embeddings to indexing workers
        let (embed_forward_tx, embed_forward_rx) = bounded(self.config.channel_size);

        // Embedding workers
        let embed_workers: Vec<_> = (0..self.config.embedding_threads)
            .map(|i| {
                let input_rx = self.input_rx.clone();
                let embed_tx = self.embed_tx.clone();
                let embed_forward_tx_clone = embed_forward_tx.clone();
                let embed_fn = embed_fn.clone();
                let batch_size = self.config.batch_size;

                thread::spawn(move || {
                    debug!("Embedding worker {} started", i);

                    let mut batch = Vec::with_capacity(batch_size);

                    loop {
                        // Collect batch
                        batch.clear();

                        // Try to fill batch
                        for _ in 0..batch_size {
                            match input_rx.try_recv() {
                                Ok(item) => batch.push(item),
                                Err(_) => break,
                            }
                        }

                        // If no items and channel is disconnected, exit
                        if batch.is_empty() {
                            match input_rx.recv_timeout(std::time::Duration::from_millis(100)) {
                                Ok(item) => batch.push(item),
                                Err(_) => break,
                            }
                        }

                        // Process batch
                        if !batch.is_empty() {
                            match embed_fn(batch.clone()) {
                                Ok(embeddings) => {
                                    for embedding in embeddings {
                                        if embed_tx.send(embedding.clone()).is_err() {
                                            return;
                                        }
                                        if embed_forward_tx_clone.send(embedding).is_err() {
                                            return;
                                        }
                                    }
                                }
                                Err(e) => {
                                    debug!("Embedding error: {}", e);
                                }
                            }
                        }
                    }

                    debug!("Embedding worker {} stopped", i);
                })
            })
            .collect();

        // Indexing workers
        let index_workers: Vec<_> = (0..self.config.indexing_threads)
            .map(|i| {
                let embed_rx = embed_forward_rx.clone();
                let index_tx = self.index_tx.clone();
                let index_fn = index_fn.clone();
                let batch_size = self.config.batch_size;

                thread::spawn(move || {
                    debug!("Indexing worker {} started", i);

                    let mut batch = Vec::with_capacity(batch_size);

                    loop {
                        // Collect batch
                        batch.clear();

                        // Try to fill batch
                        for _ in 0..batch_size {
                            match embed_rx.try_recv() {
                                Ok(item) => batch.push(item),
                                Err(_) => break,
                            }
                        }

                        // If no items and channel is disconnected, exit
                        if batch.is_empty() {
                            match embed_rx.recv_timeout(std::time::Duration::from_millis(100)) {
                                Ok(item) => batch.push(item),
                                Err(_) => break,
                            }
                        }

                        // Process batch
                        if !batch.is_empty() {
                            let batch_to_process =
                                std::mem::replace(&mut batch, Vec::with_capacity(batch_size));
                            match index_fn(batch_to_process) {
                                Ok(indices) => {
                                    for index in indices {
                                        if index_tx.send(index).is_err() {
                                            return;
                                        }
                                    }
                                }
                                Err(e) => {
                                    debug!("Indexing error: {}", e);
                                }
                            }
                        }
                    }

                    debug!("Indexing worker {} stopped", i);
                })
            })
            .collect();

        // Wait for workers to complete
        for worker in embed_workers {
            worker.join().ok();
        }

        for worker in index_workers {
            worker.join().ok();
        }

        Ok(())
    }
}

/// Batch processor for parallel operations
pub struct BatchProcessor<T> {
    items: Arc<Mutex<Vec<T>>>,
    batch_size: usize,
}

impl<T: Send + Sync> BatchProcessor<T> {
    pub fn new(batch_size: usize) -> Self {
        Self {
            items: Arc::new(Mutex::new(Vec::new())),
            batch_size,
        }
    }

    /// Add items to processor
    pub fn add(&self, items: Vec<T>) {
        let mut buffer = self.items.lock();
        buffer.extend(items);
    }

    /// Process all items in batches
    pub fn process_all<F, R>(&self, process_fn: F) -> Result<Vec<R>>
    where
        F: Fn(&[T]) -> Result<Vec<R>> + Send + Sync,
        R: Send,
    {
        let mut buffer = self.items.lock();
        let items: Vec<T> = buffer.drain(..).collect();
        drop(buffer);

        let pool = embedding_pool();

        let results: Result<Vec<Vec<R>>> = pool.install(|| {
            items
                .chunks(self.batch_size)
                .map(|batch| process_fn(batch))
                .collect()
        });

        results.map(|vecs| vecs.into_iter().flatten().collect())
    }
}

/// Performance monitoring
pub mod monitor {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Instant;

    pub struct ThroughputMonitor {
        start_time: Instant,
        processed_items: AtomicU64,
        processed_bytes: AtomicU64,
    }

    impl ThroughputMonitor {
        pub fn new() -> Self {
            Self {
                start_time: Instant::now(),
                processed_items: AtomicU64::new(0),
                processed_bytes: AtomicU64::new(0),
            }
        }

        pub fn record_items(&self, count: u64) {
            self.processed_items.fetch_add(count, Ordering::Relaxed);
        }

        pub fn record_bytes(&self, bytes: u64) {
            self.processed_bytes.fetch_add(bytes, Ordering::Relaxed);
        }

        pub fn throughput(&self) -> (f64, f64) {
            let elapsed = self.start_time.elapsed().as_secs_f64();
            let items = self.processed_items.load(Ordering::Relaxed);
            let bytes = self.processed_bytes.load(Ordering::Relaxed);

            let items_per_sec = items as f64 / elapsed;
            let mb_per_sec = (bytes as f64 / 1_048_576.0) / elapsed;

            (items_per_sec, mb_per_sec)
        }

        pub fn report(&self) -> String {
            let (items_per_sec, mb_per_sec) = self.throughput();
            format!(
                "Throughput: {:.2} items/s, {:.2} MB/s",
                items_per_sec, mb_per_sec
            )
        }
    }
}
