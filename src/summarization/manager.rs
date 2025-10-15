use crate::summarization::{
    config::SummarizationConfig,
    types::*,
    methods::{SummarizationMethodTrait, ExtractiveSummarizer, KeywordSummarizer, SentenceSummarizer, AbstractiveSummarizer},
};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

/// Gerenciador principal do sistema de sumarização
pub struct SummarizationManager {
    config: SummarizationConfig,
    extractive: ExtractiveSummarizer,
    keyword: KeywordSummarizer,
    sentence: SentenceSummarizer,
    abstractive: AbstractiveSummarizer,
    pub summaries: HashMap<String, SummarizationResult>, // Cache de sumários
}

impl SummarizationManager {
    /// Criar novo gerenciador de sumarização
    pub fn new(config: SummarizationConfig) -> Result<Self, SummarizationError> {
        config.validate()?;
        
        Ok(Self {
            config,
            extractive: ExtractiveSummarizer::new(),
            keyword: KeywordSummarizer::new(),
            sentence: SentenceSummarizer::new(),
            abstractive: AbstractiveSummarizer::new(),
            summaries: HashMap::new(),
        })
    }
    
    /// Criar gerenciador com configuração padrão
    pub fn with_default_config() -> Self {
        Self::new(SummarizationConfig::default()).unwrap()
    }

    /// Create with enabled summarization config for testing
    pub fn with_enabled_config() -> Self {
        let mut config = SummarizationConfig::default();
        config.enabled = true;
        config.auto_summarize = true;
        Self::new(config).unwrap()
    }

    /// Obter referência para a configuração atual
    pub fn get_config(&self) -> &SummarizationConfig {
        &self.config
    }
    
    /// Sumarizar texto
    pub fn summarize_text(&mut self, params: SummarizationParams) -> Result<SummarizationResult, SummarizationError> {
        if !self.config.enabled {
            return Err(SummarizationError::ConfigurationError { 
                message: "Summarization is disabled".to_string() 
            });
        }
        
        // Validar parâmetros
        self.validate_params(&params)?;
        
        // Obter configuração do método
        let method_config = self.config.get_method_config(&params.method)
            .ok_or_else(|| SummarizationError::UnsupportedMethod { 
                method: params.method.to_string() 
            })?;
        
        if !method_config.enabled {
            return Err(SummarizationError::MethodDisabled { 
                method: params.method.to_string() 
            });
        }
        
        // Executar sumarização
        let summary_text = match &params.method {
            SummarizationMethod::Extractive => {
                self.extractive.summarize(&params, method_config)?
            },
            SummarizationMethod::Keyword => {
                self.keyword.summarize(&params, method_config)?
            },
            SummarizationMethod::Sentence => {
                self.sentence.summarize(&params, method_config)?
            },
            SummarizationMethod::Abstractive => {
                self.abstractive.summarize(&params, method_config)?
            },
        };
        
        // Criar resultado
        let summary_id = Uuid::new_v4().to_string();
        let original_length = params.text.len();
        let summary_length = summary_text.len();
        let compression_ratio = summary_length as f32 / original_length as f32;
        let language = params.language.clone().unwrap_or_else(|| "en".to_string());
        
        let mut metadata = params.metadata.clone();
        self.add_metadata(&mut metadata, &params, &summary_id, compression_ratio);
        
        let result = SummarizationResult {
            summary_id: summary_id.clone(),
            original_text: params.text,
            summary: summary_text,
            method: params.method,
            original_length,
            summary_length,
            compression_ratio,
            language,
            metadata,
            created_at: Utc::now(),
        };
        
        // Armazenar no cache
        self.summaries.insert(summary_id.clone(), result.clone());
        
        Ok(result)
    }
    
    /// Sumarizar contexto
    pub fn summarize_context(&mut self, params: ContextSummarizationParams) -> Result<SummarizationResult, SummarizationError> {
        let text_params = SummarizationParams {
            text: params.context,
            method: params.method,
            max_length: params.max_length,
            compression_ratio: params.compression_ratio,
            language: params.language,
            metadata: params.metadata,
        };
        
        self.summarize_text(text_params)
    }
    
    /// Obter sumário por ID
    pub fn get_summary(&self, summary_id: &str) -> Option<&SummarizationResult> {
        self.summaries.get(summary_id)
    }
    
    /// Listar sumários com filtros
    pub fn list_summaries(&self, method: Option<&str>, language: Option<&str>, limit: Option<usize>, offset: Option<usize>) -> Vec<SummaryInfo> {
        let mut summaries: Vec<&SummarizationResult> = self.summaries.values().collect();
        
        // Aplicar filtros
        if let Some(method_filter) = method {
            summaries.retain(|s| s.method.to_string() == method_filter);
        }
        
        if let Some(lang_filter) = language {
            summaries.retain(|s| s.language == lang_filter);
        }
        
        // Ordenar por data de criação (mais recente primeiro)
        summaries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        // Aplicar paginação
        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(100);
        
        summaries
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|s| SummaryInfo {
                summary_id: s.summary_id.clone(),
                method: s.method.clone(),
                language: s.language.clone(),
                original_length: s.original_length,
                summary_length: s.summary_length,
                compression_ratio: s.compression_ratio,
                created_at: s.created_at,
                metadata: s.metadata.clone(),
            })
            .collect()
    }
    
    /// Verificar se sumarização automática está habilitada
    pub fn is_auto_summarization_enabled(&self) -> bool {
        self.config.enabled && self.config.auto_summarize
    }
    
    /// Obter nome da collection de sumários
    pub fn get_summary_collection_name(&self) -> &str {
        &self.config.summary_collection
    }
    
    /// Obter método padrão
    pub fn get_default_method(&self) -> SummarizationMethod {
        self.config.default_method.parse().unwrap_or(SummarizationMethod::Extractive)
    }
    
    /// Validar parâmetros de sumarização
    fn validate_params(&self, params: &SummarizationParams) -> Result<(), SummarizationError> {
        if params.text.len() < 10 {
            return Err(SummarizationError::TextTooShort { length: params.text.len() });
        }
        
        if params.text.len() > 100000 {
            return Err(SummarizationError::TextTooLong { length: params.text.len() });
        }
        
        if let Some(ratio) = params.compression_ratio {
            if ratio < 0.1 || ratio > 0.9 {
                return Err(SummarizationError::InvalidCompressionRatio { ratio });
            }
        }
        
        if let Some(language) = &params.language {
            if !self.config.languages.contains_key(language) {
                return Err(SummarizationError::UnsupportedLanguage { 
                    language: language.clone() 
                });
            }
        }
        
        Ok(())
    }
    
    /// Adicionar metadados ao sumário
    fn add_metadata(&self, metadata: &mut HashMap<String, String>, params: &SummarizationParams, summary_id: &str, compression_ratio: f32) {
        if self.config.metadata.include_timestamp {
            metadata.insert("created_at".to_string(), Utc::now().to_rfc3339());
        }
        
        if self.config.metadata.include_method {
            metadata.insert("method".to_string(), params.method.to_string());
        }
        
        if self.config.metadata.include_compression_ratio {
            metadata.insert("compression_ratio".to_string(), compression_ratio.to_string());
        }
        
        if let Some(language) = &params.language {
            metadata.insert("language".to_string(), language.clone());
        }
        
        // Adicionar flag de sumário
        metadata.insert("is_summary".to_string(), "true".to_string());
        metadata.insert("summary_id".to_string(), summary_id.to_string());
    }
    
    /// Criar metadados para sumário automático durante indexação
    pub fn create_auto_summary_metadata(&self, original_id: &str, file_path: Option<&str>) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        
        metadata.insert("is_summary".to_string(), "true".to_string());
        metadata.insert("auto_generated".to_string(), "true".to_string());
        
        if self.config.metadata.include_original_id {
            metadata.insert("original_id".to_string(), original_id.to_string());
        }
        
        if self.config.metadata.include_file_path {
            if let Some(path) = file_path {
                metadata.insert("original_file_path".to_string(), path.to_string());
            }
        }
        
        if self.config.metadata.include_timestamp {
            metadata.insert("created_at".to_string(), Utc::now().to_rfc3339());
        }
        
        if self.config.metadata.include_method {
            metadata.insert("method".to_string(), self.config.default_method.clone());
        }
        
        metadata
    }
    
    /// Sumarizar texto automaticamente durante indexação
    pub fn auto_summarize(&mut self, text: &str, original_id: &str, file_path: Option<&str>) -> Result<SummarizationResult, SummarizationError> {
        if !self.is_auto_summarization_enabled() {
            return Err(SummarizationError::ConfigurationError { 
                message: "Auto summarization is disabled".to_string() 
            });
        }
        
        let method = self.get_default_method();
        let mut metadata = self.create_auto_summary_metadata(original_id, file_path);
        
        let params = SummarizationParams {
            text: text.to_string(),
            method,
            max_length: None,
            compression_ratio: None,
            language: Some("en".to_string()),
            metadata,
        };
        
        self.summarize_text(params)
    }
    
    /// Obter estatísticas de sumarização
    pub fn get_stats(&self) -> SummarizationStats {
        let total_summaries = self.summaries.len();
        let mut method_counts: HashMap<String, usize> = HashMap::new();
        let mut language_counts: HashMap<String, usize> = HashMap::new();
        let mut total_compression_ratio = 0.0;
        
        for summary in self.summaries.values() {
            *method_counts.entry(summary.method.to_string()).or_insert(0) += 1;
            *language_counts.entry(summary.language.clone()).or_insert(0) += 1;
            total_compression_ratio += summary.compression_ratio;
        }
        
        let avg_compression_ratio = if total_summaries > 0 {
            total_compression_ratio / total_summaries as f32
        } else {
            0.0
        };
        
        SummarizationStats {
            total_summaries,
            method_counts,
            language_counts,
            average_compression_ratio: avg_compression_ratio,
            auto_summarization_enabled: self.is_auto_summarization_enabled(),
        }
    }
}

/// Estatísticas de sumarização
#[derive(Debug, Clone)]
pub struct SummarizationStats {
    pub total_summaries: usize,
    pub method_counts: HashMap<String, usize>,
    pub language_counts: HashMap<String, usize>,
    pub average_compression_ratio: f32,
    pub auto_summarization_enabled: bool,
}
