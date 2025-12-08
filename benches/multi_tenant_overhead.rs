//! Multi-Tenant Overhead Benchmarks
//!
//! Measures the performance overhead of multi-tenant features:
#![allow(
    clippy::uninlined_format_args,
    clippy::get_first,
    clippy::inefficient_to_string
)]
//! - Service header validation
//! - Tenant context creation
//! - Collection name parsing
//! - Owner validation
//! - Quota checks (simulated)
//!
//! Run with: `cargo bench --bench multi_tenant_overhead`

use std::hint::black_box;
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use uuid::Uuid;

/// Benchmark service header validation
fn bench_service_header_check(c: &mut Criterion) {
    c.bench_function("service_header_check", |b| {
        let service_key = "test-service-key";
        let expected = "test-service-key";

        b.iter(|| {
            // Simulate service header validation
            black_box(service_key == expected)
        });
    });
}

/// Benchmark tenant ID extraction from header
fn bench_tenant_id_extraction(c: &mut Criterion) {
    c.bench_function("tenant_id_extraction", |b| {
        let header_value = "user_12345678-1234-5678-1234-567812345678";

        b.iter(|| {
            // Simulate extracting tenant ID from header
            black_box(header_value.to_string())
        });
    });
}

/// Benchmark collection name parsing
fn bench_collection_name_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("collection_name_parsing");

    // Test different name lengths
    for name_len in [10, 50, 100].iter() {
        let tenant_id = "user_12345678-1234-5678-1234-567812345678";
        let collection_name = "a".repeat(*name_len);
        let full_name = format!("{tenant_id}:{collection_name}");

        group.bench_with_input(
            BenchmarkId::from_parameter(name_len),
            &full_name,
            |b, full_name| {
                b.iter(|| {
                    // Parse tenant and collection
                    let parts: Vec<&str> = full_name.splitn(2, ':').collect();
                    let tenant = parts.first().map(|s| (*s).to_string());
                    let collection = parts.get(1).map(|s| (*s).to_string());
                    black_box((tenant, collection))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark UUID parsing (for owner validation)
fn bench_uuid_parsing(c: &mut Criterion) {
    c.bench_function("uuid_parsing", |b| {
        let uuid_str = "12345678-1234-5678-1234-567812345678";

        b.iter(|| black_box(Uuid::parse_str(uuid_str).ok()));
    });
}

/// Benchmark owner ID comparison
fn bench_owner_validation(c: &mut Criterion) {
    c.bench_function("owner_validation", |b| {
        let owner_id = Uuid::parse_str("12345678-1234-5678-1234-567812345678").unwrap();
        let collection_owner = Uuid::parse_str("12345678-1234-5678-1234-567812345678").unwrap();

        b.iter(|| {
            // Simple equality check
            black_box(owner_id == collection_owner)
        });
    });
}

/// Benchmark collection filtering by owner
fn bench_collection_filtering(c: &mut Criterion) {
    let mut group = c.benchmark_group("collection_filtering");

    // Test with different collection counts
    for count in [10, 100, 1000, 10000].iter() {
        let owner_id = Uuid::parse_str("12345678-1234-5678-1234-567812345678").unwrap();

        // Generate test collections
        let collections: Vec<(String, Option<Uuid>)> = (0..*count)
            .map(|i| {
                let name = format!("collection_{}", i);
                let owner = if i % 2 == 0 {
                    Some(owner_id)
                } else {
                    Some(Uuid::new_v4())
                };
                (name, owner)
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            &collections,
            |b, collections| {
                b.iter(|| {
                    // Filter collections by owner
                    let filtered: Vec<&String> = collections
                        .iter()
                        .filter(|(_, owner)| owner.as_ref() == Some(&owner_id))
                        .map(|(name, _)| name)
                        .collect();
                    black_box(filtered)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark complete tenant context creation
fn bench_tenant_context_creation(c: &mut Criterion) {
    c.bench_function("tenant_context_creation", |b| {
        let tenant_id = "user_12345678-1234-5678-1234-567812345678".to_string();
        let tenant_name = "Test Tenant".to_string();
        let api_key_id = "key_abcdefgh".to_string();

        b.iter(|| {
            // Simulate creating tenant context struct
            let context: (String, String, String, Vec<String>, Option<u32>) = (
                tenant_id.clone(),
                tenant_name.clone(),
                api_key_id.clone(),
                vec![], // permissions
                None,   // rate_limits
            );
            black_box(context)
        });
    });
}

/// Benchmark cache lookup simulation
fn bench_cache_lookup(c: &mut Criterion) {
    use std::collections::HashMap;

    let mut group = c.benchmark_group("cache_lookup");

    // Test with different cache sizes
    for size in [100, 1000, 10000].iter() {
        let mut cache = HashMap::new();

        // Populate cache
        for i in 0..*size {
            let key = format!("key_{}", i);
            cache.insert(key, i);
        }

        let lookup_key = "key_500".to_string();

        group.bench_with_input(BenchmarkId::from_parameter(size), &cache, |b, cache| {
            b.iter(|| black_box(cache.get(&lookup_key)));
        });
    }

    group.finish();
}

/// Benchmark hash computation for cache keys
fn bench_key_hashing(c: &mut Criterion) {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    c.bench_function("key_hashing", |b| {
        let api_key = "hh_live_1234567890abcdefghijklmnopqrstuvwxyz1234567890";

        b.iter(|| {
            let mut hasher = DefaultHasher::new();
            api_key.hash(&mut hasher);
            black_box(hasher.finish())
        });
    });
}

/// Benchmark string allocation for collection names
fn bench_string_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_allocation");

    for len in [10, 50, 100, 500].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(len), len, |b, len| {
            let tenant_id = "user_12345678";
            let collection = "a".repeat(*len);

            b.iter(|| {
                // Format string (common operation)
                black_box(format!("{}:{}", tenant_id, collection))
            });
        });
    }

    group.finish();
}

/// Benchmark complete request overhead (simulated)
fn bench_complete_request_overhead(c: &mut Criterion) {
    c.bench_function("complete_request_overhead", |b| {
        let service_key = "test-service-key";
        let tenant_id = "user_12345678-1234-5678-1234-567812345678";
        let full_collection_name = "user_12345678-1234-5678-1234-567812345678:documents";
        let owner_id = Uuid::parse_str("12345678-1234-5678-1234-567812345678").unwrap();

        b.iter(|| {
            // 1. Service header check (~0.05ms)
            let valid_service = service_key == "test-service-key";

            // 2. Tenant ID extraction (~0.01ms)
            let tenant = tenant_id.to_string();

            // 3. Collection name parsing (~0.005ms)
            let parts: Vec<&str> = full_collection_name.splitn(2, ':').collect();
            let parsed = (
                parts.get(0).map(|s| s.to_string()),
                parts.get(1).map(|s| s.to_string()),
            );

            // 4. Owner validation (~0.05ms)
            let collection_owner = owner_id;
            let owner_match = owner_id == collection_owner;

            // 5. Tenant context creation (~0.02ms)
            let context = (
                tenant_id.to_string(),
                "Test Tenant".to_string(),
                "key_12345678".to_string(),
            );

            black_box((valid_service, tenant, parsed, owner_match, context))
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(1000);
    targets =
        bench_service_header_check,
        bench_tenant_id_extraction,
        bench_collection_name_parsing,
        bench_uuid_parsing,
        bench_owner_validation,
        bench_collection_filtering,
        bench_tenant_context_creation,
        bench_cache_lookup,
        bench_key_hashing,
        bench_string_allocation,
        bench_complete_request_overhead,
}

criterion_main!(benches);
