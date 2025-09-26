# Vectorizer Scripts

This directory contains scripts for managing Vectorizer servers across different operating systems.

## Scripts Overview

### Linux/macOS Scripts
- `start.sh` - Start all Vectorizer servers (GRPC architecture) - Production mode
- `start-dev.sh` - Start all Vectorizer servers (GRPC architecture) - Development mode
- `stop.sh` - Stop all Vectorizer servers - Production mode
- `stop-dev.sh` - Stop all Vectorizer servers - Development mode
- `status.sh` - Check status of all servers
- `build.sh` - Build optimized binaries for production

### Windows Scripts
- `start.bat` - Start all Vectorizer servers (GRPC architecture) - Production mode
- `start-dev.bat` - Start all Vectorizer servers (GRPC architecture) - Development mode
- `stop.bat` - Stop all Vectorizer servers - Production mode
- `stop-dev.bat` - Stop all Vectorizer servers - Development mode
- `status.bat` - Check status of all servers
- `build.bat` - Build optimized binaries for production

## Architecture (v0.13.0)

The scripts support the new GRPC-based microservices architecture:

```
Client → REST/MCP → GRPC → vzr → Vector Store
```

### Services
- **vzr** (Port 15003): GRPC orchestrator and indexing engine
- **vectorizer-server** (Port 15001): REST API server (GRPC client)
- **vectorizer-mcp-server** (Port 15002): MCP server (GRPC client)

## Usage

### Development Mode (Always uses cargo run)
```bash
# Linux/macOS
./scripts/start-dev.sh

# Windows
scripts\start-dev.bat
```

### Production Mode (Uses compiled binaries when available)
```bash
# Build optimized binaries
./scripts/build.sh    # Linux/macOS
scripts\build.bat     # Windows

# Start with compiled binaries (falls back to cargo run if not available)
./scripts/start.sh    # Linux/macOS
scripts\start.bat     # Windows
```

## Features

### Automatic Binary Detection
- Scripts automatically detect if compiled binaries exist
- Falls back to `cargo run` if binaries not found
- Shows clear indication of which mode is being used

### Cross-Platform Support
- Linux, macOS, and Windows support
- OS-specific binary extensions (.exe on Windows)
- Platform-appropriate process management

### GRPC Architecture Support
- Starts vzr orchestrator first (GRPC server)
- Waits for proper initialization before starting clients
- Proper service dependency management

### Production Ready
- Uses optimized release binaries when available
- Proper process management and cleanup
- Health checks and status monitoring

## Examples

### Start with specific workspace
```bash
# Linux/macOS
./scripts/start.sh --workspace ../my-project/vectorize-workspace.yml
./scripts/start-dev.sh --workspace ../my-project/vectorize-workspace.yml

# Windows
scripts\start.bat --workspace ..\my-project\vectorize-workspace.yml
scripts\start-dev.bat --workspace ..\my-project\vectorize-workspace.yml
```

### Check server status
```bash
# Linux/macOS
./scripts/status.sh

# Windows
scripts\status.bat
```

### Stop servers
```bash
# Linux/macOS
./scripts/stop.sh      # Production mode
./scripts/stop-dev.sh   # Development mode

# Windows
scripts\stop.bat        # Production mode
scripts\stop-dev.bat    # Development mode
```

## Performance Benefits

When using compiled binaries:
- **Faster startup**: No compilation overhead
- **Lower memory usage**: Optimized release builds
- **Better performance**: Release optimizations enabled
- **Production ready**: Stable, optimized binaries

## Troubleshooting

### Binaries not found
If you see "Compiled binaries not found", run:
```bash
./scripts/build.sh    # Linux/macOS
scripts\build.bat     # Windows
```

### Port conflicts
Scripts automatically detect and kill processes using Vectorizer ports (15001, 15002, 15003).

### Service startup order
Scripts ensure proper startup order:
1. vzr orchestrator (GRPC server)
2. MCP server (GRPC client)
3. REST server (GRPC client)

This ensures GRPC communication is properly established before clients attempt to connect.
