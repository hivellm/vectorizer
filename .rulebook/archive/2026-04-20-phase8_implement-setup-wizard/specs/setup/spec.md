# Setup Wizard Specification

## Purpose

This specification defines the Setup Wizard feature for Vectorizer, including automatic first-start detection, web-based configuration wizard, API documentation with sandbox, and CLI setup command. The goal is to reduce time-to-value for new users and improve API discoverability.

---

## ADDED Requirements

### Requirement: Project Analysis
The system MUST be able to analyze a given directory path to detect project types, languages, frameworks, and gather directory statistics.

#### Scenario: Analyze Rust Project
Given a directory containing a `Cargo.toml` file
When the analysis is triggered via `/setup/analyze`
Then the system identifies it as a Rust project and suggests appropriate include/exclude patterns for source code and documentation collections

#### Scenario: Analyze TypeScript Project
Given a directory containing `package.json` and `tsconfig.json` files
When the analysis is triggered
Then the system identifies it as a TypeScript project and detects any frameworks (React, Vue, Next.js, etc.)

#### Scenario: Analyze Multi-Language Project
Given a directory containing multiple project markers (Cargo.toml, package.json, pyproject.toml)
When the analysis is triggered
Then the system identifies all project types and languages, generating collections for each

---

### Requirement: Setup Status API
The system MUST provide a RESTful API endpoint to check if initial setup is needed.

#### Scenario: Check Setup Status - Needs Setup
Given the server is running for the first time
When a GET request is made to `/setup/status`
Then the system returns `needs_setup: true` with deployment information

#### Scenario: Check Setup Status - Already Configured
Given the server has a valid `workspace.yml` and at least one collection
When a GET request is made to `/setup/status`
Then the system returns `needs_setup: false` with project and collection counts

---

### Requirement: First Start Auto-Detection
The system MUST detect when initial setup is required and guide users to the Setup Wizard.

#### Scenario: Terminal Guidance on First Start
Given the Vectorizer server starts for the first time
When no `workspace.yml` exists and no collections are configured
Then the terminal displays a message with the Setup Wizard URL (e.g., "🚀 First time? Open http://localhost:15002/setup to configure your workspace")

#### Scenario: Dashboard Auto-Redirect
Given a user opens the dashboard for the first time
When `needs_setup` is true
Then the dashboard automatically redirects to `/setup` wizard page

#### Scenario: Persistent Setup Banner
Given the dashboard is accessed without completing setup
When `needs_setup` is true and user navigates away from wizard
Then a persistent banner appears offering to complete setup

---

### Requirement: Configuration Application
The system MUST allow applying a simplified configuration object which is then converted into a valid `workspace.yml` file.

#### Scenario: Apply Configuration
Given a valid project configuration payload with projects and collections
When a POST request is made to `/setup/apply`
Then the system generates and writes the `workspace.yml` file and returns success with created project count

#### Scenario: Apply Configuration with Templates
Given a user selects a pre-configured template (RAG, Code Search, Documentation)
When the template is applied
Then the system generates appropriate collections with optimized settings for the use case

---

### Requirement: Configuration Templates
The system MUST provide pre-configured templates for common use cases.

#### Scenario: RAG Template Selection
Given a user selects the "RAG (Retrieval-Augmented Generation)" template
When applied to a project directory
Then collections are created optimized for document chunking, embedding, and semantic search

#### Scenario: Code Search Template Selection
Given a user selects the "Code Search" template
When applied to a project directory
Then collections are created for source code with language-aware chunking and technical search optimization

#### Scenario: Documentation Template Selection
Given a user selects the "Documentation" template
When applied to a project directory
Then collections are created for markdown, text, and documentation files with appropriate patterns

#### Scenario: Custom Template Selection
Given a user selects "Custom" template
When proceeding through the wizard
Then the system allows full customization of collections, patterns, and settings

---

### Requirement: Dashboard Setup Wizard
The dashboard MUST provide a user-friendly, multi-step interface to guide users through the setup process.

#### Scenario: Wizard Flow - Welcome
Given a new installation
When the user accesses the Setup Wizard
Then they see a welcome screen with version info and a "Get Started" button

#### Scenario: Wizard Flow - Template Selection
Given the user clicks "Get Started"
When presented with template options
Then they can choose between RAG, Code Search, Documentation, or Custom templates

#### Scenario: Wizard Flow - Folder Selection
Given a template is selected (or Custom)
When the user enters a folder path
Then the system analyzes the directory and displays detected project information

#### Scenario: Wizard Flow - Collection Review
Given the project is analyzed
When collections are suggested
Then the user can review, select/deselect, and customize collection configurations

#### Scenario: Wizard Flow - Configuration Preview
Given the user reviews collections
When they proceed to the final step
Then they see a preview of the YAML configuration that will be generated

#### Scenario: Wizard Flow - Completion
Given the configuration is applied successfully
When the wizard completes
Then the user sees next steps and links to dashboard, workspace, and documentation

---

### Requirement: API Documentation
The system MUST provide interactive API documentation accessible from the dashboard.

#### Scenario: OpenAPI Specification
Given the server is running
When a GET request is made to `/api/docs/openapi.json`
Then the system returns a complete OpenAPI 3.0 specification for all endpoints

#### Scenario: Documentation Page Access
Given a user navigates to `/docs` in the dashboard
When the page loads
Then they see interactive API documentation with all endpoints categorized

#### Scenario: Endpoint Search
Given the user is on the API documentation page
When they search for "search"
Then all search-related endpoints are filtered and displayed

#### Scenario: Endpoint Details
Given the user clicks on an endpoint
When the details expand
Then they see description, parameters, request/response schemas, and example payloads

---

### Requirement: API Sandbox
The system MUST provide a sandbox environment for testing API calls directly from the dashboard.

#### Scenario: Request Builder
Given the user is on the API Sandbox page
When they select an endpoint
Then they see a request builder with method, URL, headers, and body fields

#### Scenario: Execute Request
Given the user configures a request in the sandbox
When they click "Send Request"
Then the request is executed against the server and response is displayed

#### Scenario: Code Examples Generation
Given the user configures a request in the sandbox
When they click "Get Code Examples"
Then code snippets are generated for curl, Python, TypeScript, Rust, and Go

#### Scenario: Response Viewer
Given a request is executed
When the response is received
Then it is displayed with syntax highlighting, formatted JSON, and timing information

#### Scenario: Sandbox Isolation
Given a user executes operations in sandbox mode
When destructive operations are attempted
Then they are isolated to sandbox test data or require explicit confirmation

---

### Requirement: CLI Setup Command
The system MUST provide a CLI command for terminal-based setup.

#### Scenario: Interactive CLI Setup
Given the user runs `vectorizer setup`
When no arguments are provided
Then an interactive terminal wizard guides them through setup

#### Scenario: Open Web Wizard from CLI
Given the user runs `vectorizer setup --wizard`
When the command executes
Then the default web browser opens to the Setup Wizard URL

#### Scenario: CLI Docs Command
Given the user runs `vectorizer docs`
When the command executes
Then the default web browser opens to the API Documentation URL

#### Scenario: CLI with Path Argument
Given the user runs `vectorizer setup /path/to/project`
When a valid path is provided
Then the system analyzes the project and prompts for confirmation

---

### Requirement: Setup Verification
The system MUST provide verification of successful setup completion.

#### Scenario: Verify Setup Success
Given setup has been completed
When a GET request is made to `/setup/verify`
Then the system returns setup status, health info, workspace validity, and next steps

#### Scenario: Verify Setup Failure
Given setup has not been completed
When a GET request is made to `/setup/verify`
Then the system returns `setup_complete: false` with instructions to complete setup

---

## Response Schemas

### SetupStatus Response
```json
{
  "needs_setup": boolean,
  "version": "string",
  "deployment_type": "binary" | "docker",
  "has_workspace_config": boolean,
  "project_count": number,
  "collection_count": number
}
```

### ProjectAnalysis Response
```json
{
  "project_name": "string",
  "project_path": "string",
  "project_types": ["rust" | "typescript" | "python" | ...],
  "languages": ["rust" | "typescript" | ...],
  "frameworks": ["string"],
  "suggested_collections": [{
    "name": "string",
    "description": "string",
    "include_patterns": ["string"],
    "exclude_patterns": ["string"],
    "content_type": "string",
    "estimated_file_count": number
  }],
  "statistics": {
    "total_files": number,
    "total_directories": number,
    "total_size_bytes": number,
    "files_by_extension": {"ext": number},
    "has_git": boolean,
    "has_docs": boolean
  }
}
```

### ConfigurationTemplate Response
```json
{
  "id": "rag" | "code_search" | "documentation" | "custom",
  "name": "string",
  "description": "string",
  "icon": "string",
  "collections": [{
    "name_suffix": "string",
    "description": "string",
    "include_patterns": ["string"],
    "exclude_patterns": ["string"],
    "settings": {
      "chunk_size": number,
      "chunk_overlap": number,
      "embedding_model": "string"
    }
  }]
}
```

---

## API Endpoints Summary

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/setup/status` | Check if setup is needed |
| POST | `/setup/analyze` | Analyze a directory |
| POST | `/setup/apply` | Apply configuration |
| GET | `/setup/verify` | Verify setup completion |
| GET | `/setup/templates` | Get available templates |
| GET | `/api/docs/openapi.json` | OpenAPI specification |
| GET | `/api/docs` | API documentation HTML |
| POST | `/api/sandbox/execute` | Execute sandbox request |

---

## Non-Functional Requirements

### Performance
- Directory analysis MUST complete within 5 seconds for projects up to 10,000 files
- API documentation page MUST load within 2 seconds
- Sandbox requests MUST have configurable timeout (default 30s)

### Security
- Sandbox operations SHOULD be isolated from production data
- Directory analysis MUST validate paths to prevent directory traversal attacks
- Configuration files MUST be validated before writing

### Accessibility
- Setup Wizard MUST be keyboard navigable
- All form inputs MUST have proper labels
- Error messages MUST be clear and actionable

### Browser Support
- Dashboard MUST work on Chrome, Firefox, Safari, and Edge (latest 2 versions)
- API Documentation MUST render correctly on mobile devices
