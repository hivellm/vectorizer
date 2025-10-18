//! Test to validate Metal Native payload retrieval
//!
//! This test validates that:
//! 1. Search results return payloads
//! 2. Payloads contain content field
//! 3. Intelligent search tools work correctly

use std::sync::Arc;

use vectorizer::VectorStore;
use vectorizer::embedding::EmbeddingManager;
use vectorizer::intelligent_search::mcp_tools::*;

#[tokio::test]
async fn test_metal_native_payload_retrieval() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init()
        .ok();

    println!("\n🧪 ===== TESTE: Metal Native Payload Retrieval =====\n");

    // Create store and a fresh test collection
    let store = Arc::new(VectorStore::new());

    // Create a fresh test collection with Metal Native
    let test_collection = "test-metal-payload";
    let config = vectorizer::models::CollectionConfig {
        dimension: 512,
        metric: vectorizer::models::DistanceMetric::Cosine,
        hnsw_config: vectorizer::models::HnswConfig::default(),
        quantization: vectorizer::models::QuantizationConfig::SQ { bits: 8 },
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
    };

    println!("🆕 Creating fresh test collection: {}", test_collection);
    store
        .create_collection(test_collection, config)
        .expect("Failed to create test collection");

    // Create embedding manager with BM25 (512 dimensions)
    let mut embedding_manager = EmbeddingManager::new();
    let bm25 = vectorizer::embedding::Bm25Embedding::new(512);
    embedding_manager.register_provider("bm25".to_string(), Box::new(bm25));
    embedding_manager
        .set_default_provider("bm25")
        .expect("Failed to set default provider");
    let embedding_manager = Arc::new(embedding_manager);

    // Add test vectors with payload
    println!("📝 Adding test vectors with payload...");
    let test_docs = vec![
        ("doc1", "database schema design patterns"),
        ("doc2", "API endpoint authentication methods"),
        ("doc3", "Docker container orchestration guide"),
    ];

    let mut test_vectors = Vec::new();
    for (doc_id, content) in &test_docs {
        let embedding = embedding_manager.embed(content).expect("Failed to embed");
        let vector = vectorizer::models::Vector {
            id: format!("test-{}", doc_id),
            data: embedding,
            payload: Some(vectorizer::models::Payload::new(serde_json::json!({
                "content": content,
                "doc_id": doc_id,
                "file_path": format!("/test/{}.txt", doc_id),
            }))),
        };
        test_vectors.push(vector);
    }

    println!("💾 Inserting {} test vectors...", test_vectors.len());
    store
        .insert(test_collection, test_vectors)
        .expect("Failed to insert vectors");

    println!("✅ Test vectors inserted successfully");
    println!(
        "📊 Collection now has {} vectors",
        store
            .get_collection(test_collection)
            .unwrap()
            .metadata()
            .vector_count
    );

    // Create MCP tool handler
    let handler = MCPToolHandler::new(store.clone(), embedding_manager.clone());

    // Test 1: Simple search on test collection
    println!("\n📋 Test 1: Simple search on test collection");
    println!("   Collection: {}", test_collection);

    let query = "database design";
    println!("   Query: '{}'", query);

    // Embed query
    let query_embedding = embedding_manager
        .embed(query)
        .expect("Failed to embed query");

    // Direct search
    let direct_results = store
        .search(test_collection, &query_embedding, 3)
        .expect("Search failed");

    println!("\n   Results: {} found", direct_results.len());

    let mut valid_results = 0;
    let mut results_with_payload = 0;
    let mut results_with_content = 0;

    for (i, result) in direct_results.iter().enumerate() {
        valid_results += 1;

        print!(
            "   [{:2}] ID: {} | Score: {:.4} | ",
            i + 1,
            result.id,
            result.score
        );

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
    assert_eq!(
        results_with_payload, valid_results,
        "❌ Not all results have payload"
    );
    assert_eq!(
        results_with_content, valid_results,
        "❌ Not all results have content"
    );

    println!("   ✅ Test 1 PASSED: All results have payload and content\n");

    // Test 2: Intelligent Search
    println!("\n📋 Test 2: Intelligent Search (MCP Tool)");

    let intelligent_search_request = IntelligentSearchTool {
        query: "authentication methods".to_string(),
        collections: Some(vec![test_collection.to_string()]),
        max_results: Some(3),
        domain_expansion: Some(false),
        technical_focus: Some(true),
        mmr_enabled: Some(false),
        mmr_lambda: Some(0.7),
    };

    println!("   Query: '{}'", intelligent_search_request.query);
    println!(
        "   Collections: {:?}",
        intelligent_search_request.collections
    );

    match handler
        .handle_intelligent_search(intelligent_search_request)
        .await
    {
        Ok(response) => {
            println!("\n   Results: {} found", response.results.len());
            println!("   Metadata:");
            println!("      - Total queries: {}", response.metadata.total_queries);
            println!(
                "      - Collections searched: {}",
                response.metadata.collections_searched
            );
            println!(
                "      - Total results found: {}",
                response.metadata.total_results_found
            );
            println!(
                "      - After dedup: {}",
                response.metadata.results_after_dedup
            );
            println!(
                "      - Final count: {}",
                response.metadata.final_results_count
            );

            let mut has_content_count = 0;

            for (i, result) in response.results.iter().take(5).enumerate() {
                print!(
                    "   [{:2}] Collection: {} | Score: {:.4} | ",
                    i + 1,
                    result.collection,
                    result.score
                );

                if !result.content.is_empty() {
                    has_content_count += 1;
                    let preview = result.content.chars().take(60).collect::<String>();
                    println!("Content: \"{}...\"", preview);
                } else {
                    println!("Content: ❌ (empty)");
                }
            }

            println!("\n   📊 Summary:");
            println!(
                "      - Results with content: {}/{}",
                has_content_count,
                response.results.len()
            );

            assert!(
                response.results.len() > 0,
                "❌ No results from intelligent search"
            );
            assert_eq!(
                has_content_count,
                response.results.len(),
                "❌ Not all intelligent search results have content"
            );

            println!("   ✅ Test 2 PASSED: Intelligent search returns content\n");
        }
        Err(e) => {
            println!("   ❌ Test 2 FAILED: {}", e);
            panic!("Intelligent search failed: {}", e);
        }
    }

    // Test 3: Multi-collection search (using single test collection)
    println!("\n📋 Test 3: Multi-Collection Search");

    let multi_search_request = MultiCollectionSearchTool {
        query: "container orchestration".to_string(),
        collections: vec![test_collection.to_string()],
        max_per_collection: Some(2),
        max_total_results: Some(3),
        cross_collection_reranking: Some(true),
    };

    println!("   Query: '{}'", multi_search_request.query);
    println!(
        "   Collections: {} collections",
        multi_search_request.collections.len()
    );

    match handler
        .handle_multi_collection_search(multi_search_request)
        .await
    {
        Ok(response) => {
            println!("\n   Results: {} found", response.results.len());

            let mut content_count = 0;
            let mut collection_distribution: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();

            for result in &response.results {
                if !result.content.is_empty() {
                    content_count += 1;
                }
                *collection_distribution
                    .entry(result.collection.clone())
                    .or_insert(0) += 1;
            }

            println!("   📊 Distribution by collection:");
            for (collection, count) in &collection_distribution {
                println!("      - {}: {} results", collection, count);
            }

            println!("\n   📊 Summary:");
            println!("      - Total results: {}", response.results.len());
            println!("      - With content: {}", content_count);
            println!(
                "      - Collections represented: {}",
                collection_distribution.len()
            );

            assert!(
                response.results.len() > 0,
                "❌ No results from multi-collection search"
            );
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

    // Load persisted collections from data directory
    println!("📂 Loading persisted collections...");
    match store.load_all_persisted_collections() {
        Ok(count) => {
            println!("✅ Loaded {} collections from disk\n", count);
        }
        Err(e) => {
            println!("⚠️  Failed to load collections: {}\n", e);
        }
    }

    let collections = store.list_collections();
    let mimir_collections: Vec<String> = collections
        .into_iter()
        .filter(|name| name.starts_with("mimir-"))
        .collect();

    if mimir_collections.is_empty() {
        println!("⚠️  No Mimir collections found, skipping test");
        return;
    }

    println!(
        "Testing payload structure in {} collections\n",
        mimir_collections.len()
    );

    let mut embedding_manager = EmbeddingManager::new();
    let bm25 = vectorizer::embedding::Bm25Embedding::new(512);
    embedding_manager.register_provider("bm25".to_string(), Box::new(bm25));
    embedding_manager
        .set_default_provider("bm25")
        .expect("Failed to set default provider");

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
                                            let preview =
                                                content.chars().take(50).collect::<String>();
                                            println!(
                                                "         - content: \"{}...\" ({} chars)",
                                                preview,
                                                content.len()
                                            );
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
                            assert!(
                                payload.data.get("content").is_some(),
                                "Missing 'content' field"
                            );
                            assert!(
                                payload.data.get("file_path").is_some(),
                                "Missing 'file_path' field"
                            );
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
