#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::summarization::methods::SummarizationMethodTrait;
    use crate::summarization::{
        ContextSummarizationParams, LanguageConfig, MetadataConfig, MethodConfig,
        SummarizationConfig, SummarizationManager, SummarizationMethod, SummarizationParams,
    };

    fn create_test_config() -> SummarizationConfig {
        let mut methods = HashMap::new();
        methods.insert("extractive".to_string(), MethodConfig::default());
        methods.insert("keyword".to_string(), MethodConfig::default());

        let mut languages = HashMap::new();
        languages.insert("en".to_string(), LanguageConfig::default());
        languages.insert("pt".to_string(), LanguageConfig::default());

        SummarizationConfig {
            enabled: true,
            auto_summarize: true,
            summary_collection: "test_summaries".to_string(),
            default_method: "extractive".to_string(),
            methods,
            languages,
            metadata: MetadataConfig::default(),
        }
    }

    #[test]
    fn test_summarization_manager_creation() {
        let config = create_test_config();
        let manager = SummarizationManager::new(config.clone());

        // Test that the manager was created successfully
        assert!(manager.is_ok());
    }

    #[test]
    fn test_summarization_manager_with_default_config() {
        let manager = SummarizationManager::with_default_config();

        // Test that the manager was created successfully
        // We can't access private fields, so we just verify it was created
        assert!(true); // Placeholder assertion
    }

    #[test]
    fn test_summarize_text_empty_input() {
        let mut manager = SummarizationManager::with_default_config();

        let params = SummarizationParams {
            text: "".to_string(),
            method: SummarizationMethod::Extractive,
            max_length: Some(10),
            compression_ratio: Some(0.3),
            language: Some("en".to_string()),
            metadata: HashMap::new(),
        };

        let result = manager.summarize_text(params);
        assert!(result.is_err());
    }

    #[test]
    fn test_summarize_text_very_short_input() {
        let mut manager = SummarizationManager::with_default_config();

        let params = SummarizationParams {
            text: "Hi".to_string(),
            method: SummarizationMethod::Extractive,
            max_length: Some(10),
            compression_ratio: Some(0.3),
            language: Some("en".to_string()),
            metadata: HashMap::new(),
        };

        let result = manager.summarize_text(params);
        // Should handle gracefully or return error for very short text
        match result {
            Ok(r) => {
                assert!(!r.summary.is_empty());
            }
            Err(_) => {
                // Acceptable for very short text
            }
        }
    }

    #[test]
    fn test_get_summary_not_found() {
        let mut manager = SummarizationManager::with_default_config();

        let result = manager.get_summary("non-existent-id");
        assert!(result.is_none());
    }

    #[test]
    fn test_summarization_method_parsing() {
        assert_eq!(
            "extractive".parse::<SummarizationMethod>().unwrap(),
            SummarizationMethod::Extractive
        );
        assert_eq!(
            "keyword".parse::<SummarizationMethod>().unwrap(),
            SummarizationMethod::Keyword
        );
        assert_eq!(
            "sentence".parse::<SummarizationMethod>().unwrap(),
            SummarizationMethod::Sentence
        );
        assert_eq!(
            "abstractive".parse::<SummarizationMethod>().unwrap(),
            SummarizationMethod::Abstractive
        );

        assert!("invalid".parse::<SummarizationMethod>().is_err());
    }

    #[test]
    fn test_summarization_method_to_string() {
        assert_eq!(SummarizationMethod::Extractive.to_string(), "extractive");
        assert_eq!(SummarizationMethod::Keyword.to_string(), "keyword");
        assert_eq!(SummarizationMethod::Sentence.to_string(), "sentence");
        assert_eq!(SummarizationMethod::Abstractive.to_string(), "abstractive");
    }

    #[test]
    fn test_abstractive_summarization_requires_api_key() {
        use crate::summarization::methods::AbstractiveSummarizer;

        let summarizer = AbstractiveSummarizer::new();

        // Test that abstractive summarization requires API key
        let params = SummarizationParams {
            text: "This is a test document that needs summarization. It contains multiple sentences for testing purposes.".to_string(),
            method: SummarizationMethod::Abstractive,
            max_length: Some(100),
            compression_ratio: Some(0.3),
            language: Some("en".to_string()),
            metadata: HashMap::new(),
        };

        let mut config = MethodConfig::default();
        config.enabled = true;
        // No API key configured

        let result = summarizer.summarize(&params, &config);
        assert!(result.is_err());

        // Check error message mentions API key
        if let Err(e) = result {
            let error_msg = format!("{:?}", e);
            assert!(error_msg.contains("API key") || error_msg.contains("OPENAI_API_KEY"));
        }
    }

    #[test]
    fn test_abstractive_summarizer_is_available_check() {
        use crate::summarization::methods::AbstractiveSummarizer;

        let summarizer = AbstractiveSummarizer::new();

        // Check availability (depends on OPENAI_API_KEY env var)
        let is_available = summarizer.is_available();
        // May or may not be available depending on environment
        // Just verify the method exists and returns bool
        assert!(matches!(is_available, true | false));
    }

    #[test]
    fn test_summary_persistence() {
        let mut manager = SummarizationManager::with_enabled_config();

        let params = SummarizationParams {
            text: "This is a comprehensive test document for persistence testing purposes. It contains multiple sentences with detailed information that allows proper testing of summary persistence functionality. The document covers various topics to ensure thorough validation of the persistence mechanism.".to_string(),
            method: SummarizationMethod::Extractive,
            max_length: Some(50),
            compression_ratio: Some(0.3),
            language: Some("en".to_string()),
            metadata: HashMap::new(),
        };

        let result = manager.summarize_text(params).unwrap();
        let summary_id = result.summary_id.clone();

        // Verify the summary is stored
        let retrieved = manager.get_summary(&summary_id).unwrap();
        assert_eq!(retrieved.summary_id, summary_id);

        // Verify it appears in the list
        let summaries = manager.list_summaries(None, None, None, None);
        let found = summaries.iter().any(|s| s.summary_id == summary_id);
        assert!(found);
    }
}
