# Outer RwLock over internally-synchronized index = pure reader-blocking (phase38)

**Category**: architecture
**Tags**: performance, locking, simd, phase38, analysis:2026-07-11-improvement-analysis

## Description

Collection held Arc&lt;RwLock&lt;OptimizedHnswIndex&gt;&gt; and insert_batch took .write() for the whole batch, but OptimizedHnswIndex is already internally synchronized (every field behind its own lock, all methods &amp;self). The outer write lock added zero safety and blocked every concurrent search for the batch duration. Fix: writers take .read() + a dedicated writer Mutex for bookkeeping atomicity (is-new/order/count); searches contend only on the index's narrow internal locks. Lesson: before narrowing a hot write-lock's scope, check whether the guarded type is internally synchronized — the lock may be removable from the read path entirely. Also from phase38: crate::simd::cosine_similarity is a clamped dot product that ASSUMES unit-length inputs; feeding it raw vectors scores every pair 1.0. Compute dot/(|a||b|) for raw data. And: criterion 0.8 deprecates criterion::black_box → std::hint::black_box (clippy -D warnings rejects it at commit).

## When to Use

When touching Collection insert/search paths, any Arc&lt;RwLock&lt;T&gt;&gt; hot path, crate::simd cosine callers, or criterion benches.
