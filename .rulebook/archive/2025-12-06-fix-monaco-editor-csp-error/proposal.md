# Proposal: fix-monaco-editor-csp-error

## Why

The Monaco Editor in the dashboard is failing to load due to Content Security Policy (CSP) violations. The editor attempts to load resources from `cdn.jsdelivr.net`, but the current CSP only allows scripts from `'self'`, causing the editor to fail initialization.

**Current Error**:
```
Loading the script 'https://cdn.jsdelivr.net/npm/monaco-editor@0.55.1/min/vs/loader.js' 
violates the following Content Security Policy directive: 
"script-src 'self' 'unsafe-inline' 'unsafe-eval'"
```

**Problem Scenarios**:
- Monaco Editor shows "Loading editor..." but never loads
- Console shows multiple CSP violation errors
- Editor falls back to textarea, losing syntax highlighting and advanced features
- User experience is degraded with basic textarea instead of full-featured editor

**Root Cause**:
- Monaco Editor uses `@monaco-editor/loader` which tries to load workers and resources from CDN
- CSP in `src/server/mod.rs` only allows `'self'` for scripts
- Monaco Editor needs to load workers from external CDN or bundled locally

## What Changes

### 1. Update Content Security Policy

Add `cdn.jsdelivr.net` to the CSP `script-src` directive to allow Monaco Editor to load its workers and resources:

**Current CSP**:
```
script-src 'self' 'unsafe-inline' 'unsafe-eval'
```

**Updated CSP**:
```
script-src 'self' 'unsafe-inline' 'unsafe-eval' https://cdn.jsdelivr.net
```

### 2. Alternative: Bundle Monaco Editor Locally (Optional)

If we want to avoid external CDN dependencies, we can configure Monaco Editor to use local bundled resources instead of CDN. This requires:
- Configuring `@monaco-editor/loader` to use local paths
- Ensuring Monaco Editor workers are bundled with the application
- Updating Vite config if needed

**Recommendation**: Start with adding CDN to CSP (simpler, faster), then consider local bundling if security requirements demand it.

### 3. Update CSP for Worker Scripts

Monaco Editor also uses Web Workers, which may need additional CSP directives:
- Add `worker-src` directive if needed
- Ensure workers can be loaded from CDN or local bundle

## Impact

- **Affected code**:
  - Modified `src/server/mod.rs` - Update CSP header in `security_headers_middleware`
- **Breaking change**: NO - Only adds allowed source, doesn't remove anything
- **User benefit**: Monaco Editor loads correctly, providing full code editing experience with syntax highlighting
