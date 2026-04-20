# Setup Wizard Implementation Tasks

## 1. Backend - Core Setup (COMPLETED)
- [x] 1.1 Create `project_analyzer` module in `src/workspace/`
- [x] 1.2 Implement language and framework detection
- [x] 1.3 Create Setup API endpoints (`status`, `analyze`, `apply`, `verify`)
- [x] 1.4 Implement error handling and response types
- [x] 1.5 Register new routes in `src/server/mod.rs`

## 2. Dashboard - Setup Wizard (COMPLETED)
- [x] 2.1 Create `useSetup` hook
- [x] 2.2 Implement `SetupWizardPage` component
- [x] 2.3 Add `/setup` route to `AppRouter`
- [x] 2.4 Verify UI flow and API integration

## 3. Auto-Redirect & First Start Experience (COMPLETED)
- [x] 3.1 Add startup detection for `needs_setup` status
- [x] 3.2 Display wizard URL in terminal on first start
- [x] 3.3 Implement auto-redirect in dashboard when `needs_setup=true`
- [x] 3.4 Add "Open Setup Wizard" button in dashboard header when unconfigured
- [x] 3.5 Create welcome banner for first-time users

## 4. Configuration Templates (COMPLETED)
- [x] 4.1 Create template definitions (RAG, Code Search, Documentation, Custom)
- [x] 4.2 Add template selector step in Setup Wizard
- [x] 4.3 Implement template preview with expected collections
- [x] 4.4 Add "Quick Setup" option with one-click templates

## 5. API Documentation Page (COMPLETED)
- [x] 5.1 Create OpenAPI/Swagger spec endpoint (docs/api/openapi.yaml + json mirror)
- [x] 5.2 Implement API Documentation page in dashboard
- [x] 5.3 Integrate Swagger UI or Redoc component (custom implementation with sandbox)
- [x] 5.4 Add endpoint categorization (Collections, Vectors, Search, Discovery)
- [x] 5.5 Implement search/filter for API endpoints

## 6. API Sandbox (COMPLETED)
- [x] 6.1 Create sandbox API handlers with isolated test data
- [x] 6.2 Implement request builder component (method, URL, headers, body)
- [x] 6.3 Create response viewer with syntax highlighting
- [x] 6.4 Add code examples generator (curl, Python, TypeScript, Rust, Go)
- [x] 6.5 Implement "Try it out" button on each endpoint documentation
- [x] 6.6 Add request history and favorites (useSandboxHistory hook + per-endpoint panel + star toggle)

## 7. CLI Implementation (COMPLETED)
- [x] 7.1 Implement `vectorizer setup` command
- [x] 7.2 Add interactive terminal prompts for setup
- [x] 7.3 Implement `vectorizer setup --wizard` to open web browser
- [x] 7.4 Add `vectorizer docs` command to open API documentation
- [x] 7.5 Reuse `project_analyzer` logic for CLI

## 8. Enhanced Wizard UX (COMPLETED)
- [x] 8.1 Add configuration preview (YAML output) before applying
- [x] 8.2 Implement real-time validation feedback (debounced path validation, duplicate-name detection across selected collections, semantic color styling)
- [x] 8.3 Add "Add Another Project" flow without leaving wizard
- [x] 8.4 Create progress persistence (resume interrupted wizard via useWizardProgress hook + welcome-step resume banner)
- [x] 8.5 Add success animations and celebratory feedback
- [x] 8.6 Glassmorphism visual design with animated background
- [x] 8.7 Bypass-Setup option with confirmation modal
- [x] 8.8 GraphRAG toggle for collections
- [x] 8.9 Dedicated WizardLayout without sidebar

## 9. Testing Phase (COMPLETED)
- [x] 9.1 Write unit tests for `project_analyzer` (4 tests in crates/vectorizer/src/workspace/project_analyzer.rs)
- [x] 9.2 Write unit tests for template configurations (5 tests in crates/vectorizer/src/workspace/templates.rs)
- [x] 9.3 Write integration tests for Setup API endpoints (6 tests in crates/vectorizer-server/src/server/setup_handlers.rs #[cfg(test)] mod tests)
- [x] 9.4 Write integration tests for API documentation endpoints (5 tests in crates/vectorizer-server/tests/api_docs_spec.rs)
- [x] 9.5 Verify dashboard wizard with E2E tests (Playwright spec at dashboard/e2e/setup-wizard.spec.ts + 12 hook unit tests)
- [x] 9.6 Test CLI setup command (2 tests in crates/vectorizer-cli/src/cli/setup.rs)

## 10. Documentation Phase (COMPLETED)
- [x] 10.1 Create Setup Wizard user guide in `docs/users/getting-started/SETUP_WIZARD.md`
- [x] 10.2 Document API Sandbox usage (docs/users/getting-started/API_SANDBOX.md)
- [x] 10.3 Update Quick Start guide with wizard instructions
- [x] 10.4 Add configuration templates reference (included in SETUP_WIZARD.md)
- [x] 10.5 Update CHANGELOG.md

## 11. Embedded Dashboard (COMPLETED - v2.3.0)
- [x] 11.1 Add rust-embed for static asset embedding
- [x] 11.2 Create embedded_assets.rs module with file serving
- [x] 11.3 Implement SPA fallback routing for React Router
- [x] 11.4 Add optimized cache headers (immutable for fingerprinted assets)
- [x] 11.5 Remove ServeDir dependency for dashboard
- [x] 11.6 Test single binary distribution

## Progress Summary
- Total tasks: 51
- Completed: 51
- Progress: 100%

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation (API_SANDBOX.md)
- [x] Write tests covering the new behavior (12 dashboard hook tests, 6 setup_handlers tests, 5 openapi tests, 2 CLI tests)
- [x] Run tests and confirm they pass (dashboard 32/32, workspace 36/36, setup_handlers 6/6, api_docs_spec 5/5, CLI setup 2/2)
