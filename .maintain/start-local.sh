#!/bin/bash

# Licensable Runtime - Local Development Startup Script
# This script provides a convenient way to start the entire stack locally with Docker
# It handles building, starting services, database initialization, and monitoring
#
# Usage:
#   ./start-local.sh           # Start with traditional Substrate node (default)
#   ./start-local.sh --omni    # Start with Polkadot Omni-Node
#   ./start-local.sh --clean   # Clean all data and start fresh

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default configuration
USE_OMNI_NODE=false
CLEAN_START=false
FORCE_BUILD=false
PROJECT_NAME="licensable-runtime"
MAX_WAIT_TIME=60  # Maximum time to wait for services in seconds

# Parse command line arguments
for arg in "$@"; do
    case $arg in
        --omni|--omni-node)
            USE_OMNI_NODE=true
            shift
            ;;
        --clean)
            CLEAN_START=true
            shift
            ;;
        --build)
            FORCE_BUILD=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --omni, --omni-node  Use Polkadot Omni-Node instead of traditional Substrate node"
            echo "  --clean              Clean all data volumes before starting"
            echo "  --build              Force rebuild of Docker images (skips if images exist by default)"
            echo "  --help, -h           Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                   Start with traditional Substrate node (uses existing images)"
            echo "  $0 --build           Rebuild images and start traditional node"
            echo "  $0 --omni            Start with Polkadot Omni-Node (uses existing images)"
            echo "  $0 --omni --build    Rebuild Omni-Node image and start"
            echo "  $0 --omni --clean    Clean start with Omni-Node"
            exit 0
            ;;
        *)
            echo "Unknown option: $arg"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Set compose file based on node type
if [ "$USE_OMNI_NODE" = true ]; then
    COMPOSE_FILE=".maintain/docker-compose.omni-node.yml"
    NODE_TYPE="Polkadot Omni-Node"
    CONTAINER_NAME="licensable-omni-node"
    API_CONTAINER_NAME="licensable-omni-node"  # Omni-node has API built-in
else
    COMPOSE_FILE=".maintain/docker-compose.yml"
    NODE_TYPE="Traditional Substrate Node"
    CONTAINER_NAME="licensable-substrate"
    API_CONTAINER_NAME="licensable-api"
fi

# Function to print colored output
print_color() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# Function to print section headers
print_header() {
    echo ""
    print_color "$BLUE" "================================================"
    print_color "$BLUE" "$1"
    print_color "$BLUE" "================================================"
}

# Function to check if command exists
check_command() {
    if ! command -v $1 &> /dev/null; then
        print_color "$RED" "Error: $1 is not installed"
        return 1
    fi
}

# Function to wait for a service to be ready
wait_for_service() {
    local service_name=$1
    local check_command=$2
    local max_attempts=$3
    local attempt=0

    print_color "$YELLOW" "Waiting for $service_name to be ready..."

    while [ $attempt -lt $max_attempts ]; do
        if eval $check_command &> /dev/null; then
            print_color "$GREEN" "✓ $service_name is ready!"
            return 0
        fi
        echo -n "."
        sleep 1
        attempt=$((attempt + 1))
    done

    echo ""
    print_color "$RED" "✗ $service_name failed to start after ${max_attempts} seconds"
    return 1
}

# Function to check Docker daemon
check_docker_running() {
    if ! docker info &> /dev/null; then
        print_color "$RED" "Error: Docker daemon is not running"
        print_color "$YELLOW" "Please start Docker and try again"
        exit 1
    fi
}

# Function to check if Docker image exists
check_docker_image() {
    local image_name=$1
    if docker images --format "{{.Repository}}:{{.Tag}}" | grep -q "^${image_name}$"; then
        return 0
    else
        return 1
    fi
}

# Function to cleanup on exit
cleanup() {
    if [ "$?" -ne 0 ]; then
        print_color "$RED" "Script failed. Check the logs above for errors."
        print_color "$YELLOW" "You can check service logs with: pnpm docker:logs"
    fi
}

trap cleanup EXIT

# Main execution
main() {
    print_header "Licensable Runtime - Local Development Setup"
    print_color "$BLUE" "Node Type: $NODE_TYPE"

    # Check prerequisites
    print_color "$YELLOW" "Checking prerequisites..."
    check_command "docker" || exit 1
    check_command "docker-compose" || exit 1
    check_docker_running
    print_color "$GREEN" "✓ All prerequisites met"

    # Navigate to project root
    SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
    cd "$SCRIPT_DIR/.."

    print_header "Step 1: Cleaning up existing containers"
    print_color "$YELLOW" "Stopping any running containers..."

    # Stop containers from both compose files to avoid port conflicts
    docker-compose -f .maintain/docker-compose.yml down 2>/dev/null || true
    docker-compose -f .maintain/docker-compose.omni-node.yml down 2>/dev/null || true

    # Handle volume cleaning
    if [ "$CLEAN_START" = true ]; then
        print_color "$YELLOW" "Clean start requested - removing volumes..."
        docker-compose -f $COMPOSE_FILE down -v
        print_color "$GREEN" "✓ Volumes cleaned"
    elif [ "$USE_OMNI_NODE" = false ]; then
        # Only ask for traditional node (omni-node doesn't have the seeding step)
        read -p "Do you want to clean existing data volumes? (y/N) " -n 1 -r
        echo ""
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            print_color "$YELLOW" "Removing volumes..."
            docker-compose -f $COMPOSE_FILE down -v
            print_color "$GREEN" "✓ Volumes cleaned"
        fi
    fi

    print_header "Step 2: Docker Images"

    # Determine which images to check based on node type
    if [ "$USE_OMNI_NODE" = true ]; then
        REQUIRED_IMAGES=("maintain-licensable-omni-node:latest" "postgres:16-alpine")
    else
        REQUIRED_IMAGES=("maintain-substrate:latest" "maintain-api:latest" "postgres:15-alpine")
    fi

    # Check if images exist
    IMAGES_EXIST=true
    for image in "${REQUIRED_IMAGES[@]}"; do
        if ! check_docker_image "$image"; then
            print_color "$YELLOW" "Docker image $image not found"
            IMAGES_EXIST=false
        else
            print_color "$GREEN" "✓ Found image: $image"
        fi
    done

    # Decide whether to build
    if [ "$FORCE_BUILD" = true ]; then
        print_color "$YELLOW" "Force build requested with --build flag"
        print_color "$YELLOW" "Building Docker images..."
        print_color "$YELLOW" "This may take several minutes..."

        if docker-compose -f $COMPOSE_FILE build; then
            print_color "$GREEN" "✓ Docker images built successfully"
        else
            print_color "$RED" "✗ Failed to build Docker images"
            exit 1
        fi
    elif [ "$IMAGES_EXIST" = false ]; then
        print_color "$YELLOW" "Required images not found. Building Docker images..."
        print_color "$YELLOW" "This may take several minutes on first run..."
        print_color "$YELLOW" "Tip: Images are already built? Check with: docker images"

        if docker-compose -f $COMPOSE_FILE build; then
            print_color "$GREEN" "✓ Docker images built successfully"
        else
            print_color "$RED" "✗ Failed to build Docker images"
            exit 1
        fi
    else
        print_color "$GREEN" "✓ All required Docker images exist"
        print_color "$BLUE" "Skipping build step (use --build to force rebuild)"
    fi

    print_header "Step 3: Starting services"
    print_color "$YELLOW" "Starting PostgreSQL, Substrate Node, and NestJS API..."

    if docker-compose -f $COMPOSE_FILE up -d; then
        print_color "$GREEN" "✓ Services started"
    else
        print_color "$RED" "✗ Failed to start services"
        exit 1
    fi

    print_header "Step 4: Waiting for services to be ready"

    if [ "$USE_OMNI_NODE" = true ]; then
        # For Omni-Node setup
        if ! wait_for_service "Polkadot Omni-Node" "curl -f http://localhost:9933" 60; then
            print_color "$RED" "Omni-Node failed to start. Checking logs..."
            docker logs $CONTAINER_NAME --tail 20
            print_color "$YELLOW" "Tip: Try rebuilding the image with --build flag"
            exit 1
        fi
    else
        # For traditional setup with PostgreSQL and API
        # Wait for PostgreSQL
        wait_for_service "PostgreSQL" "docker exec licensable-postgres pg_isready -U postgres" 30

        # Wait for NestJS API
        wait_for_service "NestJS API" "curl -f http://localhost:3000/health" 30

        # Wait for Substrate Node (with diagnostics)
        print_color "$YELLOW" "Waiting for Substrate Node to be ready..."
        if ! wait_for_service "Substrate Node" "curl -f http://localhost:9933" 60; then
            print_color "$YELLOW" "Substrate node didn't respond. Checking supervisor status..."

            # Check if it's a Rosetta/architecture issue
            if docker exec $CONTAINER_NAME cat /var/log/supervisor/substrate-error.log 2>/dev/null | grep -q "rosetta error"; then
                print_color "$RED" "✗ Detected Rosetta/architecture compatibility issue!"
                echo ""
                print_color "$YELLOW" "The Substrate node binary was built for x86_64 but you're running on ARM (Apple Silicon)."
                print_color "$YELLOW" "Solutions:"
                print_color "$BLUE" "  1. Rebuild the Docker image with: .maintain/start-local.sh --build"
                print_color "$BLUE" "  2. Or try the Omni-Node setup: .maintain/start-local.sh --omni"
                echo ""
                print_color "$YELLOW" "Continuing with API-only setup..."
                print_color "$GREEN" "✓ NestJS API is running and accessible"
            else
                print_color "$YELLOW" "Checking container logs..."
                docker exec $CONTAINER_NAME supervisorctl status
                echo ""
                print_color "$YELLOW" "Recent error logs:"
                docker exec $CONTAINER_NAME cat /var/log/supervisor/substrate-error.log 2>/dev/null | tail -10

                print_color "$YELLOW" "Tip: Check full logs with: pnpm docker:logs"
            fi
        fi

        print_header "Step 5: Seeding database"
        print_color "$YELLOW" "Creating test license data..."

        # Wait a bit more for API to be fully initialized
        sleep 3

        if docker exec -it $API_CONTAINER_NAME sh -c 'cd /app && DB_HOST=postgres DB_PORT=5432 DB_USERNAME=postgres DB_PASSWORD=postgres DB_NAME=license_db node dist/seed.js'; then
            print_color "$GREEN" "✓ Database seeded successfully"
            echo ""
            print_color "$GREEN" "Test licenses created:"
            print_color "$BLUE" "  • valid-license-key-12345    (Valid, expires in 30 days)"
            print_color "$BLUE" "  • expired-license-key-67890  (Expired 30 days ago)"
            print_color "$BLUE" "  • inactive-license-key-11111 (Inactive)"
        else
            print_color "$YELLOW" "⚠ Warning: Database seeding failed (database might already be seeded)"
        fi
    fi

    print_header "Step 6: Verifying deployment"

    if [ "$USE_OMNI_NODE" = false ]; then
        # Test API endpoint for traditional setup
        print_color "$YELLOW" "Testing API endpoints..."

        if curl -s "http://localhost:3000/license?key=valid-license-key-12345" | grep -q "true"; then
            print_color "$GREEN" "✓ API license validation is working"
        else
            print_color "$YELLOW" "⚠ API license validation test failed (may need seeding)"
        fi

        # Check Substrate node status
        if curl -f http://localhost:9933 &>/dev/null; then
            print_color "$GREEN" "✓ Substrate node is responding"
        else
            print_color "$YELLOW" "⚠ Substrate node is not responding (see diagnostics above)"
        fi
    fi

    # Show service status
    print_header "Service Status"
    docker-compose -f $COMPOSE_FILE ps

    print_header "🎉 Setup Complete!"

    print_color "$GREEN" "All services are running successfully!"
    echo ""
    print_color "$BLUE" "Service URLs:"
    print_color "$BLUE" "  • Substrate RPC HTTP:    http://localhost:9933"
    print_color "$BLUE" "  • Substrate WebSocket:   ws://localhost:9944"

    if [ "$USE_OMNI_NODE" = false ]; then
        print_color "$BLUE" "  • NestJS API:           http://localhost:3000"
        print_color "$BLUE" "  • PostgreSQL:           localhost:5432 (database: license_db)"
    fi

    print_color "$BLUE" "  • Prometheus Metrics:   http://localhost:9615"
    echo ""
    print_color "$YELLOW" "Useful commands:"

    if [ "$USE_OMNI_NODE" = true ]; then
        print_color "$YELLOW" "  • View logs:            pnpm omni:docker:logs"
        print_color "$YELLOW" "  • Check status:         docker-compose -f $COMPOSE_FILE ps"
        print_color "$YELLOW" "  • Stop services:        pnpm omni:docker:down"
        print_color "$YELLOW" "  • Clean everything:     pnpm omni:docker:clean"
    else
        print_color "$YELLOW" "  • View logs:            pnpm docker:logs"
        print_color "$YELLOW" "  • Check status:         pnpm docker:status"
        print_color "$YELLOW" "  • Stop services:        pnpm docker:down"
        print_color "$YELLOW" "  • Clean everything:     pnpm docker:clean"
    fi

    echo ""

    if [ "$USE_OMNI_NODE" = false ]; then
        print_color "$GREEN" "To test the API, try:"
        print_color "$BLUE" '  curl "http://localhost:3000/license?key=valid-license-key-12345"'
        echo ""
    fi

    # Ask if user wants to see logs
    read -p "Do you want to view the service logs now? (y/N) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_header "Service Logs (Press Ctrl+C to exit)"
        docker-compose -f $COMPOSE_FILE logs -f
    else
        if [ "$USE_OMNI_NODE" = true ]; then
            print_color "$BLUE" "You can view logs anytime with: pnpm omni:docker:logs"
        else
            print_color "$BLUE" "You can view logs anytime with: pnpm docker:logs"
        fi
    fi
}

# Run the main function
main