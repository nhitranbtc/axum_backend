#!/bin/bash

# gRPC CreateUser Stress Test (Optimized)
# Tests the gRPC UserService/CreateUser endpoint under load

set -e

# Configuration
GRPC_HOST="${GRPC_HOST:-localhost:50051}"
NUM_USERS="${NUM_USERS:-500}"
CONCURRENT="${CONCURRENT:-50}"
RESULTS_DIR="${RESULTS_DIR:-/tmp/grpc-stress-test}"
TEST_NAME="grpc_create_user"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Create results directory with timestamp
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
TEST_RUN_DIR="$RESULTS_DIR/${TEST_NAME}_${TIMESTAMP}"
mkdir -p "$TEST_RUN_DIR"

echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║     gRPC CreateUser Stress Test (Optimized)           ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${CYAN}Configuration:${NC}"
echo "  • gRPC Host: $GRPC_HOST"
echo "  • Total Users: $NUM_USERS"
echo "  • Concurrent Requests: $CONCURRENT"
echo "  • Results Directory: $TEST_RUN_DIR"
echo ""

# Check dependencies
if ! command -v grpcurl &> /dev/null; then
    echo -e "${RED}❌ Error: grpcurl is not installed${NC}"
    echo "Install: go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest"
    exit 1
fi

# Check server
echo -e "${YELLOW}🔍 Checking gRPC server...${NC}"
if ! grpcurl -plaintext "$GRPC_HOST" list > /dev/null 2>&1; then
    echo -e "${RED}❌ Cannot connect to gRPC server at $GRPC_HOST${NC}"
    echo "Start server: cargo run --bin grpc_server"
    exit 1
fi
echo -e "${GREEN}✅ Server is running${NC}"
echo ""

# Create temp directory
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

# Function to create user
create_user() {
    local user_id=$1
    local start_time=$(date +%s%N)
    
    local response=$(grpcurl -plaintext \
        -d "{\"email\": \"stress_test_${user_id}@example.com\", \"name\": \"Stress Test User ${user_id}\", \"password\": \"SecurePassword123!\"}" \
        "$GRPC_HOST" \
        user.UserService/CreateUser 2>&1)
    
    local end_time=$(date +%s%N)
    local duration=$(( (end_time - start_time) / 1000000 ))
    
    if echo "$response" | grep -q "ERROR"; then
        echo "FAIL,$user_id,$duration" >> "$TEMP_DIR/results.csv"
    else
        echo "SUCCESS,$user_id,$duration" >> "$TEMP_DIR/results.csv"
    fi
}

export -f create_user
export GRPC_HOST TEMP_DIR

# Initialize results
echo "status,user_id,latency_ms" > "$TEMP_DIR/results.csv"

echo -e "${YELLOW}📊 Running stress test...${NC}"
START_TIME=$(date +%s)

# Run tests
if command -v parallel &> /dev/null; then
    seq 1 "$NUM_USERS" | parallel -j "$CONCURRENT" --bar create_user {}
else
    echo -e "${YELLOW}⚠️  Using xargs (install GNU parallel for progress bar)${NC}"
    seq 1 "$NUM_USERS" | xargs -n 1 -P "$CONCURRENT" -I {} bash -c '
        user_id={}
        start_time=$(date +%s%N)
        response=$(grpcurl -plaintext \
            -d "{\"email\": \"stress_test_${user_id}@example.com\", \"name\": \"Stress Test User ${user_id}\", \"password\": \"SecurePassword123!\"}" \
            "'"$GRPC_HOST"'" \
            user.UserService/CreateUser 2>&1)
        end_time=$(date +%s%N)
        duration=$(( (end_time - start_time) / 1000000 ))
        if echo "$response" | grep -q "ERROR"; then
            echo "FAIL,$user_id,$duration" >> "'"$TEMP_DIR"'/results.csv"
        else
            echo "SUCCESS,$user_id,$duration" >> "'"$TEMP_DIR"'/results.csv"
        fi
    '
fi

END_TIME=$(date +%s)
TOTAL_DURATION=$((END_TIME - START_TIME))

# Calculate statistics
TOTAL_REQUESTS=$(tail -n +2 "$TEMP_DIR/results.csv" | wc -l)
SUCCESS_COUNT=$(grep "^SUCCESS" "$TEMP_DIR/results.csv" 2>/dev/null | wc -l)
FAIL_COUNT=$(grep "^FAIL" "$TEMP_DIR/results.csv" 2>/dev/null | wc -l)

if [ "$TOTAL_REQUESTS" -eq 0 ]; then
    echo -e "${RED}❌ No requests completed${NC}"
    exit 1
fi

SUCCESS_RATE=$(awk "BEGIN {printf \"%.1f\", ($SUCCESS_COUNT / $TOTAL_REQUESTS) * 100}")
LATENCIES=$(grep "^SUCCESS" "$TEMP_DIR/results.csv" 2>/dev/null | cut -d',' -f3 | sort -n)

if [ -n "$LATENCIES" ] && [ "$SUCCESS_COUNT" -gt 0 ]; then
    MIN_LATENCY=$(echo "$LATENCIES" | head -n 1)
    MAX_LATENCY=$(echo "$LATENCIES" | tail -n 1)
    AVG_LATENCY=$(echo "$LATENCIES" | awk '{sum+=$1} END {printf "%.0f", sum/NR}')
    P50_INDEX=$(awk "BEGIN {printf \"%.0f\", $SUCCESS_COUNT * 0.50}")
    P95_INDEX=$(awk "BEGIN {printf \"%.0f\", $SUCCESS_COUNT * 0.95}")
    P99_INDEX=$(awk "BEGIN {printf \"%.0f\", $SUCCESS_COUNT * 0.99}")
    P50_LATENCY=$(echo "$LATENCIES" | sed -n "${P50_INDEX}p")
    P95_LATENCY=$(echo "$LATENCIES" | sed -n "${P95_INDEX}p")
    P99_LATENCY=$(echo "$LATENCIES" | sed -n "${P99_INDEX}p")
fi

THROUGHPUT=$(awk "BEGIN {printf \"%.2f\", $SUCCESS_COUNT / $TOTAL_DURATION}")

# Print results
echo ""
echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                    TEST RESULTS                        ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}📊 Summary:${NC}"
echo "   ✅ Successful: $SUCCESS_COUNT ($SUCCESS_RATE%)"
echo "   ❌ Failed: $FAIL_COUNT"
echo "   ⏱️  Duration: ${TOTAL_DURATION}s"
echo "   🚀 Throughput: $THROUGHPUT req/s"
echo ""

if [ -n "$LATENCIES" ]; then
    echo -e "${GREEN}⏱️  Latency (ms):${NC}"
    echo "   Min: ${MIN_LATENCY}ms | Avg: ${AVG_LATENCY}ms | Max: ${MAX_LATENCY}ms"
    echo "   P50: ${P50_LATENCY}ms | P95: ${P95_LATENCY}ms | P99: ${P99_LATENCY}ms"
    echo ""
fi

# Performance targets
echo -e "${GREEN}🎯 Performance Targets:${NC}"
[ "$SUCCESS_COUNT" -ge $((NUM_USERS * 95 / 100)) ] && \
    echo -e "   Success Rate: ${GREEN}✅ PASS${NC}" || \
    echo -e "   Success Rate: ${RED}❌ FAIL${NC}"

if [ -n "$P95_LATENCY" ]; then
    P95_SEC=$(awk "BEGIN {printf \"%.0f\", $P95_LATENCY / 1000}")
    [ "$P95_SEC" -lt 5 ] && \
        echo -e "   P95 Latency: ${GREEN}✅ PASS${NC}" || \
        echo -e "   P95 Latency: ${RED}❌ FAIL${NC}"
fi

THROUGHPUT_INT=$(awk "BEGIN {printf \"%.0f\", $THROUGHPUT}")
[ "$THROUGHPUT_INT" -ge 50 ] && \
    echo -e "   Throughput: ${GREEN}✅ PASS${NC}" || \
    echo -e "   Throughput: ${YELLOW}⚠️  WARN${NC}"

# Save results
cp "$TEMP_DIR/results.csv" "$TEST_RUN_DIR/results.csv"

# Create summary report
cat > "$TEST_RUN_DIR/summary.txt" <<EOF
gRPC CreateUser Stress Test Results
====================================
Timestamp: $TIMESTAMP
Host: $GRPC_HOST
Total Users: $NUM_USERS
Concurrency: $CONCURRENT

Results:
--------
Total Requests: $TOTAL_REQUESTS
Successful: $SUCCESS_COUNT ($SUCCESS_RATE%)
Failed: $FAIL_COUNT
Duration: ${TOTAL_DURATION}s
Throughput: $THROUGHPUT req/s

Latency Statistics (ms):
-------------------------
Min: ${MIN_LATENCY:-N/A}
Average: ${AVG_LATENCY:-N/A}
P50: ${P50_LATENCY:-N/A}
P95: ${P95_LATENCY:-N/A}
P99: ${P99_LATENCY:-N/A}
Max: ${MAX_LATENCY:-N/A}
EOF

echo ""
echo -e "${CYAN}📁 Results saved to: $TEST_RUN_DIR${NC}"
echo "   • results.csv - Detailed per-request data"
echo "   • summary.txt - Test summary report"
echo ""

# Exit code
if [ "$SUCCESS_COUNT" -ge $((NUM_USERS * 95 / 100)) ]; then
    echo -e "${GREEN}✅ Stress test PASSED!${NC}"
    exit 0
else
    echo -e "${RED}❌ Stress test FAILED!${NC}"
    exit 1
fi
