# Vectorizer Docker Guide

## Quick Start

### Basic Usage (No Persistence)

```bash
docker run -p 15002:15002 ghcr.io/hivellm/vectorizer:latest
```

Access:
- **MCP Server**: http://localhost:15002/mcp
- **REST API**: http://localhost:15002
- **Dashboard**: http://localhost:15002/
- **UMICP**: http://localhost:15002/umicp

### With Persistent Data

```bash
docker run -p 15002:15002 \
  -v $(pwd)/vectorizer-data:/vectorizer/data \
  -v $(pwd)/vectorizer-storage:/vectorizer/storage \
  ghcr.io/hivellm/vectorizer:latest
```

### With Workspace Configuration

1. Create your workspace config:
```bash
cp vectorize-workspace.example.yml vectorize-workspace.yml
# Edit vectorize-workspace.yml with your project paths
```

2. Run with workspace mounted:
```bash
docker run -p 15002:15002 \
  -v $(pwd)/vectorizer-data:/vectorizer/data \
  -v $(pwd)/vectorizer-storage:/vectorizer/storage \
  -v $(pwd)/vectorize-workspace.yml:/vectorizer/vectorize-workspace.yml:ro \
  -v $(pwd)/src:/workspace/src:ro \
  -v $(pwd)/docs:/workspace/docs:ro \
  ghcr.io/hivellm/vectorizer:latest
```

**Note**: Adjust the mounted volumes (`src`, `docs`) to match your `watch_directories` in the workspace config.

### With Monorepo / Multiple Projects

If you need to index multiple projects (e.g., HiveLLM monorepo structure where vectorizer is a subproject):

```bash
# Mount the entire parent directory to access sibling projects
docker run -p 15002:15002 \
  -v $(pwd)/vectorizer-data:/vectorizer/data \
  -v $(pwd)/vectorizer-storage:/vectorizer/storage \
  -v $(pwd)/vectorize-workspace.yml:/vectorizer/vectorize-workspace.yml:ro \
  -v $(pwd)/../..:/workspace:ro \
  ghcr.io/hivellm/vectorizer:latest
```

Then in your `vectorize-workspace.yml`:
```yaml
global_settings:
  file_watcher:
    auto_discovery: true
    enable_auto_update: true
    hot_reload: true
    watch_paths:
      - /workspace

projects:
  - name: vectorizer
    path: /workspace/vectorizer
    description: Vectorizer - Vector database
    collections:
      - name: vectorizer-source
        description: Rust source code
        include_patterns:
          - "src/**/*.rs"
          - "**/*.toml"
        exclude_patterns:
          - "target/**"
    
  - name: governance
    path: /workspace/governance
    description: Governance System
    collections:
      - name: governance-source
        description: TypeScript source code
        include_patterns:
          - "src/**/*.ts"
          - "**/*.json"
        exclude_patterns:
          - "node_modules/**"
    
  - name: task-queue
    path: /workspace/task-queue
    description: Task Queue
    collections:
      - name: task-queue-source
        description: Rust source code
        include_patterns:
          - "src/**/*.rs"
          - "**/*.toml"
        exclude_patterns:
          - "target/**"
```

**Security Note**: Mounting `../../` gives the container read access to all parent directories. Always use read-only (`:ro`) and be mindful of sensitive data.

## Environment Variables

```bash
docker run -p 15002:15002 \
  -e VECTORIZER_HOST=0.0.0.0 \
  -e VECTORIZER_PORT=15002 \
  -e TZ=America/New_York \
  ghcr.io/hivellm/vectorizer:latest
```

| Variable | Default | Description |
|----------|---------|-------------|
| `VECTORIZER_HOST` | `0.0.0.0` | Server host |
| `VECTORIZER_PORT` | `15002` | Server port |
| `TZ` | `Etc/UTC` | Timezone |
| `RUN_MODE` | `production` | Run mode |

## Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  vectorizer:
    image: ghcr.io/hivellm/vectorizer:latest
    ports:
      - "15002:15002"
    volumes:
      - ./vectorizer-data:/vectorizer/data
      - ./vectorizer-storage:/vectorizer/storage
      - ./vectorizer-snapshots:/vectorizer/snapshots
      - ./vectorize-workspace.yml:/vectorizer/vectorize-workspace.yml:ro
      - ./src:/workspace/src:ro
      - ./docs:/workspace/docs:ro
    environment:
      - VECTORIZER_HOST=0.0.0.0
      - VECTORIZER_PORT=15002
      - TZ=Etc/UTC
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:15002/"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
```

Run:
```bash
docker-compose up -d
```

### Docker Compose for Monorepo

For HiveLLM monorepo structure:

```yaml
version: '3.8'

services:
  vectorizer:
    image: ghcr.io/hivellm/vectorizer:latest
    ports:
      - "15002:15002"
    volumes:
      # Data volumes
      - ./vectorizer-data:/vectorizer/data
      - ./vectorizer-storage:/vectorizer/storage
      - ./vectorizer-snapshots:/vectorizer/snapshots
      
      # Workspace config
      - ./vectorize-workspace.yml:/vectorizer/vectorize-workspace.yml:ro
      
      # Mount entire monorepo (read-only)
      - ../../:/workspace:ro
    environment:
      - VECTORIZER_HOST=0.0.0.0
      - VECTORIZER_PORT=15002
      - TZ=Etc/UTC
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:15002/"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
```

## Volume Structure

```
.
├── vectorizer-data/          # Collection data (.vecdb format)
│   └── vectorizer.vecdb      # Compressed archive
├── vectorizer-storage/       # Additional storage
├── vectorizer-snapshots/     # Automatic snapshots
└── vectorize-workspace.yml   # Workspace configuration
```

## Unprivileged Image

For enhanced security, use the unprivileged variant:

```bash
docker run -p 15002:15002 \
  --user 1000:1000 \
  -v $(pwd)/vectorizer-data:/vectorizer/data \
  ghcr.io/hivellm/vectorizer:latest-unprivileged
```

## Building from Source

```bash
# Single platform
docker build -t vectorizer:local .

# Multi-platform
docker buildx build --platform linux/amd64,linux/arm64 -t vectorizer:local .

# With specific features
docker build --build-arg FEATURES="wgpu-gpu" -t vectorizer:gpu .
```

## Troubleshooting

### Container exits immediately
Check logs:
```bash
docker logs <container-id>
```

### Permission denied on volumes
Ensure the mounted directories are writable:
```bash
chmod -R 755 vectorizer-data vectorizer-storage vectorizer-snapshots
```

Or use the unprivileged image with matching user ID:
```bash
docker run --user $(id -u):$(id -g) ...
```

### Workspace not loading
1. Verify the workspace file is mounted correctly:
```bash
docker exec <container-id> cat /vectorizer/vectorize-workspace.yml
```

2. Check that watched directories are mounted:
```bash
docker exec <container-id> ls -la /workspace/
```

3. Ensure paths in `vectorize-workspace.yml` match mounted volumes.

### Data not persisting
Ensure volumes are mounted correctly:
```bash
docker inspect <container-id> | grep Mounts -A 20
```

## Advanced Usage

### Custom Entrypoint

```bash
docker run -p 15002:15002 \
  --entrypoint /bin/bash \
  ghcr.io/hivellm/vectorizer:latest \
  -c "ls -la && ./entrypoint.sh"
```

### Development Mode

```bash
docker run -p 15002:15002 \
  -v $(pwd):/workspace:ro \
  -v $(pwd)/vectorizer-data:/vectorizer/data \
  --rm -it \
  ghcr.io/hivellm/vectorizer:latest
```

### Network Mode

```bash
docker run --network host \
  -v $(pwd)/vectorizer-data:/vectorizer/data \
  ghcr.io/hivellm/vectorizer:latest
```

## Security Best Practices

1. **Use unprivileged images** in production
2. **Mount volumes read-only** when possible (`:ro`)
3. **Limit container resources**:
```bash
docker run -p 15002:15002 \
  --memory="2g" \
  --cpus="2" \
  ghcr.io/hivellm/vectorizer:latest
```
4. **Use secrets for sensitive data** instead of environment variables
5. **Run behind a reverse proxy** (nginx, traefik) for TLS termination

## Migration from v0.9.x

The container automatically handles migration from legacy `.bin` format to the new `.vecdb` format on first run. The migration is interactive; if running in a non-TTY environment, it defaults to "yes".

To force migration in automated deployments, the container will auto-migrate and create a backup in `./data/.bak.<timestamp>`.

