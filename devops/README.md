# DevOps Configuration for Vectorizer v0.3.2

This directory contains all DevOps configurations for Vectorizer, including Docker and Kubernetes deployments.

## 📁 Directory Structure

```
devops/
├── docker/                    # Docker configurations
│   ├── Dockerfile            # CPU-only production image
│   ├── Dockerfile.dev       # Development image
│   ├── docker-compose.yml   # Multi-profile compose file
│   ├── build.sh             # Build script for all images
│   └── run.sh               # Run script for different profiles
└── kubernetes/               # Kubernetes configurations
    ├── deployment.yaml      # Kubernetes deployment
    ├── service.yaml         # Kubernetes services
    ├── configmap.yaml      # Configuration maps
    ├── pvc.yaml            # Persistent volume claims
    └── namespace.yaml       # Namespace configuration
```

## 🐳 Docker Options

Vectorizer provides multiple Docker configurations to suit different needs:

### Production Images

#### CPU-Only Production (`Dockerfile`)
- **Base**: `rust:1.82-slim` → `debian:bookworm-slim`
- **Features**: REST API Architecture, Automatic Summarization, Batch Operations
- **Use Case**: Maximum compatibility, no GPU required
- **Size**: Smaller image size
- **Performance**: CPU-optimized operations

### Development Images

#### Development (`Dockerfile.dev`)
- **Base**: `rust:1.82-slim`
- **Features**: Development tools, hot reload, debugging support
- **Use Case**: Development and testing

## 🎮 GPU Acceleration

Vectorizer implements cross-platform GPU acceleration through **wgpu**, providing support for:

### Supported Platforms
- **Vulkan** (Linux, Windows, Android)
- **DirectX 12** (Windows)
- **Metal** (macOS, iOS)

### GPU Features

#### WebGPU/wgpu (`wgpu-gpu`)
- **Backend**: Cross-platform GPU via wgpu
- **Technology**: WebGPU standard implementation
- **Support**: Vulkan, DirectX 12, Metal
- **Performance**: Significant acceleration in vector operations
- **Compatibility**: Works across multiple platforms without specific drivers

### Enabling GPU

To enable GPU support, compile with the appropriate features:

```bash
# Feature wgpu-gpu (Vulkan, DirectX, Metal)
cargo build --release --features wgpu-gpu

# Feature metal (macOS/iOS specific)
cargo build --release --features metal

# Feature gpu-accel (alias for wgpu-gpu)
cargo build --release --features gpu-accel
```

### Available Features in Cargo.toml

```toml
[features]
default = ["gpu_real"]
gpu = []
gpu_real = ["gpu"]
metal = ["wgpu", "pollster", "bytemuck", "futures"]
wgpu-gpu = ["wgpu", "pollster", "bytemuck", "futures", "ctrlc"]
gpu-accel = ["wgpu-gpu"]
```

### GPU Dependencies

```toml
# WebGPU/Metal dependencies (cross-platform GPU acceleration)
wgpu = { version = "27.0", features = ["wgsl"], optional = true }
pollster = { version = "0.4", optional = true }
bytemuck = { version = "1.22", features = ["derive"], optional = true }
futures = { version = "0.3", optional = true }
```

## 📝 Note about CUDA

**Status**: CUDA has been temporarily removed from the project.

The CUDA implementation (including CUHNSW) was temporarily removed in favor of a cross-platform solution using wgpu. This decision enables:

- Better cross-platform compatibility (Linux, Windows, macOS)
- Reduced dependency complexity
- Support for multiple GPU backends without vendor lock-in
- Easier development and deployment

CUDA support may return in future versions if there is specific demand.

## 🚀 Quick Start

### Building Images

```bash
# Build specific image
./devops/docker/build.sh cpu        # CPU-only production
./devops/docker/build.sh dev        # Development image
./devops/docker/build.sh all        # All images
```

### Running Services

```bash
# Start specific environment
./devops/docker/run.sh cpu          # CPU-only production
./devops/docker/run.sh dev          # Development
./devops/docker/run.sh stop         # Stop all services
./devops/docker/run.sh status       # Show status
```

### Manual Docker Compose

```bash
# CPU-only production
docker-compose --profile cpu-only up -d

# Development
docker-compose --profile dev up -d
```

## 🔧 Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Log level | `info` (prod), `debug` (dev) |
| `RUST_BACKTRACE` | Enable backtraces | `1` |
| `VECTORIZER_ENV` | Environment | `production` or `development` |
| `WGPU_BACKEND` | GPU backend | `auto` (Vulkan, DirectX, Metal) |
| `WGPU_POWER_PREF` | GPU power preference | `high-performance` or `low-power` |

### Ports

| Port | Service | Description |
|------|---------|-------------|
| 15002 | Unified Server | REST API + MCP (Model Context Protocol) |

### Volumes

| Volume | Description |
|--------|-------------|
| `vectorizer-data` | Persistent data storage |
| `vectorizer-cache` | Cache storage |
| `vectorizer-logs` | Log files |
| `cargo-cache` | Rust dependency cache (dev only) |

## ☸️ Kubernetes Deployment

### Prerequisites

- Kubernetes cluster (v1.20+)
- kubectl configured
- Persistent volume support

### Deploy to Kubernetes

```bash
# Create namespace
kubectl apply -f devops/kubernetes/namespace.yaml

# Create persistent volume claims
kubectl apply -f devops/kubernetes/pvc.yaml

# Create configuration maps
kubectl apply -f devops/kubernetes/configmap.yaml

# Deploy application
kubectl apply -f devops/kubernetes/deployment.yaml

# Create services
kubectl apply -f devops/kubernetes/service.yaml
```

### Access Services

```bash
# Port forward for local access
kubectl port-forward -n vectorizer svc/vectorizer-service 15002:15002

# Access via NodePort (if enabled)
# Unified Server: http://<node-ip>:30002
```

## 🔍 Monitoring

### Health Checks

All containers include health checks:

```bash
# Check container health
docker ps --format "table {{.Names}}\t{{.Status}}"

# Check Kubernetes pod health
kubectl get pods -n vectorizer
kubectl describe pod <pod-name> -n vectorizer
```

### Logs

```bash
# Docker logs
docker logs vectorizer-cpu-prod
docker logs vectorizer-dev

# Kubernetes logs
kubectl logs -n vectorizer deployment/vectorizer
kubectl logs -n vectorizer deployment/vectorizer -f  # Follow logs
```

## 🛠️ Troubleshooting

### Common Issues

#### GPU Issues
```bash
# Check wgpu backend availability
RUST_LOG=debug cargo run --features wgpu-gpu

# Test GPU compute
cargo test --features wgpu-gpu gpu_tests

# Vulkan on Linux - check drivers
vulkaninfo

# DirectX on Windows - check support
dxdiag

# Metal on macOS - check support
system_profiler SPDisplaysDataType
```

#### Port Conflicts
```bash
# Check port usage
netstat -tulpn | grep :15002
lsof -i :15002

# Use different ports
docker run -p 16002:15002 vectorizer:v0.29.0
```

#### Permission Issues
```bash
# Fix volume permissions
docker-compose down
sudo chown -R 1000:1000 ./data
docker-compose --profile cpu-only up -d
```

## 📊 Performance Comparison

| Environment | CPU Usage | Memory Usage | GPU Usage | Performance |
|-------------|-----------|--------------|-----------|-------------|
| CPU-Only | High | Medium | None | Baseline |
| wgpu-gpu (Vulkan/DirectX/Metal) | Low | Medium-High | Medium-High | 2-4x faster |

## 🔒 Security

### Production Security

- Non-root user in containers
- Minimal base images
- No unnecessary packages
- Health checks enabled
- Resource limits configured

### Development Security

- Development tools included
- Debug symbols available
- Hot reload enabled
- Volume mounts for live development

## 📝 Notes

- CPU images work on any Docker installation
- Development images include debugging tools
- Production images are optimized for size and security
- All images support REST API architecture
- Automatic summarization works in all variants
- GPU acceleration via wgpu supports Vulkan, DirectX 12, and Metal
- Cross-platform GPU support without vendor lock-in
- CUDA support was temporarily removed in favor of wgpu cross-platform solution
