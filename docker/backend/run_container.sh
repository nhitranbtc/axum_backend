#!/bin/bash
set -e

# ==============================================================================
# Configuration
# ==============================================================================
APP_NAME="axum_backend"
IMAGE_NAME="axum_backend:latest"
CONTAINER_NAME="axum_backend_app"
DB_CONTAINER_NAME="axum_db"
DB_IMAGE_NAME="postgres_with_init:latest"

# Database Credentials (should match docker-compose.yml)
DB_USER="axum"
DB_PASS="axum123"
DB_NAME="axum_backend"

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

cleanup_container() {
    local container_name=$1
    if [ "$(docker ps -aq -f name=$container_name)" ]; then
        log_info "Removing existing container: $container_name"
        docker stop $container_name > /dev/null 2>&1 || true
        docker rm $container_name > /dev/null 2>&1 || true
    fi
}

# ==============================================================================
# Main Script
# ==============================================================================

# 1. Navigate to Project Root
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$SCRIPT_DIR/../.."
cd "$PROJECT_ROOT" || { log_error "Failed to navigate to project root"; exit 1; }

echo "============================================="
echo "   � Starting Axum Backend Setup"
echo "============================================="
echo ""

# 2. Database Setup
echo "============================================="
echo "   🐘 Database Setup"
echo "============================================="

cleanup_container $DB_CONTAINER_NAME

log_info "Building Database Image..."
docker build -t $DB_IMAGE_NAME -f docker/postgres-docker/Dockerfile docker/postgres-docker

log_info "Starting Database Container..."
if docker run -d \
    --name $DB_CONTAINER_NAME \
    -p 5432:5432 \
    -e POSTGRES_USER=$DB_USER \
    -e POSTGRES_PASSWORD=$DB_PASS \
    -e POSTGRES_DB=$DB_NAME \
    -v postgres_data:/var/lib/postgresql/data \
    --restart unless-stopped \
    $DB_IMAGE_NAME > /dev/null; then
        log_success "Database started successfully."
else
        log_error "Failed to start database."
        exit 1
fi

log_info "Waiting 5s for Database to initialize..."
sleep 5

# 3. Application Build
echo ""
echo "============================================="
echo "   🔨 Application Build"
echo "============================================="

# Determine Rust version
if [ -f "rust-toolchain.toml" ]; then
    RUST_VERSION=$(grep 'channel' rust-toolchain.toml | awk -F '"' '{print $2}')
    RUST_VERSION=${RUST_VERSION:-bookworm}
else
    RUST_VERSION="bookworm"
fi
log_info "Using Rust version: $RUST_VERSION"

log_info "Building Application Image..."
docker build -f docker/backend/Dockerfile \
    --build-arg RUST_VERSION="$RUST_VERSION" \
    -t "$IMAGE_NAME" .

# 4. Application Run
echo ""
echo "============================================="
echo "   🏃 Running Application"
echo "============================================="

cleanup_container $CONTAINER_NAME

log_info "Starting Application Container..."
# Using --network host for easiest local dev connection to the DB container running on 5432
if docker run -d \
  --name $CONTAINER_NAME \
  --network host \
  --env-file .env \
  --restart unless-stopped \
  $IMAGE_NAME > /dev/null; then
    log_success "Application is running in Standalone Mode."
else
    log_error "Failed to start application."
    exit 1
fi

echo "============================================="
echo "   🎉 Deployment Complete"
echo "============================================="
echo "   - Main App:   http://localhost:3000"
echo "   - Postgres:   localhost:5432 ($DB_USER / $DB_PASS)"
echo "   - Logs:       docker logs -f $CONTAINER_NAME"
echo "============================================="

