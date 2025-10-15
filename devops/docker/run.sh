#!/bin/bash
# Docker Compose management script for Vectorizer
# Usage: ./run.sh [prod|dev|stop|status]

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

# Function to run production
run_prod() {
    print_status "Starting production environment..."
    docker-compose up -d
    print_success "Production environment started!"
    print_status "Services available at:"
    echo "  - Unified Server: http://localhost:15002"
}

# Function to run development
run_dev() {
    print_status "Starting development environment..."
    docker-compose --profile dev up -d
    print_success "Development environment started!"
    print_status "Services available at:"
    echo "  - Unified Server: http://localhost:15002"
    print_status "To access the development container:"
    echo "  docker exec -it vectorizer-dev bash"
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
    echo "Usage: $0 [prod|dev|stop|status]"
    echo ""
    echo "Options:"
    echo "  prod     - Start production environment"
    echo "  dev      - Start development environment"
    echo "  stop     - Stop all services"
    echo "  status   - Show services status"
    echo ""
    echo "Examples:"
    echo "  $0 prod       # Start production"
    echo "  $0 dev        # Start development"
    echo "  $0 stop       # Stop all services"
    echo "  $0 status     # Show status"
}

# Main script logic
case "${1:-}" in
    prod)
        run_prod
        ;;
    dev)
        run_dev
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
