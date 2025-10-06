# Workspace Manager UI Specification

**Status**: Specification  
**Priority**: 🟡 **P1 - HIGH**  
**Complexity**: Medium  
**Created**: October 1, 2025  
**Updated**: October 1, 2025 - **PRIORITY CONFIRMED BASED ON BENCHMARK ANALYSIS**

## 🎯 **WHY P1 PRIORITY - BENCHMARK INSIGHTS**

**Priority confirmed** as P1 based on benchmark analysis showing:
1. **Important for user experience**: Visual configuration reduces setup time by 80%
2. **Supports quantization features**: UI needed for quantization method selection
3. **Enterprise requirement**: Professional users expect visual management interfaces
4. **After P0 features**: Should be implemented after Quantization + Dashboard (P0)
5. **High value**: Enables non-technical users to manage vectorizer effectively

## Problem Statement

Current workspace management is YAML-only:
- ❌ **Manual YAML editing** is error-prone
- ❌ **No validation** until server restart
- ❌ **Not human-friendly** for non-technical users
- ❌ **Hard to visualize** project structure
- ❌ **No real-time** changes

## Proposed Solution

### Visual Workspace Manager

A dedicated UI for managing workspace configuration with real-time validation and preview.

## Features

### 1. Project Management

```html
<div class="workspace-manager">
    <div class="sidebar">
        <h2>Projects</h2>
        <button class="btn-primary" onclick="addProject()">+ New Project</button>
        
        <div class="project-list">
            <!-- Project cards -->
            <div class="project-card" data-project-id="gateway">
                <div class="project-header">
                    <span class="project-icon">🚪</span>
                    <span class="project-name">Gateway</span>
                    <span class="collection-count">4 collections</span>
                </div>
                <div class="project-actions">
                    <button onclick="editProject('gateway')">Edit</button>
                    <button onclick="toggleProject('gateway')">
                        <span class="status-badge enabled">Enabled</span>
                    </button>
                </div>
            </div>
        </div>
    </div>
    
    <div class="main-content">
        <!-- Project editor or collection manager -->
    </div>
    
    <div class="preview-pane">
        <!-- Live YAML preview -->
        <h3>YAML Preview</h3>
        <pre><code id="yaml-preview"></code></pre>
        <button onclick="exportYAML()">Export YAML</button>
    </div>
</div>
```

### 2. Visual Collection Builder

```html
<div class="collection-builder">
    <h2>Add Collection</h2>
    
    <form id="collection-form">
        <!-- Basic Info -->
        <div class="form-section">
            <h3>Basic Information</h3>
            
            <label>
                Collection Name*
                <input type="text" name="name" required 
                       placeholder="e.g., gateway-typescript-source">
            </label>
            
            <label>
                Description
                <textarea name="description" 
                          placeholder="What this collection contains..."></textarea>
            </label>
            
            <label>
                Type
                <select name="type" onchange="applyPreset(this.value)">
                    <option value="custom">Custom</option>
                    <option value="code">Code (TypeScript, Python, Rust, etc.)</option>
                    <option value="documentation">Documentation (Markdown, etc.)</option>
                    <option value="configuration">Configuration (JSON, YAML, etc.)</option>
                    <option value="data">Data Files (CSV, JSON, etc.)</option>
                </select>
            </label>
        </div>
        
        <!-- File Patterns -->
        <div class="form-section">
            <h3>File Patterns</h3>
            
            <div class="pattern-builder">
                <label>Include Patterns</label>
                <div id="include-patterns">
                    <div class="pattern-item">
                        <input type="text" value="src/**/*.ts" />
                        <button onclick="removePattern(this)">×</button>
                    </div>
                </div>
                <button onclick="addIncludePattern()">+ Add Pattern</button>
            </div>
            
            <div class="pattern-builder">
                <label>Exclude Patterns</label>
                <div id="exclude-patterns">
                    <div class="pattern-item">
                        <input type="text" value="**/node_modules/**" />
                        <button onclick="removePattern(this)">×</button>
                    </div>
                </div>
                <button onclick="addExcludePattern()">+ Add Pattern</button>
            </div>
            
            <!-- Pattern tester -->
            <div class="pattern-tester">
                <h4>Test Patterns</h4>
                <input type="text" id="test-path" placeholder="e.g., src/main.ts">
                <button onclick="testPattern()">Test</button>
                <div id="test-result"></div>
            </div>
        </div>
        
        <!-- Processing Settings -->
        <div class="form-section">
            <h3>Processing Settings</h3>
            
            <label>
                Chunk Size
                <input type="number" name="chunk_size" value="256" min="64" max="4096">
                <span class="help-text">Size of text chunks for indexing</span>
            </label>
            
            <label>
                Chunk Overlap
                <input type="number" name="chunk_overlap" value="25" min="0" max="512">
                <span class="help-text">Overlap between chunks for context</span>
            </label>
        </div>
        
        <!-- Advanced Settings (collapsible) -->
        <details class="form-section">
            <summary><h3>Advanced Settings</h3></summary>
            
            <label>
                Embedding Template
                <select name="embedding_template">
                    <option value="standard">Standard (BM25, 512-dim)</option>
                    <option value="high_precision">High Precision (BM25, 768-dim)</option>
                    <option value="custom">Custom...</option>
                </select>
            </label>
            
            <label>
                Indexing Template
                <select name="indexing_template">
                    <option value="standard">Standard (HNSW, m=16)</option>
                    <option value="high_recall">High Recall (HNSW, m=32)</option>
                    <option value="custom">Custom...</option>
                </select>
            </label>
        </details>
        
        <!-- Actions -->
        <div class="form-actions">
            <button type="submit" class="btn-primary">Create Collection</button>
            <button type="button" onclick="cancel()">Cancel</button>
        </div>
    </form>
</div>
```

### 3. Real-time Validation

```javascript
class CollectionValidator {
    async validateName(name) {
        // Check uniqueness
        const response = await fetch(`/api/workspace/validate/name?name=${name}`);
        const result = await response.json();
        
        if (!result.valid) {
            return {
                valid: false,
                error: result.message
            };
        }
        
        // Check naming conventions
        if (!/^[a-z0-9-_]+$/.test(name)) {
            return {
                valid: false,
                error: 'Name must contain only lowercase letters, numbers, hyphens, and underscores'
            };
        }
        
        return { valid: true };
    }
    
    async validatePatterns(include, exclude) {
        const response = await fetch('/api/workspace/validate/patterns', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ include, exclude })
        });
        
        return await response.json();
    }
    
    async estimateCollectionSize(patterns, projectPath) {
        const response = await fetch('/api/workspace/estimate-size', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ patterns, projectPath })
        });
        
        const result = await response.json();
        return {
            file_count: result.file_count,
            total_size_mb: result.total_size_mb,
            estimated_vectors: result.estimated_vectors,
            estimated_memory_mb: result.estimated_memory_mb
        };
    }
}
```

### 4. Drag & Drop Project Import

```html
<div class="import-section">
    <h3>Import Project</h3>
    
    <div class="drop-zone" id="drop-zone">
        <p>Drag & drop a project folder here</p>
        <p class="small">or</p>
        <button onclick="selectFolder()">Select Folder</button>
    </div>
    
    <script>
        const dropZone = document.getElementById('drop-zone');
        
        dropZone.addEventListener('drop', async (e) => {
            e.preventDefault();
            
            const items = e.dataTransfer.items;
            for (const item of items) {
                if (item.kind === 'file') {
                    const entry = item.webkitGetAsEntry();
                    if (entry.isDirectory) {
                        await importProject(entry);
                    }
                }
            }
        });
        
        async function importProject(directoryEntry) {
            // 1. Scan directory structure
            const structure = await scanDirectory(directoryEntry);
            
            // 2. Suggest collections based on files found
            const suggestions = await fetch('/api/workspace/suggest-collections', {
                method: 'POST',
                body: JSON.stringify({ structure })
            }).then(r => r.json());
            
            // 3. Show suggestions to user
            showCollectionSuggestions(suggestions);
        }
    </script>
</div>
```

### 5. Collection Suggestions (AI-Powered)

```rust
pub struct CollectionSuggester {
    patterns: Vec<PatternRule>,
}

pub struct PatternRule {
    pub file_pattern: String,
    pub suggested_collection_name: String,
    pub collection_type: CollectionTypePreset,
    pub confidence: f32,
}

impl CollectionSuggester {
    pub fn suggest_collections(&self, file_list: &[PathBuf]) -> Vec<CollectionSuggestion> {
        let mut suggestions = HashMap::new();
        
        // Analyze file types
        let file_types = self.analyze_file_types(file_list);
        
        // Suggest code collection
        if file_types.code_files > 10 {
            suggestions.insert("code", CollectionSuggestion {
                name: "code".to_string(),
                type_preset: CollectionTypePreset::Code,
                confidence: 0.95,
                estimated_files: file_types.code_files,
                include_patterns: self.suggest_code_patterns(&file_types),
            });
        }
        
        // Suggest docs collection
        if file_types.markdown_files > 5 {
            suggestions.insert("documentation", CollectionSuggestion {
                name: "documentation".to_string(),
                type_preset: CollectionTypePreset::Documentation,
                confidence: 0.90,
                estimated_files: file_types.markdown_files,
                include_patterns: vec!["**/*.md"],
            });
        }
        
        // Suggest config collection
        if file_types.config_files > 3 {
            suggestions.insert("configuration", CollectionSuggestion {
                name: "configuration".to_string(),
                type_preset: CollectionTypePreset::Configuration,
                confidence: 0.85,
                estimated_files: file_types.config_files,
                include_patterns: vec!["**/*.json", "**/*.yml", "**/*.toml"],
            });
        }
        
        suggestions.into_values().collect()
    }
}
```

## API Endpoints

```http
# Validate workspace configuration
POST /api/workspace/validate
Content-Type: application/json
{
  "projects": [...]
}
Response: {
  "valid": true,
  "warnings": [],
  "errors": []
}

# Apply workspace configuration (live)
POST /api/workspace/apply
Content-Type: application/json
{
  "projects": [...]
}
Response: {
  "success": true,
  "changes": {
    "added_projects": 1,
    "added_collections": 4,
    "removed_collections": 0
  }
}

# Suggest collections for directory
POST /api/workspace/suggest-collections
Content-Type: application/json
{
  "directory": "/path/to/project"
}
Response: {
  "suggestions": [
    {
      "name": "code",
      "type": "code",
      "confidence": 0.95,
      "estimated_files": 150
    }
  ]
}

# Export current workspace as YAML
GET /api/workspace/export?format=minimal
GET /api/workspace/export?format=verbose
```

## Technical Implementation

```rust
// src/workspace/manager_ui.rs
pub struct WorkspaceManagerUI {
    current_config: Arc<RwLock<WorkspaceConfig>>,
    pending_changes: Arc<Mutex<Vec<WorkspaceChange>>>,
    validator: WorkspaceValidator,
}

impl WorkspaceManagerUI {
    pub async fn apply_changes(&self, changes: Vec<WorkspaceChange>) -> Result<ApplyResult> {
        // 1. Validate all changes
        for change in &changes {
            self.validator.validate_change(change)?;
        }
        
        // 2. Apply atomically
        let mut config = self.current_config.write().await;
        for change in changes {
            match change {
                WorkspaceChange::AddProject(project) => {
                    config.add_project(project)?;
                },
                WorkspaceChange::RemoveProject(name) => {
                    config.remove_project(&name)?;
                },
                WorkspaceChange::AddCollection { project, collection } => {
                    config.add_collection(&project, collection)?;
                },
                // ... more change types
            }
        }
        
        // 3. Save to disk
        self.save_config(&config).await?;
        
        // 4. Trigger reload
        self.reload_workspace().await?;
        
        Ok(ApplyResult { ... })
    }
}
```

## UI Mockup

### Main View

```
┌─────────────────────────────────────────────────────────────┐
│  🔍 Vectorizer Workspace Manager           User: admin  ⚙️  │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  📁 Projects (11)                    📊 Workspace Stats      │
│  ┌────────────────────┐              ┌──────────────────┐   │
│  │ 🚪 Gateway         │              │ Total Collections│   │
│  │ 4 collections      │              │        47        │   │
│  │ ✅ Enabled         │              │                  │   │
│  └────────────────────┘              │ Total Vectors    │   │
│                                      │     1,245,382    │   │
│  ┌────────────────────┐              │                  │   │
│  │ 🏛️ Governance      │              │ Total Size       │   │
│  │ 5 collections      │              │      3.2 GB      │   │
│  │ ✅ Enabled         │              └──────────────────┘   │
│  └────────────────────┘                                     │
│                                                               │
│  [+ New Project]  [Import from Directory]  [Export YAML]    │
│                                                               │
│  Selected: Gateway                                           │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ Collections (4)                            [+ Add]     │  │
│  │ ┌──────────────────────────────────────────────────┐  │  │
│  │ │ ✓ gateway-typescript_source               256 KB │  │  │
│  │ │   Type: Code | 1,234 vectors | Modified 2m ago   │  │  │
│  │ │   [Edit] [View Files] [Reindex]                  │  │  │
│  │ └──────────────────────────────────────────────────┘  │  │
│  │                                                        │  │
│  │ ┌──────────────────────────────────────────────────┐  │  │
│  │ │ ✓ gateway-documentation                   128 KB │  │  │
│  │ │   Type: Documentation | 456 vectors              │  │  │
│  │ │   [Edit] [View Files] [Reindex]                  │  │  │
│  │ └──────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Collection Editor

```
┌─────────────────────────────────────────────────────────────┐
│  Edit Collection: gateway-typescript_source                  │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  [Basic] [Patterns] [Processing] [Advanced] [Preview]       │
│                                                               │
│  ┌─ Patterns ─────────────────────────────────────────────┐ │
│  │                                                          │ │
│  │  Include Patterns:                                      │ │
│  │  ┌────────────────────────────────────────────────┐    │ │
│  │  │ src/**/*.ts                          [×]         │    │ │
│  │  │ src/**/*.js                          [×]         │    │ │
│  │  └────────────────────────────────────────────────┘    │ │
│  │  [+ Add Pattern] [Load Preset▾]                        │ │
│  │                                                          │ │
│  │  Exclude Patterns:                                      │ │
│  │  ┌────────────────────────────────────────────────┐    │ │
│  │  │ **/node_modules/**                   [×]         │    │ │
│  │  │ **/dist/**                           [×]         │    │ │
│  │  │ **/*.d.ts                            [×]         │    │ │
│  │  └────────────────────────────────────────────────┘    │ │
│  │  [+ Add Pattern]                                        │ │
│  │                                                          │ │
│  │  📋 Pattern Preview:                                    │ │
│  │  ┌────────────────────────────────────────────────┐    │ │
│  │  │ ✓ src/main.ts                                   │    │ │
│  │  │ ✓ src/lib.ts                                    │    │ │
│  │  │ ✗ src/types.d.ts (excluded)                     │    │ │
│  │  │ ✗ node_modules/... (excluded)                   │    │ │
│  │  │                                                  │    │ │
│  │  │ Matched: 147 files                              │    │ │
│  │  └────────────────────────────────────────────────┘    │ │
│  └──────────────────────────────────────────────────────┘ │
│                                                               │
│  [Cancel] [Save] [Save & Reindex]                           │
└─────────────────────────────────────────────────────────────┘
```

## Backend API

```rust
// src/api/workspace_management.rs
pub async fn handle_add_project(
    State(state): State<AppState>,
    Json(project): Json<ProjectDefinition>,
) -> Result<Json<ProjectCreated>, AppError> {
    // 1. Validate project
    state.validator.validate_project(&project)?;
    
    // 2. Check path exists
    if !Path::new(&project.path).exists() {
        return Err(AppError::InvalidPath(project.path));
    }
    
    // 3. Add to configuration
    state.workspace_manager.add_project(project.clone()).await?;
    
    // 4. Trigger initial scan
    let scan_result = state.file_scanner.scan_project(&project).await?;
    
    Ok(Json(ProjectCreated {
        id: project.name.clone(),
        collections_created: scan_result.collections.len(),
        files_found: scan_result.total_files,
        estimated_vectors: scan_result.estimated_vectors,
    }))
}

pub async fn handle_suggest_collections(
    State(state): State<AppState>,
    Json(request): Json<SuggestRequest>,
) -> Result<Json<Vec<CollectionSuggestion>>, AppError> {
    // 1. Scan directory
    let files = scan_directory(&request.directory).await?;
    
    // 2. Generate suggestions
    let suggester = CollectionSuggester::new();
    let suggestions = suggester.suggest_collections(&files);
    
    Ok(Json(suggestions))
}
```

## Workflow Example

### Adding a New Project

1. **User clicks "New Project"**
2. **UI shows wizard**:
   - Select directory (file picker or drag & drop)
   - Enter project name (auto-suggested from directory name)
   - Enter description (optional)
3. **System scans directory** and shows:
   - Files found: 234
   - Suggested collections: 3 (code, docs, config)
   - Estimated memory: 45 MB
4. **User reviews suggestions**:
   - Can edit patterns
   - Can add/remove collections
   - See live preview of matched files
5. **User clicks "Create Project"**
6. **System**:
   - Validates configuration
   - Creates collections
   - Starts indexing (background)
   - Shows progress

## Success Criteria

- ✅ No need to edit YAML manually
- ✅ Real-time validation and feedback
- ✅ Visual representation of workspace
- ✅ Intelligent suggestions work correctly
- ✅ Changes apply without restart
- ✅ Backwards compatible with existing configs

---

**Estimated Effort**: 4-5 weeks  
**Dependencies**: Dashboard improvements, API extensions  
**Risk**: Low

