//! Example demonstrating real transformer model embeddings
//!
//! Run with: cargo run --example real_models_example --features real-models

#[cfg(feature = "real-models")]
use vectorizer::embedding::{EmbeddingProvider, RealModelEmbedder, RealModelType};

#[cfg(feature = "real-models")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Testing Real Transformer Models");
    println!("=====================================");

    // Test texts in Portuguese and English
    let texts = vec![
        "Como funciona a busca semântica?", // Portuguese
        "What is semantic search?",         // English
        "Busca por similaridade de texto",  // Portuguese
        "Text similarity search",           // English
    ];

    // Test different models
    let models = vec![
        ("MiniLM Multilingual", RealModelType::MiniLMMultilingual),
        ("E5 Small", RealModelType::E5SmallMultilingual),
    ];

    for (name, model_type) in models {
        println!("\n🔍 Testing {} ({})", name, model_type.model_id());
        println!("{}", "-".repeat(50));

        match RealModelEmbedder::new(model_type.clone()) {
            Ok(embedder) => {
                println!("✅ Model loaded successfully!");
                println!("📏 Dimension: {}D", embedder.dimension());

                // Generate embeddings for test texts
                for (i, text) in texts.iter().enumerate() {
                    match embedder.embed(text) {
                        Ok(embedding) => {
                            println!(
                                "  [{}] \"{}\" -> {:.3}... ({} dims)",
                                i + 1,
                                text,
                                embedding[0],
                                embedding.len()
                            );
                        }
                        Err(e) => {
                            println!("  ❌ Failed to embed \"{}\": {}", text, e);
                        }
                    }
                }

                // Test similarity between Portuguese and English versions
                let pt_embedding = embedder.embed(&texts[0])?;
                let en_embedding = embedder.embed(&texts[1])?;
                let similarity = cosine_similarity(&pt_embedding, &en_embedding);
                println!("  🔗 Similarity PT-EN: {:.3}", similarity);
            }
            Err(e) => {
                println!("❌ Failed to load model: {}", e);
            }
        }
    }

    println!("\n✅ Real models example completed!");
    Ok(())
}

#[cfg(not(feature = "real-models"))]
fn main() {
    println!("❌ Real models feature not enabled!");
    println!("   Compile with: cargo run --example real_models_example --features real-models");
    println!("   This will download and test actual transformer models.");
}

#[cfg(feature = "real-models")]
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot_product / (norm_a * norm_b)
}
