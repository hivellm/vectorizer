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
  -v $(pwd)/vectorizer-snapshots:/vectorizer/snapshots \
  -v $(pwd)/vectorizer-dashboard:/vectorizer/dashboard \
  ghcr.io/hivellm/vectorizer:latest
```

### With Monorepo Access (Recommended)

This setup allows Vectorizer to index multiple projects in a monorepo structure.

**Step 1**: Create Docker-specific workspace config:
```bash
# Copy example workspace config
cp vectorize-workspace.docker.example.yml vectorize-workspace.docker.yml

# Edit vectorize-workspace.docker.yml with /workspace/* paths
# All paths in the config should use container paths starting with /workspace/
```

**Step 2**: Run with monorepo access:
```bash
docker run -d \
  --name vectorizer \
  -p 15002:15002 \
  -p 15003:15003 \
  -v $(pwd)/vectorizer-data:/vectorizer/data \
  -v $(pwd)/vectorizer-storage:/vectorizer/storage \
  -v $(pwd)/vectorizer-snapshots:/vectorizer/snapshots \
  -v $(pwd)/vectorizer-dashboard:/vectorizer/dashboard \
  -v $(pwd)/vectorize-workspace.docker.yml:/vectorizer/vectorize-workspace.yml:ro \
  -v $(pwd)/../../:/workspace:ro \
  -e VECTORIZER_HOST=0.0.0.0 \
  -e VECTORIZER_PORT=15002 \
  -e TZ=America/Sao_Paulo \
  --restart unless-stopped \
  ghcr.io/hivellm/vectorizer:latest
```

**Step 3**: Verify workspace is loaded:
```bash
# View logs to confirm workspace loading
docker logs vectorizer | grep -i workspace

# Check if workspace file is accessible
docker exec vectorizer cat /vectorizer/vectorize-workspace.yml

# Verify mounted directories
docker exec vectorizer ls -la /workspace/
```

**Path mapping**:
- Host: `../../` → Container: `/workspace/`
- Host: `../../cmmv/cmmv` → Container: `/workspace/cmmv/cmmv`
- Host: `../../hivellm/governance` → Container: `/workspace/hivellm/governance`

**Important**: 
- The workspace file **must** be mounted to `/vectorizer/vectorize-workspace.yml`
- All paths in `vectorize-workspace.docker.yml` should use container paths (e.g., `/workspace/project-name`)
- Use `:ro` (read-only) flag for security when mounting source directories

### With Workspace Configuration

**Important**: Workspace paths are different for Docker vs. local execution:
- **Local execution**: Use relative paths like `../../cmmv/cmmv`
- **Docker execution**: Use container paths like `/workspace/cmmv/cmmv`

**Default workspace file location**: The container looks for `vectorize-workspace.yml` at `/vectorizer/vectorize-workspace.yml` (inside the container).

1. Create workspace configs:
```bash
# For local execution (relative paths)
cp vectorize-workspace.example.yml vectorize-workspace.yml

# For Docker execution (container paths /workspace/*)
cp vectorize-workspace.docker.example.yml vectorize-workspace.docker.yml

# Edit both files according to your needs
```

2. Run with Docker (mount workspace file):
```bash
docker run -d -p 15002:15002 \
  --name vectorizer \
  -v $(pwd)/vectorizer-data:/vectorizer/data \
  -v $(pwd)/vectorizer-storage:/vectorizer/storage \
  -v $(pwd)/vectorizer-snapshots:/vectorizer/snapshots \
  -v $(pwd)/vectorize-workspace.docker.yml:/vectorizer/vectorize-workspace.yml:ro \
  -v $(pwd)/src:/workspace/src:ro \
  -v $(pwd)/docs:/workspace/docs:ro \
  ghcr.io/hivellm/vectorizer:latest
```

**Note**: 
- The workspace file must be mounted to `/vectorizer/vectorize-workspace.yml` inside the container
- Adjust the mounted volumes (`src`, `docs`) to match your `watch_paths` in the workspace config
- Use `:ro` (read-only) flag for workspace file and source directories for security

3. Verify workspace is loaded:
```bash
# Check if workspace file is accessible
docker exec vectorizer cat /vectorizer/vectorize-workspace.yml

# Check if workspace directories are mounted
docker exec vectorizer ls -la /workspace/

# View logs to confirm workspace loading
docker logs vectorizer | grep -i workspace
```

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

### Production (Pre-built Image)

A `docker-compose.yml` file is already included in the repository. Just run:

```bash
# Start in background
docker-compose up -d

# View logs
docker-compose logs -f

# Stop
docker-compose down
```

The included `docker-compose.yml`:
- Uses the published image from GitHub Container Registry
- Mounts `../../` as `/workspace` for monorepo support
- Includes health checks and resource limits
- Persists data in `./vectorizer-data`, `./vectorizer-storage`, `./vectorizer-snapshots`

### Development (Build from Source)

For local development with live builds:

```bash
# Build and start
docker-compose -f docker-compose.dev.yml up --build

# Rebuild after code changes
docker-compose -f docker-compose.dev.yml up --build --force-recreate

# Stop
docker-compose -f docker-compose.dev.yml down
```

The `docker-compose.dev.yml`:
- Builds from local Dockerfile
- Includes debug logging (`RUST_LOG=debug`)
- Mounts logs directory
- Higher resource limits for development

### Customizing Docker Compose

To customize the Docker Compose configuration:

1. **Change timezone**: Edit `TZ` environment variable
2. **Adjust resource limits**: Modify `deploy.resources` section
3. **Mount different paths**: Edit volume mappings
4. **Change ports**: Modify `ports` section

Example: Use named volumes instead of bind mounts:
```yaml
volumes:
  - vectorizer-data:/vectorizer/data
  - vectorizer-storage:/vectorizer/storage

volumes:
  vectorizer-data:
  vectorizer-storage:
```

## Volume Structure

```
.
├── vectorizer-data/          # Collection data (.vecdb format)
│   └── vectorizer.vecdb      # Compressed archive
├── vectorizer-storage/       # Additional storage
├── vectorizer-snapshots/     # Automatic snapshots
├── vectorizer-dashboard/     # Dashboard data and assets
└── vectorize-workspace.yml   # Workspace configuration (must be mounted to /vectorizer/vectorize-workspace.yml)
```

### Container Directory Structure

Inside the container:
```
/vectorizer/
├── vectorizer              # Executable
├── vectorize-workspace.yml # Workspace config (default location)
├── data/                  # Collection data
├── storage/               # Additional storage
├── snapshots/             # Automatic snapshots
├── dashboard/             # Dashboard data
└── .logs/                 # Log files

/workspace/                # Mount point for source code (optional)
├── src/                   # Your source code
├── docs/                  # Documentation
└── ...                    # Other projects/directories
```

**Important**: The workspace configuration file must be mounted to `/vectorizer/vectorize-workspace.yml` for the container to find it automatically.

## Unprivileged Image

For enhanced security, use the unprivileged variant:

```bash
docker run -p 15002:15002 \
  --user 1000:1000 \
  -v $(pwd)/vectorizer-data:/vectorizer/data \
  ghcr.io/hivellm/vectorizer:latest-unprivileged
```

## Building from Source

**Important**: Always use `-t` flag to name your image, otherwise it will be unnamed (`<none>`).

```bash
# Single platform (amd64)
docker build -t vectorizer:local .

# Single platform (arm64)
docker build -t vectorizer:local-arm64 --platform linux/arm64 .

# Multi-platform (requires buildx)
docker buildx create --use  # First time only
docker buildx build --platform linux/amd64,linux/arm64 -t vectorizer:latest --load .

# With specific features
docker build -t vectorizer:gpu --build-arg FEATURES="wgpu-gpu" .

# Tag with version
docker build -t vectorizer:v0.9.6 -t vectorizer:latest .

# Build and push to registry
docker buildx build --platform linux/amd64,linux/arm64 \
  -t ghcr.io/YOUR_USERNAME/vectorizer:latest \
  -t ghcr.io/YOUR_USERNAME/vectorizer:v0.9.6 \
  --push .
```

## Troubleshooting

### Image has no name (`<none>`)

If you build without `-t` flag, the image will be unnamed:

```bash
# Wrong - image will be unnamed
docker build .

# Correct - image will be named
docker build -t vectorizer:local .
```

To fix existing unnamed images:
```bash
# Find unnamed images
docker images | grep "<none>"

# Tag an unnamed image
docker tag <image-id> vectorizer:local

# Or rebuild with proper name
docker build -t vectorizer:local .
```

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

1. **Verify the workspace file is mounted correctly**:
```bash
docker exec <container-id> cat /vectorizer/vectorize-workspace.yml
```

If the file doesn't exist, mount it:
```bash
docker run ... -v $(pwd)/vectorize-workspace.yml:/vectorizer/vectorize-workspace.yml:ro ...
```

2. **Check that watched directories are mounted**:
```bash
docker exec <container-id> ls -la /workspace/
```

3. **Ensure paths in `vectorize-workspace.yml` match mounted volumes**:
   - Container paths should start with `/workspace/` (e.g., `/workspace/src`)
   - Host paths should match your `-v` mount points
   - Example: If you mount `-v $(pwd)/src:/workspace/src:ro`, use `/workspace/src` in workspace config

4. **Check workspace loading logs**:
```bash
docker logs <container-id> | grep -i workspace
```

5. **Common issues**:
   - **File not found**: Workspace file must be at `/vectorizer/vectorize-workspace.yml` inside container
   - **Path mismatch**: Container paths in config don't match mounted volumes
   - **Permission denied**: Use `:ro` flag for read-only mounts, ensure directories are readable
   - **YAML syntax error**: Validate YAML syntax before mounting

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

