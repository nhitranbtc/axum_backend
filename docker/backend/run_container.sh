#!/bin/bash
# ==============================================================================
# Axum Backend — Container Orchestration Script
#
# Usage:  ./docker/backend/run_container.sh [options]
# Run from any directory; the script always resolves the project root.
# ==============================================================================
set -euo pipefail

# ==============================================================================
# Constants
# ==============================================================================

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
readonly ENV_FILE="${PROJECT_ROOT}/.env"

readonly COMPOSE_FILE="docker/backend/docker-compose.yml"
readonly SCYLLA_SINGLE_COMPOSE="docker/scylla/docker-compose.yml"
readonly SCYLLA_CLUSTER_COMPOSE="docker/scylla-cluster/docker-compose.yml"

readonly BACKEND_CONTAINER="axum_backend"
readonly SCYLLA_CONTAINER="axum_scylla1"
readonly REDIS_CONTAINER="axum_redis"
readonly SCYLLA_MANAGER_CONTAINER="axum_scylla_manager"
readonly SCYLLA_CLUSTER_NAME="axum-cluster"
readonly NETWORK_NAME="axum_net"

readonly REDIS_SERVICE="redis"
readonly NATS_SERVICE="nats"

readonly BACKEND_PORT=3000
readonly HEALTH_MAX_WAIT=30   # seconds
readonly SCYLLA_CODE_RETRIES=10

# ==============================================================================
# Logging
# ==============================================================================

log_info()    { echo "ℹ️  $*"; }
log_success() { echo "✅ $*"; }
log_warn()    { echo "⚠️  $*"; }
log_error()   { echo "❌ $*" >&2; }

print_banner() {
    echo ""
    echo "============================================="
    echo "   $*"
    echo "============================================="
}

# ==============================================================================
# Usage
# ==============================================================================

usage() {
    cat <<EOF
Usage: $(basename "$0") [options]

Options:
  --single   Use single-node ScyllaDB  (docker/scylla)
  --cluster  Use 3-node ScyllaDB cluster (docker/scylla-cluster) [default]
  --clean    Drop ScyllaDB keyspace and flush Redis before starting
  --build    Force rebuild of the backend image  (implies --clean)
  --stop     Stop all containers (without removing them)
  --restart  Stop all containers then redeploy  (combinable with --single/--cluster)
  --remove   Stop containers, remove volumes, and delete locally built images
  --test     Run the full API smoke test (register → verify → login)
  --help     Show this help message
EOF
    exit 0
}

# ==============================================================================
# Docker Compose helpers
# ==============================================================================

# Wrapper so every compose call shares the same --env-file flag
dc() {
    docker compose --env-file "${ENV_FILE}" "$@"
}

# Load a single value from the .env file, with an optional fallback
env_val() {
    local key="$1" fallback="${2:-}"
    grep -E "^${key}=" "${ENV_FILE}" 2>/dev/null | cut -d= -f2- | tr -d "'\""  || echo "${fallback}"
}

# ==============================================================================
# Infrastructure helpers
# ==============================================================================

ensure_network() {
    if ! docker network ls --format '{{.Name}}' | grep -q "^${NETWORK_NAME}$"; then
        log_info "Creating external Docker network '${NETWORK_NAME}'..."
        docker network create "${NETWORK_NAME}"
    fi
}

# Stop the *other* Scylla mode stack to prevent container-name conflicts
stop_conflicting_scylla() {
    local other_compose="$1"
    if dc -f "${other_compose}" ps -q 2>/dev/null | grep -q .; then
        log_info "Stopping conflicting ScyllaDB stack (${other_compose})..."
        dc -f "${other_compose}" down --remove-orphans
    fi
}

wait_for_scylla_healthy() {
    log_info "Waiting for ScyllaDB to become healthy..."
    while [ "$(docker inspect -f '{{.State.Health.Status}}' "${SCYLLA_CONTAINER}" 2>/dev/null || echo starting)" != "healthy" ]; do
        sleep 2
    done
    log_success "ScyllaDB is healthy."
}

wait_for_cluster_ring() {
    log_info "Waiting for all 3 ScyllaDB nodes to join the ring..."
    local scylla_user scylla_pass
    scylla_user="$(env_val SCYLLA_USERNAME cassandra)"
    scylla_pass="$(env_val SCYLLA_PASSWORD cassandra)"
    while true; do
        local peers
        peers=$(docker exec "${SCYLLA_CONTAINER}" cqlsh \
            -u "${scylla_user}" -p "${scylla_pass}" \
            -e "SELECT count(*) FROM system.peers;" 2>/dev/null \
            | grep -oE "[0-9]+" | head -n1 || echo 0)
        [ "${peers}" -ge 2 ] && break
        sleep 5
    done
    log_success "All 3 ScyllaDB nodes are ready."
}

start_scylla() {
    local compose_file="$1"
    dc -f "${compose_file}" up -d
    wait_for_scylla_healthy
    if [[ "${compose_file}" == *"cluster"* ]]; then
        wait_for_cluster_ring
    fi
}

register_scylla_manager() {
    local scylla_user scylla_pass
    scylla_user="$(env_val SCYLLA_USERNAME cassandra)"
    scylla_pass="$(env_val SCYLLA_PASSWORD cassandra)"

    log_info "Registering cluster with Scylla Manager..."
    for _ in $(seq 1 15); do
        if docker exec "${SCYLLA_MANAGER_CONTAINER}" sctool status -c "${SCYLLA_CLUSTER_NAME}" >/dev/null 2>&1; then
            log_success "Cluster '${SCYLLA_CLUSTER_NAME}' already registered."
            return 0
        fi
        sleep 2
        if docker exec "${SCYLLA_MANAGER_CONTAINER}" sctool version >/dev/null 2>&1; then
            docker exec "${SCYLLA_MANAGER_CONTAINER}" sctool cluster add \
                --host scylla1 \
                --name "${SCYLLA_CLUSTER_NAME}" \
                --auth-token super-secret-token \
                --username "${scylla_user}" \
                --password "${scylla_pass}" \
            && log_success "Cluster '${SCYLLA_CLUSTER_NAME}' registered with Scylla Manager." \
            || log_warn "Manager registration failed — register manually: sctool cluster add --host scylla1 --name ${SCYLLA_CLUSTER_NAME} --auth-token super-secret-token"
            return 0
        fi
    done
    log_warn "Scylla Manager not reachable after retries — skipping registration."
}

# ==============================================================================
# Actions
# ==============================================================================

action_stop() {
    local scylla_compose="$1"
    log_info "Stopping all services..."
    dc -f "${COMPOSE_FILE}" stop
    dc -f "${scylla_compose}" stop
    log_success "Services stopped."
}

action_restart() {
    local scylla_compose="$1" build_flag="$2"
    log_info "Restarting all services..."
    action_stop "${scylla_compose}"
    if "${CLEAN_DB}"; then
        action_clean_db "${scylla_compose}"
    fi
    action_deploy "${scylla_compose}" "${build_flag}"
    log_success "Restart complete."
}

action_remove() {
    local scylla_compose="$1" scylla_mode="$2"
    log_warn "Removing all containers, volumes, and locally built images..."

    dc -f "${COMPOSE_FILE}"   down --remove-orphans --volumes
    dc -f "${scylla_compose}" down --remove-orphans --volumes
    log_success "Containers and volumes removed."

    local images=("${BACKEND_CONTAINER}:latest")
    if [ "${scylla_mode}" = "single" ]; then
        images+=("scylla-scylla1")
    else
        images+=("scylla-cluster-scylla1" "scylla-cluster-scylla2" "scylla-cluster-scylla3")
    fi

    for img in "${images[@]}"; do
        if docker image inspect "${img}" >/dev/null 2>&1; then
            docker rmi "${img}" && log_success "Removed image: ${img}" || log_warn "Could not remove image: ${img}"
        else
            log_info "Image not found (skipped): ${img}"
        fi
    done
    log_success "Done. Run '--build' to rebuild from scratch."
}

action_clean_db() {
    local scylla_compose="$1"
    local scylla_keyspace
    scylla_keyspace="$(env_val SCYLLA_KEYSPACE axum_backend)"

    log_warn "Cleaning up database data..."
    log_info "Starting infrastructure for cleanup..."
    dc -f "${scylla_compose}" up -d
    dc -f "${COMPOSE_FILE}"   up -d "${REDIS_SERVICE}" "${NATS_SERVICE}"

    wait_for_scylla_healthy
    if [[ "${scylla_compose}" == *"cluster"* ]]; then
        wait_for_cluster_ring
    fi

    log_info "Dropping ScyllaDB keyspace '${scylla_keyspace}'..."
    docker exec "${SCYLLA_CONTAINER}" sh -c \
        "cqlsh -u \${SCYLLA_USERNAME} -p \${SCYLLA_PASSWORD} -e \"DROP KEYSPACE IF EXISTS ${scylla_keyspace};\"" \
        || log_error "Failed to drop ScyllaDB keyspace (continuing)"

    log_info "Flushing Redis data..."
    docker exec "${REDIS_CONTAINER}" redis-cli FLUSHALL \
        || log_error "Failed to flush Redis (continuing)"

    log_success "Database cleanup complete."
}

action_deploy() {
    local scylla_compose="$1" build_flag="$2"
    log_info "Launching services..."
    start_scylla "${scylla_compose}"
    # shellcheck disable=SC2086
    dc -f "${COMPOSE_FILE}" up -d ${build_flag} --force-recreate --remove-orphans
    register_scylla_manager
}

# ==============================================================================
# Smoke test
# ==============================================================================

# Wait until the backend health endpoint responds
wait_for_backend() {
    log_info "Waiting for backend on port ${BACKEND_PORT}..."
    local elapsed=0
    until curl -s "http://localhost:${BACKEND_PORT}/health" >/dev/null; do
        sleep 2
        elapsed=$((elapsed + 2))
        if [ "${elapsed}" -ge "${HEALTH_MAX_WAIT}" ]; then
            log_error "Backend not reachable after ${HEALTH_MAX_WAIT}s."
            exit 1
        fi
    done
    log_success "Backend is up."
}

# Poll ScyllaDB for the 6-digit confirmation code written for a given email
fetch_confirmation_code() {
    local email="$1"
    local scylla_user scylla_pass code=""
    scylla_user="$(env_val SCYLLA_USERNAME cassandra)"
    scylla_pass="$(env_val SCYLLA_PASSWORD cassandra)"

    for _ in $(seq 1 "${SCYLLA_CODE_RETRIES}"); do
        sleep 1
        code=$(docker exec "${SCYLLA_CONTAINER}" cqlsh \
            -u "${scylla_user}" -p "${scylla_pass}" \
            -e "SELECT confirmation_code FROM axum_backend.users WHERE email='${email}' ALLOW FILTERING;" \
            2>/dev/null | grep -E "^\s+[0-9]{6}" | tr -d ' ')
        [ -n "${code}" ] && break
    done
    echo "${code}"
}

# Assert a curl response body contains an expected string; exit on failure
assert_response() {
    local step="$1" body="$2" expect="$3"
    if ! echo "${body}" | grep -q "${expect}"; then
        log_error "Step ${step} FAILED."
        echo "${body}"
        exit 1
    fi
    log_success "Step ${step} PASSED."
}

run_smoke_test() {
    local test_email="smoke_test_$(date +%s)@test.local"
    local test_name="SmokeUser"
    local test_pass="Test@1234"

    print_banner "🔍 Smoke Test  (register → verify → login)"

    wait_for_backend

    # Step 1 — Register
    log_info "Step 1 — Registering: ${test_email}"
    local reg_resp
    reg_resp=$(curl -sf -i -X POST "http://localhost:${BACKEND_PORT}/api/auth/register" \
        -H 'Content-Type: application/json' \
        -d "{\"email\":\"${test_email}\",\"name\":\"${test_name}\",\"password\":\"${test_pass}\"}")
    assert_response "1 (register)" "${reg_resp}" "HTTP/1.1 201 Created"

    # Step 2 — Fetch code from DB
    log_info "Step 2 — Fetching confirmation code from ScyllaDB..."
    local code
    code=$(fetch_confirmation_code "${test_email}")
    if [ -z "${code}" ]; then
        log_error "Step 2 FAILED — confirmation code not found in DB."
        exit 1
    fi
    log_success "Step 2 PASSED — Code: ${code}"

    # Step 3 — Verify email
    log_info "Step 3 — Verifying email..."
    local ver_resp
    ver_resp=$(curl -sf -X POST "http://localhost:${BACKEND_PORT}/api/auth/verify" \
        -H 'Content-Type: application/json' \
        -d "{\"email\":\"${test_email}\",\"code\":\"${code}\"}")
    assert_response "3 (verify)" "${ver_resp}" '"success":true'

    # Step 4 — Login
    log_info "Step 4 — Logging in..."
    local login_resp
    login_resp=$(curl -sf -X POST "http://localhost:${BACKEND_PORT}/api/auth/login" \
        -H 'Content-Type: application/json' \
        -d "{\"email\":\"${test_email}\",\"password\":\"${test_pass}\"}")
    assert_response "4 (login)" "${login_resp}" '"access_token"'

    print_banner "✅ All smoke tests PASSED"
    echo ""
    log_info "Backend logs (tail):"
    docker logs --tail 15 "${BACKEND_CONTAINER}"
}

# ==============================================================================
# Argument parsing
# ==============================================================================

CLEAN_DB=false
FORCE_BUILD=false
STOP_ONLY=false
RESTART=false
REMOVE_ALL=false
RUN_TEST=false
TEST_ONLY=false
SCYLLA_MODE="cluster"

while [[ "$#" -gt 0 ]]; do
    case "$1" in
        --single)  SCYLLA_MODE="single" ;;
        --cluster) SCYLLA_MODE="cluster" ;;
        --clean)   CLEAN_DB=true ;;
        --build)   FORCE_BUILD=true; CLEAN_DB=true ;;
        --stop)    STOP_ONLY=true ;;
        --restart) RESTART=true ;;
        --remove)  REMOVE_ALL=true ;;
        --test)
            RUN_TEST=true
            if ! "${CLEAN_DB}" && ! "${FORCE_BUILD}" && ! "${STOP_ONLY}" && ! "${RESTART}" && ! "${REMOVE_ALL}"; then
                TEST_ONLY=true
            fi
            ;;
        --help)    usage ;;
        *)         log_error "Unknown option: $1"; usage ;;
    esac
    shift
done

# Resolve Scylla compose paths from selected mode
if [ "${SCYLLA_MODE}" = "single" ]; then
    SCYLLA_COMPOSE="${SCYLLA_SINGLE_COMPOSE}"
    OTHER_SCYLLA_COMPOSE="${SCYLLA_CLUSTER_COMPOSE}"
    export SCYLLA_NODES_DOCKER="scylla1:9042"
else
    SCYLLA_COMPOSE="${SCYLLA_CLUSTER_COMPOSE}"
    OTHER_SCYLLA_COMPOSE="${SCYLLA_SINGLE_COMPOSE}"
    export SCYLLA_NODES_DOCKER="scylla1:9042,scylla2:9042,scylla3:9042"
fi

# ==============================================================================
# Entry point
# ==============================================================================

cd "${PROJECT_ROOT}" || { log_error "Cannot cd to project root: ${PROJECT_ROOT}"; exit 1; }

print_banner "🚀 Axum Backend  |  ScyllaDB ${SCYLLA_MODE^}"

# Short-circuit: --test alone skips deployment and just runs the test suite
if "${TEST_ONLY}"; then
    run_smoke_test
    exit 0
fi

ensure_network
stop_conflicting_scylla "${OTHER_SCYLLA_COMPOSE}"

if "${STOP_ONLY}"; then
    action_stop "${SCYLLA_COMPOSE}"
    exit 0
fi

if "${RESTART}"; then
    BUILD_FLAG=""
    "${FORCE_BUILD}" && BUILD_FLAG="--build"
    action_restart "${SCYLLA_COMPOSE}" "${BUILD_FLAG}"
    print_banner "🎉 Restart Complete"
    cat <<EOF
   Swagger UI : http://localhost:${BACKEND_PORT}/swagger-ui/
   Health     : http://localhost:${BACKEND_PORT}/health
   ScyllaDB   : localhost:9042
   Redis      : localhost:6379
   NATS       : localhost:4222
   Logs       : docker logs -f ${BACKEND_CONTAINER}
=============================================
EOF
    "${RUN_TEST}" && run_smoke_test
    exit 0
fi

if "${REMOVE_ALL}"; then
    action_remove "${SCYLLA_COMPOSE}" "${SCYLLA_MODE}"
    exit 0
fi

if "${CLEAN_DB}"; then
    action_clean_db "${SCYLLA_COMPOSE}"
fi

BUILD_FLAG=""
"${FORCE_BUILD}" && BUILD_FLAG="--build"

action_deploy "${SCYLLA_COMPOSE}" "${BUILD_FLAG}"

print_banner "🎉 Deployment Complete"
cat <<EOF
   Swagger UI : http://localhost:${BACKEND_PORT}/swagger-ui/
   Health     : http://localhost:${BACKEND_PORT}/health
   ScyllaDB   : localhost:9042
   Redis      : localhost:6379
   NATS       : localhost:4222
   Logs       : docker logs -f ${BACKEND_CONTAINER}
=============================================
EOF

"${CLEAN_DB}" && log_info "Schema will be recreated automatically by the backend on startup."

"${RUN_TEST}" && run_smoke_test

exit 0
