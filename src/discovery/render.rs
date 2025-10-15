//! LLM prompt rendering

use super::config::PromptRenderConfig;
use super::types::{AnswerPlan, DiscoveryResult};

/// Generate compact prompt for LLM formatting
pub fn render_llm_prompt(plan: &AnswerPlan, config: &PromptRenderConfig) -> DiscoveryResult<String> {
    let mut prompt = String::new();
    
    // Header
    prompt.push_str("# Context from Vector Database\n\n");
    prompt.push_str(&format!(
        "Found {} relevant pieces of information from {} sources.\n\n",
        plan.total_bullets,
        plan.sources.len()
    ));
    
    // Instructions
    prompt.push_str("## Instructions\n");
    prompt.push_str("Format the following information into a clear, concise answer. ");
    prompt.push_str("Keep citations [source_id] intact. Organize by the sections provided.\n\n");
    
    // Sections
    prompt.push_str("## Evidence\n\n");
    for section in &plan.sections {
        prompt.push_str(&format!("### {}\n\n", section.title));
        
        for (idx, bullet) in section.bullets.iter().enumerate() {
            if config.include_sources {
                prompt.push_str(&format!("{}. {} [{}]\n", idx + 1, bullet.text, bullet.source_id));
            } else {
                prompt.push_str(&format!("{}. {}\n", idx + 1, bullet.text));
            }
        }
        prompt.push_str("\n");
    }
    
    // Sources index
    if config.include_sources {
        prompt.push_str("## Sources\n\n");
        for source in &plan.sources {
            prompt.push_str(&format!("- {}\n", source));
        }
    }
    
    // Truncate if too long
    let final_prompt = truncate_to_token_limit(prompt, config.max_prompt_tokens);
    
    Ok(final_prompt)
}

/// Truncate prompt to token limit
fn truncate_to_token_limit(prompt: String, max_tokens: usize) -> String {
    // Rough token estimation: 1 token ≈ 4 characters
    let estimated_tokens = prompt.len() / 4;
    
    if estimated_tokens > max_tokens {
        let max_chars = max_tokens * 4;
        let truncated: String = prompt.chars().take(max_chars).collect();
        format!(
            "{}\n\n[... truncated to {} tokens ...]",
            truncated, max_tokens
        )
    } else {
        prompt
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::types::{Bullet, BulletCategory, Section, SectionType};
    
    fn create_test_plan() -> AnswerPlan {
        let bullet = Bullet {
            text: "Test bullet".to_string(),
            source_id: "test#0".to_string(),
            collection: "test".to_string(),
            file_path: "test.md".to_string(),
            score: 0.9,
            category: BulletCategory::Definition,
        };
        
        let section = Section {
            title: "📋 Definition".to_string(),
            section_type: SectionType::Definition,
            bullets: vec![bullet],
            priority: 1,
        };
        
        AnswerPlan {
            sections: vec![section],
            total_bullets: 1,
            sources: vec!["[test#0]".to_string()],
        }
    }
    
    #[test]
    fn test_render_llm_prompt() {
        let plan = create_test_plan();
        let config = PromptRenderConfig::default();
        
        let prompt = render_llm_prompt(&plan, &config).unwrap();
        
        assert!(prompt.contains("Context from Vector Database"));
        assert!(prompt.contains("Definition"));
        assert!(prompt.contains("Test bullet"));
        assert!(prompt.contains("[test#0]"));
    }
    
    #[test]
    fn test_truncate_long_prompt() {
        let long_prompt = "a".repeat(20000);
        let truncated = truncate_to_token_limit(long_prompt, 1000);
        
        assert!(truncated.len() < 20000);
        assert!(truncated.contains("truncated"));
    }
}

