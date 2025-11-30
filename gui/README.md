# Vectorizer GUI

Desktop application for managing Hive Vectorizer - High-performance vector database.

## Features

- **Connection Management**: Connect to local or remote Vectorizer instances
- **Collection Browser**: View and manage vector collections
- **Semantic Search**: Perform searches using multiple algorithms (Basic, Semantic, Intelligent, Discover)
- **Workspace Manager**: Add directories and auto-index files in real-time
- **Configuration Editor**: Edit config.yml with visual interface
- **Logs Viewer**: Monitor vectorizer logs in real-time
- **Backup Manager**: Create and restore snapshots

## Prerequisites

- Node.js 16+ or Node.js 20+ (recommended)
- pnpm (recommended) or npm
- Vectorizer binary (automatically bundled in releases)

## Development

### Install Dependencies

```bash
pnpm install
```

### Run in Development Mode

```bash
pnpm electron:dev
```

This will:
1. Start Vite dev server on port 5173
2. Compile TypeScript for main process
3. Launch Electron with hot-reload

### Build for Production

```bash
# Build for current platform
pnpm electron:build

# Build for specific platforms
pnpm electron:build:win   # Windows (MSI)
pnpm electron:build:mac   # macOS (DMG)
pnpm electron:build:linux # Linux (DEB)
```

## Project Structure

```
gui/
├── src/
│   ├── main/              # Electron main process
│   │   ├── main.ts        # Main entry point
│   │   ├── preload.ts     # Preload script (context bridge)
│   │   └── vectorizer-manager.ts  # Vectorizer process manager
│   ├── renderer/          # Vue 3 application
│   │   ├── views/         # Page components
│   │   ├── components/    # Reusable components
│   │   ├── stores/        # Pinia stores
│   │   ├── styles/        # CSS styles
│   │   └── main.ts        # Renderer entry point
│   └── shared/            # Shared types and utilities
│       └── types.ts       # TypeScript interfaces
├── assets/
│   └── icons/             # Application icons
├── installers/            # Platform-specific installers
│   ├── windows/           # MSI installer config
│   ├── linux/             # DEB installer config
│   └── macos/             # DMG installer config
├── package.json
├── tsconfig.json          # TypeScript config for renderer
├── tsconfig.main.json     # TypeScript config for main process
└── vite.config.js         # Vite configuration
```

## Technology Stack

- **Electron**: Desktop application framework
- **Vue 3**: Progressive JavaScript framework
- **TypeScript**: Type-safe development
- **Pinia**: State management
- **Vite**: Fast build tool
- **@hivellm/vectorizer-client**: Official Vectorizer SDK

## Architecture

### Main Process (Electron)
- Manages application lifecycle
- Controls vectorizer process (start/stop/restart)
- Provides file system dialogs
- Handles IPC communication

### Renderer Process (Vue 3)
- User interface and interactions
- State management with Pinia
- API communication via @hivellm/vectorizer-client
- Real-time updates and polling

### Communication Flow

```
┌─────────────────┐
│   Renderer      │
│   (Vue 3 UI)    │
└────────┬────────┘
         │ IPC
┌────────┴────────┐      ┌──────────────┐
│   Main Process  │      │  Vectorizer  │
│   (Electron)    │◄────►│   Process    │
└─────────────────┘ REST └──────────────┘
```

## Distribution

### Windows (MSI)
- Creates service for vectorizer
- Adds desktop shortcut
- Configures auto-start (optional)

### macOS (DMG)
- App bundle with vectorizer
- LaunchAgent for daemon
- Code signing support

### Linux (DEB)
- Systemd service
- Desktop entry
- Install to /usr/local/bin/

## License

MIT License - see LICENSE file for details

