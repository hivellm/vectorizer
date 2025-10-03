#!/bin/bash
# Docker Compose management script for Vectorizer
# Usage: ./run.sh [cpu|cuda|dev-cpu|dev-cuda|stop|status]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to run CPU-only production
run_cpu() {
    print_status "Starting CPU-only production environment..."
    docker-compose --profile cpu-only up -d
    print_success "CPU-only production environment started!"
    print_status "Services available at:"
    echo "  - REST API: http://localhost:15001"
    echo "  - MCP Server: http://localhost:15002"
    echo "  - GRPC Server: http://localhost:15003"
}

# Function to run CUDA production
run_cuda() {
    print_status "Starting CUDA production environment..."
    docker-compose --profile cuda up -d
    print_success "CUDA production environment started!"
    print_status "Services available at:"
    echo "  - REST API: http://localhost:15001"
    echo "  - MCP Server: http://localhost:15002"
    echo "  - GRPC Server: http://localhost:15003"
}

# Function to run CPU-only development
run_dev_cpu() {
    print_status "Starting CPU-only development environment..."
    docker-compose --profile dev-cpu up -d
    print_success "CPU-only development environment started!"
    print_status "Services available at:"
    echo "  - REST API: http://localhost:15001"
    echo "  - MCP Server: http://localhost:15002"
    echo "  - GRPC Server: http://localhost:15003"
    print_status "To access the development container:"
    echo "  docker exec -it vectorizer-dev-cpu bash"
}

# Function to run CUDA development
run_dev_cuda() {
    print_status "Starting CUDA development environment..."
    docker-compose --profile dev-cuda up -d
    print_success "CUDA development environment started!"
    print_status "Services available at:"
    echo "  - REST API: http://localhost:15001"
    echo "  - MCP Server: http://localhost:15002"
    echo "  - GRPC Server: http://localhost:15003"
    print_status "To access the development container:"
    echo "  docker exec -it vectorizer-dev-cuda bash"
}

# Function to stop all services
stop_all() {
    print_status "Stopping all Vectorizer services..."
    docker-compose down
    print_success "All services stopped!"
}

# Function to show status
show_status() {
    print_status "Vectorizer services status:"
    docker-compose ps
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [cpu|cuda|dev-cpu|dev-cuda|stop|status]"
    echo ""
    echo "Options:"
    echo "  cpu      - Start CPU-only production environment"
    echo "  cuda     - Start CUDA production environment"
    echo "  dev-cpu  - Start CPU-only development environment"
    echo "  dev-cuda - Start CUDA development environment"
    echo "  stop     - Stop all services"
    echo "  status   - Show services status"
    echo ""
    echo "Examples:"
    echo "  $0 cpu        # Start CPU-only production"
    echo "  $0 cuda       # Start CUDA production"
    echo "  $0 dev-cpu    # Start CPU-only development"
    echo "  $0 stop       # Stop all services"
    echo "  $0 status     # Show status"
}

# Main script logic
case "${1:-}" in
    cpu)
        run_cpu
        ;;
    cuda)
        run_cuda
        ;;
    dev-cpu)
        run_dev_cpu
        ;;
    dev-cuda)
        run_dev_cuda
        ;;
    stop)
        stop_all
        ;;
    status)
        show_status
        ;;
    *)
        show_usage
        exit 1
        ;;
esac
