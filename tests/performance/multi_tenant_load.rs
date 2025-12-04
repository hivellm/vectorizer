//! Multi-Tenant Load Tests
//!
//! These tests verify system performance and stability with 100+ concurrent tenants.
#![allow(clippy::uninlined_format_args)]
//! They test:
//! - Concurrent tenant operations (create collections, insert vectors, search)
//! - Quota enforcement under load
//! - Authentication performance
//! - Isolation integrity under high concurrency
//! - Resource usage and memory limits
//!
//! Note: These are stress tests and may take several minutes to complete.
//! Run with: `cargo test --release --test all_tests performance::multi_tenant_load -- --ignored --test-threads=1`

use reqwest::blocking::Client;
use serde_json::{Value, json};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use uuid::Uuid;

const HIVEHUB_API_URL: &str = "http://localhost:12000";
const VECTORIZER_API_URL: &str = "http://localhost:15002";
const SERVICE_API_KEY: &str = "test-service-key";

/// Test statistics for load testing
#[derive(Debug)]
struct LoadTestStats {
    total_requests: AtomicUsize,
    successful_requests: AtomicUsize,
    failed_requests: AtomicUsize,
    total_latency_ms: AtomicUsize,
}

impl LoadTestStats {
    fn new() -> Self {
        Self {
            total_requests: AtomicUsize::new(0),
            successful_requests: AtomicUsize::new(0),
            failed_requests: AtomicUsize::new(0),
            total_latency_ms: AtomicUsize::new(0),
        }
    }

    fn record_request(&self, success: bool, latency_ms: u64) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        if success {
            self.successful_requests.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_requests.fetch_add(1, Ordering::Relaxed);
        }
        self.total_latency_ms
            .fetch_add(latency_ms as usize, Ordering::Relaxed);
    }

    fn print_summary(&self, test_name: &str, duration: Duration) {
        let total = self.total_requests.load(Ordering::Relaxed);
        let successful = self.successful_requests.load(Ordering::Relaxed);
        let failed = self.failed_requests.load(Ordering::Relaxed);
        let total_latency = self.total_latency_ms.load(Ordering::Relaxed);

        let avg_latency = if total > 0 { total_latency / total } else { 0 };
        let throughput = total as f64 / duration.as_secs_f64();
        let success_rate = if total > 0 {
            (successful as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        println!("\n=== Load Test Results: {} ===", test_name);
        println!("Duration: {:?}", duration);
        println!("Total Requests: {}", total);
        println!("Successful: {} ({:.2}%)", successful, success_rate);
        println!("Failed: {}", failed);
        println!("Average Latency: {}ms", avg_latency);
        println!("Throughput: {:.2} req/s", throughput);
        println!("=====================================\n");
    }
}

/// Helper to create HTTP client
fn create_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
}

/// Helper to register a test user in HiveHub
fn register_test_user(base_email: &str) -> (String, String) {
    let client = create_client();

    let unique_email = format!(
        "{}+{}@test.local",
        base_email.split('@').next().unwrap_or("test"),
        Uuid::new_v4()
    );

    let response = client
        .post(format!("{}/api/auth/register", HIVEHUB_API_URL))
        .json(&json!({
            "email": unique_email,
            "username": base_email.split('@').next().unwrap_or("test"),
            "password": "test123456",
            "full_name": "Load Test User"
        }))
        .send()
        .expect("Failed to register user");

    assert!(
        response.status() == 200 || response.status() == 201,
        "User registration failed"
    );

    let body: Value = response
        .json()
        .expect("Failed to parse registration response");
    let user_id = body["user"]["id"]
        .as_str()
        .expect("Missing user_id")
        .to_string();
    let token = body["access_token"]
        .as_str()
        .expect("Missing access_token")
        .to_string();

    (user_id, token)
}

/// Helper to create a collection for a tenant
fn create_collection_for_tenant(
    client: &Client,
    user_id: &str,
    collection_name: &str,
    dimension: usize,
) -> Result<(), String> {
    let response = client
        .post(format!("{}/collections", VECTORIZER_API_URL))
        .header("x-hivehub-service", SERVICE_API_KEY)
        .header("x-hivehub-user-id", user_id)
        .json(&json!({
            "name": collection_name,
            "dimension": dimension,
            "metric": "cosine"
        }))
        .send()
        .map_err(|e| e.to_string())?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("Create collection failed: {}", response.status()))
    }
}

/// Helper to insert vectors for a tenant
fn insert_vectors_for_tenant(
    client: &Client,
    user_id: &str,
    collection_name: &str,
    count: usize,
    dimension: usize,
) -> Result<(), String> {
    let full_name = format!("user_{}:{}", user_id, collection_name);
    let vectors: Vec<Value> = (0..count)
        .map(|i| {
            let data: Vec<f32> = (0..dimension)
                .map(|j| ((i + j) as f32 * 0.01) % 1.0)
                .collect();
            json!({
                "id": format!("vec_{}", i),
                "data": data
            })
        })
        .collect();

    let response = client
        .post(format!(
            "{}/collections/{}/vectors",
            VECTORIZER_API_URL, full_name
        ))
        .header("x-hivehub-service", SERVICE_API_KEY)
        .header("x-hivehub-user-id", user_id)
        .json(&json!({ "vectors": vectors }))
        .send()
        .map_err(|e| e.to_string())?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("Insert vectors failed: {}", response.status()))
    }
}

/// Helper to search vectors for a tenant
fn search_vectors_for_tenant(
    client: &Client,
    user_id: &str,
    collection_name: &str,
    dimension: usize,
) -> Result<(), String> {
    let full_name = format!("user_{}:{}", user_id, collection_name);
    let query_vector: Vec<f32> = (0..dimension).map(|i| (i as f32 * 0.01) % 1.0).collect();

    let response = client
        .post(format!("{}/search", VECTORIZER_API_URL))
        .header("x-hivehub-service", SERVICE_API_KEY)
        .header("x-hivehub-user-id", user_id)
        .json(&json!({
            "collection": full_name,
            "vector": query_vector,
            "limit": 10
        }))
        .send()
        .map_err(|e| e.to_string())?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("Search failed: {}", response.status()))
    }
}

#[test]
#[ignore = "requires running servers and is resource intensive"]
fn test_load_100_concurrent_tenants_basic_operations() {
    println!("\n=== Starting Load Test: 100 Concurrent Tenants - Basic Operations ===\n");

    let num_tenants = 100;
    let stats = Arc::new(LoadTestStats::new());
    let start_time = Instant::now();

    // Register all tenants
    println!("Registering {} tenants...", num_tenants);
    let tenants: Vec<(String, String)> = (0..num_tenants)
        .map(|i| {
            let (user_id, _token) = register_test_user(&format!("loadtest_{}", i));
            (user_id, format!("collection_{}", i))
        })
        .collect();

    println!("All tenants registered. Starting concurrent operations...\n");

    // Phase 1: Concurrent collection creation
    println!("Phase 1: Creating collections for all tenants...");
    let mut handles = Vec::new();

    for (user_id, collection_name) in tenants.iter() {
        let user_id = user_id.clone();
        let collection_name = collection_name.clone();
        let stats = Arc::clone(&stats);

        let handle = std::thread::spawn(move || {
            let client = create_client();
            let start = Instant::now();
            let result = create_collection_for_tenant(&client, &user_id, &collection_name, 128);
            let latency = start.elapsed().as_millis() as u64;
            stats.record_request(result.is_ok(), latency);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread should complete");
    }

    stats.print_summary("Phase 1: Collection Creation", start_time.elapsed());

    // Phase 2: Concurrent vector insertion
    println!("Phase 2: Inserting vectors for all tenants...");
    let stats2 = Arc::new(LoadTestStats::new());
    let mut handles = Vec::new();

    for (user_id, collection_name) in tenants.iter() {
        let user_id = user_id.clone();
        let collection_name = collection_name.clone();
        let stats = Arc::clone(&stats2);

        let handle = std::thread::spawn(move || {
            let client = create_client();
            let start = Instant::now();
            let result = insert_vectors_for_tenant(&client, &user_id, &collection_name, 50, 128);
            let latency = start.elapsed().as_millis() as u64;
            stats.record_request(result.is_ok(), latency);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread should complete");
    }

    stats2.print_summary("Phase 2: Vector Insertion", start_time.elapsed());

    // Phase 3: Concurrent searches
    println!("Phase 3: Performing concurrent searches...");
    let stats3 = Arc::new(LoadTestStats::new());
    let mut handles = Vec::new();

    for (user_id, collection_name) in tenants.iter() {
        let user_id = user_id.clone();
        let collection_name = collection_name.clone();
        let stats = Arc::clone(&stats3);

        let handle = std::thread::spawn(move || {
            let client = create_client();
            // Perform 10 searches per tenant
            for _ in 0..10 {
                let start = Instant::now();
                let result = search_vectors_for_tenant(&client, &user_id, &collection_name, 128);
                let latency = start.elapsed().as_millis() as u64;
                stats.record_request(result.is_ok(), latency);
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread should complete");
    }

    stats3.print_summary("Phase 3: Concurrent Searches", start_time.elapsed());

    let total_duration = start_time.elapsed();
    println!("\n=== Total Test Duration: {:?} ===", total_duration);

    // Assert minimum success rates
    let create_success_rate = stats.successful_requests.load(Ordering::Relaxed) as f64
        / stats.total_requests.load(Ordering::Relaxed) as f64;
    let insert_success_rate = stats2.successful_requests.load(Ordering::Relaxed) as f64
        / stats2.total_requests.load(Ordering::Relaxed) as f64;
    let search_success_rate = stats3.successful_requests.load(Ordering::Relaxed) as f64
        / stats3.total_requests.load(Ordering::Relaxed) as f64;

    assert!(
        create_success_rate >= 0.95,
        "Collection creation success rate should be >= 95%, got {:.2}%",
        create_success_rate * 100.0
    );
    assert!(
        insert_success_rate >= 0.90,
        "Vector insertion success rate should be >= 90%, got {:.2}%",
        insert_success_rate * 100.0
    );
    assert!(
        search_success_rate >= 0.95,
        "Search success rate should be >= 95%, got {:.2}%",
        search_success_rate * 100.0
    );
}

#[test]
#[ignore = "requires running servers and is resource intensive"]
fn test_load_sustained_concurrent_operations() {
    println!("\n=== Starting Load Test: Sustained Concurrent Operations ===\n");

    let num_tenants = 50;
    let duration_secs = 60; // 1 minute sustained load

    // Register tenants
    println!("Registering {} tenants...", num_tenants);
    let tenants: Vec<(String, String)> = (0..num_tenants)
        .map(|i| {
            let (user_id, _token) = register_test_user(&format!("sustained_{}", i));
            let collection_name = format!("sustained_collection_{}", i);

            // Create collection
            let client = create_client();
            let _ = create_collection_for_tenant(&client, &user_id, &collection_name, 128);

            // Insert initial vectors
            let _ = insert_vectors_for_tenant(&client, &user_id, &collection_name, 100, 128);

            (user_id, collection_name)
        })
        .collect();

    println!("Setup complete. Starting sustained load...\n");

    let stats = Arc::new(LoadTestStats::new());
    let start_time = Instant::now();
    let duration = Duration::from_secs(duration_secs);

    // Spawn threads for each tenant
    let mut handles = Vec::new();

    for (user_id, collection_name) in tenants {
        let stats = Arc::clone(&stats);
        let end_time = start_time + duration;

        let handle = std::thread::spawn(move || {
            let client = create_client();
            let mut operation_count = 0;

            while Instant::now() < end_time {
                let start = Instant::now();

                // Randomly choose operation: 70% search, 20% insert, 10% list
                let op_type = operation_count % 10;
                let result = match op_type {
                    0..=6 => {
                        // Search operation
                        search_vectors_for_tenant(&client, &user_id, &collection_name, 128)
                    }
                    7..=8 => {
                        // Insert operation
                        insert_vectors_for_tenant(&client, &user_id, &collection_name, 5, 128)
                    }
                    _ => {
                        // List collections operation
                        let full_name = format!("user_{}:{}", user_id, collection_name);
                        client
                            .get(format!("{}/collections/{}", VECTORIZER_API_URL, full_name))
                            .header("x-hivehub-service", SERVICE_API_KEY)
                            .header("x-hivehub-user-id", &user_id)
                            .send()
                            .map(|_| ())
                            .map_err(|e| e.to_string())
                    }
                };

                let latency = start.elapsed().as_millis() as u64;
                stats.record_request(result.is_ok(), latency);
                operation_count += 1;

                // Small delay to avoid overwhelming the system
                std::thread::sleep(Duration::from_millis(100));
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread should complete");
    }

    let total_duration = start_time.elapsed();
    stats.print_summary("Sustained Concurrent Operations", total_duration);

    // Assert performance targets
    let success_rate = stats.successful_requests.load(Ordering::Relaxed) as f64
        / stats.total_requests.load(Ordering::Relaxed) as f64;
    let avg_latency = stats.total_latency_ms.load(Ordering::Relaxed)
        / stats.total_requests.load(Ordering::Relaxed);

    assert!(
        success_rate >= 0.95,
        "Success rate should be >= 95%, got {:.2}%",
        success_rate * 100.0
    );
    assert!(
        avg_latency <= 500,
        "Average latency should be <= 500ms, got {}ms",
        avg_latency
    );
}

#[test]
#[ignore = "requires running servers and is resource intensive"]
fn test_load_quota_enforcement_under_pressure() {
    println!("\n=== Starting Load Test: Quota Enforcement Under Pressure ===\n");

    let num_tenants = 20;
    let collections_per_tenant = 15; // Should exceed quota

    println!("Registering {} tenants...", num_tenants);
    let tenants: Vec<String> = (0..num_tenants)
        .map(|i| {
            let (user_id, _token) = register_test_user(&format!("quota_{}", i));
            user_id
        })
        .collect();

    println!(
        "Testing quota enforcement with {} collections per tenant...\n",
        collections_per_tenant
    );

    let stats = Arc::new(LoadTestStats::new());
    let start_time = Instant::now();
    let mut handles = Vec::new();

    for user_id in tenants {
        let stats = Arc::clone(&stats);

        let handle = std::thread::spawn(move || {
            let client = create_client();

            for i in 0..collections_per_tenant {
                let collection_name = format!("quota_test_{}", i);
                let start = Instant::now();
                let result = create_collection_for_tenant(&client, &user_id, &collection_name, 128);
                let latency = start.elapsed().as_millis() as u64;
                stats.record_request(result.is_ok(), latency);

                if result.is_err() {
                    // Quota exceeded expected after a certain number
                    break;
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread should complete");
    }

    stats.print_summary("Quota Enforcement Test", start_time.elapsed());

    // Verify that some requests failed due to quota (expected behavior)
    let total = stats.total_requests.load(Ordering::Relaxed);
    let failed = stats.failed_requests.load(Ordering::Relaxed);

    println!("Total collection creation attempts: {}", total);
    println!("Failed due to quota: {}", failed);

    // We expect some failures due to quota limits
    assert!(failed > 0, "Should have some quota-exceeded failures");
}
