//! Test to validate Metal Native payload retrieval
//! 
//! This test validates that:
//! 1. Search results return payloads
//! 2. Payloads contain content field
//! 3. Intelligent search tools work correctly

use vectorizer::{VectorStore, embedding::EmbeddingManager, intelligent_search::mcp_tools::*};
use std::sync::Arc;

#[tokio::test]
async fn test_metal_native_payload_retrieval() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init()
        .ok();

    println!("\n🧪 ===== TESTE: Metal Native Payload Retrieval =====\n");

    // Create store and embedding manager
    let store = Arc::new(VectorStore::new());
    
    // Create embedding manager with BM25 (512 dimensions)
    let mut embedding_manager = EmbeddingManager::new();
    let bm25 = vectorizer::embedding::Bm25Embedding::new(512);
    embedding_manager.register_provider("bm25".to_string(), Box::new(bm25));
    embedding_manager.set_default_provider("bm25").expect("Failed to set default provider");
    let embedding_manager = Arc::new(embedding_manager);

    // List available collections
    let collections = store.list_collections();
    println!("📚 Available collections: {}", collections.len());
    for collection in &collections {
        if let Ok(coll) = store.get_collection(collection) {
            let metadata = coll.metadata();
            println!("  - {}: {} vectors", collection, metadata.vector_count);
        }
    }

    // Filter Mimir collections
    let mimir_collections: Vec<String> = collections
        .into_iter()
        .filter(|name| name.starts_with("mimir-"))
        .collect();

    if mimir_collections.is_empty() {
        println!("❌ No Mimir collections found!");
        panic!("No collections to test");
    }

    println!("\n🎯 Testing with {} Mimir collections", mimir_collections.len());

    // Create MCP tool handler
    let handler = MCPToolHandler::new(store.clone(), embedding_manager.clone());

    // Test 1: Simple search on first collection
    println!("\n📋 Test 1: Simple search on first collection");
    let first_collection = &mimir_collections[0];
    println!("   Collection: {}", first_collection);

    let query = "database schema";
    println!("   Query: '{}'", query);

    // Embed query
    let query_embedding = embedding_manager.embed(query).expect("Failed to embed query");
    
    // Direct search
    let direct_results = store.search(first_collection, &query_embedding, 5)
        .expect("Search failed");

    println!("\n   Results: {} found", direct_results.len());
    
    let mut valid_results = 0;
    let mut results_with_payload = 0;
    let mut results_with_content = 0;

    for (i, result) in direct_results.iter().enumerate() {
        valid_results += 1;
        
        print!("   [{:2}] ID: {} | Score: {:.4} | ", i + 1, result.id, result.score);
        
        if let Some(ref payload) = result.payload {
            results_with_payload += 1;
            print!("Payload: ✓ | ");
            
            if let Some(content) = payload.data.get("content").and_then(|v| v.as_str()) {
                results_with_content += 1;
                let preview = content.chars().take(60).collect::<String>();
                println!("Content: \"{}...\"", preview);
            } else {
                println!("Content: ❌ (missing)");
            }
        } else {
            println!("Payload: ❌ (null)");
        }
    }

    println!("\n   📊 Summary:");
    println!("      - Valid results: {}", valid_results);
    println!("      - With payload: {}", results_with_payload);
    println!("      - With content: {}", results_with_content);

    // Validate results
    assert!(valid_results > 0, "❌ No valid results found");
    assert_eq!(results_with_payload, valid_results, "❌ Not all results have payload");
    assert_eq!(results_with_content, valid_results, "❌ Not all results have content");

    println!("   ✅ Test 1 PASSED: All results have payload and content\n");

    // Test 2: Intelligent Search
    println!("📋 Test 2: Intelligent Search (MCP Tool)");
    
    let intelligent_search_request = IntelligentSearchTool {
        query: "authentication system".to_string(),
        collections: Some(mimir_collections.clone()),
        max_results: Some(5),
        domain_expansion: Some(false),
        technical_focus: Some(true),
        mmr_enabled: Some(false),
        mmr_lambda: Some(0.7),
    };

    println!("   Query: '{}'", intelligent_search_request.query);
    println!("   Collections: {:?}", intelligent_search_request.collections);
    
    match handler.handle_intelligent_search(intelligent_search_request).await {
        Ok(response) => {
            println!("\n   Results: {} found", response.results.len());
            println!("   Metadata:");
            println!("      - Total queries: {}", response.metadata.total_queries);
            println!("      - Collections searched: {}", response.metadata.collections_searched);
            println!("      - Total results found: {}", response.metadata.total_results_found);
            println!("      - After dedup: {}", response.metadata.results_after_dedup);
            println!("      - Final count: {}", response.metadata.final_results_count);

            let mut has_content_count = 0;
            
            for (i, result) in response.results.iter().take(5).enumerate() {
                print!("   [{:2}] Collection: {} | Score: {:.4} | ", 
                       i + 1, result.collection, result.score);
                
                if !result.content.is_empty() {
                    has_content_count += 1;
                    let preview = result.content.chars().take(60).collect::<String>();
                    println!("Content: \"{}...\"", preview);
                } else {
                    println!("Content: ❌ (empty)");
                }
            }

            println!("\n   📊 Summary:");
            println!("      - Results with content: {}/{}", has_content_count, response.results.len());

            assert!(response.results.len() > 0, "❌ No results from intelligent search");
            assert_eq!(has_content_count, response.results.len(), 
                      "❌ Not all intelligent search results have content");

            println!("   ✅ Test 2 PASSED: Intelligent search returns content\n");
        }
        Err(e) => {
            println!("   ❌ Test 2 FAILED: {}", e);
            panic!("Intelligent search failed: {}", e);
        }
    }

    // Test 3: Multi-collection search
    println!("📋 Test 3: Multi-Collection Search");
    
    let multi_search_request = MultiCollectionSearchTool {
        query: "docker configuration".to_string(),
        collections: mimir_collections.clone(),
        max_per_collection: Some(3),
        max_total_results: Some(10),
        cross_collection_reranking: Some(true),
    };

    println!("   Query: '{}'", multi_search_request.query);
    println!("   Collections: {} collections", multi_search_request.collections.len());

    match handler.handle_multi_collection_search(multi_search_request).await {
        Ok(response) => {
            println!("\n   Results: {} found", response.results.len());

            let mut content_count = 0;
            let mut collection_distribution: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

            for result in &response.results {
                if !result.content.is_empty() {
                    content_count += 1;
                }
                *collection_distribution.entry(result.collection.clone()).or_insert(0) += 1;
            }

            println!("   📊 Distribution by collection:");
            for (collection, count) in &collection_distribution {
                println!("      - {}: {} results", collection, count);
            }

            println!("\n   📊 Summary:");
            println!("      - Total results: {}", response.results.len());
            println!("      - With content: {}", content_count);
            println!("      - Collections represented: {}", collection_distribution.len());

            assert!(response.results.len() > 0, "❌ No results from multi-collection search");
            assert!(content_count > 0, "❌ No results with content");

            println!("   ✅ Test 3 PASSED: Multi-collection search works\n");
        }
        Err(e) => {
            println!("   ❌ Test 3 FAILED: {}", e);
            panic!("Multi-collection search failed: {}", e);
        }
    }

    println!("\n🎉 ===== ALL TESTS PASSED =====\n");
    println!("✅ Metal Native payload retrieval is working correctly");
    println!("✅ Intelligent search returns content");
    println!("✅ Multi-collection search works");
    println!("\n===============================================\n");
}

#[tokio::test]
async fn test_payload_structure() {
    println!("\n🧪 ===== TESTE: Payload Structure Validation =====\n");

    let store = Arc::new(VectorStore::new());
    
    let collections = store.list_collections();
    let mimir_collections: Vec<String> = collections
        .into_iter()
        .filter(|name| name.starts_with("mimir-"))
        .collect();

    if mimir_collections.is_empty() {
        println!("⚠️  No Mimir collections found, skipping test");
        return;
    }

    println!("Testing payload structure in {} collections\n", mimir_collections.len());

    let mut embedding_manager = EmbeddingManager::new();
    let bm25 = vectorizer::embedding::Bm25Embedding::new(512);
    embedding_manager.register_provider("bm25".to_string(), Box::new(bm25));
    embedding_manager.set_default_provider("bm25").expect("Failed to set default provider");

    for collection_name in &mimir_collections {
        println!("📂 Collection: {}", collection_name);

        // Get collection metadata
        if let Ok(collection) = store.get_collection(collection_name) {
            let metadata = collection.metadata();
            println!("   Vectors: {}", metadata.vector_count);

            if metadata.vector_count == 0 {
                println!("   ⚠️  Empty collection, skipping\n");
                continue;
            }

            // Create a simple query
            let query_text = "test";
            let query_embedding = match embedding_manager.embed(query_text) {
                Ok(emb) => emb,
                Err(e) => {
                    println!("   ❌ Embedding failed: {}\n", e);
                    continue;
                }
            };

            // Search
            match store.search(collection_name, &query_embedding, 3) {
                Ok(results) => {
                    println!("   Results: {}", results.len());

                    for (i, result) in results.iter().enumerate() {
                        println!("\n   Result {}:", i + 1);
                        println!("      ID: {}", result.id);
                        println!("      Score: {:.4}", result.score);

                        if let Some(ref payload) = result.payload {
                            println!("      Payload fields:");
                            for (key, value) in payload.data.as_object().unwrap() {
                                match key.as_str() {
                                    "content" => {
                                        if let Some(content) = value.as_str() {
                                            let preview = content.chars().take(50).collect::<String>();
                                            println!("         - content: \"{}...\" ({} chars)", 
                                                   preview, content.len());
                                        }
                                    }
                                    "file_path" => {
                                        println!("         - file_path: {}", value);
                                    }
                                    "chunk_index" => {
                                        println!("         - chunk_index: {}", value);
                                    }
                                    _ => {
                                        println!("         - {}: {:?}", key, value);
                                    }
                                }
                            }

                            // Validate required fields
                            assert!(payload.data.get("content").is_some(), 
                                   "Missing 'content' field");
                            assert!(payload.data.get("file_path").is_some(), 
                                   "Missing 'file_path' field");
                        } else {
                            panic!("❌ Result {} has no payload!", i + 1);
                        }
                    }

                    println!("\n   ✅ All results have valid payloads");
                }
                Err(e) => {
                    println!("   ❌ Search failed: {}", e);
                    panic!("Search failed for collection {}", collection_name);
                }
            }
        }

        println!();
    }

    println!("🎉 All payload structures are valid!\n");
}

