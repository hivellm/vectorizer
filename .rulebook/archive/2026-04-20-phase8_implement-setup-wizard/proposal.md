# Proposal: Implement Setup Wizard with API Documentation Sandbox

## Why

Currently, setting up Vectorizer requires manual configuration of `workspace.yml`, which is error-prone and daunting for new users. Additionally, understanding the extensive API capabilities requires reading documentation externally, making it harder for users to experiment and learn.

A comprehensive Setup Wizard with integrated API Documentation and Sandbox is needed to:
1. **Automate first-time configuration** - Detect projects, languages, and frameworks automatically
2. **Reduce time-to-value** - From minutes of manual config editing to seconds of guided setup
3. **Improve API discoverability** - Interactive documentation with live testing sandbox
4. **Lower barrier to entry** - Make Vectorizer accessible to users of all skill levels

## What Changes

This task implements a complete Setup Wizard and API Documentation Sandbox across the stack:

### Backend Changes
- **ADDED**: Auto-redirect detection on first start with terminal guidance
- **ADDED**: API Documentation endpoints serving OpenAPI/Swagger spec
- **ADDED**: Sandbox API endpoints for safe testing with isolated data
- **MODIFIED**: Server startup to check setup status and display wizard URL
- **MODIFIED**: `project_analyzer` module with enhanced detection capabilities
- **ADDED**: Template configurations for common use cases (RAG, Code Search, Documentation)

### Frontend/Dashboard Changes
- **ADDED**: Auto-redirect to Setup Wizard on first access when `needs_setup=true`
- **ADDED**: API Documentation page with interactive Swagger UI
- **ADDED**: API Sandbox with request builder and response viewer
- **ADDED**: Code examples generator (curl, Python, TypeScript, Rust, Go)
- **ADDED**: Configuration templates selector in Setup Wizard
- **MODIFIED**: Enhanced Setup Wizard UX with better feedback and validation

### CLI Changes
- **ADDED**: `vectorizer setup` command for terminal-based interactive setup
- **ADDED**: `vectorizer setup --wizard` to open web wizard in browser
- **ADDED**: `vectorizer docs` command to open API documentation

### Documentation Changes
- **ADDED**: Setup Wizard user guide
- **ADDED**: API Sandbox usage guide
- **MODIFIED**: Getting Started guide with wizard instructions

## Impact

- **Affected specs**: `setup` (new/enhanced)
- **Affected code**: 
  - `src/workspace/*` (enhanced)
  - `src/server/*` (enhanced + new handlers)
  - `src/cli/*` (new commands)
  - `dashboard/*` (new pages + components)
- **Breaking change**: NO
- **User benefit**: 
  - Reduces setup time from minutes to seconds
  - Provides interactive API learning environment
  - Enables safe experimentation with sandbox
  - Improves onboarding experience significantly

## Success Metrics

1. **First-time setup completion rate**: Target 95%+ users complete wizard successfully
2. **Time to first search**: Target < 5 minutes from installation to first query
3. **API sandbox usage**: Track engagement with documentation sandbox
4. **Support ticket reduction**: Expect 50%+ reduction in setup-related issues
