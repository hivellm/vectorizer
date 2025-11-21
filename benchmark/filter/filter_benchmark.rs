//! Benchmark for payload filter performance
//!
//! This benchmark measures the performance of different filter types:
//! - Keyword filters (exact match)
//! - Range filters (integer and float)
//! - Text filters (full-text search)
//! - Geo filters (bounding box and radius)
//! - Nested field filters

use std::collections::HashSet;
use std::sync::Arc;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use serde_json::json;
use vectorizer::db::payload_index::{PayloadIndex, PayloadIndexConfig, PayloadIndexType};
use vectorizer::models::Payload;

fn setup_index_with_vectors(count: usize) -> PayloadIndex {
    let index = PayloadIndex::new();

    // Configure indexes
    index.add_index_config(PayloadIndexConfig::new(
        "status".to_string(),
        PayloadIndexType::Keyword,
    ));
    index.add_index_config(PayloadIndexConfig::new(
        "age".to_string(),
        PayloadIndexType::Integer,
    ));
    index.add_index_config(PayloadIndexConfig::new(
        "price".to_string(),
        PayloadIndexType::Float,
    ));
    index.add_index_config(PayloadIndexConfig::new(
        "description".to_string(),
        PayloadIndexType::Text,
    ));
    index.add_index_config(PayloadIndexConfig::new(
        "location".to_string(),
        PayloadIndexType::Geo,
    ));
    index.add_index_config(PayloadIndexConfig::new(
        "user.age".to_string(),
        PayloadIndexType::Integer,
    ));

    // Insert vectors
    for i in 0..count {
        let payload = Payload {
            data: json!({
                "status": if i % 2 == 0 { "active" } else { "inactive" },
                "age": 20 + (i % 50),
                "price": 10.0 + (i as f64 * 0.5),
                "description": format!("product {} description text", i),
                "location": {
                    "lat": -23.5 + (i as f64 * 0.01),
                    "lon": -46.6 + (i as f64 * 0.01)
                },
                "user": {
                    "age": 25 + (i % 30)
                }
            }),
        };
        index.index_vector(format!("v{}", i), &payload);
    }

    index
}

fn benchmark_keyword_filter(c: &mut Criterion) {
    let index = setup_index_with_vectors(10_000);

    c.bench_function("keyword_filter_10k", |b| {
        b.iter(|| {
            black_box(index.get_ids_for_keyword("status", "active").unwrap());
        });
    });
}

fn benchmark_integer_range_filter(c: &mut Criterion) {
    let index = setup_index_with_vectors(10_000);

    c.bench_function("integer_range_filter_10k", |b| {
        b.iter(|| {
            black_box(index.get_ids_in_range("age", Some(25), Some(35)).unwrap());
        });
    });
}

fn benchmark_float_range_filter(c: &mut Criterion) {
    let index = setup_index_with_vectors(10_000);

    c.bench_function("float_range_filter_10k", |b| {
        b.iter(|| {
            black_box(
                index
                    .get_ids_in_float_range("price", Some(20.0), Some(30.0))
                    .unwrap(),
            );
        });
    });
}

fn benchmark_text_filter(c: &mut Criterion) {
    let index = setup_index_with_vectors(10_000);

    c.bench_function("text_filter_10k", |b| {
        b.iter(|| {
            black_box(index.search_text("description", "product 100").unwrap());
        });
    });
}

fn benchmark_geo_bounding_box_filter(c: &mut Criterion) {
    let index = setup_index_with_vectors(10_000);

    c.bench_function("geo_bounding_box_filter_10k", |b| {
        b.iter(|| {
            black_box(
                index
                    .get_ids_in_geo_bounding_box("location", -24.0, -23.0, -47.0, -46.0)
                    .unwrap(),
            );
        });
    });
}

fn benchmark_geo_radius_filter(c: &mut Criterion) {
    let index = setup_index_with_vectors(10_000);

    c.bench_function("geo_radius_filter_10k", |b| {
        b.iter(|| {
            black_box(
                index
                    .get_ids_in_geo_radius("location", -23.5, -46.6, 10.0)
                    .unwrap(),
            );
        });
    });
}

fn benchmark_nested_field_filter(c: &mut Criterion) {
    let index = setup_index_with_vectors(10_000);

    c.bench_function("nested_field_filter_10k", |b| {
        b.iter(|| {
            black_box(
                index
                    .get_ids_in_range("user.age", Some(25), Some(30))
                    .unwrap(),
            );
        });
    });
}

fn benchmark_filter_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_scaling");

    for size in [1_000, 10_000, 100_000].iter() {
        let index = setup_index_with_vectors(*size);
        group.bench_with_input(
            criterion::BenchmarkId::new("keyword_filter", size),
            size,
            |b, _| {
                b.iter(|| {
                    black_box(index.get_ids_for_keyword("status", "active").unwrap());
                });
            },
        );
        group.bench_with_input(
            criterion::BenchmarkId::new("range_filter", size),
            size,
            |b, _| {
                b.iter(|| {
                    black_box(index.get_ids_in_range("age", Some(25), Some(35)).unwrap());
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_keyword_filter,
    benchmark_integer_range_filter,
    benchmark_float_range_filter,
    benchmark_text_filter,
    benchmark_geo_bounding_box_filter,
    benchmark_geo_radius_filter,
    benchmark_nested_field_filter,
    benchmark_filter_scaling
);
criterion_main!(benches);
