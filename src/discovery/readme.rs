//! README file promotion

use super::config::ReadmePromotionConfig;
use super::types::{DiscoveryResult, ScoredChunk};

/// Boost README files to the top of results
pub fn promote_readme(
    hits: &[ScoredChunk],
    config: &ReadmePromotionConfig,
) -> DiscoveryResult<Vec<ScoredChunk>> {
    let mut readme_chunks = Vec::new();
    let mut other_chunks = Vec::new();

    for chunk in hits {
        if is_readme(&chunk.metadata.file_path, config) {
            let mut promoted = chunk.clone();
            promoted.score *= config.readme_boost;
            readme_chunks.push(promoted);
        } else {
            other_chunks.push(chunk.clone());
        }
    }

    if config.always_top {
        // READMEs always at top, sorted by score
        readme_chunks.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        other_chunks.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        readme_chunks.extend(other_chunks);
        Ok(readme_chunks)
    } else {
        // READMEs get boost but mixed with others by score
        let mut all = readme_chunks;
        all.extend(other_chunks);
        all.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(all)
    }
}

/// Check if file path is a README
fn is_readme(file_path: &str, config: &ReadmePromotionConfig) -> bool {
    let filename = std::path::Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    config
        .readme_patterns
        .iter()
        .any(|pattern| filename.eq_ignore_ascii_case(pattern))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::types::ChunkMetadata;

    fn create_test_chunk(file_path: &str, score: f32) -> ScoredChunk {
        ScoredChunk {
            collection: "test".to_string(),
            doc_id: "doc1".to_string(),
            content: "test content".to_string(),
            score,
            metadata: ChunkMetadata {
                file_path: file_path.to_string(),
                chunk_index: 0,
                file_extension: "md".to_string(),
                line_range: None,
            },
        }
    }

    #[test]
    fn test_is_readme() {
        let config = ReadmePromotionConfig::default();

        assert!(is_readme("README.md", &config));
        assert!(is_readme("readme.md", &config));
        assert!(is_readme("README", &config));
        assert!(!is_readme("other.md", &config));
    }

    #[test]
    fn test_promote_readme() {
        let chunks = vec![
            create_test_chunk("other.md", 0.9),
            create_test_chunk("README.md", 0.7),
            create_test_chunk("another.md", 0.8),
        ];

        let config = ReadmePromotionConfig::default();
        let promoted = promote_readme(&chunks, &config).unwrap();

        // README should be first even with lower original score
        assert!(promoted[0].metadata.file_path.contains("README"));
        assert!(promoted[0].score > 0.7); // Boosted
    }
}
