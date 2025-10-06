# DevOps Configuration for Vectorizer v0.20.0

This directory contains all DevOps configurations for Vectorizer, including Docker and Kubernetes deployments.

## 📁 Directory Structure

```
devops/
├── docker/                    # Docker configurations
│   ├── Dockerfile            # CPU-only production image
│   ├── Dockerfile.gpu      # GPU production image
│   ├── Dockerfile.dev       # CPU-only development image
│   ├── Dockerfile.dev.gpu  # GPU development image
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

#### CUDA Production (`Dockerfile.cuda`)
- **Base**: `nvidia/cuda:12.6-devel-ubuntu22.04` → `nvidia/cuda:12.6-runtime-ubuntu22.04`
- **Features**: REST API Architecture, CUDA GPU Acceleration, Automatic Summarization, Batch Operations
- **Dependencies**: [CUHNSW](https://github.com/js1010/cuhnsw) - CUDA implementation of HNSW algorithm
- **Use Case**: Maximum performance with GPU acceleration
- **Size**: Larger image size
- **Performance**: 3-5x faster with GPU

### Development Images

#### CPU-Only Development (`Dockerfile.dev`)
- **Base**: `rust:1.82-slim`
- **Features**: Development tools, hot reload, debugging support
- **Use Case**: Development without GPU requirements

#### CUDA Development (`Dockerfile.dev.cuda`)
- **Base**: `nvidia/cuda:12.6-devel-ubuntu22.04`
- **Features**: Development tools, hot reload, debugging support, CUDA acceleration
- **Dependencies**: [CUHNSW](https://github.com/js1010/cuhnsw) - CUDA implementation of HNSW algorithm
- **Use Case**: Development with GPU testing

## 🔧 CUHNSW Integration

Vectorizer uses [CUHNSW](https://github.com/js1010/cuhnsw) for CUDA-accelerated HNSW operations. CUHNSW is automatically cloned and built during Docker image creation.

### CUHNSW Features
- **CUDA Implementation**: Efficient GPU implementation of HNSW algorithm
- **Performance**: 8-9x faster build time, 3-4x faster search time
- **Compatibility**: Compatible with hnswlib model format
- **Quality**: Deterministic results matching CPU implementations

### Automatic Installation
CUHNSW is automatically installed in CUDA Docker images:

```dockerfile
# Clone and build CUHNSW dependency
RUN git clone https://github.com/js1010/cuhnsw.git /tmp/cuhnsw && \
    cd /tmp/cuhnsw && \
    git submodule update --init && \
    pip3 install -r requirements.txt && \
    python3 -m grpc_tools.protoc --python_out cuhnsw/ --proto_path cuhnsw/proto/ config.proto && \
    python3 setup.py install && \
    cd / && rm -rf /tmp/cuhnsw
```

### Manual Installation (Development)
For local development without Docker:

```bash
# Clone CUHNSW repository
git clone https://github.com/js1010/cuhnsw.git
cd cuhnsw

# Initialize submodules
git submodule update --init

# Install Python dependencies
pip install -r requirements.txt

# Generate protobuf files
python -m grpc_tools.protoc --python_out cuhnsw/ --proto_path cuhnsw/proto/ config.proto

# Install CUHNSW
python setup.py install
```

## 🚀 Quick Start

### Building Images

```bash
# Build specific image
./devops/docker/build.sh cpu        # CPU-only production
./devops/docker/build.sh cuda       # CUDA production
./devops/docker/build.sh dev-cpu    # CPU-only development
./devops/docker/build.sh dev-cuda   # CUDA development
./devops/docker/build.sh all        # All images
```

### Running Services

```bash
# Start specific environment
./devops/docker/run.sh cpu          # CPU-only production
./devops/docker/run.sh cuda         # CUDA production
./devops/docker/run.sh dev-cpu      # CPU-only development
./devops/docker/run.sh dev-cuda     # CUDA development
./devops/docker/run.sh stop         # Stop all services
./devops/docker/run.sh status       # Show status
```

### Manual Docker Compose

```bash
# CPU-only production
docker-compose --profile cpu-only up -d

# CUDA production
docker-compose --profile cuda up -d

# CPU-only development
docker-compose --profile dev-cpu up -d

# CUDA development
docker-compose --profile dev-cuda up -d
```

## 🔧 Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Log level | `info` (prod), `debug` (dev) |
| `RUST_BACKTRACE` | Enable backtraces | `1` |
| `VECTORIZER_ENV` | Environment | `production` or `development` |
| `CUDA_ENABLED` | CUDA support | `true` (CUDA images), `false` (CPU images) |

### Ports

| Port | Service | Description |
|------|---------|-------------|
| 15001 | REST API | HTTP API endpoint |
| 15002 | MCP Server | Model Context Protocol |

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
kubectl port-forward -n vectorizer svc/vectorizer-service 15001:15001
kubectl port-forward -n vectorizer svc/vectorizer-service 15002:15002
kubectl port-forward -n vectorizer svc/vectorizer-service 15003:15003

# Access via NodePort (if enabled)
# REST API: http://<node-ip>:30001
# MCP Server: http://<node-ip>:30002
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
docker logs vectorizer-cuda-prod

# Kubernetes logs
kubectl logs -n vectorizer deployment/vectorizer
kubectl logs -n vectorizer deployment/vectorizer -f  # Follow logs
```

## 🛠️ Troubleshooting

### Common Issues

#### CUDA Issues
```bash
# Check NVIDIA Docker runtime
docker run --rm --gpus all nvidia/cuda:12.6-base-ubuntu22.04 nvidia-smi

# Check CUDA in container
docker exec -it vectorizer-cuda-prod nvidia-smi

# Check CUHNSW installation
docker exec -it vectorizer-cuda-prod python3 -c "import cuhnsw; print('CUHNSW installed successfully')"

# Rebuild CUDA image if CUHNSW fails
docker build -f devops/docker/Dockerfile.cuda -t vectorizer:cuda-latest .
```

#### Port Conflicts
```bash
# Check port usage
netstat -tulpn | grep :15001
lsof -i :15001

# Use different ports
docker-compose --profile cpu-only up -d --scale vectorizer-cpu=0
docker run -p 16001:15001 vectorizer:cpu-latest
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
| CUDA | Low | High | High | 3-5x faster |

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

- CUDA images require NVIDIA Docker runtime and [CUHNSW](https://github.com/js1010/cuhnsw) dependency
- CPU images work on any Docker installation
- Development images include debugging tools
- Production images are optimized for size and security
- All images support REST API architecture
- Automatic summarization works in all variants
- CUHNSW provides 8-9x faster build and 3-4x faster search performance
