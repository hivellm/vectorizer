# Development Guide - Vectorizer GUI

## Setup Development Environment

### Prerequisites

- Node.js 20+ (recommended) or 16+
- pnpm (recommended) or npm
- Rust toolchain (for building vectorizer)
- Git

### Initial Setup

```bash
# Clone repository
cd vectorizer/gui

# Install dependencies
pnpm install

# Build vectorizer binary (run from vectorizer root)
cd ..
cargo build --release
cd gui
```

## Development Workflow

### Start Development Server

```bash
# Start all in one command (recommended)
pnpm dev

# This will:
# 1. Start Vite dev server (port 5173)
# 2. Wait for dev server to be ready
# 3. Compile TypeScript for main process
# 4. Launch Electron with hot-reload
```

### Manual Development

If you prefer to run components separately:

```bash
# Terminal 1: Vite dev server
pnpm dev:vite

# Terminal 2: Compile main process (watch mode)
pnpm build:main -- --watch

# Terminal 3: Start Electron
pnpm dev:electron
```

## Project Structure

```
gui/
├── src/
│   ├── main/                    # Electron main process
│   │   ├── main.ts              # Main entry (window management)
│   │   ├── preload.ts           # Context bridge (security)
│   │   ├── vectorizer-manager.ts # Process manager
│   │   └── index.d.ts           # Type definitions
│   │
│   ├── renderer/                # Vue 3 application
│   │   ├── App.vue              # Root component
│   │   ├── main.ts              # Renderer entry
│   │   ├── router.ts            # Vue Router config
│   │   │
│   │   ├── views/               # Page components
│   │   │   ├── Dashboard.vue
│   │   │   ├── ConnectionManager.vue
│   │   │   ├── CollectionDetail.vue
│   │   │   ├── WorkspaceManager.vue
│   │   │   ├── ConfigEditor.vue
│   │   │   ├── LogsViewer.vue
│   │   │   └── BackupManager.vue
│   │   │
│   │   ├── components/          # Reusable components
│   │   │   ├── ToastContainer.vue
│   │   │   ├── LoadingSpinner.vue
│   │   │   └── Modal.vue
│   │   │
│   │   ├── stores/              # Pinia stores
│   │   │   ├── connections.ts
│   │   │   └── vectorizer.ts
│   │   │
│   │   ├── composables/         # Vue composables
│   │   │   ├── useAutoSave.ts
│   │   │   └── useToast.ts
│   │   │
│   │   ├── styles/              # Global styles
│   │   │   └── main.css
│   │   │
│   │   └── types/               # Type definitions
│   │       └── electron.d.ts
│   │
│   └── shared/                  # Shared code
│       └── types.ts             # Shared TypeScript types
│
├── assets/                      # Static assets
│   └── icons/                   # Application icons
│
├── build/                       # Build resources
├── installers/                  # Platform-specific installers
├── build-scripts/               # Build automation
│
├── index.html                   # HTML entry point
├── package.json                 # NPM package config
├── tsconfig.json                # TS config (renderer)
├── tsconfig.main.json           # TS config (main)
├── vite.config.js               # Vite config
├── electron-builder.yml         # Electron builder config
└── dev-runner.js                # Development runner
```

## Key Technologies

### Main Process (Electron)
- **Electron**: Desktop application framework
- **TypeScript**: Type-safe development
- **VectorizerManager**: Controls vectorizer subprocess

### Renderer Process (Vue 3)
- **Vue 3**: Composition API
- **TypeScript**: Full type safety
- **Pinia**: State management
- **Vue Router**: Navigation
- **@hivellm/vectorizer-client**: API communication
- **@vueuse/core**: Utility functions

### Build & Package
- **Vite**: Fast bundler for renderer
- **TypeScript Compiler**: Compiles main process
- **electron-builder**: Creates installers

## Development Tips

### Hot Reload

- Vue components: Auto-reload
- TypeScript (renderer): Auto-reload
- TypeScript (main): Requires manual restart

To restart Electron after main process changes:
1. Close Electron window
2. Recompile: `pnpm build:main`
3. Restart: `pnpm dev:electron`

### Debugging

#### Renderer Process
- Open DevTools automatically in development
- Use Vue DevTools extension
- Console logs visible in DevTools

#### Main Process
- Use VSCode debugger
- Add `console.log()` statements
- Logs appear in terminal

#### Vectorizer Process
- Logs captured in VectorizerManager
- View in GUI Logs page
- Or check vectorizer log files

### Type Checking

```bash
# Check types without emitting files
pnpm type-check

# Watch mode
pnpm type-check -- --watch
```

### Linting

```bash
# Add ESLint (optional)
pnpm add -D eslint @typescript-eslint/parser @typescript-eslint/eslint-plugin

# Create .eslintrc.json
# Run linting
pnpm eslint src --ext .ts,.vue
```

## API Integration

### Using @hivellm/vectorizer-client

The GUI uses the official TypeScript SDK:

```typescript
import { VectorizerClient } from '@hivellm/vectorizer-client';

// Initialize client
const client = new VectorizerClient({
  baseURL: 'http://localhost:15002',
  apiKey: 'optional-token',
  timeout: 30000
});

// Use client
const collections = await client.listCollections();
const results = await client.searchByText('my-collection', {
  query: 'search query',
  limit: 10
});
```

### Adding New API Methods

1. Check if method exists in SDK
2. If not, add to `vectorizer/client-sdks/typescript`
3. Use in store or component

## State Management

### Pinia Stores

#### connections.ts
- Manages connection configurations
- Health checking
- Active connection switching

#### vectorizer.ts  
- Wraps @hivellm/vectorizer-client
- Collections CRUD
- Search operations
- Loading states

### Usage in Components

```vue
<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { useVectorizerStore } from '@/stores/vectorizer';

const store = useVectorizerStore();
const { collections, loading } = storeToRefs(store);

// Call actions
await store.loadCollections();
</script>
```

## Adding New Features

### 1. Add API Endpoint (Rust)

Edit `vectorizer/src/server/rest_handlers.rs`:

```rust
pub async fn my_new_endpoint(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // Implementation
    Ok(Json(json!({"result": "success"})))
}
```

Add route in `vectorizer/src/server/mod.rs`:

```rust
.route("/api/my-endpoint", post(rest_handlers::my_new_endpoint))
```

### 2. Add Store Method

Edit store (e.g., `src/renderer/stores/vectorizer.ts`):

```typescript
async function myNewFeature(): Promise<void> {
  if (!client.value) throw new Error('Not connected');
  
  const response = await client.value.request('/api/my-endpoint', {
    method: 'POST',
    body: JSON.stringify({ data: 'value' })
  });
  
  // Update state
}
```

### 3. Add UI Component

Create Vue component in `src/renderer/views/`:

```vue
<template>
  <div class="my-feature">
    <h1>My Feature</h1>
    <button @click="doSomething">Action</button>
  </div>
</template>

<script setup lang="ts">
import { useVectorizerStore } from '@/stores/vectorizer';

const store = useVectorizerStore();

async function doSomething() {
  await store.myNewFeature();
}
</script>
```

### 4. Add Route

Edit `src/renderer/router.ts`:

```typescript
{
  path: '/my-feature',
  name: 'MyFeature',
  component: () => import('./views/MyFeature.vue')
}
```

## Testing

### Manual Testing

1. Start vectorizer: `cd .. && cargo run --release`
2. Start GUI: `pnpm dev`
3. Test features in GUI

### Automated Testing (TODO)

```bash
# Unit tests
pnpm test

# E2E tests
pnpm test:e2e
```

## Building for Production

### Build All Platforms

```bash
# From gui directory
pnpm electron:build

# Or use master build script
cd build-scripts
./build-all.sh    # Linux/Mac
build-all.bat     # Windows
```

### Build Specific Platform

```bash
pnpm electron:build:win     # Windows MSI
pnpm electron:build:mac     # macOS DMG
pnpm electron:build:linux   # Linux DEB
```

### Output

Built packages appear in `gui/dist-release/`:
- Windows: `Vectorizer-GUI-Setup-{version}.msi`
- macOS: `Vectorizer-GUI-{version}-{arch}.dmg`
- Linux: `vectorizer-gui_{version}_amd64.deb`

## Troubleshooting

### "Cannot find module" errors

```bash
# Reinstall dependencies
rm -rf node_modules
pnpm install
```

### TypeScript errors

```bash
# Check types
pnpm type-check

# Rebuild
pnpm clean
pnpm build
```

### Electron won't start

```bash
# Ensure main process is compiled
pnpm build:main

# Check for errors in terminal
# Check dist-electron/ directory exists
```

### Vite dev server issues

```bash
# Kill any process using port 5173
# Windows: netstat -ano | findstr :5173
# Linux/Mac: lsof -ti:5173 | xargs kill

# Restart dev server
pnpm dev:vite
```

## Performance Optimization

### Production Build

- Vite automatically minifies code
- Tree-shaking removes unused code
- Code splitting for faster loads

### Electron Optimization

- Use `nodeIntegration: false` (security)
- Use `contextIsolation: true` (security)
- Preload only what's needed

### Vue Optimization

- Use `v-show` instead of `v-if` for frequently toggled elements
- Lazy load routes with dynamic imports
- Use computed properties for derived state

## Contributing

1. Create feature branch
2. Make changes
3. Test thoroughly
4. Submit pull request

See CONTRIBUTING.md for detailed guidelines.

