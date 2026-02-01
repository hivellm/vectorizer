//! Setup configuration module
//!
//! Provides structures and functions for applying the initial setup configuration,
//! shared between the REST API and CLI.

use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::workspace::project_analyzer::ProjectAnalysis;

/// Apply configuration request (simplified workspace config for setup)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyConfigRequest {
    /// Projects to add
    pub projects: Vec<SetupProject>,
    /// Global settings
    pub global_settings: Option<SetupGlobalSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupProject {
    pub name: String,
    pub path: String,
    pub description: String,
    pub collections: Vec<SetupCollection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupCollection {
    pub name: String,
    pub description: String,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    /// Enable automatic graph relationship discovery (GraphRAG)
    #[serde(default)]
    pub enable_graph: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupGlobalSettings {
    pub file_watcher: Option<SetupFileWatcherSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupFileWatcherSettings {
    pub auto_discovery: Option<bool>,
    pub enable_auto_update: Option<bool>,
    pub hot_reload: Option<bool>,
    pub watch_paths: Option<Vec<String>>,
}

impl ApplyConfigRequest {
    /// Create a setup configuration from a project analysis result
    pub fn from_analysis(analysis: &ProjectAnalysis) -> Self {
        let collections = analysis
            .suggested_collections
            .iter()
            .map(|c| SetupCollection {
            name: c.name.clone(),
            description: c.description.clone(),
            include_patterns: c.include_patterns.clone(),
            exclude_patterns: c.exclude_patterns.clone(),
                enable_graph: None, // Graph relationships disabled by default
            })
            .collect();

        let project = SetupProject {
            name: analysis.project_name.clone(),
            path: analysis.project_path.clone(),
            description: format!(
                "{} project with {} languages detected",
                analysis
                    .project_types
                    .first()
                    .map(|t| format!("{:?}", t))
                    .unwrap_or_else(|| "Mixed".to_string()),
                analysis.languages.len()
            ),
            collections,
        };

        ApplyConfigRequest {
            projects: vec![project],
            global_settings: None,
        }
    }
}

/// Generate workspace.yml content from the setup configuration
pub fn generate_workspace_yaml(
    config: &ApplyConfigRequest,
) -> Result<String, Box<dyn std::error::Error>> {
    // Build workspace.yml content
    let mut projects_yaml: Vec<Value> = Vec::new();

    for project in &config.projects {
        let mut collections_yaml: Vec<Value> = Vec::new();

        for collection in &project.collections {
            let mut collection_json = json!({
                "name": collection.name,
                "description": collection.description,
                "include_patterns": collection.include_patterns,
                "exclude_patterns": collection.exclude_patterns,
            });

            // Add enable_graph if set to true
            if collection.enable_graph == Some(true) {
                collection_json["enable_graph"] = json!(true);
            }

            collections_yaml.push(collection_json);
        }

        projects_yaml.push(json!({
            "name": project.name,
            "path": project.path,
            "description": project.description,
            "collections": collections_yaml,
        }));
    }

    // Build global settings
    let mut global_settings_yaml = json!({
        "file_watcher": {
            "auto_discovery": true,
            "enable_auto_update": true,
            "hot_reload": true,
            "watch_paths": [],
            "exclude_patterns": []
        }
    });

    if let Some(ref global) = config.global_settings {
        if let Some(ref fw) = global.file_watcher {
            if let Some(auto_discovery) = fw.auto_discovery {
                global_settings_yaml["file_watcher"]["auto_discovery"] = json!(auto_discovery);
            }
            if let Some(enable_auto_update) = fw.enable_auto_update {
                global_settings_yaml["file_watcher"]["enable_auto_update"] =
                    json!(enable_auto_update);
            }
            if let Some(hot_reload) = fw.hot_reload {
                global_settings_yaml["file_watcher"]["hot_reload"] = json!(hot_reload);
            }
            if let Some(ref watch_paths) = fw.watch_paths {
                global_settings_yaml["file_watcher"]["watch_paths"] = json!(watch_paths);
            }
        }
    }

    // Construct the full workspace config
    let workspace_config = json!({
        "global_settings": global_settings_yaml,
        "projects": projects_yaml,
    });

    // Convert to YAML
    let yaml_content = serde_yaml::to_string(&workspace_config)?;

    // Add header comment
    let yaml_with_header = format!(
        "# Vectorizer Workspace Configuration\n\
         # Generated by Setup Wizard on {}\n\n{}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        yaml_content
    );

    Ok(yaml_with_header)
}

/// Write the workspace configuration to a file
pub fn write_workspace_config(
    config: &ApplyConfigRequest,
    path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = generate_workspace_yaml(config)?;
    std::fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::workspace::project_analyzer::{
        DirectoryStats, ProgrammingLanguage, ProjectAnalysis, ProjectType, SuggestedCollection,
    };

    #[test]
    fn test_from_analysis() {
        let analysis = ProjectAnalysis {
            project_name: "test-project".to_string(),
            project_path: "/tmp/test-project".to_string(),
            project_types: vec![ProjectType::Rust],
            languages: vec![ProgrammingLanguage::Rust],
            frameworks: vec![],
            suggested_collections: vec![SuggestedCollection {
                    name: "source".to_string(),
                    description: "Source code".to_string(),
                    include_patterns: vec!["src/**/*.rs".to_string()],
                    exclude_patterns: vec![],
                    content_type: "rust".to_string(),
                    estimated_file_count: 10,
            }],
            statistics: DirectoryStats {
                total_files: 10,
                total_directories: 2,
                total_size_bytes: 1000,
                files_by_extension: HashMap::new(),
                has_git: true,
                has_docs: true,
            },
        };

        let config = ApplyConfigRequest::from_analysis(&analysis);
        
        assert_eq!(config.projects.len(), 1);
        let project = &config.projects[0];
        assert_eq!(project.name, "test-project");
        assert_eq!(project.path, "/tmp/test-project");
        assert_eq!(project.collections.len(), 1);
        assert_eq!(project.collections[0].name, "source");
    }
}
