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

