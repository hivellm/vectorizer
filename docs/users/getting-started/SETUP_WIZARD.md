---
title: Setup Wizard Guide
module: getting-started
id: setup-wizard-guide
order: 3
description: Complete guide to using the Vectorizer Setup Wizard for first-time configuration
tags: [setup, wizard, configuration, getting-started, workspace]
---

# Setup Wizard Guide

The Setup Wizard provides a guided, step-by-step interface for configuring your Vectorizer workspace. It's the easiest way to get started with Vectorizer.

## Overview

The Setup Wizard helps you:

- **Automatically detect** project types, languages, and frameworks
- **Create collections** based on your project structure
- **Configure workspace** settings with sensible defaults
- **Generate workspace.yml** configuration file

## Accessing the Setup Wizard

### Web Dashboard (Recommended)

The Setup Wizard is available at:

```
http://localhost:15002/setup
```

**Auto-Redirect:** When you first access the dashboard without a `workspace.yml` file and no existing collections, you'll be automatically redirected to the Setup Wizard.

### CLI Alternative

You can also use the CLI for setup:

```bash
# Interactive setup via CLI
vectorizer-cli setup /path/to/your/project

# Open web wizard in browser
vectorizer-cli setup --wizard
```

### Terminal Guidance

On first start, Vectorizer displays setup guidance in the terminal:

```
╔══════════════════════════════════════════════════════════════════╗
║                                                                  ║
║  🚀 Welcome to Vectorizer!                                       ║
║                                                                  ║
║  First time setup detected.                                      ║
║  Configure your workspace using the Setup Wizard:                ║
║                                                                  ║
║  👉 http://localhost:15002/setup                                 ║
║                                                                  ║
║  Or use the CLI:                                                 ║
║  $ vectorizer-cli setup /path/to/your/project                    ║
║                                                                  ║
╚══════════════════════════════════════════════════════════════════╝
```

## Wizard Steps

### Step 1: Welcome

The welcome screen displays:

- Current Vectorizer version
- Deployment type (binary or docker)
- Number of existing collections

Click **"Get Started"** to begin configuration.

### Step 2: Choose a Template

Select a configuration template that matches your use case:

| Template | Description | Best For |
|----------|-------------|----------|
| **RAG** | Optimized for document retrieval and LLM integration | Chatbots, Q&A systems |
| **Code Search** | Semantic search across source code | IDE plugins, code analysis |
| **Documentation** | Index and search documentation files | Knowledge bases, docs sites |
| **Custom** | Full control over configuration | Advanced users |

#### Template Details

**RAG Template:**
- Creates `documents` collection
- Includes: `*.md`, `*.txt`, `*.pdf`, `*.docx`
- Chunk size: 512 tokens
- Optimized for retrieval

**Code Search Template:**
- Creates `source-code` collection
- Includes: `*.rs`, `*.py`, `*.ts`, `*.js`, `*.go`, etc.
- Excludes: `node_modules/`, `target/`, `.git/`
- Preserves code structure

**Documentation Template:**
- Creates `docs` collection
- Includes: `*.md`, `*.rst`, `*.adoc`
- Optimized for documentation structure

### Step 3: Add Project Folder

Enter the path to your project folder:

```
/path/to/your/project
```

**Tips:**
- Use absolute paths for reliability
- The wizard will analyze the folder structure
- You can add multiple projects

Click **"Analyze"** to scan the directory.

### Step 4: Review Analysis

The wizard shows detected:

- **Project Types:** Library, Application, Monorepo, etc.
- **Languages:** Rust, Python, TypeScript, etc.
- **Frameworks:** React, FastAPI, Axum, etc.
- **Statistics:** Total files, directories, size

**Collections Preview:**

Each project generates suggested collections based on detected content:

```
my-project/
├── my-project-source     (Source code files)
├── my-project-docs       (Documentation files)
└── my-project-config     (Configuration files)
```

You can:
- ✅ Select/deselect projects
- ✅ Select/deselect individual collections
- ✅ Add more projects

### Step 5: Review & Apply

Review the final configuration:

- Selected template
- Number of projects
- Number of collections

**Important:** The wizard creates a `workspace.yml` file. The server may need to restart to apply changes.

Click **"Apply Configuration"** to create the workspace.

### Step 6: Complete

🎉 Setup is complete! Next steps:

1. **Restart the server** to apply workspace configuration
2. **Visit the Workspace page** to manage projects
3. **Use the Search page** to query your data

## Configuration Output

The wizard generates a `workspace.yml` file:

```yaml
# Vectorizer Workspace Configuration
# Generated by Setup Wizard

global_settings:
  file_watcher:
    auto_discovery: true
    enable_auto_update: true
    hot_reload: true

projects:
  - name: my-project
    path: /path/to/my-project
    description: "Rust project"
    collections:
      - name: my-project-source
        description: "Source code files"
        include_patterns:
          - "**/*.rs"
          - "**/*.toml"
        exclude_patterns:
          - "**/target/**"
          - "**/node_modules/**"
      - name: my-project-docs
        description: "Documentation files"
        include_patterns:
          - "**/*.md"
          - "**/*.txt"
```

## Troubleshooting

### Wizard Not Loading

**Symptoms:**
- Page shows loading spinner indefinitely
- Cannot access `/setup`

**Solutions:**

1. **Verify server is running:**
   ```bash
   curl http://localhost:15002/health
   ```

2. **Check server logs:**
   ```bash
   # Linux
   journalctl -u vectorizer -f
   
   # Or check console output
   ```

3. **Clear browser cache:**
   - Hard refresh: `Ctrl+Shift+R` (Windows/Linux) or `Cmd+Shift+R` (Mac)
   - Clear site data in browser settings

4. **Check port availability:**
   ```bash
   # Check if port is in use
   lsof -i :15002
   
   # Or on Windows
   netstat -ano | findstr :15002
   ```

### Analysis Fails

**Symptoms:**
- Error message when analyzing directory
- "Failed to analyze directory" error

**Solutions:**

1. **Check path exists:**
   ```bash
   ls -la /path/to/your/project
   ```

2. **Use absolute path:**
   ```
   # ✅ Correct
   /home/user/projects/my-project
   
   # ❌ Wrong
   ~/projects/my-project
   ./my-project
   ```

3. **Check permissions:**
   ```bash
   # Ensure read access
   chmod -R +r /path/to/your/project
   ```

### Configuration Not Applied

**Symptoms:**
- workspace.yml created but not active
- Collections not appearing

**Solutions:**

1. **Restart the server:**
   ```bash
   # Linux systemd
   sudo systemctl restart vectorizer
   
   # Manual
   pkill vectorizer
   ./vectorizer
   ```

2. **Verify workspace.yml:**
   ```bash
   cat workspace.yml
   ```

3. **Check for YAML errors:**
   ```bash
   # Validate YAML syntax
   python -c "import yaml; yaml.safe_load(open('workspace.yml'))"
   ```

### Skip Auto-Redirect

If you want to access other pages without completing setup:

1. **Direct URL access:**
   ```
   http://localhost:15002/overview
   http://localhost:15002/collections
   ```

2. **Create empty workspace.yml:**
   ```yaml
   # workspace.yml
   global_settings:
     file_watcher:
       enabled: false
   projects: []
   ```

## API Reference

The Setup Wizard uses these REST API endpoints:

### GET /setup/status

Check if setup is needed.

**Response:**
```json
{
  "needs_setup": true,
  "version": "1.3.0",
  "deployment_type": "binary",
  "has_workspace_config": false,
  "project_count": 0,
  "collection_count": 0
}
```

### POST /setup/analyze

Analyze a directory.

**Request:**
```json
{
  "path": "/path/to/project"
}
```

**Response:**
```json
{
  "project_types": ["library"],
  "languages": ["rust", "typescript"],
  "frameworks": ["axum", "react"],
  "project_name": "my-project",
  "project_path": "/path/to/project",
  "statistics": {
    "total_files": 150,
    "total_directories": 25,
    "total_size_bytes": 1048576
  },
  "suggested_collections": [...]
}
```

### POST /setup/apply

Apply configuration.

**Request:**
```json
{
  "projects": [...],
  "global_settings": {...}
}
```

### GET /setup/verify

Verify setup completion.

### GET /setup/templates

Get available configuration templates.

## Related Topics

- [Quick Start Guide](./QUICK_START.md) - Get started quickly
- [First Steps](./FIRST_STEPS.md) - Post-installation guide
- [Configuration Guide](../configuration/CONFIGURATION.md) - Advanced configuration
- [Workspace Management](../api/WORKSPACE.md) - Workspace API reference
