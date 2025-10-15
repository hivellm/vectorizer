# Changelog - Vectorizer GUI

All notable changes to the Vectorizer GUI will be documented in this file.

## [0.1.0] - 2025-10-15

### Added
- Initial release of Vectorizer GUI
- Connection management (local and remote vectorizer instances)
- Collection browser and management
- Semantic search with multiple algorithms (Basic, Semantic, Intelligent, Discover)
- Workspace manager with auto-indexing
- Configuration editor with YAML support
- Real-time logs viewer
- Backup and snapshot management
- Cross-platform support (Windows MSI, macOS DMG, Linux DEB)
- Windows Service integration
- macOS LaunchAgent support
- Linux systemd daemon
- TypeScript implementation with full type safety
- Integration with @hivellm/vectorizer-client SDK

### Features
- **Connection Manager**: Connect to local or remote Vectorizer instances
- **Dashboard**: Overview of collections, vectors, and system status
- **Collections**: Browse, create, and delete collections
- **Search**: Perform semantic searches across collections
- **Workspace**: Add directories for real-time indexing
- **Configuration**: Edit config.yml with visual interface
- **Logs**: Monitor vectorizer logs in real-time
- **Backups**: Create and restore snapshots

### Technical
- Built with Electron 28.0
- Vue 3 for reactive UI
- TypeScript for type safety
- Pinia for state management
- Vite for fast builds
- electron-builder for packaging

