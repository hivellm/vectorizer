# Refactor Dashboard with Vite + React - Proposal

## Why

The current dashboard implementation uses Vue.js 3 via CDN with vanilla JavaScript, making it difficult to maintain, test, and extend. The codebase is monolithic with all functionality in a single `app.js` file (1543 lines) and inline HTML templates, which creates several problems:

1. **Maintenance Complexity**: Single large file makes it hard to locate and fix bugs
2. **No Component Reusability**: Code duplication across different pages
3. **No Type Safety**: Vanilla JavaScript lacks type checking, leading to runtime errors
4. **Graph Visualization Issues**: Current graph-relationships page has bugs and doesn't work properly
5. **No Build System**: CDN dependencies make it hard to optimize and bundle
6. **No Modern Tooling**: Missing hot reload, TypeScript, linting, and testing infrastructure
7. **Poor Developer Experience**: No component isolation, difficult to debug

A complete refactor using Vite + React with modern tooling will provide:
- Better code organization with component-based architecture
- Type safety with TypeScript
- Modern build system with hot module replacement
- Better graph visualization with react-vis
- Consistent UI with UntitledUI component library
- Persistent routing for better UX
- Easier maintenance and testing

## What Changes

This task completely rebuilds the dashboard from scratch using modern React tooling:

1. **New Tech Stack**:
   - Vite as build tool (fast HMR, optimized builds)
   - React 18+ with TypeScript
   - React Router with persistent navigation
   - UntitledUI for consistent UI components
   - react-vis for graph visualization (replacing vis-network)
   - Modern state management (Zustand or React Context)

2. **Component Architecture**:
   - Separate components for each page/view
   - Reusable UI components (buttons, cards, modals, etc.)
   - Shared layout components (sidebar, header, etc.)
   - API client as React hooks
   - Graph visualization component

3. **Pages to Implement** (matching current functionality):
   - Overview (stats, quick actions)
   - Collections (list, create, manage)
   - Search (vector search interface)
   - Vectors (browse, view, edit)
   - File Watcher (status, configuration)
   - Graph Relationships (NEW - fully functional graph visualization)
   - Connections (server connections)
   - Workspace (project management)
   - Configuration (settings)
   - Logs (system logs viewer)
   - Backups (backup management)

4. **Graph Visualization**:
   - Fix current graph-relationships page bugs
   - Implement proper node/edge rendering with react-vis
   - Interactive graph navigation
   - Relationship filtering and search
   - Graph layout algorithms (force-directed, hierarchical)

5. **Server Integration**:
   - Update server to serve `/dashboard/dist` instead of `/dashboard`
   - Remove old dashboard directory after migration
   - Ensure API compatibility (no backend changes needed)

6. **Build & Deployment**:
   - Vite build configuration
   - Production optimizations
   - Asset bundling and code splitting
   - Environment configuration

## Impact

- **Affected specs**: 
  - `specs/api-rest/spec.md` - Dashboard API endpoints remain unchanged
- **Affected code**: 
  - **NEW**: `dashboard-v2/` directory with complete React application
  - **MODIFIED**: `src/server/mod.rs` - Update dashboard route to serve `/dashboard/dist`
  - **DELETED**: `dashboard/` directory (old implementation)
- **Breaking change**: NO (same API, same routes, same functionality)
- **User benefit**: 
  - Better performance and faster load times
  - More reliable graph visualization
  - Better mobile responsiveness
  - Improved user experience with persistent routing
  - Easier to add new features in the future

