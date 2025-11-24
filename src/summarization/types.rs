use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Tipos de métodos de sumarização disponíveis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SummarizationMethod {
    /// Sumarização extrativa - seleciona as frases mais importantes
    Extractive,
    /// Sumarização abstrativa - gera novo texto resumido
    Abstractive,
    /// Extração de palavras-chave
    Keyword,
    /// Seleção de frases representativas
    Sentence,
}

impl std::fmt::Display for SummarizationMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SummarizationMethod::Extractive => write!(f, "extractive"),
            SummarizationMethod::Abstractive => write!(f, "abstractive"),
            SummarizationMethod::Keyword => write!(f, "keyword"),
            SummarizationMethod::Sentence => write!(f, "sentence"),
        }
    }
}

impl std::str::FromStr for SummarizationMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "extractive" => Ok(SummarizationMethod::Extractive),
            "abstractive" => Ok(SummarizationMethod::Abstractive),
            "keyword" => Ok(SummarizationMethod::Keyword),
            "sentence" => Ok(SummarizationMethod::Sentence),
            _ => Err(format!("Invalid summarization method: {}", s)),
        }
    }
}

/// Configuração para um método específico de sumarização
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodConfig {
    pub enabled: bool,
    pub compression_ratio: f32,
    pub max_sentences: Option<usize>,
    pub min_sentence_length: Option<usize>,
    pub max_keywords: Option<usize>,
    pub min_keyword_length: Option<usize>,
    pub use_tfidf: Option<bool>,
    pub use_stopwords: Option<bool>,
    pub use_position_weight: Option<bool>,
    pub language: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
}

impl Default for MethodConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            compression_ratio: 0.3,
            max_sentences: Some(5),
            min_sentence_length: Some(10),
            max_keywords: Some(10),
            min_keyword_length: Some(3),
            use_tfidf: Some(true),
            use_stopwords: Some(true),
            use_position_weight: Some(true),
            language: Some("en".to_string()),
            model: None,
            api_key: None,
            max_tokens: Some(150),
            temperature: Some(0.3),
        }
    }
}

/// Configuração de idioma para sumarização
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    pub stopwords: bool,
    pub stemming: bool,
}

impl Default for LanguageConfig {
    fn default() -> Self {
        Self {
            stopwords: true,
            stemming: true,
        }
    }
}

/// Configuração de metadados para sumários
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataConfig {
    pub include_original_id: bool,
    pub include_file_path: bool,
    pub include_timestamp: bool,
    pub include_method: bool,
    pub include_compression_ratio: bool,
}

impl Default for MetadataConfig {
    fn default() -> Self {
        Self {
            include_original_id: true,
            include_file_path: true,
            include_timestamp: true,
            include_method: true,
            include_compression_ratio: true,
        }
    }
}

/// Resultado de uma operação de sumarização
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummarizationResult {
    pub summary_id: String,
    pub original_text: String,
    pub summary: String,
    pub method: SummarizationMethod,
    pub original_length: usize,
    pub summary_length: usize,
    pub compression_ratio: f32,
    pub language: String,
    pub metadata: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Informações de um sumário para listagem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryInfo {
    pub summary_id: String,
    pub method: SummarizationMethod,
    pub language: String,
    pub original_length: usize,
    pub summary_length: usize,
    pub compression_ratio: f32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

/// Parâmetros para sumarização de texto
#[derive(Debug, Clone)]
pub struct SummarizationParams {
    pub text: String,
    pub method: SummarizationMethod,
    pub max_length: Option<usize>,
    pub compression_ratio: Option<f32>,
    pub language: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Parâmetros para sumarização de contexto
#[derive(Debug, Clone)]
pub struct ContextSummarizationParams {
    pub context: String,
    pub method: SummarizationMethod,
    pub max_length: Option<usize>,
    pub compression_ratio: Option<f32>,
    pub language: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Erros de sumarização
#[derive(Debug, thiserror::Error)]
pub enum SummarizationError {
    #[error("Summarization method not supported: {method}")]
    UnsupportedMethod { method: String },

    #[error("Summarization method disabled: {method}")]
    MethodDisabled { method: String },

    #[error("Text too short for summarization: {length} characters")]
    TextTooShort { length: usize },

    #[error("Text too long for summarization: {length} characters")]
    TextTooLong { length: usize },

    #[error("Invalid compression ratio: {ratio} (must be between 0.1 and 0.9)")]
    InvalidCompressionRatio { ratio: f32 },

    #[error("Language not supported: {language}")]
    UnsupportedLanguage { language: String },

    #[error("External API error: {message}")]
    ExternalApiError { message: String },

    #[error("Summarization failed: {message}")]
    SummarizationFailed { message: String },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
}

impl From<String> for SummarizationError {
    fn from(message: String) -> Self {
        SummarizationError::ConfigurationError { message }
    }
}

impl From<crate::error::VectorizerError> for SummarizationError {
    fn from(error: crate::error::VectorizerError) -> Self {
        SummarizationError::SummarizationFailed {
            message: format!("Embedding error: {}", error),
        }
    }
}

/// Resultado de sumarização com possibilidade de erro
pub type SummarizationResultType = Result<SummarizationResult, SummarizationError>;
