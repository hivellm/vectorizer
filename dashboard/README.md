# Vectorizer Dashboard

Modern web dashboard for Vectorizer built with Vite, React, and TypeScript.

## 🚀 Quick Start

### Development

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Dashboard available at http://localhost:5173/
```

### Production Build

```bash
# Build for production
npm run build

# Output: dashboard/dist/
# Server will serve from this directory at /dashboard/
```

## 🛠️ Tech Stack

- **Vite 7** - Fast build tool and dev server
- **React 19** - UI library
- **TypeScript 5.9** - Type safety
- **Tailwind CSS 4** - Utility-first CSS
- **React Router 7** - Client-side routing
- **Zustand** - State management
- **Untitled UI** - Design system and icons
- **Visx** - Data visualization (for graph relationships)
- **Monaco Editor** - Code editor for JSON payloads

## 📁 Project Structure

```
dashboard/
├── src/
│   ├── components/       # Reusable UI components
│   │   ├── ui/          # Base UI components (Button, Card, Modal, etc.)
│   │   ├── layout/      # Layout components (Sidebar, Header, MainLayout)
│   │   └── modals/      # Modal components
│   ├── pages/           # Page components
│   ├── hooks/           # Custom React hooks
│   ├── stores/          # Zustand stores
│   ├── lib/             # API client and utilities
│   ├── providers/       # Context providers
│   ├── router/          # React Router configuration
│   ├── styles/          # Global styles and theme
│   └── utils/           # Utility functions
├── dist/                # Production build output
├── package.json
├── vite.config.ts       # Vite configuration
└── tsconfig.json        # TypeScript configuration
```

## 🎨 Features

### Pages

- **Overview** - Dashboard overview with stats and quick actions
- **Collections** - Manage collections (create, view, delete)
- **Search** - Search vectors with text, vector, or hybrid search
- **Vectors** - Browse and manage vectors in collections
- **File Watcher** - Monitor file changes and indexing
- **Graph** - Visualize vector relationships (coming soon)
- **Connections** - Manage connections to other Vectorizer servers
- **Workspace** - Manage workspace configuration
- **Configuration** - Server configuration management
- **Logs** - View server logs with filtering
- **Backups** - Create and restore backups

### Components

- **Button** - Multiple variants and sizes
- **Card** - Container component with dark mode
- **Modal** - Reusable modal dialog
- **Toast** - Notification system
- **Table** - Data table component
- **Input/Select** - Form components
- **CodeEditor** - Monaco Editor integration for JSON
- **LoadingSpinner** - Loading states

## 🔧 Development

### Code Style

- TypeScript strict mode enabled
- ESLint for code quality
- Prettier for formatting
- Tailwind CSS for styling

### API Integration

The dashboard uses a custom API client (`src/lib/api-client.ts`) that:
- Handles authentication
- Provides type-safe API methods
- Manages error handling
- Supports request/response interceptors

### State Management

- **Zustand** for global state (collections, connections, etc.)
- **React Context** for theme and toast notifications
- **React Router** for navigation state

## 📦 Build Optimization

The dashboard is optimized for production:

- **Code Splitting**: Routes are lazy-loaded
- **Chunk Optimization**: Vendor chunks separated (React, Monaco, Visx)
- **Minification**: ESBuild for fast minification
- **Tree Shaking**: Unused code removed
- **Asset Optimization**: CSS and JS optimized

### Build Output

```
dist/
├── index.html           # Main HTML file
└── assets/
    ├── css/            # CSS files
    └── js/             # JavaScript chunks
        ├── index-*.js   # Main entry point
        ├── react-vendor-*.js  # React chunk
        ├── vendor-*.js  # Other vendors
        └── [page]-*.js # Page chunks (lazy-loaded)
```

## 🌐 Integration with Rust Server

The dashboard is served by the Rust server:

- **Base Path**: `/dashboard/` (production)
- **Static Files**: Served from `dashboard/dist/`
- **SPA Routing**: Fallback handler serves `index.html` for client-side routes
- **API Calls**: Relative paths handled by React Router

See [Dashboard Integration Guide](../docs/features/DASHBOARD_INTEGRATION.md) for details.

## 🐛 Troubleshooting

### Build Errors

```bash
# Clear cache and rebuild
rm -rf node_modules dist
npm install
npm run build
```

### Type Errors

```bash
# Check TypeScript errors
npm run build
# Fix errors shown in output
```

### Development Server Issues

```bash
# Clear Vite cache
rm -rf node_modules/.vite
npm run dev
```

## 📚 Resources

- [Vite Documentation](https://vite.dev/)
- [React Documentation](https://react.dev/)
- [Tailwind CSS Documentation](https://tailwindcss.com/)
- [React Router Documentation](https://reactrouter.com/)
- [Untitled UI](https://www.untitledui.com/)

## 🤝 Contributing

When adding new features:

1. Create components in `src/components/`
2. Add pages in `src/pages/`
3. Update router in `src/router/AppRouter.tsx`
4. Add API methods in `src/lib/api-client.ts` if needed
5. Update this README if adding major features

## Hybrid styling (console + Tailwind)

The dashboard currently runs **two style systems side-by-side**. New code
should pick the right one based on where it lives.

### Primary: console design system

Source of truth for every page and shell chrome:

- `src/styles/console.css` — design tokens, layout primitives, dark theme.
- `src/components/console/*` — `Card`, `Stat`, `Tbl`, `Btn`, `Tag`, `Field`,
  `ConsoleSidebar`, `ConsoleTopbar`, `CommandPalette`, etc.

All routes mounted under `ConsoleLayout` (Phase 1.8) and every page rewritten
in Phase 2/3 use this system **exclusively**. Pages must NOT introduce
Tailwind utility classes — use console primitives + inline `style={{ }}`
escape hatches that rely on the `--c-*` CSS variables defined in
`console.css`.

### Legacy: Tailwind v4

Still required by 24 shared components that have not been migrated to
console primitives yet:

- `src/components/ui/*` — `Modal`, `Input`, `Select`, `Checkbox`, `Dropdown`,
  `Toast`, `StatCard`, `PasswordStrengthIndicator`, `CodeEditor`.
- `src/components/modals/*` — every modal (Create/Delete/Details for
  collections, vectors, edges, files, plus `DiscoveryConfigModal`,
  `PathFinderModal`, `FileUploadModal`).
- `src/components/FileBrowser.tsx`, `WelcomeBanner.tsx`, `ProtectedRoute.tsx`,
  `ErrorBoundary.tsx`, `LoadingState.tsx`.

Tailwind v4 is wired through:

- `dashboard/src/styles/theme.css` — `@import "tailwindcss"` plus the
  `@theme {}` block that pins primary/gray to grayscale-only tokens.
- `dashboard/vite.config.ts` — `@tailwindcss/vite` plugin.
- `package.json` deps: `tailwindcss`, `@tailwindcss/vite`, `tailwind-merge`,
  `tailwindcss-animate`, `tailwindcss-react-aria-components`.

### TODO markers

Several Phase 2/3 page rewrites left `// TODO(actions)`,
`// TODO(workspace-modal)`, `// TODO(graph-modals)`, and
`// TODO(api-docs-section)` comments where modal triggers still mount the
old Tailwind-styled modals. Each marker is a future migration point: replace
the trigger with a console-native modal primitive, then delete the
Tailwind-only shared component once nothing imports it. When the last
shared component is migrated, drop Tailwind v4 entirely (delete `theme.css`,
remove the Vite plugin, prune the five `tailwindcss*` packages).

## Screenshots

The dashboard's full set of console-design pages is auto-captured by
`e2e/screenshots.spec.ts` (run with `pnpm exec playwright test
e2e/screenshots.spec.ts`). The PNGs live under `dashboard/docs/screenshots/`:

| Page | Preview |
|------|---------|
| Overview     | ![Overview](docs/screenshots/overview.png) |
| Collections  | ![Collections](docs/screenshots/collections.png) |
| Search       | ![Search](docs/screenshots/search.png) |
| Vectors      | ![Vectors](docs/screenshots/vectors.png) |
| Monitoring   | ![Monitoring](docs/screenshots/monitoring.png) |
| Replication  | ![Replication](docs/screenshots/replication.png) |
| API Keys     | ![API Keys](docs/screenshots/api-keys.png) |
| MCP Tools    | ![MCP Tools](docs/screenshots/mcp-tools.png) |
| Settings     | ![Settings](docs/screenshots/settings.png) |
| File Watcher | ![File Watcher](docs/screenshots/file-watcher.png) |
| Graph        | ![Graph](docs/screenshots/graph.png) |
| Connections  | ![Connections](docs/screenshots/connections.png) |
| Workspace    | ![Workspace](docs/screenshots/workspace.png) |
| Logs         | ![Logs](docs/screenshots/logs.png) |
| Backups      | ![Backups](docs/screenshots/backups.png) |
| Users        | ![Users](docs/screenshots/users.png) |
| API Docs     | ![API Docs](docs/screenshots/api-docs.png) |

