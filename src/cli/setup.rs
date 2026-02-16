//! CLI Setup Wizard
//!
//! Interactive terminal-based setup wizard for Vectorizer configuration

use std::io::{self, Write};
use std::path::PathBuf;

use crate::workspace::project_analyzer::analyze_directory;
use crate::workspace::setup_config::{ApplyConfigRequest, write_workspace_config};
use crate::workspace::templates::{ConfigTemplate, get_template_by_id, get_templates};

/// Run the interactive setup wizard
pub async fn run(path: PathBuf) -> anyhow::Result<()> {
    println!();
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║                    Vectorizer Setup Wizard                       ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();

    // Check if path was provided or if we need to prompt
    let project_path = if path.as_os_str().is_empty() || path.to_string_lossy() == "." {
        prompt_for_path()?
    } else {
        path
    };

    println!("🔍 Analyzing directory: {}", project_path.display());
    println!();

    let analysis = match analyze_directory(&project_path) {
        Ok(a) => a,
        Err(e) => {
            println!("❌ Failed to analyze directory: {}", e);
            return Ok(());
        }
    };

    // Display analysis results
    println!("╭─────────────────────────────────────────────────────────────────╮");
    println!("│                     Analysis Complete                           │");
    println!("╰─────────────────────────────────────────────────────────────────╯");
    println!();
    println!("   📁 Project Name: {}", analysis.project_name);
    println!("   📂 Project Path: {}", analysis.project_path);
    println!("   🏷️  Project Type: {:?}", analysis.project_types);
    println!("   💻 Languages:    {:?}", analysis.languages);
    println!();
    println!("   📊 Statistics:");
    println!(
        "      • Total Files:       {}",
        analysis.statistics.total_files
    );
    println!(
        "      • Total Directories: {}",
        analysis.statistics.total_directories
    );
    println!(
        "      • Total Size:        {} bytes",
        analysis.statistics.total_size_bytes
    );
    println!(
        "      • Has Git:           {}",
        if analysis.statistics.has_git {
            "Yes"
        } else {
            "No"
        }
    );
    println!(
        "      • Has Docs:          {}",
        if analysis.statistics.has_docs {
            "Yes"
        } else {
            "No"
        }
    );
    println!();

    // Ask about template
    let template = prompt_for_template()?;

    // Determine collections based on template
    let collections = if let Some(ref t) = template {
        if t.id == "custom" {
            // Use detected collections for custom template
            analysis.suggested_collections.clone()
        } else {
            // Convert template collections
            t.collections
                .iter()
                .map(
                    |tc| crate::workspace::project_analyzer::SuggestedCollection {
                        name: format!("{}-{}", analysis.project_name, tc.name_suffix),
                        description: tc.description.clone(),
                        include_patterns: tc.include_patterns.clone(),
                        exclude_patterns: tc.exclude_patterns.clone(),
                        content_type: tc.content_type.clone(),
                        estimated_file_count: 0,
                    },
                )
                .collect()
        }
    } else {
        analysis.suggested_collections.clone()
    };

    // Display suggested collections
    println!();
    println!("   📦 Suggested Collections ({}):", collections.len());
    for col in &collections {
        println!("      • {}", col.name);
        println!("        Description: {}", col.description);
        println!("        Type:        {}", col.content_type);
        println!("        Include:     {:?}", col.include_patterns);
        println!();
    }

    // Confirm and apply
    println!();
    print!("❓ Apply this configuration and create workspace.yml? [Y/n]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    if input == "y" || input == "yes" || input.is_empty() {
        // Create modified analysis with selected collections
        let mut modified_analysis = analysis.clone();
        modified_analysis.suggested_collections = collections;

        let config = ApplyConfigRequest::from_analysis(&modified_analysis);
        match write_workspace_config(&config, "workspace.yml") {
            Ok(_) => {
                println!();
                println!("╭─────────────────────────────────────────────────────────────────╮");
                println!("│                   ✅ Setup Complete!                            │");
                println!("╰─────────────────────────────────────────────────────────────────╯");
                println!();
                println!("   workspace.yml has been created successfully!");
                println!();
                println!("   📋 Next Steps:");
                println!("   1. Start the server:     vectorizer");
                println!("   2. Open the dashboard:   http://localhost:15002");
                println!("   3. View collections:     http://localhost:15002/collections");
                println!("   4. Start searching:      http://localhost:15002/search");
                println!();
            }
            Err(e) => {
                println!("❌ Failed to write config: {}", e);
            }
        }
    } else {
        println!();
        println!("❌ Setup cancelled.");
    }

    Ok(())
}

/// Run the setup wizard with web browser
pub async fn run_wizard() -> anyhow::Result<()> {
    println!();
    println!("🌐 Opening Setup Wizard in your browser...");
    println!();
    println!("   URL: http://localhost:15002/setup");
    println!();

    // Try to open the browser
    let url = "http://localhost:15002/setup";

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }

    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", url])
            .spawn();
    }

    println!("   If the browser doesn't open automatically, please visit the URL above.");
    println!();

    Ok(())
}

/// Open API documentation in browser
pub async fn run_docs(sandbox: bool) -> anyhow::Result<()> {
    let url = if sandbox {
        "http://localhost:15002/docs/sandbox"
    } else {
        "http://localhost:15002/docs"
    };

    println!();
    println!("🌐 Opening API Documentation...");
    println!();
    println!("   URL: {}", url);
    println!();

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }

    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", url])
            .spawn();
    }

    println!("   If the browser doesn't open automatically, please visit the URL above.");
    println!();

    Ok(())
}

/// Prompt user for project path
fn prompt_for_path() -> io::Result<PathBuf> {
    print!("📁 Enter project path [.]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        Ok(PathBuf::from("."))
    } else {
        Ok(PathBuf::from(input))
    }
}

/// Prompt user for template selection
fn prompt_for_template() -> io::Result<Option<ConfigTemplate>> {
    let templates = get_templates();

    println!();
    println!("╭─────────────────────────────────────────────────────────────────╮");
    println!("│                    Select a Template                           │");
    println!("╰─────────────────────────────────────────────────────────────────╯");
    println!();

    for (i, template) in templates.iter().enumerate() {
        println!("   {}. {} {}", i + 1, template.icon, template.name);
        println!("      {}", template.description);
        println!();
    }

    println!("   0. Skip (use auto-detected collections)");
    println!();

    print!("   Enter choice [0]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() || input == "0" {
        Ok(None)
    } else if let Ok(choice) = input.parse::<usize>() {
        if choice > 0 && choice <= templates.len() {
            Ok(Some(templates[choice - 1].clone()))
        } else {
            println!("Invalid choice, using auto-detected collections.");
            Ok(None)
        }
    } else {
        // Try to match by template ID
        Ok(get_template_by_id(input))
    }
}

/// Check if setup is needed
pub fn needs_setup() -> bool {
    !std::path::Path::new("workspace.yml").exists()
}

/// Display setup hint in terminal
pub fn display_setup_hint() {
    if needs_setup() {
        println!();
        println!("💡 Tip: Run 'vectorizer-cli setup' to configure your workspace");
        println!("        Or visit http://localhost:15002/setup for the web wizard");
        println!();
    }
}
