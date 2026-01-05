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
- [ ] 3.4 Add "Open Setup Wizard" button in dashboard header when unconfigured
- [ ] 3.5 Create welcome banner for first-time users

## 4. Configuration Templates (COMPLETED)
- [x] 4.1 Create template definitions (RAG, Code Search, Documentation, Custom)
- [x] 4.2 Add template selector step in Setup Wizard
- [x] 4.3 Implement template preview with expected collections
- [ ] 4.4 Add "Quick Setup" option with one-click templates

## 5. API Documentation Page
- [ ] 5.1 Create OpenAPI/Swagger spec endpoint (`/api/docs/openapi.json`)
- [ ] 5.2 Implement API Documentation page in dashboard
- [ ] 5.3 Integrate Swagger UI or Redoc component
- [ ] 5.4 Add endpoint categorization (Collections, Vectors, Search, Discovery, etc.)
- [ ] 5.5 Implement search/filter for API endpoints

## 6. API Sandbox
- [ ] 6.1 Create sandbox API handlers with isolated test data
- [ ] 6.2 Implement request builder component (method, URL, headers, body)
- [ ] 6.3 Create response viewer with syntax highlighting
- [ ] 6.4 Add code examples generator (curl, Python, TypeScript, Rust, Go)
- [ ] 6.5 Implement "Try it out" button on each endpoint documentation
- [ ] 6.6 Add request history and favorites

## 7. CLI Implementation (COMPLETED)
- [x] 7.1 Implement `vectorizer setup` command
- [x] 7.2 Add interactive terminal prompts for setup
- [x] 7.3 Implement `vectorizer setup --wizard` to open web browser
- [x] 7.4 Add `vectorizer docs` command to open API documentation
- [x] 7.5 Reuse `project_analyzer` logic for CLI

## 8. Enhanced Wizard UX
- [ ] 8.1 Add configuration preview (YAML output) before applying
- [ ] 8.2 Implement real-time validation feedback
- [x] 8.3 Add "Add Another Project" flow without leaving wizard
- [ ] 8.4 Create progress persistence (resume interrupted wizard)
- [ ] 8.5 Add success animations and celebratory feedback

## 9. Testing Phase
- [ ] 9.1 Write unit tests for `project_analyzer`
- [ ] 9.2 Write unit tests for template configurations
- [ ] 9.3 Write integration tests for Setup API endpoints
- [ ] 9.4 Write integration tests for API documentation endpoints
- [ ] 9.5 Verify dashboard wizard with E2E tests
- [ ] 9.6 Test CLI setup command

## 10. Documentation Phase
- [ ] 10.1 Create Setup Wizard user guide in `docs/users/getting-started/`
- [ ] 10.2 Document API Sandbox usage
- [ ] 10.3 Update Quick Start guide with wizard instructions
- [ ] 10.4 Add configuration templates reference
- [ ] 10.5 Update CHANGELOG.md

## Progress Summary
- Total tasks: 45
- Completed: 22 (Phases 1-4 core, Phase 7, partial Phase 3 & 8)
- In progress: 0
- Pending: 23 (Phases 5-6, 8-10, remaining Phase 3)
- Progress: ~49%
