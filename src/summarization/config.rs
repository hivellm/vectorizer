use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::summarization::types::{MethodConfig, LanguageConfig, MetadataConfig, SummarizationMethod};

/// Configuração completa do sistema de sumarização
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummarizationConfig {
    /// Habilitar sumarização automática
    pub enabled: bool,
    /// Sumarizar automaticamente durante indexação
    pub auto_summarize: bool,
    /// Nome da collection para armazenar sumários
    pub summary_collection: String,
    /// Método padrão de sumarização
    pub default_method: String,
    /// Configurações específicas por método
    pub methods: HashMap<String, MethodConfig>,
    /// Configurações por idioma
    pub languages: HashMap<String, LanguageConfig>,
    /// Configurações de metadados
    pub metadata: MetadataConfig,
}

impl Default for SummarizationConfig {
    fn default() -> Self {
        let mut methods = HashMap::new();
        methods.insert("extractive".to_string(), MethodConfig::default());
        methods.insert("keyword".to_string(), MethodConfig::default());
        methods.insert("sentence".to_string(), MethodConfig::default());
        
        let mut languages = HashMap::new();
        languages.insert("en".to_string(), LanguageConfig::default());
        languages.insert("pt".to_string(), LanguageConfig::default());
        languages.insert("es".to_string(), LanguageConfig::default());
        languages.insert("fr".to_string(), LanguageConfig::default());
        
        Self {
            enabled: false,
            auto_summarize: false,
            summary_collection: "summaries".to_string(),
            default_method: "extractive".to_string(),
            methods,
            languages,
            metadata: MetadataConfig::default(),
        }
    }
}

impl SummarizationConfig {
    /// Criar configuração a partir de valores YAML
    pub fn from_yaml(config: &serde_yaml::Value) -> Result<Self, String> {
        let mut summarization_config = SummarizationConfig::default();
        
        if let Some(summarization) = config.get("summarization") {
            if let Some(enabled) = summarization.get("enabled").and_then(|v| v.as_bool()) {
                summarization_config.enabled = enabled;
            }
            
            if let Some(auto_summarize) = summarization.get("auto_summarize").and_then(|v| v.as_bool()) {
                summarization_config.auto_summarize = auto_summarize;
            }
            
            if let Some(summary_collection) = summarization.get("summary_collection").and_then(|v| v.as_str()) {
                summarization_config.summary_collection = summary_collection.to_string();
            }
            
            if let Some(default_method) = summarization.get("default_method").and_then(|v| v.as_str()) {
                summarization_config.default_method = default_method.to_string();
            }
            
            // Configurações de métodos
            if let Some(methods) = summarization.get("methods") {
                for (method_name, method_config) in methods.as_mapping().unwrap() {
                    let method_name = method_name.as_str().unwrap();
                    let mut config = MethodConfig::default();
                    
                    if let Some(enabled) = method_config.get("enabled").and_then(|v| v.as_bool()) {
                        config.enabled = enabled;
                    }
                    
                    if let Some(compression_ratio) = method_config.get("compression_ratio").and_then(|v| v.as_f64()) {
                        config.compression_ratio = compression_ratio as f32;
                    }
                    
                    if let Some(max_sentences) = method_config.get("max_sentences").and_then(|v| v.as_u64()) {
                        config.max_sentences = Some(max_sentences as usize);
                    }
                    
                    if let Some(min_sentence_length) = method_config.get("min_sentence_length").and_then(|v| v.as_u64()) {
                        config.min_sentence_length = Some(min_sentence_length as usize);
                    }
                    
                    if let Some(max_keywords) = method_config.get("max_keywords").and_then(|v| v.as_u64()) {
                        config.max_keywords = Some(max_keywords as usize);
                    }
                    
                    if let Some(min_keyword_length) = method_config.get("min_keyword_length").and_then(|v| v.as_u64()) {
                        config.min_keyword_length = Some(min_keyword_length as usize);
                    }
                    
                    if let Some(use_tfidf) = method_config.get("use_tfidf").and_then(|v| v.as_bool()) {
                        config.use_tfidf = Some(use_tfidf);
                    }
                    
                    if let Some(use_stopwords) = method_config.get("use_stopwords").and_then(|v| v.as_bool()) {
                        config.use_stopwords = Some(use_stopwords);
                    }
                    
                    if let Some(use_position_weight) = method_config.get("use_position_weight").and_then(|v| v.as_bool()) {
                        config.use_position_weight = Some(use_position_weight);
                    }
                    
                    if let Some(language) = method_config.get("language").and_then(|v| v.as_str()) {
                        config.language = Some(language.to_string());
                    }
                    
                    if let Some(model) = method_config.get("model").and_then(|v| v.as_str()) {
                        config.model = Some(model.to_string());
                    }
                    
                    if let Some(api_key) = method_config.get("api_key").and_then(|v| v.as_str()) {
                        config.api_key = Some(api_key.to_string());
                    }
                    
                    if let Some(max_tokens) = method_config.get("max_tokens").and_then(|v| v.as_u64()) {
                        config.max_tokens = Some(max_tokens as usize);
                    }
                    
                    if let Some(temperature) = method_config.get("temperature").and_then(|v| v.as_f64()) {
                        config.temperature = Some(temperature as f32);
                    }
                    
                    summarization_config.methods.insert(method_name.to_string(), config);
                }
            }
            
            // Configurações de idiomas
            if let Some(languages) = summarization.get("languages") {
                for (lang_code, lang_config) in languages.as_mapping().unwrap() {
                    let lang_code = lang_code.as_str().unwrap();
                    let mut config = LanguageConfig::default();
                    
                    if let Some(stopwords) = lang_config.get("stopwords").and_then(|v| v.as_bool()) {
                        config.stopwords = stopwords;
                    }
                    
                    if let Some(stemming) = lang_config.get("stemming").and_then(|v| v.as_bool()) {
                        config.stemming = stemming;
                    }
                    
                    summarization_config.languages.insert(lang_code.to_string(), config);
                }
            }
            
            // Configurações de metadados
            if let Some(metadata) = summarization.get("metadata") {
                if let Some(include_original_id) = metadata.get("include_original_id").and_then(|v| v.as_bool()) {
                    summarization_config.metadata.include_original_id = include_original_id;
                }
                
                if let Some(include_file_path) = metadata.get("include_file_path").and_then(|v| v.as_bool()) {
                    summarization_config.metadata.include_file_path = include_file_path;
                }
                
                if let Some(include_timestamp) = metadata.get("include_timestamp").and_then(|v| v.as_bool()) {
                    summarization_config.metadata.include_timestamp = include_timestamp;
                }
                
                if let Some(include_method) = metadata.get("include_method").and_then(|v| v.as_bool()) {
                    summarization_config.metadata.include_method = include_method;
                }
                
                if let Some(include_compression_ratio) = metadata.get("include_compression_ratio").and_then(|v| v.as_bool()) {
                    summarization_config.metadata.include_compression_ratio = include_compression_ratio;
                }
            }
        }
        
        Ok(summarization_config)
    }
    
    /// Verificar se um método está habilitado
    pub fn is_method_enabled(&self, method: &SummarizationMethod) -> bool {
        let method_name = method.to_string();
        self.methods.get(&method_name)
            .map(|config| config.enabled)
            .unwrap_or(false)
    }
    
    /// Obter configuração de um método específico
    pub fn get_method_config(&self, method: &SummarizationMethod) -> Option<&MethodConfig> {
        let method_name = method.to_string();
        self.methods.get(&method_name)
    }
    
    /// Obter configuração de idioma
    pub fn get_language_config(&self, language: &str) -> Option<&LanguageConfig> {
        self.languages.get(language)
    }
    
    /// Validar configuração
    pub fn validate(&self) -> Result<(), String> {
        if self.summary_collection.is_empty() {
            return Err("Summary collection name cannot be empty".to_string());
        }
        
        if self.default_method.is_empty() {
            return Err("Default method cannot be empty".to_string());
        }
        
        // Validar método padrão
        if let Err(_) = self.default_method.parse::<SummarizationMethod>() {
            return Err(format!("Invalid default method: {}", self.default_method));
        }
        
        // Validar configurações de métodos
        for (method_name, config) in &self.methods {
            if config.compression_ratio < 0.1 || config.compression_ratio > 0.9 {
                return Err(format!("Invalid compression ratio for method {}: {}", method_name, config.compression_ratio));
            }
        }
        
        Ok(())
    }
}
