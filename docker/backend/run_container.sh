#!/bin/bash
set -e

# ==============================================================================
# Configuration
# ==============================================================================
COMPOSE_FILE="docker/backend/docker-compose.yml"
BACKEND_SERVICE="backend"
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
    echo "  --single      Use single-node ScyllaDB (docker/scylla)"
    echo "  --cluster     Use 3-node ScyllaDB cluster (docker/scylla-cluster) [default]"
    echo "  --clean       Cleanup database (ScyllaDB keyspace drop and Redis flush) before starting"
    echo "  --build       Force rebuild of the application image (implies --clean)"
    echo "  --stop        Stop and remove all containers"
    echo "  --remove      Stop containers, remove volumes, and delete all locally built images"
    echo "  --test        Run API registration test after startup"
    echo "  --help        Show this help message"
    exit 1
}

run_smoke_test() {
    echo ""
    echo "============================================="
    echo "   🔍 API Smoke Test"
    echo "============================================="

    # Verify backend is reachable
    log_info "Checking backend health on port 3000..."
    max_retries=10
    count=0
    while ! curl -s http://localhost:3000/health > /dev/null; do
        sleep 2
        count=$((count + 1))
        if [ $count -ge $max_retries ]; then
            log_error "Backend is not reachable on port 3000."
            exit 1
        fi
    done
    log_success "Backend is up."

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
}

# ==============================================================================
# Argument Parsing
# ==============================================================================

CLEAN_DB=false
FORCE_BUILD=false
STOP_ONLY=false
REMOVE_ALL=false
RUN_TEST=false
SCYLLA_MODE="cluster"   # default
TEST_ONLY=false

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --single) SCYLLA_MODE="single" ;;
        --cluster) SCYLLA_MODE="cluster" ;;
        --clean) CLEAN_DB=true ;;
        --build) 
            FORCE_BUILD=true
            CLEAN_DB=true 
            ;;
        --stop) STOP_ONLY=true ;;
        --remove) REMOVE_ALL=true ;;
        --test)
            RUN_TEST=true
            # If no other action flag is set yet, mark as test-only
            if [ "$CLEAN_DB" = false ] && [ "$FORCE_BUILD" = false ] && [ "$STOP_ONLY" = false ] && [ "$REMOVE_ALL" = false ]; then
                TEST_ONLY=true
            fi
            ;;
        --help) usage ;;
        *) echo "Unknown parameter passed: $1"; usage ;;
    esac
    shift
done

# Resolve Scylla compose file based on selected mode
if [ "$SCYLLA_MODE" = "single" ]; then
    SCYLLA_COMPOSE_FILE="docker/scylla/docker-compose.yml"
    OTHER_SCYLLA_COMPOSE_FILE="docker/scylla-cluster/docker-compose.yml"
else
    SCYLLA_COMPOSE_FILE="docker/scylla-cluster/docker-compose.yml"
    OTHER_SCYLLA_COMPOSE_FILE="docker/scylla/docker-compose.yml"
fi

# ==============================================================================
# Main Script
# ==============================================================================

# 1. Navigate to Project Root
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$SCRIPT_DIR/../.."
cd "$PROJECT_ROOT" || { log_error "Failed to navigate to project root"; exit 1; }

echo "============================================="
echo "   🚀 Axum Backend Deployment Script"
echo "   Mode: ScyllaDB ${SCYLLA_MODE^}"
echo "============================================="
echo ""

# Short-circuit: --test with no other action flags → just run the smoke test
if [ "$TEST_ONLY" = true ]; then
    run_smoke_test
    exit 0
fi

# 1.5. Ensure external network exists
if ! docker network ls --format '{{.Name}}' | grep -q '^axum_net$'; then
    log_info "Creating external network 'axum_net'..."
    docker network create axum_net
fi

# 1.6. Stop the other mode's Scylla stack to avoid container name conflicts
# (both modes use the same container names: axum_scylla1, axum_scylla_manager)
if docker compose --env-file .env -f $OTHER_SCYLLA_COMPOSE_FILE ps -q 2>/dev/null | grep -q .; then
    log_info "Stopping other ScyllaDB mode stack to avoid name conflicts..."
    docker compose --env-file .env -f $OTHER_SCYLLA_COMPOSE_FILE down --remove-orphans
fi

# 2. Stop services if requested
if [ "$STOP_ONLY" = true ]; then
    log_info "Stopping all services..."
    docker compose --env-file .env -f $COMPOSE_FILE down --remove-orphans
    docker compose --env-file .env -f $SCYLLA_COMPOSE_FILE down --remove-orphans
    log_success "Services stopped."
    exit 0
fi

# 2b. Remove everything (containers + volumes + built images)
if [ "$REMOVE_ALL" = true ]; then
    log_warn "Removing all containers, volumes, and locally built images..."

    # Stop containers and remove volumes
    docker compose --env-file .env -f $COMPOSE_FILE down --remove-orphans --volumes
    docker compose --env-file .env -f $SCYLLA_COMPOSE_FILE down --remove-orphans --volumes
    log_success "Containers and volumes removed."

    # Delete locally built images (the ones produced by --build)
    BUILT_IMAGES=("axum_backend:latest")
    if [ "$SCYLLA_MODE" = "single" ]; then
        BUILT_IMAGES+=("scylla-scylla1")
    else
        BUILT_IMAGES+=("scylla-cluster-scylla1" "scylla-cluster-scylla2" "scylla-cluster-scylla3")
    fi
    for img in "${BUILT_IMAGES[@]}"; do
        if docker image inspect "$img" > /dev/null 2>&1; then
            docker rmi "$img" && log_success "Removed image: $img" || log_warn "Could not remove image: $img"
        else
            log_info "Image not found (skipped): $img"
        fi
    done

    log_success "All locally built images removed. Run '--build' to rebuild from scratch."
    exit 0
fi

# 3. Handle Database Cleanup
if [ "$CLEAN_DB" = true ]; then
    log_warn "Cleaning up database data..."
    
    # Start infra services first to perform cleanup
    log_info "Starting infrastructure for cleanup..."
    docker compose --env-file .env -f $SCYLLA_COMPOSE_FILE up -d
    docker compose --env-file .env -f $COMPOSE_FILE up -d $REDIS_SERVICE $NATS_SERVICE
    
    # Wait for Scylla to be ready
    log_info "Waiting for ScyllaDB cluster to be healthy..."
    while [ "$(docker inspect -f {{.State.Health.Status}} axum_scylla1 2>/dev/null || echo 'starting')" != "healthy" ]; do
        sleep 2
    done

    if [[ "$SCYLLA_COMPOSE_FILE" == *"cluster"* ]]; then
        log_info "Waiting for all 3 nodes in the ScyllaDB cluster to join the ring..."
        while [ "$(docker exec axum_scylla1 sh -c 'cqlsh -u ${SCYLLA_USERNAME} -p ${SCYLLA_PASSWORD} -e "SELECT count(*) FROM system.peers;" 2>/dev/null | grep -oE "[0-9]+" | head -n1 || echo 0')" -lt 2 ]; do
            sleep 5
        done
        log_success "All 3 ScyllaDB nodes are ready!"
    fi

    # Cleanup ScyllaDB
    log_info "Dropping ScyllaDB keyspace 'axum_backend'..."
    docker exec axum_scylla1 sh -c 'cqlsh -u ${SCYLLA_USERNAME} -p ${SCYLLA_PASSWORD} -e "DROP KEYSPACE IF EXISTS ${SCYLLA_KEYSPACE};"' || log_error "Failed to drop ScyllaDB keyspace"
    
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
docker compose --env-file .env -f $SCYLLA_COMPOSE_FILE up -d

log_info "Waiting for ScyllaDB cluster to be healthy..."
while [ "$(docker inspect -f {{.State.Health.Status}} axum_scylla1 2>/dev/null || echo 'starting')" != "healthy" ]; do
    sleep 2
done

if [[ "$SCYLLA_COMPOSE_FILE" == *"cluster"* ]]; then
    log_info "Waiting for all 3 nodes in the ScyllaDB cluster to join the ring..."
    while [ "$(docker exec axum_scylla1 sh -c 'cqlsh -u ${SCYLLA_USERNAME} -p ${SCYLLA_PASSWORD} -e "SELECT count(*) FROM system.peers;" 2>/dev/null | grep -oE "[0-9]+" | head -n1 || echo 0')" -lt 2 ]; do
        sleep 5
    done
    log_success "All 3 ScyllaDB nodes are ready!"
fi

docker compose --env-file .env -f $COMPOSE_FILE up -d $BUILD_FLAG --force-recreate --remove-orphans

# Register the cluster with Manager (per docs: https://manager.docs.scylladb.com/stable/add-a-cluster.html)
log_info "Registering cluster with Scylla Manager..."
# Wait for manager to be ready
for i in $(seq 1 15); do
    if docker exec axum_scylla_manager sctool status -c axum-cluster > /dev/null 2>&1; then
        log_success "Cluster 'axum-cluster' already registered."
        break
    fi
    sleep 2
    # Try to register on last attempt or once manager is reachable
    if docker exec axum_scylla_manager sctool version > /dev/null 2>&1; then
        if ! docker exec axum_scylla_manager sctool status -c axum-cluster > /dev/null 2>&1; then
            SCYLLA_USER=$(grep '^SCYLLA_USERNAME=' .env 2>/dev/null | cut -d= -f2 || echo "cassandra")
            SCYLLA_PASS=$(grep '^SCYLLA_PASSWORD=' .env 2>/dev/null | cut -d= -f2 || echo "cassandra")
            docker exec axum_scylla_manager sctool cluster add \
                --host scylla1 \
                --name axum-cluster \
                --auth-token super-secret-token \
                --username "$SCYLLA_USER" \
                --password "$SCYLLA_PASS" && \
            log_success "Cluster 'axum-cluster' registered with Scylla Manager." || \
            log_warn "Manager registration failed - you can register manually with: sctool cluster add --host scylla1 --name axum-cluster --auth-token super-secret-token"
            break
        fi
    fi
done

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
    run_smoke_test
fi
