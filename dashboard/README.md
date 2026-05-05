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
- **Console design system** - Hand-rolled CSS in `src/styles/console.css` + primitives in `src/components/console/`
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
- Console design system (`src/styles/console.css` + `src/components/console/*`) for styling

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
- [React Router Documentation](https://reactrouter.com/)
- [Untitled UI](https://www.untitledui.com/) (icons only)

## 🤝 Contributing

When adding new features:

1. Create components in `src/components/`
2. Add pages in `src/pages/`
3. Update router in `src/router/AppRouter.tsx`
4. Add API methods in `src/lib/api-client.ts` if needed
5. Update this README if adding major features

## Console design system

Tailwind has been **fully removed** from the dashboard. The single source
of truth for styling is the console design system:

- `src/styles/console.css` — design tokens (`--bg-1`, `--border`, `--teal`,
  `--magenta`, `--green`, `--red`, `--amber`, `--text`, `--text-2`, etc.),
  shell layout, `.btn`, `.card`, `.tbl`, `.input`, `.pill`, `.spinner`, and
  the rest of the console primitives' base classes.
- `src/components/console/*` — typed React primitives (`Card`, `CardHead`,
  `CardBody`, `Tbl`, `Th`, `Td`, `Pill`, `Modal`, `Field`, `Kpi`, `Bar`,
  `Sparkline`, `Ring`, `StatusPill`, `KeyValue`, `HexLogo`,
  `ConsoleLayout`, `ConsoleSidebar`, `ConsoleTopbar`, `CommandPalette`).

Shared `ui/*` wrappers (`Button`, `Card`, `Table`, `Modal`, `Input`,
`Select`, `Checkbox`, `Toast`, `StatCard`, etc.) are thin pass-throughs over
the console primitives, preserving their legacy prop API so consumers that
still use `<Button>` / `<Modal>` keep working.

### Conventions for new code

- Compose styling via console class names (`btn`, `btn primary`, `card`,
  `tbl`, `pill`, `input`, `icon-btn`, `spinner`).
- Use console CSS variables (`var(--bg-1)`, `var(--teal)`, etc.) inside
  inline `style={{ }}` for one-off layout/escape-hatch styling.
- Prefer a console primitive (`Card`, `Pill`, `Tbl`, `Modal`) over inline
  HTML+CSS when one exists.
- **Do not** add Tailwind packages or utility classes back into the
  codebase. The repo is Tailwind-free and the Tailwind plugin is no longer
  wired into `vite.config.ts`.

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

