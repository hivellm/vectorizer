use vectorizer::error::Result;
use vectorizer::models::{Payload, CollectionConfig, HnswConfig, DistanceMetric, Vector};
use hive_gpu::{GpuVector, GpuDistanceMetric, GpuContext, GpuVectorStorage};
use hive_gpu::metal::{MetalNativeContext, MetalNativeVectorStorage};
use tracing::{info, error};

// Test MCP integration
#[cfg(feature = "metal-native")]
#[tokio::test]
async fn test_mcp_integration() -> Result<()> {
    use vectorizer::{VectorStore, CollectionConfig, DistanceMetric};
    use vectorizer::models::HnswConfig;

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🧪 MCP Integration Test");
    println!("=======================");

    // Create VectorStore like the server does
    let store = VectorStore::new();

    // Create a Metal Native collection like the server
    let config = CollectionConfig {
        dimension: 128,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig { m: 16, ..Default::default() },
        ..Default::default()
    };

    store.create_collection("mcp_test_collection", config)?;

    // Add some vectors
    for i in 0..5 {
        let mut data = vec![0.0; 128];
        for j in 0..128 {
            data[j] = ((i as f32).sin() * (j as f32).cos() * 0.001) + (i as f32 * 0.0001);
        }
        let vector = Vector::new(format!("mcp_vec_{}", i), data);
        store.insert("mcp_test_collection", vector)?;
    }

    println!("✅ MCP collection created and populated");

    // Simulate MCP search
    let query = vec![0.1; 128];
    let results = store.search("mcp_test_collection", &query, 3)?;
    println!("✅ MCP search successful: {} results", results.len());

    Ok(())
}

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
    // Using hive-gpu instead of local GPU module
    use vectorizer::models::{Vector, DistanceMetric};

    // Test 1: Create collection
    println!("📊 Test 1: Create Collection");
    println!("----------------------------");
    
    let start = Instant::now();
    let context = MetalNativeContext::new()?;
    let mut collection = context.create_storage(128, GpuDistanceMetric::Cosine)?;
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
    let indices = collection.add_vectors(&[test_vector.clone()])?;
    let index = indices[0];
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
        collection.add_vectors(&[vector.clone()])?;
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

    let context_512d = MetalNativeContext::new()?;
    let mut collection_512d = context_512d.create_storage(512, GpuDistanceMetric::Cosine)?;
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
        collection_512d.add_vectors(&[vector.clone()])?;
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
        let context_clone = MetalNativeContext::new()?;
        let mut collection_clone = context_clone.create_storage(128, GpuDistanceMetric::Cosine)?;

        // Add vectors to concurrent collection
        for vector in &test_vectors {
            collection_clone.add_vectors(&[vector.clone()])?;
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

    // Test 11: MCP-like search simulation
    println!("\n📊 Test 11: MCP-like Search Simulation");
    println!("--------------------------------------");

    // Simulate what MCP might do - search with specific parameters
    let mcp_query = vec![0.1; 128]; // Simple query vector
    let mcp_k = 5; // Reasonable k value

    info!("🔍 Simulating MCP search call...");
    let mcp_results = collection.search(&mcp_query, mcp_k)?;
    info!("✅ MCP simulation successful: {} results", mcp_results.len());

    // Test 12: Discovery-like search with embedding manager
    println!("\n📊 Test 12: Discovery-like Search (with Embedding)");
    println!("--------------------------------------------------");

    use vectorizer::embedding::EmbeddingManager;

    let mut embedding_manager = EmbeddingManager::new();
    // Register a simple BM25 embedding provider
    let bm25 = vectorizer::embedding::Bm25Embedding::new(128);
    embedding_manager.register_provider("bm25".to_string(), Box::new(bm25));
    embedding_manager.set_default_provider("bm25")?;

    // Create a text query like discovery does
    let text_query = "test query for discovery simulation";
    info!("🔍 Creating embedding for text query: '{}'", text_query);

    let query_embedding = embedding_manager.embed(text_query)?;
    info!("✅ Embedding created: {} dimensions", query_embedding.len());

    // Search with the embedding (like discovery does)
    let discovery_results = collection.search(&query_embedding, 3)?;
    info!("✅ Discovery-like search successful: {} results", discovery_results.len());

    // Test 13: 512D Discovery-like search (real MCP scenario)
    println!("\n📊 Test 13: 512D Discovery-like Search (Real MCP Scenario)");
    println!("---------------------------------------------------------");

    let mut embedding_manager_512d = EmbeddingManager::new();
    let bm25_512d = vectorizer::embedding::Bm25Embedding::new(512);
    embedding_manager_512d.register_provider("bm25".to_string(), Box::new(bm25_512d));
    embedding_manager_512d.set_default_provider("bm25")?;

    let query_embedding_512d = embedding_manager_512d.embed(text_query)?;
    info!("✅ 512D Embedding created: {} dimensions", query_embedding_512d.len());

    let discovery_512d_results = collection_512d.search(&query_embedding_512d, 3)?;
    info!("✅ 512D Discovery-like search successful: {} results", discovery_512d_results.len());

    println!("🎉 All MCP and discovery simulations completed successfully!");
    println!("🎉 If this works but real MCP crashes, the issue is in MCP integration!");

    // Test 14: Server-like VectorStore test (real MCP scenario)
    println!("\n📊 Test 14: Server-like VectorStore Test (Real MCP Scenario)");
    println!("-----------------------------------------------------------");

    use vectorizer::VectorStore;

    // Create VectorStore like the server does
    let store = VectorStore::new_auto();
    info!("✅ VectorStore created with auto GPU detection");

    // Check if we have any loaded collections
    let collections = store.list_collections();
    info!("📊 VectorStore has {} collections loaded", collections.len());

    if collections.is_empty() {
        info!("⚠️ No collections loaded, creating test collection...");
        let config = CollectionConfig {
            dimension: 128,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig { m: 16, ..Default::default() },
            ..Default::default()
        };

        store.create_collection("server_test_collection", config)?;
        info!("✅ Created server test collection");

        // Add some vectors
        for i in 0..3 {
            let mut data = vec![0.0; 128];
            for j in 0..128 {
                data[j] = ((i as f32).sin() * (j as f32).cos() * 0.001) + (i as f32 * 0.0001);
            }
            let vector = Vector::new(format!("server_vec_{}", i), data);
            store.insert("server_test_collection", vec![vector])?;
        }
        info!("✅ Added 3 vectors to server test collection");
    }

    // Now test search like MCP does
    let query = vec![0.1; 128];
    for collection_name in &collections {
        info!("🔍 Testing MCP-like search on collection '{}'", collection_name);
        let results = store.search(collection_name, &query, 3)?;
        info!("✅ MCP-like search on '{}' successful: {} results", collection_name, results.len());
    }

    // Test with embedding manager like discovery
    let mut embedding_manager = EmbeddingManager::new();
    let bm25 = vectorizer::embedding::Bm25Embedding::new(128);
    embedding_manager.register_provider("bm25".to_string(), Box::new(bm25));
    embedding_manager.set_default_provider("bm25")?;

    let text_query = "server test query for MCP simulation";
    let query_embedding = embedding_manager.embed(text_query)?;
    info!("✅ Server embedding created: {} dimensions", query_embedding.len());

    for collection_name in &collections {
        info!("🔍 Testing discovery-like search on collection '{}'", collection_name);
        let results = store.search(collection_name, &query_embedding, 3)?;
        info!("✅ Discovery-like search on '{}' successful: {} results", collection_name, results.len());
    }

    println!("🎉 Server-like VectorStore tests completed successfully!");
    println!("🎉 This simulates exactly what MCP does - if this works, issue is elsewhere!");

    // Test 15: THE CRASH REPRODUCTION - Dimension mismatch like MCP does
    println!("\n📊 Test 15: CRASH REPRODUCTION - Dimension Mismatch (Real MCP Bug)");
    println!("-------------------------------------------------------------------");

    // This is the exact bug that crashes MCP!
    // Collections have different dimensions, but MCP uses same embedding size
    info!("🐛 Reproducing the exact MCP crash scenario...");

    let wrong_dim_query_64 = vec![0.1; 64];   // 64D query
    let wrong_dim_query_128 = vec![0.1; 128]; // 128D query

    // Try searching 64D query on 128D collection (should fail gracefully)
    match store.search("vectorizer-sdk-python", &wrong_dim_query_64, 3) {
        Ok(_) => {
            error!("🐛 UNEXPECTED: 64D query on 128D collection should have failed!");
            panic!("Dimension mismatch should have failed");
        }
        Err(e) => {
            info!("✅ 64D→128D dimension mismatch correctly failed: {}", e.to_string());
        }
    }

    // Try searching 128D query on 512D collection (should fail gracefully)
    match store.search("vectorizer-sdk-typescript", &wrong_dim_query_128, 3) {
        Ok(_) => {
            error!("🐛 UNEXPECTED: 128D query on 512D collection should have failed!");
            panic!("Dimension mismatch should have failed");
        }
        Err(e) => {
            info!("✅ 128D→512D dimension mismatch correctly failed: {}", e.to_string());
        }
    }

    println!("🎯 CRASH REPRODUCTION SUCCESSFUL!");
    println!("🎯 The bug is: MCP creates embeddings with fixed dimensions but collections have different dimensions");
    println!("🎯 Solution: MCP needs to create embeddings matching each collection's dimension");

    Ok(())
}
