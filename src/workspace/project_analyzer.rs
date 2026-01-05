//! Project Analyzer Module
//!
//! Analyzes project directories to detect project types, languages,
//! and generate appropriate collection configurations for the setup wizard.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Result of analyzing a project directory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAnalysis {
    /// Detected project type(s)
    pub project_types: Vec<ProjectType>,
    /// Detected programming languages
    pub languages: Vec<ProgrammingLanguage>,
    /// Detected frameworks
    pub frameworks: Vec<String>,
    /// Suggested collections based on detection
    pub suggested_collections: Vec<SuggestedCollection>,
    /// Directory statistics
    pub statistics: DirectoryStats,
    /// Project name (derived from directory name)
    pub project_name: String,
    /// Full path to the project
    pub project_path: String,
}

/// Types of projects that can be detected
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ProjectType {
    /// Rust project (Cargo.toml)
    Rust,
    /// TypeScript/JavaScript project (package.json with typescript)
    TypeScript,
    /// JavaScript project (package.json without typescript)
    JavaScript,
    /// Python project (pyproject.toml, setup.py, requirements.txt)
    Python,
    /// Go project (go.mod)
    Go,
    /// C/C++ project (CMakeLists.txt, Makefile with .c/.cpp)
    Cpp,
    /// Java project (pom.xml, build.gradle)
    Java,
    /// C# / .NET project (.csproj, .sln)
    CSharp,
    /// Documentation project (primarily .md files)
    Documentation,
    /// Web frontend (React, Vue, Angular, etc.)
    WebFrontend,
    /// Mixed/Unknown project type
    Mixed,
}

/// Programming languages detected in project
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ProgrammingLanguage {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    Cpp,
    C,
    Java,
    Kotlin,
    CSharp,
    Ruby,
    Php,
    Swift,
    Markdown,
    Html,
    Css,
    Yaml,
    Json,
    Toml,
    Shell,
}

/// A suggested collection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedCollection {
    /// Collection name
    pub name: String,
    /// Collection description
    pub description: String,
    /// File patterns to include
    pub include_patterns: Vec<String>,
    /// File patterns to exclude
    pub exclude_patterns: Vec<String>,
    /// Primary language/content type
    pub content_type: String,
    /// Estimated file count
    pub estimated_file_count: usize,
}

/// Statistics about the analyzed directory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryStats {
    /// Total number of files
    pub total_files: usize,
    /// Total number of directories
    pub total_directories: usize,
    /// Total size in bytes
    pub total_size_bytes: u64,
    /// File count by extension
    pub files_by_extension: HashMap<String, usize>,
    /// Has .git directory
    pub has_git: bool,
    /// Has documentation directory
    pub has_docs: bool,
}

/// Project markers - files that indicate project type
struct ProjectMarker {
    filename: &'static str,
    project_type: ProjectType,
    language: Option<ProgrammingLanguage>,
}

const PROJECT_MARKERS: &[ProjectMarker] = &[
    ProjectMarker {
        filename: "Cargo.toml",
        project_type: ProjectType::Rust,
        language: Some(ProgrammingLanguage::Rust),
    },
    ProjectMarker {
        filename: "package.json",
        project_type: ProjectType::JavaScript,
        language: Some(ProgrammingLanguage::JavaScript),
    },
    ProjectMarker {
        filename: "tsconfig.json",
        project_type: ProjectType::TypeScript,
        language: Some(ProgrammingLanguage::TypeScript),
    },
    ProjectMarker {
        filename: "pyproject.toml",
        project_type: ProjectType::Python,
        language: Some(ProgrammingLanguage::Python),
    },
    ProjectMarker {
        filename: "requirements.txt",
        project_type: ProjectType::Python,
        language: Some(ProgrammingLanguage::Python),
    },
    ProjectMarker {
        filename: "setup.py",
        project_type: ProjectType::Python,
        language: Some(ProgrammingLanguage::Python),
    },
    ProjectMarker {
        filename: "go.mod",
        project_type: ProjectType::Go,
        language: Some(ProgrammingLanguage::Go),
    },
    ProjectMarker {
        filename: "CMakeLists.txt",
        project_type: ProjectType::Cpp,
        language: Some(ProgrammingLanguage::Cpp),
    },
    ProjectMarker {
        filename: "pom.xml",
        project_type: ProjectType::Java,
        language: Some(ProgrammingLanguage::Java),
    },
    ProjectMarker {
        filename: "build.gradle",
        project_type: ProjectType::Java,
        language: Some(ProgrammingLanguage::Java),
    },
    ProjectMarker {
        filename: "build.gradle.kts",
        project_type: ProjectType::Java,
        language: Some(ProgrammingLanguage::Kotlin),
    },
];

/// Templates for collection suggestions based on project type
fn get_collection_templates(project_type: &ProjectType) -> Vec<SuggestedCollection> {
    match project_type {
        ProjectType::Rust => vec![
            SuggestedCollection {
                name: "source".to_string(),
                description: "Rust source code".to_string(),
                include_patterns: vec!["src/**/*.rs".to_string(), "**/*.toml".to_string()],
                exclude_patterns: vec!["target/**".to_string()],
                content_type: "rust".to_string(),
                estimated_file_count: 0,
            },
            SuggestedCollection {
                name: "docs".to_string(),
                description: "Documentation".to_string(),
                include_patterns: vec!["**/*.md".to_string(), "docs/**/*".to_string()],
                exclude_patterns: vec!["target/**".to_string()],
                content_type: "documentation".to_string(),
                estimated_file_count: 0,
            },
        ],
        ProjectType::TypeScript | ProjectType::JavaScript => vec![
            SuggestedCollection {
                name: "source".to_string(),
                description: "TypeScript/JavaScript source code".to_string(),
                include_patterns: vec![
                    "src/**/*.ts".to_string(),
                    "src/**/*.tsx".to_string(),
                    "src/**/*.js".to_string(),
                    "src/**/*.jsx".to_string(),
                ],
                exclude_patterns: vec![
                    "node_modules/**".to_string(),
                    "dist/**".to_string(),
                    "build/**".to_string(),
                    ".next/**".to_string(),
                ],
                content_type: "typescript".to_string(),
                estimated_file_count: 0,
            },
            SuggestedCollection {
                name: "config".to_string(),
                description: "Configuration files".to_string(),
                include_patterns: vec![
                    "**/*.json".to_string(),
                    "**/*.yaml".to_string(),
                    "**/*.yml".to_string(),
                ],
                exclude_patterns: vec![
                    "node_modules/**".to_string(),
                    "package-lock.json".to_string(),
                ],
                content_type: "config".to_string(),
                estimated_file_count: 0,
            },
            SuggestedCollection {
                name: "docs".to_string(),
                description: "Documentation".to_string(),
                include_patterns: vec!["**/*.md".to_string()],
                exclude_patterns: vec!["node_modules/**".to_string()],
                content_type: "documentation".to_string(),
                estimated_file_count: 0,
            },
        ],
        ProjectType::Python => vec![
            SuggestedCollection {
                name: "source".to_string(),
                description: "Python source code".to_string(),
                include_patterns: vec!["**/*.py".to_string()],
                exclude_patterns: vec![
                    "__pycache__/**".to_string(),
                    ".venv/**".to_string(),
                    "venv/**".to_string(),
                    ".tox/**".to_string(),
                    "*.egg-info/**".to_string(),
                ],
                content_type: "python".to_string(),
                estimated_file_count: 0,
            },
            SuggestedCollection {
                name: "docs".to_string(),
                description: "Documentation".to_string(),
                include_patterns: vec!["**/*.md".to_string(), "**/*.rst".to_string()],
                exclude_patterns: vec![".venv/**".to_string()],
                content_type: "documentation".to_string(),
                estimated_file_count: 0,
            },
        ],
        ProjectType::Go => vec![
            SuggestedCollection {
                name: "source".to_string(),
                description: "Go source code".to_string(),
                include_patterns: vec!["**/*.go".to_string(), "go.mod".to_string()],
                exclude_patterns: vec!["vendor/**".to_string()],
                content_type: "go".to_string(),
                estimated_file_count: 0,
            },
            SuggestedCollection {
                name: "docs".to_string(),
                description: "Documentation".to_string(),
                include_patterns: vec!["**/*.md".to_string()],
                exclude_patterns: vec!["vendor/**".to_string()],
                content_type: "documentation".to_string(),
                estimated_file_count: 0,
            },
        ],
        ProjectType::Documentation => vec![SuggestedCollection {
            name: "docs".to_string(),
            description: "Documentation files".to_string(),
            include_patterns: vec![
                "**/*.md".to_string(),
                "**/*.txt".to_string(),
                "**/*.rst".to_string(),
            ],
            exclude_patterns: vec![],
            content_type: "documentation".to_string(),
            estimated_file_count: 0,
        }],
        ProjectType::Cpp => vec![
            SuggestedCollection {
                name: "source".to_string(),
                description: "C/C++ source code".to_string(),
                include_patterns: vec![
                    "**/*.cpp".to_string(),
                    "**/*.c".to_string(),
                    "**/*.h".to_string(),
                    "**/*.hpp".to_string(),
                    "**/*.cc".to_string(),
                ],
                exclude_patterns: vec!["build/**".to_string(), "cmake-build-*/**".to_string()],
                content_type: "cpp".to_string(),
                estimated_file_count: 0,
            },
            SuggestedCollection {
                name: "docs".to_string(),
                description: "Documentation".to_string(),
                include_patterns: vec!["**/*.md".to_string()],
                exclude_patterns: vec!["build/**".to_string()],
                content_type: "documentation".to_string(),
                estimated_file_count: 0,
            },
        ],
        ProjectType::Java => vec![
            SuggestedCollection {
                name: "source".to_string(),
                description: "Java source code".to_string(),
                include_patterns: vec![
                    "**/*.java".to_string(),
                    "**/*.kt".to_string(),
                    "**/*.gradle".to_string(),
                    "**/*.gradle.kts".to_string(),
                ],
                exclude_patterns: vec![
                    "build/**".to_string(),
                    "target/**".to_string(),
                    ".gradle/**".to_string(),
                ],
                content_type: "java".to_string(),
                estimated_file_count: 0,
            },
            SuggestedCollection {
                name: "docs".to_string(),
                description: "Documentation".to_string(),
                include_patterns: vec!["**/*.md".to_string()],
                exclude_patterns: vec!["build/**".to_string(), "target/**".to_string()],
                content_type: "documentation".to_string(),
                estimated_file_count: 0,
            },
        ],
        ProjectType::CSharp => vec![
            SuggestedCollection {
                name: "source".to_string(),
                description: "C# source code".to_string(),
                include_patterns: vec!["**/*.cs".to_string(), "**/*.csproj".to_string()],
                exclude_patterns: vec!["bin/**".to_string(), "obj/**".to_string()],
                content_type: "csharp".to_string(),
                estimated_file_count: 0,
            },
            SuggestedCollection {
                name: "docs".to_string(),
                description: "Documentation".to_string(),
                include_patterns: vec!["**/*.md".to_string()],
                exclude_patterns: vec!["bin/**".to_string(), "obj/**".to_string()],
                content_type: "documentation".to_string(),
                estimated_file_count: 0,
            },
        ],
        _ => vec![
            SuggestedCollection {
                name: "all".to_string(),
                description: "All project files".to_string(),
                include_patterns: vec![
                    "**/*.md".to_string(),
                    "**/*.txt".to_string(),
                    "**/*.json".to_string(),
                    "**/*.yaml".to_string(),
                    "**/*.yml".to_string(),
                ],
                exclude_patterns: vec![
                    "node_modules/**".to_string(),
                    "target/**".to_string(),
                    ".git/**".to_string(),
                ],
                content_type: "mixed".to_string(),
                estimated_file_count: 0,
            },
        ],
    }
}

/// Map file extensions to programming languages
fn extension_to_language(ext: &str) -> Option<ProgrammingLanguage> {
    match ext.to_lowercase().as_str() {
        "rs" => Some(ProgrammingLanguage::Rust),
        "ts" | "tsx" => Some(ProgrammingLanguage::TypeScript),
        "js" | "jsx" | "mjs" | "cjs" => Some(ProgrammingLanguage::JavaScript),
        "py" | "pyw" => Some(ProgrammingLanguage::Python),
        "go" => Some(ProgrammingLanguage::Go),
        "cpp" | "cc" | "cxx" | "hpp" => Some(ProgrammingLanguage::Cpp),
        "c" | "h" => Some(ProgrammingLanguage::C),
        "java" => Some(ProgrammingLanguage::Java),
        "kt" | "kts" => Some(ProgrammingLanguage::Kotlin),
        "cs" => Some(ProgrammingLanguage::CSharp),
        "rb" => Some(ProgrammingLanguage::Ruby),
        "php" => Some(ProgrammingLanguage::Php),
        "swift" => Some(ProgrammingLanguage::Swift),
        "md" | "markdown" => Some(ProgrammingLanguage::Markdown),
        "html" | "htm" => Some(ProgrammingLanguage::Html),
        "css" | "scss" | "sass" | "less" => Some(ProgrammingLanguage::Css),
        "yaml" | "yml" => Some(ProgrammingLanguage::Yaml),
        "json" => Some(ProgrammingLanguage::Json),
        "toml" => Some(ProgrammingLanguage::Toml),
        "sh" | "bash" | "zsh" => Some(ProgrammingLanguage::Shell),
        _ => None,
    }
}

/// Analyze a project directory
pub fn analyze_directory<P: AsRef<Path>>(path: P) -> Result<ProjectAnalysis, String> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }

    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", path.display()));
    }

    let project_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let project_path = path.to_string_lossy().to_string();

    // Collect statistics
    let mut stats = DirectoryStats {
        total_files: 0,
        total_directories: 0,
        total_size_bytes: 0,
        files_by_extension: HashMap::new(),
        has_git: false,
        has_docs: false,
    };

    // Detected project types and languages
    let mut project_types: Vec<ProjectType> = Vec::new();
    let mut languages: Vec<ProgrammingLanguage> = Vec::new();
    let mut frameworks: Vec<String> = Vec::new();

    // Walk directory (limited depth to avoid long scans)
    walk_directory(
        path,
        &mut stats,
        &mut project_types,
        &mut languages,
        &mut frameworks,
        0,
        5, // Max depth
    );

    // Deduplicate
    project_types.sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));
    project_types.dedup();
    languages.sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));
    languages.dedup();
    frameworks.sort();
    frameworks.dedup();

    // If no specific project type detected, check if it's documentation-focused
    if project_types.is_empty() {
        let md_count = stats.files_by_extension.get("md").copied().unwrap_or(0);
        let total_code_files: usize = stats
            .files_by_extension
            .iter()
            .filter(|(ext, _)| {
                matches!(
                    ext.as_str(),
                    "rs" | "py" | "js" | "ts" | "go" | "java" | "cpp" | "c"
                )
            })
            .map(|(_, count)| *count)
            .sum();

        if md_count > total_code_files {
            project_types.push(ProjectType::Documentation);
        } else {
            project_types.push(ProjectType::Mixed);
        }
    }

    // Generate suggested collections based on detected types
    let mut suggested_collections: Vec<SuggestedCollection> = Vec::new();
    for project_type in &project_types {
        let templates = get_collection_templates(project_type);
        for mut template in templates {
            // Add project name prefix to collection name
            template.name = format!("{}-{}", project_name, template.name);
            suggested_collections.push(template);
        }
    }

    // Deduplicate collections by name
    suggested_collections.sort_by(|a, b| a.name.cmp(&b.name));
    suggested_collections.dedup_by(|a, b| a.name == b.name);

    Ok(ProjectAnalysis {
        project_types,
        languages,
        frameworks,
        suggested_collections,
        statistics: stats,
        project_name,
        project_path,
    })
}

/// Recursively walk directory and collect information
fn walk_directory(
    path: &Path,
    stats: &mut DirectoryStats,
    project_types: &mut Vec<ProjectType>,
    languages: &mut Vec<ProgrammingLanguage>,
    frameworks: &mut Vec<String>,
    current_depth: usize,
    max_depth: usize,
) {
    if current_depth > max_depth {
        return;
    }

    let entries = match std::fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        let file_name = entry
            .file_name()
            .to_str()
            .unwrap_or_default()
            .to_string();

        // Skip hidden files/directories (except .git for detection)
        if file_name.starts_with('.') && file_name != ".git" {
            continue;
        }

        if entry_path.is_dir() {
            stats.total_directories += 1;

            // Check for special directories
            if file_name == ".git" {
                stats.has_git = true;
                continue; // Don't recurse into .git
            }
            if file_name == "docs" || file_name == "documentation" {
                stats.has_docs = true;
            }

            // Skip common non-source directories
            if matches!(
                file_name.as_str(),
                "node_modules"
                    | "target"
                    | "build"
                    | "dist"
                    | "__pycache__"
                    | ".venv"
                    | "venv"
                    | "vendor"
                    | ".next"
                    | "bin"
                    | "obj"
            ) {
                continue;
            }

            walk_directory(
                &entry_path,
                stats,
                project_types,
                languages,
                frameworks,
                current_depth + 1,
                max_depth,
            );
        } else if entry_path.is_file() {
            stats.total_files += 1;

            // Get file size
            if let Ok(metadata) = entry.metadata() {
                stats.total_size_bytes += metadata.len();
            }

            // Check for project markers
            for marker in PROJECT_MARKERS {
                if file_name == marker.filename {
                    project_types.push(marker.project_type.clone());
                    if let Some(ref lang) = marker.language {
                        languages.push(lang.clone());
                    }

                    // Detect frameworks from package.json
                    if file_name == "package.json" {
                        if let Ok(content) = std::fs::read_to_string(&entry_path) {
                            detect_js_frameworks(&content, frameworks, project_types);
                        }
                    }

                    // Check for TypeScript in package.json
                    if file_name == "package.json" {
                        if let Ok(content) = std::fs::read_to_string(&entry_path) {
                            if content.contains("\"typescript\"") {
                                project_types.push(ProjectType::TypeScript);
                                languages.push(ProgrammingLanguage::TypeScript);
                            }
                        }
                    }
                }
            }

            // Count by extension
            if let Some(ext) = entry_path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                *stats.files_by_extension.entry(ext_lower.clone()).or_insert(0) += 1;

                // Detect language from extension
                if let Some(lang) = extension_to_language(&ext_lower) {
                    languages.push(lang);
                }
            }
        }
    }
}

/// Detect JavaScript/TypeScript frameworks from package.json content
fn detect_js_frameworks(content: &str, frameworks: &mut Vec<String>, project_types: &mut Vec<ProjectType>) {
    // Common frameworks to detect
    let framework_markers = [
        ("react", "React"),
        ("vue", "Vue"),
        ("@angular/core", "Angular"),
        ("svelte", "Svelte"),
        ("next", "Next.js"),
        ("nuxt", "Nuxt"),
        ("gatsby", "Gatsby"),
        ("express", "Express"),
        ("fastify", "Fastify"),
        ("nestjs", "NestJS"),
        ("electron", "Electron"),
    ];

    for (marker, name) in framework_markers {
        if content.contains(&format!("\"{}\"", marker)) {
            frameworks.push(name.to_string());
            if matches!(name, "React" | "Vue" | "Angular" | "Svelte" | "Next.js" | "Nuxt" | "Gatsby") {
                project_types.push(ProjectType::WebFrontend);
            }
        }
    }
}

/// Convert analysis to simplified workspace project config
pub fn analysis_to_project_config(
    analysis: &ProjectAnalysis,
) -> crate::workspace::simplified_config::SimplifiedProjectConfig {
    use crate::workspace::simplified_config::SimplifiedCollectionConfig;

    let collections: Vec<SimplifiedCollectionConfig> = analysis
        .suggested_collections
        .iter()
        .map(|c| SimplifiedCollectionConfig {
            name: c.name.clone(),
            description: c.description.clone(),
            include_patterns: c.include_patterns.clone(),
            exclude_patterns: c.exclude_patterns.clone(),
            embedding: None,
            dimension: None,
            metric: None,
            indexing: None,
            processing: None,
        })
        .collect();

    crate::workspace::simplified_config::SimplifiedProjectConfig {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_to_language() {
        assert_eq!(extension_to_language("rs"), Some(ProgrammingLanguage::Rust));
        assert_eq!(extension_to_language("ts"), Some(ProgrammingLanguage::TypeScript));
        assert_eq!(extension_to_language("py"), Some(ProgrammingLanguage::Python));
        assert_eq!(extension_to_language("unknown"), None);
    }

    #[test]
    fn test_get_collection_templates() {
        let rust_templates = get_collection_templates(&ProjectType::Rust);
        assert!(rust_templates.len() >= 2);
        assert!(rust_templates.iter().any(|c| c.name == "source"));
        assert!(rust_templates.iter().any(|c| c.name == "docs"));
    }

    #[test]
    fn test_analyze_nonexistent_directory() {
        let result = analyze_directory("/nonexistent/path/12345");
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_js_frameworks() {
        let mut frameworks = Vec::new();
        let mut project_types = Vec::new();
        
        let package_json = r#"{"dependencies": {"react": "^18.0.0", "next": "^13.0.0"}}"#;
        detect_js_frameworks(package_json, &mut frameworks, &mut project_types);
        
        assert!(frameworks.contains(&"React".to_string()));
        assert!(frameworks.contains(&"Next.js".to_string()));
        assert!(project_types.contains(&ProjectType::WebFrontend));
    }
}
