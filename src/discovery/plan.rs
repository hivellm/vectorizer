//! Answer plan generation

use super::config::AnswerPlanConfig;
use super::types::{AnswerPlan, Bullet, BulletCategory, DiscoveryResult, Section, SectionType};
use std::collections::{HashMap, HashSet};

/// Structure bullets into organized sections
pub fn build_answer_plan(bullets: &[Bullet], config: &AnswerPlanConfig) -> DiscoveryResult<AnswerPlan> {
    let mut sections = Vec::new();
    let mut sources = HashSet::new();
    
    // Group bullets by category
    let mut bullets_by_category: HashMap<BulletCategory, Vec<Bullet>> = HashMap::new();
    for bullet in bullets {
        bullets_by_category
            .entry(bullet.category.clone())
            .or_insert_with(Vec::new)
            .push(bullet.clone());
        
        sources.insert(format!("[{}]", bullet.source_id));
    }
    
    // Create sections in priority order
    for section_type in &config.sections {
        let category = match section_type {
            SectionType::Definition => BulletCategory::Definition,
            SectionType::Features => BulletCategory::Feature,
            SectionType::Architecture => BulletCategory::Architecture,
            SectionType::Performance => BulletCategory::Performance,
            SectionType::Integrations => BulletCategory::Integration,
            SectionType::UseCases => BulletCategory::UseCase,
        };
        
        if let Some(mut section_bullets) = bullets_by_category.remove(&category) {
            // Sort by score and limit
            section_bullets.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
            section_bullets.truncate(config.max_bullets_per_section);
            
            if section_bullets.len() >= config.min_bullets_per_section {
                sections.push(Section {
                    title: section_title(section_type),
                    section_type: section_type.clone(),
                    bullets: section_bullets,
                    priority: sections.len() + 1,
                });
            }
        }
    }
    
    Ok(AnswerPlan {
        sections,
        total_bullets: bullets.len(),
        sources: sources.into_iter().collect(),
    })
}

/// Get section title
fn section_title(section_type: &SectionType) -> String {
    match section_type {
        SectionType::Definition => "📋 Definition".to_string(),
        SectionType::Features => "✨ Key Features".to_string(),
        SectionType::Architecture => "🏗️ Architecture".to_string(),
        SectionType::Performance => "⚡ Performance".to_string(),
        SectionType::Integrations => "🔗 Integrations".to_string(),
        SectionType::UseCases => "🎯 Use Cases".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_bullet(text: &str, category: BulletCategory, score: f32) -> Bullet {
        Bullet {
            text: text.to_string(),
            source_id: "test#0".to_string(),
            collection: "test".to_string(),
            file_path: "test.md".to_string(),
            score,
            category,
        }
    }
    
    #[test]
    fn test_build_answer_plan() {
        let bullets = vec![
            create_test_bullet("A definition", BulletCategory::Definition, 0.9),
            create_test_bullet("A feature", BulletCategory::Feature, 0.8),
            create_test_bullet("Another feature", BulletCategory::Feature, 0.85),
        ];
        
        let config = AnswerPlanConfig::default();
        let plan = build_answer_plan(&bullets, &config).unwrap();
        
        assert!(!plan.sections.is_empty());
        assert_eq!(plan.total_bullets, 3);
        assert!(!plan.sources.is_empty());
    }
    
    #[test]
    fn test_section_title() {
        assert_eq!(section_title(&SectionType::Definition), "📋 Definition");
        assert_eq!(section_title(&SectionType::Features), "✨ Key Features");
    }
}

