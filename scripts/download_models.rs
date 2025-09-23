//! Script to download and test all real transformer models
//!
//! Usage: cargo run --bin download_models --features real-models

use vectorizer::embedding::{EmbeddingProvider, RealModelEmbedder, RealModelType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Model Download and Test Script");
    println!("==================================");
    println!("This script will download and test all 7 recommended transformer models.");
    println!("Models will be cached in ./models/ directory.\n");

    let test_text = "Este é um teste em português brasileiro para verificar se os modelos funcionam corretamente.";
    let test_texts = vec![
        test_text,
        "This is an English test to verify model functionality.",
        "Ceci est un test en français pour vérifier les modèles.",
    ];

    let models = vec![
        ("MiniLM-Multilingual", RealModelType::MiniLMMultilingual, 384),
        ("E5-Small", RealModelType::E5SmallMultilingual, 384),
        ("DistilUSE-Multilingual", RealModelType::DistilUseMultilingual, 512),
        ("MPNet-Multilingual", RealModelType::MPNetMultilingualBase, 768),
        ("E5-Base", RealModelType::E5BaseMultilingual, 768),
        ("GTE-Base", RealModelType::GTEMultilingualBase, 768),
        ("LaBSE", RealModelType::LaBSE, 768),
    ];

    let mut results = Vec::new();

    for (i, (name, model_type, expected_dim)) in models.iter().enumerate() {
        println!("\n📥 [{}/{}] Testing {} ({})", i + 1, models.len(), name, model_type.model_id());
        println!("Expected dimension: {}D", expected_dim);

        let start_time = std::time::Instant::now();

        match RealModelEmbedder::new(model_type.clone()) {
            Ok(embedder) => {
                let load_time = start_time.elapsed();
                println!("✅ Model loaded in {:.2}s", load_time.as_secs_f32());

                // Test embedding with progress feedback
                println!("🧮 Computing embeddings...");
                let embed_start = std::time::Instant::now();

                match embedder.embed(test_text) {
                    Ok(embedding) => {
                        let embed_time = embed_start.elapsed();
                        let actual_dim = embedding.len();

                        if actual_dim == expected_dim {
                            println!("✅ Embedding successful: {}D in {:.3}s",
                                actual_dim, embed_time.as_secs_f32());

                            // Test multilingual similarity with progress
                            print!("🌍 Testing multilingual similarity...");
                            let mut similarities = Vec::new();
                            for (j, text) in test_texts[1..].iter().enumerate() {
                                print!(" {}", j + 1);
                                std::io::Write::flush(&mut std::io::stdout()).ok();
                                let other_embedding = embedder.embed(text)?;
                                let similarity = cosine_similarity(&embedding, &other_embedding);
                                similarities.push(similarity);
                            }
                            println!();

                            println!("🔗 Similarities: EN={:.3}, FR={:.3}",
                                similarities[0], similarities[1]);

                            results.push((name.to_string(), true, load_time, embed_time));
                        } else {
                            println!("❌ Dimension mismatch: expected {}D, got {}D",
                                expected_dim, actual_dim);
                            results.push((name.to_string(), false, load_time, embed_time));
                        }
                    }
                    Err(e) => {
                        println!("❌ Embedding failed: {}", e);
                        results.push((name.to_string(), false, load_time, std::time::Duration::from_secs(0)));
                    }
                }
            }
            Err(e) => {
                println!("❌ Model loading failed: {}", e);
                results.push((name.to_string(), false, std::time::Duration::from_secs(0), std::time::Duration::from_secs(0)));
            }
        }
    }

    // Summary
    println!("\n📊 Summary Report");
    println!("=================");
    println!("{:<25} {:<8} {:<10} {:<10}", "Model", "Status", "Load Time", "Embed Time");
    println!("{:-<25} {:-<8} {:-<10} {:-<10}", "", "", "", "");

    let mut success_count = 0;
    for (name, success, load_time, embed_time) in &results {
        let status = if *success { "✅" } else { "❌" };
        println!("{:<25} {:<8} {:.2}s      {:.3}s",
            name, status,
            load_time.as_secs_f32(),
            embed_time.as_secs_f32());
        if *success { success_count += 1; }
    }

    println!("\n🎯 Results: {}/{} models loaded and tested successfully", success_count, results.len());

    if success_count == results.len() {
        println!("🎉 All models are working correctly!");
        println!("💡 You can now use these models in your applications.");
    } else {
        println!("⚠️  Some models failed. Check the errors above.");
    }

    println!("\n💾 Models cached in ./models/ directory for future use.");

    Ok(())
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}
