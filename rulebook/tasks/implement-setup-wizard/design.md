# Technical Design: Setup Wizard with API Documentation Sandbox

## Overview

This document describes the technical design decisions for implementing the Setup Wizard, API Documentation, and Sandbox features in Vectorizer.

---

## Architecture

### Component Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Vectorizer Server                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐  │
│  │  Setup Module    │  │  Docs Module     │  │  Sandbox Module  │  │
│  │                  │  │                  │  │                  │  │
│  │ - StatusHandler  │  │ - OpenAPIHandler │  │ - ExecuteHandler │  │
│  │ - AnalyzeHandler │  │ - DocsPageHandler│  │ - IsolationLayer │  │
│  │ - ApplyHandler   │  │ - SearchHandler  │  │ - CodeGenerator  │  │
│  │ - VerifyHandler  │  │                  │  │                  │  │
│  │ - TemplateHandler│  │                  │  │                  │  │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘  │
│           │                     │                     │             │
│           └─────────────────────┼─────────────────────┘             │
│                                 │                                    │
│                     ┌───────────▼───────────┐                       │
│                     │   REST API Router     │                       │
│                     │  (Axum/Tower)         │                       │
│                     └───────────────────────┘                       │
│                                 │                                    │
└─────────────────────────────────┼────────────────────────────────────┘
                                  │
┌─────────────────────────────────▼────────────────────────────────────┐
│                        Dashboard (React)                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐  │
│  │  SetupWizard     │  │  APIDocs         │  │  APISandbox      │  │
│  │                  │  │                  │  │                  │  │
│  │ - WelcomeStep    │  │ - SwaggerUI      │  │ - RequestBuilder │  │
│  │ - TemplateStep   │  │ - EndpointList   │  │ - ResponseViewer │  │
│  │ - FolderStep     │  │ - SearchFilter   │  │ - CodeExamples   │  │
│  │ - AnalysisStep   │  │ - CategoryNav    │  │ - History        │  │
│  │ - ReviewStep     │  │                  │  │                  │  │
│  │ - CompleteStep   │  │                  │  │                  │  │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘  │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Backend Design

### 1. Setup Module (`src/server/setup_handlers.rs`)

#### First Start Detection

```rust
// In main.rs or server startup
pub async fn check_and_display_setup_guidance(state: &VectorizerServer) {
    let needs_setup = !Path::new("workspace.yml").exists() 
        && state.store.list_collections().is_empty();
    
    if needs_setup {
        info!("╔════════════════════════════════════════════════════════════╗");
        info!("║  🚀 Welcome to Vectorizer!                                  ║");
        info!("║                                                             ║");
        info!("║  First time setup detected.                                 ║");
        info!("║  Open the Setup Wizard to configure your workspace:         ║");
        info!("║                                                             ║");
        info!("║  👉 http://localhost:15002/setup                            ║");
        info!("║                                                             ║");
        info!("║  Or run: vectorizer setup --wizard                          ║");
        info!("╚════════════════════════════════════════════════════════════╝");
    }
}
```

#### Configuration Templates

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub collections: Vec<TemplateCollection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateCollection {
    pub name_suffix: String,
    pub description: String,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub settings: CollectionSettings,
}

pub fn get_templates() -> Vec<ConfigTemplate> {
    vec![
        ConfigTemplate {
            id: "rag".into(),
            name: "RAG (Retrieval-Augmented Generation)".into(),
            description: "Optimized for document retrieval and LLM integration".into(),
            icon: "🤖".into(),
            collections: vec![
                TemplateCollection {
                    name_suffix: "documents".into(),
                    description: "Main document collection".into(),
                    include_patterns: vec!["**/*.md", "**/*.txt", "**/*.pdf", "**/*.docx"],
                    exclude_patterns: vec!["**/node_modules/**", "**/target/**"],
                    settings: CollectionSettings {
                        chunk_size: 512,
                        chunk_overlap: 50,
                        embedding_model: "default".into(),
                    },
                },
            ],
        },
        ConfigTemplate {
            id: "code_search".into(),
            name: "Code Search".into(),
            description: "Semantic search across source code".into(),
            icon: "💻".into(),
            collections: vec![/* ... */],
        },
        ConfigTemplate {
            id: "documentation".into(),
            name: "Documentation".into(),
            description: "Index and search documentation files".into(),
            icon: "📚".into(),
            collections: vec![/* ... */],
        },
        ConfigTemplate {
            id: "custom".into(),
            name: "Custom".into(),
            description: "Full control over configuration".into(),
            icon: "⚙️".into(),
            collections: vec![],
        },
    ]
}
```

### 2. API Documentation Module (`src/server/docs_handlers.rs`)

#### OpenAPI Specification Generation

```rust
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        // Collections
        list_collections,
        create_collection,
        get_collection,
        delete_collection,
        // Vectors
        insert_vector,
        get_vector,
        update_vector,
        delete_vector,
        // Search
        search_vectors,
        intelligent_search,
        semantic_search,
        multi_collection_search,
        // Discovery
        discover,
        filter_collections,
        expand_queries,
        // Setup
        get_setup_status,
        analyze_project_directory,
        apply_setup_config,
        verify_setup,
        // ... more endpoints
    ),
    components(
        schemas(
            Collection,
            Vector,
            SearchRequest,
            SearchResponse,
            SetupStatus,
            ProjectAnalysis,
            // ... more schemas
        )
    ),
    tags(
        (name = "Collections", description = "Collection management"),
        (name = "Vectors", description = "Vector operations"),
        (name = "Search", description = "Search operations"),
        (name = "Discovery", description = "Discovery pipeline"),
        (name = "Setup", description = "Setup wizard"),
        (name = "Files", description = "File operations"),
        (name = "Workspace", description = "Workspace management"),
    ),
    info(
        title = "Vectorizer API",
        version = "1.0.0",
        description = "High-performance vector database API",
        license(name = "MIT"),
    )
)]
pub struct ApiDoc;

/// GET /api/docs/openapi.json
pub async fn openapi_spec() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
```

### 3. Sandbox Module (`src/server/sandbox_handlers.rs`)

#### Request Execution

```rust
#[derive(Debug, Deserialize)]
pub struct SandboxRequest {
    pub method: String,
    pub endpoint: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Value>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct SandboxResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Value,
    pub timing_ms: u64,
    pub code_examples: CodeExamples,
}

#[derive(Debug, Serialize)]
pub struct CodeExamples {
    pub curl: String,
    pub python: String,
    pub typescript: String,
    pub rust: String,
    pub go: String,
}

/// POST /api/sandbox/execute
pub async fn execute_sandbox_request(
    State(state): State<VectorizerServer>,
    Json(req): Json<SandboxRequest>,
) -> Result<Json<SandboxResponse>, ErrorResponse> {
    let start = Instant::now();
    
    // Build internal request
    let response = execute_internal_request(&state, &req).await?;
    
    // Generate code examples
    let examples = generate_code_examples(&req);
    
    Ok(Json(SandboxResponse {
        status: response.status,
        headers: response.headers,
        body: response.body,
        timing_ms: start.elapsed().as_millis() as u64,
        code_examples: examples,
    }))
}

fn generate_code_examples(req: &SandboxRequest) -> CodeExamples {
    let base_url = "http://localhost:15002";
    
    CodeExamples {
        curl: generate_curl_example(base_url, req),
        python: generate_python_example(base_url, req),
        typescript: generate_typescript_example(base_url, req),
        rust: generate_rust_example(base_url, req),
        go: generate_go_example(base_url, req),
    }
}
```

---

## Frontend Design

### 1. Setup Wizard Enhanced Flow

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Welcome   │────▶│  Template   │────▶│   Folder    │
│   Step      │     │   Select    │     │   Input     │
└─────────────┘     └─────────────┘     └─────────────┘
                                               │
                                               ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Complete   │◀────│   Review    │◀────│  Analysis   │
│   Step      │     │   Config    │     │   Result    │
└─────────────┘     └─────────────┘     └─────────────┘
```

### 2. New Dashboard Routes

```typescript
// AppRouter.tsx additions
const routes = [
  // ... existing routes
  { path: '/setup', element: <SetupWizardPage /> },
  { path: '/docs', element: <ApiDocsPage /> },
  { path: '/docs/sandbox', element: <ApiSandboxPage /> },
];
```

### 3. API Documentation Component Structure

```
dashboard/src/
├── pages/
│   ├── ApiDocsPage.tsx          # Main documentation page
│   └── ApiSandboxPage.tsx       # Sandbox page
├── components/
│   └── api-docs/
│       ├── EndpointList.tsx     # List of all endpoints
│       ├── EndpointDetail.tsx   # Single endpoint view
│       ├── RequestBuilder.tsx   # Build requests
│       ├── ResponseViewer.tsx   # View responses
│       ├── CodeExamples.tsx     # Code snippets
│       ├── SchemaViewer.tsx     # JSON schema viewer
│       └── CategoryNav.tsx      # Navigation by category
└── hooks/
    ├── useApiDocs.ts            # Fetch OpenAPI spec
    └── useSandbox.ts            # Execute sandbox requests
```

### 4. Auto-Redirect Logic

```typescript
// useSetupRedirect.ts
export function useSetupRedirect() {
  const navigate = useNavigate();
  const location = useLocation();
  
  useEffect(() => {
    const checkSetup = async () => {
      try {
        const response = await fetch('/setup/status');
        const status = await response.json();
        
        // Redirect to setup if needed and not already there
        if (status.needs_setup && !location.pathname.startsWith('/setup')) {
          navigate('/setup');
        }
      } catch (error) {
        console.error('Failed to check setup status:', error);
      }
    };
    
    checkSetup();
  }, [navigate, location]);
}

// App.tsx - Use at root level
function App() {
  useSetupRedirect();
  
  return (
    <Routes>
      {/* ... */}
    </Routes>
  );
}
```

---

## CLI Design

### 1. Setup Command (`src/cli/setup.rs`)

```rust
use clap::Subcommand;
use std::process::Command;

#[derive(Subcommand)]
pub enum SetupCommands {
    /// Interactive setup wizard
    #[command(name = "")]
    Interactive {
        /// Project path to analyze
        #[arg(value_name = "PATH")]
        path: Option<String>,
    },
    
    /// Open web-based setup wizard
    #[command(name = "wizard")]
    Wizard,
}

pub fn run_setup(cmd: Option<SetupCommands>) -> Result<()> {
    match cmd {
        Some(SetupCommands::Wizard) | None if should_open_wizard() => {
            open_browser("http://localhost:15002/setup")?;
            println!("🌐 Opening Setup Wizard in your browser...");
        }
        Some(SetupCommands::Interactive { path }) => {
            run_interactive_setup(path)?;
        }
        _ => {
            run_interactive_setup(None)?;
        }
    }
    Ok(())
}

fn run_interactive_setup(path: Option<String>) -> Result<()> {
    println!("╔════════════════════════════════════════╗");
    println!("║     Vectorizer Setup Wizard            ║");
    println!("╚════════════════════════════════════════╝");
    println!();
    
    // 1. Get project path
    let project_path = match path {
        Some(p) => p,
        None => {
            print!("📁 Enter project path: ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            input.trim().to_string()
        }
    };
    
    // 2. Analyze project
    println!("🔍 Analyzing project...");
    let analysis = analyze_directory(&project_path)?;
    
    // 3. Show results
    println!("✅ Detected: {:?}", analysis.project_types);
    println!("   Languages: {:?}", analysis.languages);
    println!("   Files: {}", analysis.statistics.total_files);
    
    // 4. Confirm and apply
    print!("Apply this configuration? [Y/n]: ");
    io::stdout().flush()?;
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;
    
    if confirm.trim().to_lowercase() != "n" {
        apply_configuration(&analysis)?;
        println!("✅ Configuration applied successfully!");
        println!();
        println!("Next steps:");
        println!("  1. Restart Vectorizer: vectorizer");
        println!("  2. Open dashboard: http://localhost:15002");
    }
    
    Ok(())
}
```

### 2. Docs Command

```rust
/// Open API documentation in browser
#[derive(Parser)]
pub struct DocsCommand {
    /// Open sandbox instead of documentation
    #[arg(long)]
    sandbox: bool,
}

pub fn run_docs(cmd: DocsCommand) -> Result<()> {
    let url = if cmd.sandbox {
        "http://localhost:15002/docs/sandbox"
    } else {
        "http://localhost:15002/docs"
    };
    
    open_browser(url)?;
    println!("🌐 Opening API Documentation...");
    Ok(())
}
```

---

## File Structure

### New/Modified Files

```
src/
├── server/
│   ├── setup_handlers.rs     # Enhanced with templates
│   ├── docs_handlers.rs      # NEW: API docs endpoints
│   ├── sandbox_handlers.rs   # NEW: Sandbox endpoints
│   └── mod.rs                # Register new routes
├── workspace/
│   ├── project_analyzer.rs   # Existing (minor enhancements)
│   ├── setup_config.rs       # Existing
│   └── templates.rs          # NEW: Configuration templates
├── cli/
│   ├── mod.rs                # Add setup and docs commands
│   └── setup.rs              # NEW: CLI setup wizard
└── bin/
    └── vectorizer.rs         # Add first-start detection

dashboard/src/
├── pages/
│   ├── SetupWizardPage.tsx   # Enhanced with templates
│   ├── ApiDocsPage.tsx       # NEW
│   └── ApiSandboxPage.tsx    # NEW
├── components/
│   └── api-docs/             # NEW: Documentation components
├── hooks/
│   ├── useSetup.ts           # Existing
│   ├── useApiDocs.ts         # NEW
│   ├── useSandbox.ts         # NEW
│   └── useSetupRedirect.ts   # NEW
└── router/
    └── AppRouter.tsx         # Add new routes
```

---

## Dependencies

### Backend

```toml
# Cargo.toml additions
[dependencies]
utoipa = { version = "4", features = ["axum_extras"] }
utoipa-swagger-ui = { version = "6", features = ["axum"] }
```

### Frontend

```json
// package.json additions
{
  "dependencies": {
    "swagger-ui-react": "^5.0.0",
    "@monaco-editor/react": "^4.6.0"
  }
}
```

---

## Testing Strategy

### Unit Tests
- Template generation and validation
- OpenAPI spec generation
- Code example generators
- Project analyzer edge cases

### Integration Tests
- Setup API endpoint flow
- Documentation endpoint responses
- Sandbox request execution

### E2E Tests
- Complete wizard flow
- API documentation navigation
- Sandbox request/response cycle

---

## Rollout Plan

### Phase 1: Core Setup Enhancements
1. First-start detection and terminal guidance
2. Dashboard auto-redirect
3. Configuration templates

### Phase 2: API Documentation
1. OpenAPI spec generation
2. Documentation page with Swagger UI
3. Endpoint categorization and search

### Phase 3: API Sandbox
1. Request builder
2. Response viewer
3. Code examples generator

### Phase 4: CLI and Polish
1. CLI setup command
2. CLI docs command
3. UX polish and error handling
4. Documentation updates
