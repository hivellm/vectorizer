# Dashboard Refactor Specification

## Purpose

This specification defines the requirements for refactoring the Vectorizer dashboard from Vue.js/CDN-based implementation to a modern Vite + React + TypeScript application with proper component architecture, persistent routing, and functional graph visualization.

## ADDED Requirements

### Requirement: Modern Build System
The dashboard SHALL use Vite as the build tool with React 18+ and TypeScript for type safety and modern development experience.

#### Scenario: Development Server
Given a developer runs `npm run dev` in the dashboard directory
When the development server starts
Then it SHALL provide hot module replacement and fast refresh for React components

#### Scenario: Production Build
Given a developer runs `npm run build` in the dashboard directory
When the build completes
Then it SHALL generate optimized production files in `dist/` directory

### Requirement: Component Architecture
The dashboard SHALL be organized into reusable React components following best practices.

#### Scenario: Component Reusability
Given a UI component is needed in multiple pages
When the component is created
Then it SHALL be placed in a shared components directory and reused across pages

#### Scenario: Page Components
Given a page needs to be implemented
When the page component is created
Then it SHALL be placed in `src/pages/` directory and use shared layout components

### Requirement: Persistent Routing
The dashboard SHALL use React Router with persistent navigation state.

#### Scenario: Navigation Persistence
Given a user navigates to a page
When the user refreshes the browser
Then the application SHALL maintain the current route and restore navigation state

#### Scenario: Deep Linking
Given a user accesses a direct URL to a dashboard page
When the page loads
Then it SHALL display the correct page content without requiring navigation

### Requirement: Graph Visualization
The dashboard SHALL provide a fully functional graph relationships page using react-vis.

#### Scenario: Graph Rendering
Given a collection has graph relationships
When the user navigates to Graph Relationships page
Then the graph SHALL render all nodes and edges correctly with proper layout

#### Scenario: Graph Interaction
Given a graph is displayed
When the user interacts with nodes (click, hover, drag)
Then the graph SHALL respond appropriately with visual feedback

#### Scenario: Graph Filtering
Given a graph is displayed
When the user applies filters (relationship type, collection)
Then the graph SHALL update to show only matching nodes and edges

#### Scenario: Graph Search
Given a graph is displayed
When the user searches for a node by ID
Then the graph SHALL highlight and focus on the matching node

### Requirement: UI Component Library
The dashboard SHALL use UntitledUI components for consistent styling.

#### Scenario: Component Consistency
Given a UI component is needed
When the component is created
Then it SHALL use UntitledUI components where available for consistency

#### Scenario: Theme Consistency
Given the dashboard is rendered
When components are displayed
Then they SHALL follow the UntitledUI theme and design system

### Requirement: API Integration
The dashboard SHALL use React hooks for API communication.

#### Scenario: API Client Hook
Given a component needs to fetch data
When the component uses the API hook
Then it SHALL handle loading, error, and success states automatically

#### Scenario: Data Refresh
Given data is displayed on a page
When auto-refresh is enabled
Then the data SHALL update automatically at configured intervals

### Requirement: Server Integration
The server SHALL serve the new dashboard from `/dashboard/dist` directory.

#### Scenario: Dashboard Route
Given a user accesses `/dashboard`
When the server responds
Then it SHALL serve files from `dashboard/dist/` directory

#### Scenario: Asset Serving
Given the dashboard requests static assets
When the server responds
Then it SHALL serve assets from `dashboard/dist/` with correct MIME types

## MODIFIED Requirements

### Requirement: Dashboard Directory Structure
The dashboard directory structure SHALL be reorganized for React application.

#### Scenario: Directory Organization
Given the new dashboard is created
When files are organized
Then the structure SHALL follow:
```
dashboard/
├── src/
│   ├── components/     # Reusable UI components
│   ├── pages/          # Page components
│   ├── hooks/          # React hooks (API, state)
│   ├── stores/         # State management (Zustand)
│   ├── utils/          # Utility functions
│   ├── types/          # TypeScript types
│   └── App.tsx         # Main app component
├── public/             # Static assets
├── dist/               # Build output
├── package.json
├── vite.config.ts
└── tsconfig.json
```

## REMOVED Requirements

### Requirement: Old Dashboard Implementation
The old dashboard implementation SHALL be completely removed.

#### Scenario: Old Files Removal
Given the new dashboard is fully functional
When migration is complete
Then all files in the old `dashboard/` directory SHALL be removed except for the new React application

## Pages to Implement

### Overview Page
- System statistics (collections, vectors, etc.)
- Quick actions
- System health indicators
- Auto-refresh functionality

### Collections Page
- Collections list with table view
- Create collection modal
- Collection details view
- Delete collection functionality
- Filtering and search

### Search Page
- Search input with filters
- Search results display
- Collection selector
- Search history

### Vectors Page
- Vectors browse/list view
- Vector details modal
- Vector edit functionality
- Filtering and pagination

### File Watcher Page
- File watcher status display
- Configuration interface
- Start/stop controls

### Graph Relationships Page (Critical)
- Graph visualization with react-vis
- Node and edge rendering
- Interactive graph (click, hover, drag)
- Graph layout algorithms
- Filtering by relationship type and collection
- Node search functionality
- Graph controls (zoom, pan, reset)

### Connections Page
- Connection status display
- Connection management

### Workspace Page
- Workspace project list
- Workspace management

### Configuration Page
- Settings form
- Configuration save/load

### Logs Page
- Log viewer with filtering
- Log level filtering
- Auto-scroll and refresh

### Backups Page
- Backup list display
- Backup creation
- Backup restoration

## Technical Requirements

### Dependencies
- React 18+
- TypeScript 5+
- Vite 5+
- React Router 6+
- UntitledUI components
- react-vis for graph visualization
- Zustand for state management (or React Context)

### Build Configuration
- Output directory: `dist/`
- Base path: `/dashboard/`
- Code splitting by route
- Production optimizations enabled

### Browser Support
- Modern browsers (Chrome, Firefox, Safari, Edge)
- ES2020+ support required

