use std::env;
use std::path::PathBuf;
use vectorizer::document_loader::{DocumentLoader, LoaderConfig};
use vectorizer::embedding::{Bm25Embedding, EmbeddingManager};

fn print_usage() {
    eprintln!("Usage: build-tokenizer --project <PATH> [--embedding bm25] [--output <FILE>]");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut project: Option<String> = None;
    let mut embedding_type: String = "bm25".to_string();
    let mut output: Option<String> = None;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--project" => {
                project = args.next();
            }
            "--embedding" => {
                embedding_type = args.next().unwrap_or_else(|| "bm25".to_string());
            }
            "--output" => {
                output = args.next();
            }
            _ => {
                eprintln!("Unknown argument: {}", arg);
                print_usage();
                std::process::exit(1);
            }
        }
    }

    let project_path = project.ok_or("--project is required")?;
    let output_path = output.unwrap_or_else(|| {
        let dir = PathBuf::from(&project_path).join(".vectorizer");
        let _ = std::fs::create_dir_all(&dir);
        match embedding_type.as_str() {
            "bm25" => dir.join("tokenizer.bm25.json"),
            other => dir.join(format!("tokenizer.{}.json", other)),
        }
        .to_string_lossy()
        .to_string()
    });

    eprintln!("Building tokenizer for project: {}", project_path);
    eprintln!("Embedding type: {}", embedding_type);
    eprintln!("Output: {}", output_path);

    // Initialize loader with configured embedding type
    let mut config = LoaderConfig::default();
    config.embedding_type = embedding_type.clone();

    let loader = DocumentLoader::new(config);

    // Collect documents
    let documents = loader.collect_documents(&project_path)?;
    eprintln!("Found {} documents", documents.len());

    // Build vocabulary depending on embedding type
    let mut manager = EmbeddingManager::new();
    match embedding_type.as_str() {
        "bm25" => {
            let mut bm25 = Bm25Embedding::new(512);
            let texts: Vec<String> = documents.iter().map(|(_, c)| c.clone()).collect();
            bm25.build_vocabulary(&texts);
            eprintln!("BM25 vocabulary size: {}", bm25.vocabulary_size());
            bm25.save_vocabulary_json(&output_path)?;
        }
        "tfidf" => {
            let mut tfidf = vectorizer::embedding::TfIdfEmbedding::new(512);
            let texts: Vec<&str> = documents.iter().map(|(_, c)| c.as_str()).collect();
            tfidf.build_vocabulary(&texts);
            tfidf.save_vocabulary_json(&output_path)?;
            eprintln!("TF-IDF vocabulary saved");
        }
        "bagofwords" => {
            let mut bow = vectorizer::embedding::BagOfWordsEmbedding::new(512);
            let texts: Vec<&str> = documents.iter().map(|(_, c)| c.as_str()).collect();
            bow.build_vocabulary(&texts);
            bow.save_vocabulary_json(&output_path)?;
            eprintln!("BagOfWords vocabulary saved");
        }
        "charngram" => {
            let mut cng = vectorizer::embedding::CharNGramEmbedding::new(512, 3);
            let texts: Vec<&str> = documents.iter().map(|(_, c)| c.as_str()).collect();
            cng.build_vocabulary(&texts);
            cng.save_vocabulary_json(&output_path)?;
            eprintln!("CharNGram tokenizer saved");
        }
        other => {
            return Err(format!("Unsupported embedding for tokenizer build: {}", other).into());
        }
    }

    eprintln!("Tokenizer saved to {}", output_path);
    Ok(())
}
