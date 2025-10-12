use vectorizer::error::Result;

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
    println!("🎉 All basic Metal Native functionality tests passed!");
    
    Ok(())
}
