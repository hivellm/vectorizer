# Changelog - Vectorizer GUI

All notable changes to the Vectorizer GUI will be documented in this file.

## [Unreleased]

### Changed

- **Runs on `@hivehub/vectorizer-sdk` 3.6.0**, whose RPC transport is
  `@hivehub/thunder` 0.2.2 — the same binary protocol `vectorizer-server`
  speaks. `pnpm type-check` passes with zero errors for the first time in
  several releases.

### Fixed

- **Backups, config, logs and workspace panels talk to the server again.** All
  four views hand-rolled `fetch()` calls against `/api/backups`, `/api/config`,
  `/api/logs` and `/api/workspace/config`, borrowing the base URL from a
  `client.config` property the SDK no longer exposes. **The server has no
  `/api` prefix** — every one of those requests was a 404, so the panels were
  broken regardless of the type errors. They now call the SDK:
  `listBackupInfos`, `createBackupTyped`, `restoreBackupTyped`,
  `getBackupDirectory`, `getServerConfig`, `updateConfig`, `getLogEntries` and
  `getWorkspaceConfig`. Only the workspace-config *write* stays hand-rolled —
  the SDK has no writer for it — now against the real `/workspace/config` route
  with the base URL from the supported `client.getConfig()` accessor.
- **The typed backup methods are the ones that work.** `listBackups()` targets
  `/backups/list`, which the server does not serve, and `restoreBackup()` sends
  `{filename}` where the server reads `backup_id`. The `*Typed` variants use
  the right route and body, and `createBackupTyped` also carries the collection
  selection that the untyped call drops.
- **The backup directory shows the real path.** The response field is `path`
  (the SDK's type says `directory`), so the panel used to fall back to the
  hardcoded `./backups` on every load.
- **Text insert works with the 3.6.0 signature.** `insertText` now takes the id
  as a positional argument and answers a `Vector` whose id field is `id`; the
  store minted no id and read a `vector_id` that does not exist. Batch inserts
  fill in `BatchTextRequest.id` the same way, using the `uuid` helper the
  connections store already uses.
- **`SearchResult` matches the wire.** The local type extended `Vector`, whose
  `vector_id` is required — a field no search response carries. It now models
  what a hit actually contains (`{id, score, vector?, payload?}`), verified
  against a running 3.6.0 server rather than read off the SDK's types, which
  name the embedding `data`.

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

