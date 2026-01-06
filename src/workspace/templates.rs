//! Configuration Templates
//!
//! Pre-configured templates for common use cases (RAG, Code Search, Documentation)

use serde::{Deserialize, Serialize};

/// A configuration template for common use cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigTemplate {
    /// Unique template identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of the template
    pub description: String,
    /// Icon/emoji for the template
    pub icon: String,
    /// Use case examples
    pub use_cases: Vec<String>,
    /// Default collection configurations
    pub collections: Vec<TemplateCollection>,
}

/// Collection configuration within a template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateCollection {
    /// Suffix to add to project name for collection name
    pub name_suffix: String,
    /// Description of what this collection contains
    pub description: String,
    /// File patterns to include
    pub include_patterns: Vec<String>,
    /// File patterns to exclude
    pub exclude_patterns: Vec<String>,
    /// Content type (source, docs, config, etc.)
    pub content_type: String,
    /// Recommended settings
    pub settings: CollectionSettings,
}

/// Collection settings for a template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionSettings {
    /// Chunk size in characters
    pub chunk_size: usize,
    /// Chunk overlap in characters
    pub chunk_overlap: usize,
    /// Embedding model to use
    pub embedding_model: String,
}

impl Default for CollectionSettings {
    fn default() -> Self {
        Self {
            chunk_size: 512,
            chunk_overlap: 50,
            embedding_model: "default".to_string(),
        }
    }
}

/// Get all available configuration templates
pub fn get_templates() -> Vec<ConfigTemplate> {
    vec![
        get_rag_template(),
        get_code_search_template(),
        get_documentation_template(),
        get_custom_template(),
    ]
}

/// Get a specific template by ID
pub fn get_template_by_id(id: &str) -> Option<ConfigTemplate> {
    get_templates().into_iter().find(|t| t.id == id)
}

/// RAG (Retrieval-Augmented Generation) template
fn get_rag_template() -> ConfigTemplate {
    ConfigTemplate {
        id: "rag".to_string(),
        name: "RAG (Retrieval-Augmented Generation)".to_string(),
        description: "Optimized for document retrieval and LLM integration. Perfect for building AI assistants and chatbots.".to_string(),
        icon: "🤖".to_string(),
        use_cases: vec![
            "AI chatbots and assistants".to_string(),
            "Question answering systems".to_string(),
            "Document-based AI applications".to_string(),
            "Knowledge base search".to_string(),
        ],
        collections: vec![
            TemplateCollection {
                name_suffix: "documents".to_string(),
                description: "Main document collection for RAG retrieval".to_string(),
                include_patterns: vec![
                    "**/*.md".to_string(),
                    "**/*.txt".to_string(),
                    "**/*.rst".to_string(),
                    "**/*.html".to_string(),
                ],
                exclude_patterns: vec![
                    "**/node_modules/**".to_string(),
                    "**/target/**".to_string(),
                    "**/.git/**".to_string(),
                    "**/dist/**".to_string(),
                    "**/build/**".to_string(),
                ],
                content_type: "documentation".to_string(),
                settings: CollectionSettings {
                    chunk_size: 512,
                    chunk_overlap: 50,
                    embedding_model: "default".to_string(),
                },
            },
            TemplateCollection {
                name_suffix: "knowledge".to_string(),
                description: "Structured knowledge (JSON, YAML configs)".to_string(),
                include_patterns: vec![
                    "**/*.json".to_string(),
                    "**/*.yaml".to_string(),
                    "**/*.yml".to_string(),
                    "**/*.toml".to_string(),
                ],
                exclude_patterns: vec![
                    "**/node_modules/**".to_string(),
                    "**/package-lock.json".to_string(),
                    "**/yarn.lock".to_string(),
                    "**/.git/**".to_string(),
                ],
                content_type: "config".to_string(),
                settings: CollectionSettings {
                    chunk_size: 256,
                    chunk_overlap: 25,
                    embedding_model: "default".to_string(),
                },
            },
        ],
    }
}

/// Code Search template
fn get_code_search_template() -> ConfigTemplate {
    ConfigTemplate {
        id: "code_search".to_string(),
        name: "Code Search".to_string(),
        description: "Semantic search across source code. Find implementations, functions, and patterns quickly.".to_string(),
        icon: "💻".to_string(),
        use_cases: vec![
            "Codebase exploration".to_string(),
            "Finding similar implementations".to_string(),
            "Code review assistance".to_string(),
            "Technical documentation".to_string(),
        ],
        collections: vec![
            TemplateCollection {
                name_suffix: "source".to_string(),
                description: "Source code files".to_string(),
                include_patterns: vec![
                    "**/*.rs".to_string(),
                    "**/*.py".to_string(),
                    "**/*.ts".to_string(),
                    "**/*.tsx".to_string(),
                    "**/*.js".to_string(),
                    "**/*.jsx".to_string(),
                    "**/*.go".to_string(),
                    "**/*.java".to_string(),
                    "**/*.cpp".to_string(),
                    "**/*.c".to_string(),
                    "**/*.h".to_string(),
                    "**/*.hpp".to_string(),
                    "**/*.cs".to_string(),
                    "**/*.rb".to_string(),
                    "**/*.php".to_string(),
                    "**/*.swift".to_string(),
                    "**/*.kt".to_string(),
                ],
                exclude_patterns: vec![
                    "**/node_modules/**".to_string(),
                    "**/target/**".to_string(),
                    "**/dist/**".to_string(),
                    "**/build/**".to_string(),
                    "**/.git/**".to_string(),
                    "**/__pycache__/**".to_string(),
                    "**/.venv/**".to_string(),
                    "**/vendor/**".to_string(),
                    "**/bin/**".to_string(),
                    "**/obj/**".to_string(),
                ],
                content_type: "source".to_string(),
                settings: CollectionSettings {
                    chunk_size: 1024,
                    chunk_overlap: 100,
                    embedding_model: "default".to_string(),
                },
            },
            TemplateCollection {
                name_suffix: "docs".to_string(),
                description: "Code documentation and READMEs".to_string(),
                include_patterns: vec![
                    "**/*.md".to_string(),
                    "**/README*".to_string(),
                    "**/CHANGELOG*".to_string(),
                    "**/CONTRIBUTING*".to_string(),
                ],
                exclude_patterns: vec![
                    "**/node_modules/**".to_string(),
                    "**/target/**".to_string(),
                    "**/.git/**".to_string(),
                ],
                content_type: "documentation".to_string(),
                settings: CollectionSettings {
                    chunk_size: 512,
                    chunk_overlap: 50,
                    embedding_model: "default".to_string(),
                },
            },
            TemplateCollection {
                name_suffix: "config".to_string(),
                description: "Configuration and build files".to_string(),
                include_patterns: vec![
                    "**/Cargo.toml".to_string(),
                    "**/package.json".to_string(),
                    "**/tsconfig.json".to_string(),
                    "**/pyproject.toml".to_string(),
                    "**/go.mod".to_string(),
                    "**/pom.xml".to_string(),
                    "**/build.gradle".to_string(),
                    "**/*.yml".to_string(),
                    "**/*.yaml".to_string(),
                ],
                exclude_patterns: vec![
                    "**/node_modules/**".to_string(),
                    "**/package-lock.json".to_string(),
                    "**/.git/**".to_string(),
                ],
                content_type: "config".to_string(),
                settings: CollectionSettings {
                    chunk_size: 256,
                    chunk_overlap: 25,
                    embedding_model: "default".to_string(),
                },
            },
        ],
    }
}

/// Documentation template
fn get_documentation_template() -> ConfigTemplate {
    ConfigTemplate {
        id: "documentation".to_string(),
        name: "Documentation".to_string(),
        description:
            "Index and search documentation files. Great for wikis, guides, and knowledge bases."
                .to_string(),
        icon: "📚".to_string(),
        use_cases: vec![
            "Wiki search".to_string(),
            "Technical documentation".to_string(),
            "User guides".to_string(),
            "API documentation".to_string(),
        ],
        collections: vec![TemplateCollection {
            name_suffix: "docs".to_string(),
            description: "All documentation files".to_string(),
            include_patterns: vec![
                "**/*.md".to_string(),
                "**/*.txt".to_string(),
                "**/*.rst".to_string(),
                "**/*.adoc".to_string(),
                "**/*.org".to_string(),
                "**/docs/**/*".to_string(),
            ],
            exclude_patterns: vec!["**/node_modules/**".to_string(), "**/.git/**".to_string()],
            content_type: "documentation".to_string(),
            settings: CollectionSettings {
                chunk_size: 512,
                chunk_overlap: 50,
                embedding_model: "default".to_string(),
            },
        }],
    }
}

/// Custom template (minimal configuration)
fn get_custom_template() -> ConfigTemplate {
    ConfigTemplate {
        id: "custom".to_string(),
        name: "Custom".to_string(),
        description:
            "Full control over your configuration. Start from scratch and customize everything."
                .to_string(),
        icon: "⚙️".to_string(),
        use_cases: vec![
            "Custom file types".to_string(),
            "Specific project structures".to_string(),
            "Advanced configurations".to_string(),
        ],
        collections: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_templates() {
        let templates = get_templates();
        assert_eq!(templates.len(), 4);
        assert!(templates.iter().any(|t| t.id == "rag"));
        assert!(templates.iter().any(|t| t.id == "code_search"));
        assert!(templates.iter().any(|t| t.id == "documentation"));
        assert!(templates.iter().any(|t| t.id == "custom"));
    }

    #[test]
    fn test_get_template_by_id() {
        let rag = get_template_by_id("rag");
        assert!(rag.is_some());
        assert_eq!(rag.unwrap().name, "RAG (Retrieval-Augmented Generation)");

        let nonexistent = get_template_by_id("nonexistent");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_rag_template_has_collections() {
        let rag = get_rag_template();
        assert!(!rag.collections.is_empty());
        assert!(rag.collections.iter().any(|c| c.name_suffix == "documents"));
    }

    #[test]
    fn test_code_search_template_has_collections() {
        let code_search = get_code_search_template();
        assert!(!code_search.collections.is_empty());
        assert!(
            code_search
                .collections
                .iter()
                .any(|c| c.name_suffix == "source")
        );
    }
}
