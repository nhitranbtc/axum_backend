#!/bin/bash
set -e

# ==============================================================================
# Configuration
# ==============================================================================
COMPOSE_FILE="docker/backend/docker-compose.yml"
BACKEND_SERVICE="backend"
SCYLLA_SERVICE="scylla"
REDIS_SERVICE="redis"
NATS_SERVICE="nats"

# ==============================================================================
# Helper Functions
# ==============================================================================

log_info() {
    echo "ℹ️  $1"
}

log_success() {
    echo "✅ $1"
}

log_warn() {
    echo "⚠️  $1"
}

log_error() {
    echo "❌ $1"
}

usage() {
    echo "Usage: $0 [options]"
    echo "Options:"
    echo "  --clean       Cleanup database (ScyllaDB keyspace drop and Redis flush) before starting"
    echo "  --build       Force rebuild of the application image (implies --clean)"
    echo "  --stop        Stop and remove all containers"
    echo "  --test        Run API registration test after startup"
    echo "  --help        Show this help message"
    exit 1
}

# ==============================================================================
# Argument Parsing
# ==============================================================================

CLEAN_DB=false
FORCE_BUILD=false
STOP_ONLY=false
RUN_TEST=false

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --clean) CLEAN_DB=true ;;
        --build) 
            FORCE_BUILD=true
            CLEAN_DB=true 
            ;;
        --stop) STOP_ONLY=true ;;
        --test) RUN_TEST=true ;;
        --help) usage ;;
        *) echo "Unknown parameter passed: $1"; usage ;;
    esac
    shift
done

# ==============================================================================
# Main Script
# ==============================================================================

# 1. Navigate to Project Root
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$SCRIPT_DIR/../.."
cd "$PROJECT_ROOT" || { log_error "Failed to navigate to project root"; exit 1; }

echo "============================================="
echo "   🚀 Axum Backend Deployment Script"
echo "============================================="
echo ""

# 2. Stop services if requested
if [ "$STOP_ONLY" = true ]; then
    log_info "Stopping all services..."
    docker compose --env-file .env -f $COMPOSE_FILE down --remove-orphans
    log_success "Services stopped."
    exit 0
fi

# 3. Handle Database Cleanup
if [ "$CLEAN_DB" = true ]; then
    log_warn "Cleaning up database data..."
    
    # Start infra services first to perform cleanup
    log_info "Starting infrastructure for cleanup..."
    docker compose --env-file .env -f $COMPOSE_FILE up -d $SCYLLA_SERVICE $REDIS_SERVICE $NATS_SERVICE
    
    # Wait for Scylla to be ready
    log_info "Waiting for ScyllaDB to be healthy..."
    while [ "$(docker inspect -f {{.State.Health.Status}} axum_scylla)" != "healthy" ]; do
        sleep 2
    done

    # Cleanup ScyllaDB
    log_info "Dropping ScyllaDB keyspace 'axum_backend'..."
    docker exec axum_scylla sh -c 'cqlsh -u ${SCYLLA_USERNAME} -p ${SCYLLA_PASSWORD} -e "DROP KEYSPACE IF EXISTS ${SCYLLA_KEYSPACE};"' || log_error "Failed to drop ScyllaDB keyspace"
    
    # Cleanup Redis
    log_info "Flushing Redis data..."
    docker exec axum_redis redis-cli FLUSHALL || log_error "Failed to flush Redis"

    log_success "Database cleanup complete."
fi

# 4. Launch Services
log_info "Launching services..."

BUILD_FLAG=""
if [ "$FORCE_BUILD" = true ]; then
    BUILD_FLAG="--build"
fi

# Use --force-recreate to ensure backend restarts and re-initializes after a potential cleanup
docker compose --env-file .env -f $COMPOSE_FILE up -d $BUILD_FLAG --force-recreate --remove-orphans

# 5. Summary
echo ""
echo "============================================="
echo "   🎉 Deployment Complete"
echo "============================================="
echo "   - Swagger UI:  http://localhost:3000/swagger-ui/"
echo "   - Health:      http://localhost:3000/health"
echo "   - ScyllaDB:    localhost:9042"
echo "   - Redis:       localhost:6379"
echo "   - NATS:        localhost:4222"
echo "   - Logs:        docker logs -f axum_backend"
echo "============================================="

if [ "$CLEAN_DB" = true ]; then
    log_info "Note: Schema will be automatically recreated by the backend on startup."
fi

# 6. API Smoke Test
if [ "$RUN_TEST" = true ]; then
    echo ""
    echo "============================================="
    echo "   🔍 API Smoke Test"
    echo "============================================="
    
    # Wait for backend to be ready
    log_info "Waiting for backend to be ready on port 3000..."
    max_retries=10
    count=0
    while ! curl -s http://localhost:3000/health > /dev/null; do
        sleep 2
        count=$((count + 1))
        if [ $count -ge $max_retries ]; then
            log_error "Backend failed to become ready in time."
            exit 1
        fi
    done

    log_info "Running registration test..."
    RESPONSE=$(curl -s -i -X 'POST' \
        'http://localhost:3000/api/auth/register' \
        -H 'accept: application/json' \
        -H 'Content-Type: application/json' \
        -d '{
        "email": "user01@gmail.com",
        "name": "User01",
        "password": "Test@123"
        }')

    echo "$RESPONSE" | grep -q "HTTP/1.1 201 Created" && echo "$RESPONSE" | grep -q '{"success":true,"data":{' 
    
    if [ $? -eq 0 ]; then
        log_success "Registration API test PASSED."
    else
        log_error "Registration API test FAILED."
        echo "Response was:"
        echo "$RESPONSE"
        exit 1
    fi

    echo -e "\n--- Backend Logs (Tail) ---"
    docker logs --tail 20 axum_backend
fi
