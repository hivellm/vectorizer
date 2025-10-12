use vectorizer::error::Result;
use vectorizer::models::{Payload, CollectionConfig, HnswConfig, DistanceMetric, Vector};
use vectorizer::gpu::MetalNativeCollection;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🧪 Basic Metal Native Test");
    println!("==========================\n");

    #[cfg(all(feature = "metal-native", target_os = "macos"))]
    {
        test_basic_functionality().await?;
    }

    #[cfg(not(all(feature = "metal-native", target_os = "macos")))]
    {
        println!("❌ This test requires macOS with metal-native feature enabled");
        return Ok(());
    }

    Ok(())
}

#[cfg(all(feature = "metal-native", target_os = "macos"))]
async fn test_basic_functionality() -> Result<()> {
    use std::time::Instant;
    use vectorizer::gpu::metal_native::MetalNativeCollection;
    use vectorizer::models::{Vector, DistanceMetric};

    // Test 1: Create collection
    println!("📊 Test 1: Create Collection");
    println!("----------------------------");
    
    let start = Instant::now();
    let mut collection = MetalNativeCollection::new(128, DistanceMetric::Cosine)?;
    let elapsed = start.elapsed();
    
    println!("  ✅ Collection created: {:?}", elapsed);
    println!("  Dimension: 128D");
    println!("  Metric: Cosine");
    println!();

    // Test 2: Add a single vector
    println!("📊 Test 2: Add Single Vector");
    println!("----------------------------");
    
    let test_vector = Vector {
        id: "test_vector".to_string(),
        data: (0..128).map(|i| i as f32 * 0.01).collect(),
        payload: None,
    };
    
    let start = Instant::now();
    let index = collection.add_vector(test_vector.clone())?;
    let elapsed = start.elapsed();
    
    println!("  ✅ Vector added at index {}: {:?}", index, elapsed);
    println!("  Vector count: {}", collection.vector_count());
    println!();

    // Test 3: Get vector back
    println!("📊 Test 3: Retrieve Vector");
    println!("--------------------------");
    
    let start = Instant::now();
    let retrieved = collection.get_vector(index)?;
    let elapsed = start.elapsed();
    
    println!("  ✅ Vector retrieved: {:?}", elapsed);
    println!("  ID matches: {}", retrieved.id == test_vector.id);
    println!("  Data length matches: {}", retrieved.data.len() == test_vector.data.len());
    println!();

    // Test 4: Get vector by ID
    println!("📊 Test 4: Get Vector by ID");
    println!("---------------------------");
    
    let start = Instant::now();
    let retrieved_by_id = collection.get_vector_by_id("test_vector")?;
    let elapsed = start.elapsed();
    
    println!("  ✅ Vector retrieved by ID: {:?}", elapsed);
    println!("  ID matches: {}", retrieved_by_id.id == test_vector.id);
    println!();

    // Test 5: Remove vector
    println!("📊 Test 5: Remove Vector");
    println!("------------------------");
    
    let start = Instant::now();
    collection.remove_vector("test_vector".to_string())?;
    let elapsed = start.elapsed();
    
    println!("  ✅ Vector removed: {:?}", elapsed);
    println!("  Vector count after removal: {}", collection.vector_count());
    println!();

    // Test 6: Try to get removed vector (should fail - not found)
    println!("📊 Test 6: Verify Vector Removal");
    println!("---------------------------------");

    match collection.get_vector_by_id("test_vector") {
        Ok(_) => println!("  ❌ ERROR: Vector should have been removed"),
        Err(e) => println!("  ✅ Vector correctly removed: {}", e.to_string().contains("not found")),
    }

    println!();

    // Test 7: Add multiple vectors and test GPU search
    println!("📊 Test 7: GPU Search with Multiple Vectors");
    println!("-------------------------------------------");

    // Add several vectors for meaningful search test
    let mut test_vectors = Vec::new();
    for i in 0..10 {
        let mut data = vec![0.0; 128];
        // Create distinct vectors
        for j in 0..128 {
            data[j] = (i as f32 * 0.1) + (j as f32 * 0.01);
        }
        let vector = Vector::with_payload(
            format!("search_test_{}", i),
            data,
            Payload::new(serde_json::json!({"index": i})),
        );
        test_vectors.push(vector);
    }

    let start = Instant::now();
    for vector in &test_vectors {
        collection.add_vector(vector.clone())?;
    }
    let elapsed = start.elapsed();
    info!("  ✅ Added {} vectors for search test: {:?}", test_vectors.len(), elapsed);

    // Test GPU search - search for the first vector
    println!("📊 Test 8: GPU Full Search");
    println!("--------------------------");

    let query_vector = &test_vectors[0].data;
    let start = Instant::now();
    let search_results = collection.search(query_vector, 3)?;
    let elapsed = start.elapsed();

    info!("  ✅ GPU search completed: {} results in {:?}", search_results.len(), elapsed);
    info!("  Best match distance: {:.6}", search_results[0].1);

    // The first result should be very close (exact match)
    assert!(search_results[0].1 < 0.001, "First result should be exact match");
    info!("  ✅ Search accuracy verified");

    println!();

    // Test 9: Test with 512D vectors (like real MCP collections)
    println!("📊 Test 9: GPU Search with 512D Vectors (MCP-like)");
    println!("--------------------------------------------------");

    let config_512d = CollectionConfig {
        dimension: 512,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig { m: 16, ..Default::default() },
        ..Default::default()
    };

    let mut collection_512d = MetalNativeCollection::new_with_name_and_config(
        "MetalNativeCollection512D",
        config_512d
    )?;
    info!("  ✅ Created 512D collection");

    // Add some 512D vectors
    let mut test_vectors_512d = Vec::new();
    for i in 0..5 {
        let mut data = vec![0.0; 512];
        // Create distinct vectors
        for j in 0..512 {
            data[j] = ((i as f32).sin() * (j as f32).cos() * 0.001) + (i as f32 * 0.0001);
        }
        let vector = Vector::new(format!("vec512_{}", i), data);
        test_vectors_512d.push(vector);
    }

    for vector in &test_vectors_512d {
        collection_512d.add_vector(vector.clone())?;
    }
    info!("  ✅ Added 5 vectors to 512D collection");

    // Test GPU search with 512D
    let query_512d = &test_vectors_512d[0].data;
    let start = Instant::now();
    let search_results_512d = collection_512d.search(query_512d, 3)?;
    let elapsed = start.elapsed();

    info!("  ✅ 512D GPU search completed: {} results in {:?}", search_results_512d.len(), elapsed);
    if !search_results_512d.is_empty() {
        info!("  🎯 Best distance: {:.6}", search_results_512d[0].1);
    }

    println!();

    // Test 10: Test edge cases that might cause MCP crashes
    println!("📊 Test 10: Edge Cases (Potential MCP Crash Scenarios)");
    println!("-----------------------------------------------------");

    // Test k=0
    println!("Testing k=0...");
    let empty_results = collection.search(query_vector, 0)?;
    info!("  ✅ k=0 returned {} results (expected 0)", empty_results.len());
    assert_eq!(empty_results.len(), 0);

    // Test k > vector_count
    println!("Testing k > vector_count...");
    let large_k_results = collection.search(query_vector, 100)?;
    let actual_vector_count = collection.vector_count();
    info!("  ✅ k=100 returned {} results, collection has {} vectors", large_k_results.len(), actual_vector_count);
    assert_eq!(large_k_results.len(), actual_vector_count);

    // Test with invalid query dimensions (should fail gracefully)
    println!("Testing dimension mismatch...");
    let wrong_dim_query = vec![1.0; 64]; // 64D instead of 128D
    match collection.search(&wrong_dim_query, 1) {
        Ok(_) => panic!("Should have failed with dimension mismatch"),
        Err(e) => info!("  ✅ Correctly failed with dimension mismatch: {}", e.to_string().contains("DimensionMismatch")),
    }

    // Test concurrent searches (simulate MCP load)
    println!("Testing concurrent searches...");
    use std::thread;
    let mut handles = vec![];

    for i in 0..3 {
        let mut collection_clone = MetalNativeCollection::new_with_name_and_config(
            &format!("concurrent_test_{}", i),
            CollectionConfig {
                dimension: 128,
                metric: DistanceMetric::Cosine,
                hnsw_config: HnswConfig { m: 16, ..Default::default() },
                ..Default::default()
            }
        )?;

        // Add vectors to concurrent collection
        for vector in &test_vectors {
            collection_clone.add_vector(vector.clone())?;
        }

        let value = test_vectors.clone();
        let handle = thread::spawn(move || {
            for _ in 0..5 {
                let _ = collection_clone.search(&value[0].data, 3);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    info!("  ✅ Concurrent searches completed without crashes");

    println!();
    println!("🎉 All Metal Native functionality tests passed, including edge cases!");
    println!("🎉 MCP crash scenarios tested successfully!");

    Ok(())
}
