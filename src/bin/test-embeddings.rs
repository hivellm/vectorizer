use std::env;
use std::path::PathBuf;
use vectorizer::embedding::{
    BagOfWordsEmbedding, Bm25Embedding, CharNGramEmbedding, EmbeddingManager, TfIdfEmbedding,
};

fn main() {
    let mut project: Option<String> = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--project" => project = args.next(),
            _ => {}
        }
    }

    println!("🧪 TEST: short terms across embeddings");
    let mut manager = EmbeddingManager::new();

    // Register providers with fixed dimension 512
    manager.register_provider("tfidf".to_string(), Box::new(TfIdfEmbedding::new(512)));
    manager.register_provider("bm25".to_string(), Box::new(Bm25Embedding::new(512)));
    manager.register_provider(
        "bagofwords".to_string(),
        Box::new(BagOfWordsEmbedding::new(512)),
    );
    manager.register_provider(
        "charngram".to_string(),
        Box::new(CharNGramEmbedding::new(512, 3)),
    );

    // Load tokenizers if available
    if let Some(proj) = &project {
        let base = PathBuf::from(proj).join(".vectorizer");
        if let Some(p) = manager.get_provider_mut("bm25") {
            if let Some(bm25) = p.as_any_mut().downcast_mut::<Bm25Embedding>() {
                let path = base.join("tokenizer.bm25.json");
                if path.exists() {
                    let _ = bm25.load_vocabulary_json(&path);
                }
            }
        }
        if let Some(p) = manager.get_provider_mut("tfidf") {
            if let Some(tfidf) = p.as_any_mut().downcast_mut::<TfIdfEmbedding>() {
                let path = base.join("tokenizer.tfidf.json");
                if path.exists() {
                    let _ = tfidf.load_vocabulary_json(&path);
                }
            }
        }
        if let Some(p) = manager.get_provider_mut("bagofwords") {
            if let Some(bow) = p.as_any_mut().downcast_mut::<BagOfWordsEmbedding>() {
                let path = base.join("tokenizer.bow.json");
                if path.exists() {
                    let _ = bow.load_vocabulary_json(&path);
                }
            }
        }
        if let Some(p) = manager.get_provider_mut("charngram") {
            if let Some(cng) = p.as_any_mut().downcast_mut::<CharNGramEmbedding>() {
                let path = base.join("tokenizer.charngram.json");
                if path.exists() {
                    let _ = cng.load_vocabulary_json(&path);
                }
            }
        }
    }

    let tests = vec![
        "a", "hi", "x", "1", ".", "", "   ", "gpt-5", "gemini", "claude-4", "ai", "ok",
    ];

    let providers = vec!["bm25", "tfidf", "bagofwords", "charngram"];

    for prov in providers {
        println!("\n=== Provider: {} ===", prov);
        for t in &tests {
            match manager.embed_with_provider(prov, t) {
                Ok(v) => {
                    let dim = v.len();
                    let non_zero = v.iter().filter(|&&x| x != 0.0).count();
                    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
                    println!(
                        "'{}' -> dim: {}, non_zero: {}, norm: {:.6}",
                        t, dim, non_zero, norm
                    );
                }
                Err(e) => {
                    println!("'{}' -> ERROR: {}", t, e);
                }
            }
        }
    }
}
