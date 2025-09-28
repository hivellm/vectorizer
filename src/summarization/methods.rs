use crate::summarization::types::*;
use std::collections::HashMap;
use crate::embedding::{Bm25Embedding, TfIdfEmbedding, EmbeddingProvider};

/// Trait para implementar métodos de sumarização
pub trait SummarizationMethodTrait {
    /// Sumarizar texto usando este método
    fn summarize(&self, params: &SummarizationParams, config: &MethodConfig) -> Result<String, SummarizationError>;
    
    /// Verificar se o método está disponível
    fn is_available(&self) -> bool;
    
    /// Obter nome do método
    fn name(&self) -> &'static str;
}

/// Implementação de sumarização extrativa com algoritmo MMR
pub struct ExtractiveSummarizer {
    bm25: Bm25Embedding,
    tfidf: TfIdfEmbedding,
}

impl ExtractiveSummarizer {
    pub fn new() -> Self {
        Self {
            bm25: Bm25Embedding::new(512),
            tfidf: TfIdfEmbedding::new(512),
        }
    }
    
    /// Dividir texto em frases
    fn split_sentences(&self, text: &str) -> Vec<String> {
        text.split(&['.', '!', '?', '\n'])
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
    
    /// Calcular similaridade coseno entre dois vetores
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }
    
    /// Implementar algoritmo MMR (Maximal Marginal Relevance)
    fn mmr_selection(&self, sentences: &[String], query: &str, config: &MethodConfig) -> Result<Vec<usize>, SummarizationError> {
        if sentences.is_empty() {
            return Ok(vec![]);
        }
        
        // 1) Buscar com BM25 para obter relevância inicial
        let mut bm25_scores = Vec::new();
        for sentence in sentences {
            // Usar embedding BM25 e calcular similaridade com query
            let sentence_embedding = self.bm25.embed(sentence)?;
            let query_embedding = self.bm25.embed(query)?;
            let score = self.cosine_similarity(&sentence_embedding, &query_embedding);
            bm25_scores.push(score);
        }
        
        // 2) Construir matriz TF-IDF para todas as frases
        let mut tfidf_matrix = Vec::new();
        for sentence in sentences {
            let embedding = self.tfidf.embed(sentence)?;
            tfidf_matrix.push(embedding);
        }
        
        // 3) Vetorizar query com TF-IDF
        let query_vector = self.tfidf.embed(query)?;
        
        // 4) Calcular relevância de cada frase com a query
        let mut relevance_scores = Vec::new();
        for (i, sentence_vector) in tfidf_matrix.iter().enumerate() {
            let tfidf_relevance = self.cosine_similarity(sentence_vector, &query_vector);
            let bm25_relevance = bm25_scores[i];
            // Combinar BM25 e TF-IDF scores
            let combined_relevance = 0.6 * tfidf_relevance + 0.4 * bm25_relevance;
            relevance_scores.push(combined_relevance);
        }
        
        // 5) Algoritmo MMR
        let lambda = 0.7; // Parâmetro de balanceamento entre relevância e diversidade
        let max_sentences = config.max_sentences.unwrap_or(5);
        let compression_ratio = config.compression_ratio;
        let target_sentences = ((sentences.len() as f32 * compression_ratio).ceil() as usize)
            .min(max_sentences)
            .max(1);
        
        let mut selected_indices = Vec::new();
        
        while selected_indices.len() < target_sentences && selected_indices.len() < sentences.len() {
            let mut best_score = f32::NEG_INFINITY;
            let mut best_index = None;
            
            for i in 0..sentences.len() {
                if selected_indices.contains(&i) {
                    continue;
                }
                
                // Calcular redundância máxima com frases já selecionadas
                let mut max_redundancy: f32 = 0.0;
                for &selected_idx in &selected_indices {
                    let redundancy = self.cosine_similarity(&tfidf_matrix[i], &tfidf_matrix[selected_idx]);
                    max_redundancy = max_redundancy.max(redundancy);
                }
                
                // Calcular score MMR
                let mmr_score = lambda * relevance_scores[i] - (1.0 - lambda) * max_redundancy;
                
                if mmr_score > best_score {
                    best_score = mmr_score;
                    best_index = Some(i);
                }
            }
            
            if let Some(idx) = best_index {
                selected_indices.push(idx);
            } else {
                break;
            }
        }
        
        // 6) Ordenar por posição original para manter ordem
        selected_indices.sort();
        
        Ok(selected_indices)
    }
}

impl SummarizationMethodTrait for ExtractiveSummarizer {
    fn summarize(&self, params: &SummarizationParams, config: &MethodConfig) -> Result<String, SummarizationError> {
        if params.text.len() < 50 {
            return Err(SummarizationError::TextTooShort { length: params.text.len() });
        }
        
        let sentences = self.split_sentences(&params.text);
        if sentences.is_empty() {
            return Err(SummarizationError::SummarizationFailed { 
                message: "No sentences found in text".to_string() 
            });
        }
        
        // Filtrar frases muito curtas
        let min_length = config.min_sentence_length.unwrap_or(10);
        let filtered_sentences: Vec<String> = sentences
            .into_iter()
            .filter(|s| s.len() >= min_length)
            .collect();
        
        if filtered_sentences.is_empty() {
            return Err(SummarizationError::SummarizationFailed { 
                message: "No sentences meet minimum length requirement".to_string() 
            });
        }
        
        // Usar o texto completo como "query" para sumarização geral
        let query = &params.text;
        
        // Aplicar algoritmo MMR
        let selected_indices = self.mmr_selection(&filtered_sentences, query, config)?;
        
        if selected_indices.is_empty() {
            return Err(SummarizationError::SummarizationFailed { 
                message: "No sentences selected by MMR algorithm".to_string() 
            });
        }
        
        // Construir sumário com frases selecionadas
        let summary_sentences: Vec<String> = selected_indices
            .into_iter()
            .map(|idx| filtered_sentences[idx].clone())
            .collect();
        
        let summary = summary_sentences.join(". ") + ".";
        
        // Garantir que o sumário seja menor que o texto original
        if summary.len() >= params.text.len() {
            // Se o sumário for igual ou maior, pegar apenas as primeiras frases
            let target_length = (params.text.len() as f32 * config.compression_ratio) as usize;
            if summary_sentences.len() > 1 {
                let mut truncated_summary = String::new();
                for sentence in &summary_sentences {
                    if truncated_summary.len() + sentence.len() + 2 <= target_length {
                        if !truncated_summary.is_empty() {
                            truncated_summary.push_str(". ");
                        }
                        truncated_summary.push_str(sentence);
                    } else {
                        break;
                    }
                }
                if !truncated_summary.is_empty() {
                    return Ok(truncated_summary + ".");
                }
            }
            // Se ainda for muito grande, truncar o primeiro sentence
            if !summary_sentences.is_empty() {
                let first_sentence = &summary_sentences[0];
                if first_sentence.len() > target_length {
                    return Ok(first_sentence[..target_length.min(first_sentence.len())].to_string() + "...");
                }
            }
        }
        
        // Aplicar max_length se especificado
        if let Some(max_length) = params.max_length {
            if summary.len() > max_length {
                // Truncar para o max_length especificado
                let truncated = if max_length > 3 {
                    summary[..max_length - 3].to_string() + "..."
                } else {
                    summary[..max_length].to_string()
                };
                return Ok(truncated);
            }
        }
        
        Ok(summary)
    }
    
    fn is_available(&self) -> bool {
        true
    }
    
    fn name(&self) -> &'static str {
        "extractive"
    }
}

/// Implementação de extração de palavras-chave
pub struct KeywordSummarizer {
    // Pode incluir listas de stopwords, etc.
}

impl KeywordSummarizer {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Extrair palavras-chave do texto
    fn extract_keywords(&self, text: &str, config: &MethodConfig) -> Vec<String> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut word_counts: HashMap<String, usize> = HashMap::new();
        
        let min_length = config.min_keyword_length.unwrap_or(3);
        
        for word in words {
            let word_clean = word.to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>();
            
            if word_clean.len() >= min_length {
                *word_counts.entry(word_clean).or_insert(0) += 1;
            }
        }
        
        // Ordenar por frequência
        let mut word_freq: Vec<(String, usize)> = word_counts.into_iter().collect();
        word_freq.sort_by(|a, b| b.1.cmp(&a.1));
        
        let max_keywords = config.max_keywords.unwrap_or(10);
        word_freq
            .into_iter()
            .take(max_keywords)
            .map(|(word, _)| word)
            .collect()
    }
}

impl SummarizationMethodTrait for KeywordSummarizer {
    fn summarize(&self, params: &SummarizationParams, config: &MethodConfig) -> Result<String, SummarizationError> {
        if params.text.len() < 20 {
            return Err(SummarizationError::TextTooShort { length: params.text.len() });
        }
        
        let keywords = self.extract_keywords(&params.text, config);
        Ok(keywords.join(", "))
    }
    
    fn is_available(&self) -> bool {
        true
    }
    
    fn name(&self) -> &'static str {
        "keyword"
    }
}

/// Implementação de seleção de frases representativas
pub struct SentenceSummarizer {
    // Similar ao extrativo, mas foca em frases representativas
}

impl SentenceSummarizer {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Selecionar frases representativas
    fn select_representative_sentences(&self, text: &str, config: &MethodConfig) -> Vec<String> {
        let sentences: Vec<String> = text.split(&['.', '!', '?', '\n'])
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && s.len() >= config.min_sentence_length.unwrap_or(15))
            .collect();
        
        if sentences.is_empty() {
            return sentences;
        }
        
        let compression_ratio = config.compression_ratio;
        let max_sentences = config.max_sentences.unwrap_or(3);
        let target_sentences = ((sentences.len() as f32 * compression_ratio).ceil() as usize)
            .min(max_sentences)
            .max(1);
        
        // Selecionar frases distribuídas uniformemente
        let step = sentences.len() / target_sentences;
        let mut selected: Vec<String> = Vec::new();
        
        for i in 0..target_sentences {
            let idx = (i * step).min(sentences.len() - 1);
            selected.push(sentences[idx].clone());
        }
        
        selected
    }
}

impl SummarizationMethodTrait for SentenceSummarizer {
    fn summarize(&self, params: &SummarizationParams, config: &MethodConfig) -> Result<String, SummarizationError> {
        if params.text.len() < 50 {
            return Err(SummarizationError::TextTooShort { length: params.text.len() });
        }
        
        let sentences = self.select_representative_sentences(&params.text, config);
        if sentences.is_empty() {
            return Err(SummarizationError::SummarizationFailed { 
                message: "No suitable sentences found".to_string() 
            });
        }
        
        Ok(sentences.join(". ") + ".")
    }
    
    fn is_available(&self) -> bool {
        true
    }
    
    fn name(&self) -> &'static str {
        "sentence"
    }
}

/// Implementação de sumarização abstrativa (placeholder)
pub struct AbstractiveSummarizer {
    // Requer integração com LLM externo
}

impl AbstractiveSummarizer {
    pub fn new() -> Self {
        Self {}
    }
}

impl SummarizationMethodTrait for AbstractiveSummarizer {
    fn summarize(&self, _params: &SummarizationParams, _config: &MethodConfig) -> Result<String, SummarizationError> {
        Err(SummarizationError::SummarizationFailed { 
            message: "Abstractive summarization requires external LLM integration".to_string() 
        })
    }
    
    fn is_available(&self) -> bool {
        false // Por enquanto não disponível
    }
    
    fn name(&self) -> &'static str {
        "abstractive"
    }
}
