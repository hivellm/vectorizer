use serde_yaml;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string("vectorize-workspace.yml")?;
    println!("Read {} bytes", content.len());
    
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;
    println!("Parsed YAML successfully");
    
    if let Some(projects) = yaml.get("projects") {
        if let Some(seq) = projects.as_sequence() {
            println!("Found {} projects", seq.len());
            for (i, project) in seq.iter().enumerate() {
                if let Some(name) = project.get("name").and_then(|n| n.as_str()) {
                    println!("  Project {}: {}", i, name);
                }
            }
        } else {
            println!("Projects is not a sequence");
        }
    } else {
        println!("No projects key found");
        if let Some(mapping) = yaml.as_mapping() {
            println!("Available keys: {:?}", mapping.keys().collect::<Vec<_>>());
        }
    }
    
    Ok(())
}
