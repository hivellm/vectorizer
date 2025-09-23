//! Tests demonstrating text embeddings with vector search

use crate::{
    db::VectorStore,
    embedding::{TfIdfEmbedding, BagOfWordsEmbedding, CharNGramEmbedding, EmbeddingProvider},
    models::{CollectionConfig, DistanceMetric, HnswConfig, Payload, Vector},
};
use tempfile::tempdir;

/// Test semantic search with TF-IDF embeddings
#[test]
fn test_semantic_search_with_tfidf() {
    // Create embedding provider
    let mut tfidf = TfIdfEmbedding::new(50);
    
    // Sample documents
    let documents = vec![
        ("doc1", "Machine learning is a subset of artificial intelligence"),
        ("doc2", "Deep learning uses neural networks with multiple layers"),
        ("doc3", "Vector databases store high-dimensional embeddings"),
        ("doc4", "Natural language processing helps computers understand text"),
        ("doc5", "Computer vision enables machines to interpret visual information"),
    ];
    
    // Build vocabulary from corpus
    let corpus: Vec<&str> = documents.iter().map(|(_, text)| *text).collect();
    tfidf.build_vocabulary(&corpus);
    
    // Create vector store
    let store = VectorStore::new();
    let config = CollectionConfig {
        dimension: tfidf.dimension(),
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: None,
        compression: Default::default(),
    };
    store.create_collection("documents", config).unwrap();
    
    // Generate embeddings and insert documents
    let mut vectors = Vec::new();
    for (id, text) in &documents {
        let embedding = tfidf.embed(text).unwrap();
        let vector = Vector::with_payload(
            id.to_string(),
            embedding,
            Payload::from_value(serde_json::json!({
                "text": text,
                "type": "document"
            })).unwrap()
        );
        vectors.push(vector);
    }
    store.insert("documents", vectors).unwrap();
    
    // Test semantic search
    let query = "artificial intelligence and neural networks";
    let query_embedding = tfidf.embed(query).unwrap();
    let results = store.search("documents", &query_embedding, 3).unwrap();
    
    assert_eq!(results.len(), 3);
    
    // Results should be semantically related to the query
    println!("Query: {}", query);
    for (i, result) in results.iter().enumerate() {
        let payload = result.payload.as_ref().unwrap();
        println!("  {}. {} (score: {:.3}): {}", 
            i + 1, 
            result.id, 
            result.score,
            payload.data["text"].as_str().unwrap()
        );
    }
    
    // The most relevant documents should be about AI/ML and neural networks
    let top_ids: Vec<&str> = results.iter().take(2).map(|r| r.id.as_str()).collect();
    assert!(top_ids.contains(&"doc1") || top_ids.contains(&"doc2") || top_ids.contains(&"doc4"));
}

/// Test document clustering with embeddings
#[test]
fn test_document_clustering_with_embeddings() {
    let mut bow = BagOfWordsEmbedding::new(30);
    
    // Documents in different categories
    let documents = vec![
        // Programming languages
        ("lang1", "Python is a popular programming language for data science"),
        ("lang2", "JavaScript is used for web development and programming"),
        ("lang3", "Rust is a systems programming language with memory safety"),
        
        // Machine learning
        ("ml1", "Supervised learning uses labeled data for training models"),
        ("ml2", "Unsupervised learning discovers patterns without labels"),
        ("ml3", "Reinforcement learning trains agents through rewards"),
        
        // Databases
        ("db1", "SQL databases use structured query language"),
        ("db2", "NoSQL databases provide flexible data models"),
        ("db3", "Graph databases store data as nodes and edges"),
    ];
    
    let corpus: Vec<&str> = documents.iter().map(|(_, text)| *text).collect();
    bow.build_vocabulary(&corpus);
    
    let store = VectorStore::new();
    let config = CollectionConfig {
        dimension: bow.dimension(),
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: None,
        compression: Default::default(),
    };
    store.create_collection("clusters", config).unwrap();
    
    // Insert documents with embeddings
    let mut vectors = Vec::new();
    for (id, text) in &documents {
        let embedding = bow.embed(text).unwrap();
        let category = if id.starts_with("lang") {
            "programming"
        } else if id.starts_with("ml") {
            "machine_learning"
        } else {
            "databases"
        };
        
        let vector = Vector::with_payload(
            id.to_string(),
            embedding,
            Payload::from_value(serde_json::json!({
                "text": text,
                "category": category
            })).unwrap()
        );
        vectors.push(vector);
    }
    store.insert("clusters", vectors).unwrap();
    
    // Test finding similar documents within clusters
    let ml_query = "training machine learning models with data";
    let ml_embedding = bow.embed(ml_query).unwrap();
    let ml_results = store.search("clusters", &ml_embedding, 3).unwrap();
    
    // Should return mostly ML documents
    let ml_categories: Vec<String> = ml_results.iter()
        .map(|r| r.payload.as_ref().unwrap().data["category"].as_str().unwrap().to_string())
        .collect();
    
    let ml_count = ml_categories.iter().filter(|c| *c == "machine_learning").count();
    assert!(ml_count >= 2, "Expected at least 2 ML documents, got {}", ml_count);
}

/// Test multilingual support with character n-grams
#[test]
fn test_multilingual_embeddings() {
    let mut ngram = CharNGramEmbedding::new(100, 3);
    
    // Multilingual documents
    let documents = vec![
        ("en1", "Hello, how are you today?"),
        ("en2", "Welcome to the vector database"),
        ("es1", "Hola, ¿cómo estás hoy?"),
        ("es2", "Bienvenido a la base de datos vectorial"),
        ("fr1", "Bonjour, comment allez-vous aujourd'hui?"),
        ("fr2", "Bienvenue dans la base de données vectorielle"),
    ];
    
    let corpus: Vec<&str> = documents.iter().map(|(_, text)| *text).collect();
    ngram.build_vocabulary(&corpus);
    
    let store = VectorStore::new();
    let config = CollectionConfig {
        dimension: ngram.dimension(),
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: None,
        compression: Default::default(),
    };
    store.create_collection("multilingual", config).unwrap();
    
    // Insert documents
    let mut vectors = Vec::new();
    for (id, text) in &documents {
        let embedding = ngram.embed(text).unwrap();
        let lang = &id[..2];
        
        let vector = Vector::with_payload(
            id.to_string(),
            embedding,
            Payload::from_value(serde_json::json!({
                "text": text,
                "language": lang
            })).unwrap()
        );
        vectors.push(vector);
    }
    store.insert("multilingual", vectors).unwrap();
    
    // Test cross-lingual similarity
    // "Hello" in different languages should be somewhat similar due to character patterns
    let query = "Hola";
    let query_embedding = ngram.embed(query).unwrap();
    let results = store.search("multilingual", &query_embedding, 3).unwrap();
    
    // Should find Spanish documents first
    assert!(results[0].id.starts_with("es"));
}

/// Test persistence with real embeddings
#[test]
fn test_persistence_with_real_embeddings() {
    let mut tfidf = TfIdfEmbedding::new(20);
    
    let documents = vec![
        ("news1", "Breaking news: AI model achieves human-level performance"),
        ("news2", "Stock market reaches all-time high amid tech rally"),
        ("news3", "New breakthrough in quantum computing announced"),
    ];
    
    let corpus: Vec<&str> = documents.iter().map(|(_, text)| *text).collect();
    tfidf.build_vocabulary(&corpus);
    
    let store = VectorStore::new();
    let config = CollectionConfig {
        dimension: tfidf.dimension(),
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: None,
        compression: Default::default(),
    };
    store.create_collection("news", config).unwrap();
    
    // Insert with real embeddings
    let mut vectors = Vec::new();
    for (id, text) in &documents {
        let embedding = tfidf.embed(text).unwrap();
        let vector = Vector::with_payload(
            id.to_string(),
            embedding,
            Payload::from_value(serde_json::json!({
                "headline": text,
                "timestamp": "2025-09-23"
            })).unwrap()
        );
        vectors.push(vector);
    }
    store.insert("news", vectors).unwrap();
    
    // Save to disk
    let temp_dir = tempdir().unwrap();
    let save_path = temp_dir.path().join("news_embeddings.vdb");
    store.save(&save_path).unwrap();
    
    // Load and verify search still works
    let loaded_store = VectorStore::load(&save_path).unwrap();
    
    let query = "artificial intelligence breakthrough";
    let query_embedding = tfidf.embed(query).unwrap();
    let results = loaded_store.search("news", &query_embedding, 2).unwrap();
    
    assert_eq!(results.len(), 2);
    // AI and quantum computing news should be most relevant
    let top_ids: Vec<&str> = results.iter().map(|r| r.id.as_str()).collect();
    assert!(top_ids.contains(&"news1") || top_ids.contains(&"news3"));
}

/// Demonstrate real-world use case: FAQ search
#[test]
fn test_faq_search_system() {
    let mut tfidf = TfIdfEmbedding::new(50);
    
    // FAQ database
    let faqs = vec![
        ("faq1", "How do I reset my password?", "To reset your password, click on 'Forgot Password' on the login page and follow the instructions sent to your email."),
        ("faq2", "What payment methods do you accept?", "We accept credit cards (Visa, MasterCard, Amex), PayPal, and bank transfers."),
        ("faq3", "How can I track my order?", "You can track your order by logging into your account and clicking on 'Order History'. Each order has a tracking number."),
        ("faq4", "What is your return policy?", "We offer a 30-day return policy for all items in original condition. Shipping costs for returns are covered by the customer."),
        ("faq5", "How do I contact customer support?", "You can reach our customer support team via email at support@example.com or by phone at 1-800-123-4567."),
    ];
    
    // Build vocabulary from questions and answers
    let mut corpus = Vec::new();
    for (_, question, answer) in &faqs {
        corpus.push(*question);
        corpus.push(*answer);
    }
    tfidf.build_vocabulary(&corpus);
    
    let store = VectorStore::new();
    let config = CollectionConfig {
        dimension: tfidf.dimension(),
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: None,
        compression: Default::default(),
    };
    store.create_collection("faq", config).unwrap();
    
    // Insert FAQs
    let mut vectors = Vec::new();
    for (id, question, answer) in &faqs {
        // Combine question and answer for embedding
        let combined_text = format!("{} {}", question, answer);
        let embedding = tfidf.embed(&combined_text).unwrap();
        
        let vector = Vector::with_payload(
            id.to_string(),
            embedding,
            Payload::from_value(serde_json::json!({
                "question": question,
                "answer": answer
            })).unwrap()
        );
        vectors.push(vector);
    }
    store.insert("faq", vectors).unwrap();
    
    // Test user queries
    let user_queries = vec![
        "I forgot my password",
        "Can I pay with credit card?",
        "Where is my package?",
        "How to return item?",
    ];
    
    for user_query in user_queries {
        let query_embedding = tfidf.embed(user_query).unwrap();
        let results = store.search("faq", &query_embedding, 1).unwrap();
        
        assert!(!results.is_empty());
        let top_result = &results[0];
        let payload = top_result.payload.as_ref().unwrap();
        
        println!("User: {}", user_query);
        println!("FAQ: {}", payload.data["question"].as_str().unwrap());
        println!("Answer: {}", payload.data["answer"].as_str().unwrap());
        println!("Confidence: {:.2}%\n", top_result.score * 100.0);
    }
}
