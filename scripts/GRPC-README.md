# Stress Test Scripts

Comprehensive stress testing tools for gRPC services with performance analysis and reporting.

## 🚀 Quick Start

```bash
# Run gRPC CreateUser stress test
NUM_USERS=1000 CONCURRENT=100 ./scripts/stress_test_create_user.sh

# Run gRPC Register stress test
NUM_USERS=1000 CONCURRENT=100 ./scripts/stress_test_register_user.sh

# View results
ls -la /tmp/grpc-stress-test/
cat /tmp/grpc-stress-test/*/summary.txt
```

---

## 📋 Available Scripts

### `stress_test_create_user.sh` - gRPC CreateUser Stress Test

Tests the gRPC `UserService/CreateUser` endpoint under load.

**Features**:

- ✅ Centralized results export to `/tmp/grpc-stress-test`
- ✅ Detailed CSV results with per-request latency
- ✅ Automatic summary report generation
- ✅ Performance target validation
- ✅ Progress indicators (with GNU parallel)
- ✅ Timestamp-based unique emails

**Usage**:

```bash
# Basic usage (500 users, 50 concurrent)
./scripts/stress_test_create_user.sh

# Custom configuration
NUM_USERS=1000 CONCURRENT=100 ./scripts/stress_test_create_user.sh

# Different gRPC server
GRPC_HOST=localhost:50052 ./scripts/stress_test_create_user.sh

# Custom results directory
RESULTS_DIR=/var/log/stress-tests ./scripts/stress_test_create_user.sh
```

### `stress_test_register_user.sh` - gRPC Register Stress Test

Tests the gRPC `UserService/CreateUser` endpoint (alias for create_user test).

**Features**:

- ✅ Same as `stress_test_create_user.sh`
- ✅ Timestamp-based unique emails per test run
- ✅ No database cleanup required between runs

**Usage**:

```bash
# Basic usage
./scripts/stress_test_register_user.sh

# Custom configuration
NUM_USERS=1000 CONCURRENT=100 GRPC_HOST=localhost:50051 ./scripts/stress_test_register_user.sh
```

### `cleanup_stress_test_users.sh` - Database Cleanup

Removes all stress test users from the database.

**Usage**:

```bash
./scripts/cleanup_stress_test_users.sh
```

---

## ⚙️ Configuration

### Environment Variables

#### gRPC Stress Tests

| Variable      | Default                 | Description                     |
| ------------- | ----------------------- | ------------------------------- |
| `GRPC_HOST`   | `localhost:50051`       | gRPC server address             |
| `NUM_USERS`   | `500`                   | Total number of users to create |
| `CONCURRENT`  | `50`                    | Number of concurrent requests   |
| `RESULTS_DIR` | `/tmp/grpc-stress-test` | Results output directory        |

### Results Directory Structure

```
/tmp/grpc-stress-test/
├── grpc_create_user_20260212_110000/
│   ├── results.csv      # status,user_id,latency_ms
│   └── summary.txt      # Test summary report
├── grpc_register_user_20260212_110500/
│   ├── results.csv      # status,user_id,latency_ms
│   └── summary.txt      # Test summary report
└── ...
```

---

## 📊 gRPC Stress Test Planning Guide

### 1. Define Test Objectives

**What to Test**:

- Maximum throughput capacity
- Latency under various loads
- Error handling and recovery
- Resource utilization patterns

**Success Criteria**:

```
✅ Success Rate: ≥ 99.9%
✅ P95 Latency: < 500ms
✅ P99 Latency: < 1000ms
✅ Throughput: ≥ 50 req/s
```

### 2. Capacity Planning

#### Calculate Target Load

```bash
# Example: 100,000 daily active users
DAU=100000
PEAK_HOUR_PERCENTAGE=0.20
REQUESTS_PER_USER_PER_HOUR=10

# Peak requests per second
PEAK_RPS=$(echo "$DAU * $PEAK_HOUR_PERCENTAGE * $REQUESTS_PER_USER_PER_HOUR / 3600" | bc -l)
# Result: ~55.6 RPS

# Apply 2x safety margin
TARGET_RPS=$(echo "$PEAK_RPS * 2" | bc -l)
# Result: ~111 RPS
```

#### Estimate Resources

```bash
# CPU cores needed
CPU_PER_REQUEST_MS=10
TARGET_RPS=111
REQUIRED_CORES=$(echo "($TARGET_RPS * $CPU_PER_REQUEST_MS) / 1000" | bc -l)
# Result: ~1.11 cores → Use 2 cores with headroom

# Connection pool size
AVG_REQUEST_DURATION_MS=200
CONCURRENT_USERS=$(echo "$TARGET_RPS * $AVG_REQUEST_DURATION_MS / 1000" | bc -l)
CONNECTION_POOL=$(echo "$CONCURRENT_USERS * 1.2" | bc -l)
# Result: ~27 connections → Use 30
```

### 3. Test Scenarios

#### Baseline Test (5 minutes)

```bash
# 10% of target capacity
NUM_USERS=500 CONCURRENT=10 ./scripts/stress_test_create_user.sh
```

#### Load Test (30 minutes)

```bash
# 100% of target capacity
NUM_USERS=3000 CONCURRENT=100 ./scripts/stress_test_create_user.sh
```

#### Stress Test (Until failure)

```bash
# Gradually increase load
for concurrent in 50 100 150 200 250; do
    echo "Testing with $concurrent concurrent requests..."
    NUM_USERS=1000 CONCURRENT=$concurrent ./scripts/stress_test_create_user.sh
    sleep 60  # Cool-down period
done
```

#### Spike Test (10 minutes)

```bash
# Sudden traffic spike
NUM_USERS=500 CONCURRENT=50 ./scripts/stress_test_create_user.sh &
sleep 5
NUM_USERS=1500 CONCURRENT=150 ./scripts/stress_test_create_user.sh
```

#### Soak Test (4-24 hours)

```bash
# Long-duration test to detect memory leaks
NUM_USERS=10000 CONCURRENT=80 ./scripts/stress_test_create_user.sh
```

### 4. Performance Targets

| Metric           | Excellent   | Good       | Acceptable | Poor       |
| ---------------- | ----------- | ---------- | ---------- | ---------- |
| **Success Rate** | ≥ 99.9%     | ≥ 99%      | ≥ 95%      | < 95%      |
| **P50 Latency**  | < 100ms     | < 200ms    | < 500ms    | > 500ms    |
| **P95 Latency**  | < 500ms     | < 1000ms   | < 2000ms   | > 2000ms   |
| **P99 Latency**  | < 1000ms    | < 2000ms   | < 5000ms   | > 5000ms   |
| **Throughput**   | ≥ 100 req/s | ≥ 50 req/s | ≥ 25 req/s | < 25 req/s |

### 5. Pre-Test Checklist

- [ ] **Environment Setup**
  - [ ] Production-like infrastructure
  - [ ] Isolated test environment
  - [ ] Monitoring configured (CPU, Memory, Network)
  - [ ] Logging enabled

- [ ] **Database Preparation**
  - [ ] Connection pool configured (20-50 connections)
  - [ ] Indexes optimized
  - [ ] Backup created

- [ ] **Service Configuration**
  - [ ] Actor pool sized (default: 20)
  - [ ] gRPC server running (`cargo run --bin grpc_server`)
  - [ ] Timeouts configured
  - [ ] Rate limiting disabled for testing

### 6. Running Tests

#### Monitor in Real-Time

```bash
# Terminal 1: Run test
NUM_USERS=1000 CONCURRENT=100 ./scripts/stress_test_create_user.sh

# Terminal 2: Monitor resources
htop

# Terminal 3: Monitor database
watch -n 1 'psql -c "SELECT count(*) FROM pg_stat_activity;"'

# Terminal 4: Monitor logs
tail -f logs/grpc_server.log
```

#### Watch for Warning Signs

- ⚠️ Increasing latency over time
- ⚠️ Rising error rates
- ⚠️ Memory growth
- ⚠️ Connection pool exhaustion
- ⚠️ CPU saturation (> 90%)

### 7. Analyzing Results

#### View Summary Report

```bash
cat /tmp/grpc-stress-test/grpc_create_user_*/summary.txt
```

#### Analyze CSV Data

```bash
# Success rate
grep SUCCESS /tmp/grpc-stress-test/grpc_create_user_*/results.csv | wc -l

# Average latency
grep SUCCESS /tmp/grpc-stress-test/grpc_create_user_*/results.csv | \
  cut -d',' -f3 | awk '{sum+=$1} END {print sum/NR "ms"}'

# Latency distribution
grep SUCCESS /tmp/grpc-stress-test/grpc_create_user_*/results.csv | \
  cut -d',' -f3 | sort -n | tail -20
```

#### Interpreting Latency Patterns

**Good Distribution** (Linear scaling):

```
P50: 100ms
P95: 200ms (2x P50)
P99: 300ms (3x P50)
Max: 500ms (5x P50)
```

**Poor Distribution** (Long tail - indicates issues):

```
P50: 100ms
P95: 1000ms (10x P50) ← Contention!
P99: 5000ms (50x P50) ← Major issues!
Max: 30000ms (300x P50) ← Timeouts!
```

### 8. Common Bottlenecks

#### CPU-Bound

**Symptoms**: High CPU (> 80%), linear throughput scaling
**Solutions**:

- Optimize password hashing (use async)
- Add more CPU cores
- Implement caching

#### Memory-Bound

**Symptoms**: High memory usage, GC pauses, OOM errors
**Solutions**:

- Reduce memory allocations
- Increase heap size
- Fix memory leaks

#### I/O-Bound

**Symptoms**: Low CPU, high wait times, connection pool exhaustion
**Solutions**:

- Optimize database queries
- Add indexes
- Increase connection pool size
- Implement caching

#### Network-Bound

**Symptoms**: High network utilization, timeouts
**Solutions**:

- Reduce payload sizes
- Implement compression
- Use connection pooling

### 9. Optimization Workflow

```bash
# 1. Run baseline test
NUM_USERS=500 CONCURRENT=50 ./scripts/stress_test_create_user.sh

# 2. Identify bottleneck (CPU/Memory/I/O/Network)

# 3. Apply optimization

# 4. Re-test
NUM_USERS=500 CONCURRENT=50 ./scripts/stress_test_create_user.sh

# 5. Compare results
diff /tmp/grpc-stress-test/grpc_create_user_*/summary.txt

# 6. Repeat until targets met
```

---

## 🔧 Prerequisites

### Required Tools

- **grpcurl**: gRPC command-line client

  ```bash
  go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest
  ```

- **GNU parallel** (optional, for progress bars):

  ```bash
  # Ubuntu/Debian
  sudo apt-get install parallel

  # macOS
  brew install parallel
  ```

### Server Requirements

- gRPC server running on `localhost:50051`
  ```bash
  cargo run --bin grpc_server
  ```

---

## 📈 Example Test Results

### Successful Test Output

```
╔════════════════════════════════════════════════════════╗
║     gRPC CreateUser Stress Test (Optimized)           ║
╚════════════════════════════════════════════════════════╝

Configuration:
  • gRPC Host: localhost:50051
  • Total Users: 100
  • Concurrent Requests: 20
  • Results Directory: /tmp/grpc-stress-test/grpc_create_user_20260212_114016

📊 Summary:
   ✅ Successful: 100 (100.0%)
   ❌ Failed: 0
   ⏱️  Duration: 2s
   🚀 Throughput: 50.00 req/s

⏱️  Latency (ms):
   Min: 343ms | Avg: 483ms | Max: 964ms
   P50: 444ms | P95: 838ms | P99: 890ms

🎯 Performance Targets:
   Success Rate: ✅ PASS
   P95 Latency: ✅ PASS
   Throughput: ✅ PASS

✅ Stress test PASSED!
```

---

## � Troubleshooting

### Issue: All requests fail with "AlreadyExists"

**Cause**: Users from previous tests exist in database

**Solution**: Use timestamp-based emails (already implemented) or run cleanup:

```bash
./scripts/cleanup_stress_test_users.sh
```

### Issue: Low throughput (< 10 req/s)

**Possible Causes**:

1. Password hashing too slow → Optimize Argon2 parameters
2. Database connection pool too small → Increase to 20-50
3. Actor pool too small → Increase to 20+
4. Network latency → Test on same machine/network

### Issue: High latency (P95 > 5s)

**Possible Causes**:

1. Database query performance → Add indexes, optimize queries
2. Lock contention → Review concurrent access patterns
3. Resource exhaustion → Monitor CPU/Memory/Connections

### Issue: Connection refused

**Possible Causes**:

1. gRPC server not running → Start with `cargo run --bin grpc_server`
2. Wrong port → Check `GRPC_HOST` variable
3. Firewall blocking → Check firewall rules

---

## 📚 Additional Resources

- **Planning Guide**: See `stress_test_planning_guide.md` for detailed methodology
- **gRPC Best Practices**: https://grpc.io/docs/guides/performance/
- **Capacity Planning**: https://sre.google/workbook/capacity-planning/

---

## 🎯 Quick Reference

### Common Test Scenarios

```bash
# Quick smoke test (1 minute)
NUM_USERS=100 CONCURRENT=10 ./scripts/stress_test_create_user.sh

# Standard load test (5 minutes)
NUM_USERS=1000 CONCURRENT=100 ./scripts/stress_test_create_user.sh

# Heavy stress test (10 minutes)
NUM_USERS=5000 CONCURRENT=500 ./scripts/stress_test_create_user.sh

# Endurance test (1 hour)
NUM_USERS=10000 CONCURRENT=100 ./scripts/stress_test_create_user.sh
```

### View All Results

```bash
# List all test runs
ls -la /tmp/grpc-stress-test/

# View latest summary
cat /tmp/grpc-stress-test/grpc_create_user_*/summary.txt | tail -30

# Count successful requests
grep SUCCESS /tmp/grpc-stress-test/grpc_create_user_*/results.csv | wc -l

# Calculate average latency
grep SUCCESS /tmp/grpc-stress-test/grpc_create_user_*/results.csv | \
  cut -d',' -f3 | awk '{sum+=$1; count++} END {print sum/count "ms"}'
```

### Cleanup

```bash
# Remove test users from database
./scripts/cleanup_stress_test_users.sh

# Clear old results (keep last 7 days)
find /tmp/grpc-stress-test -type d -mtime +7 -exec rm -rf {} +
```

---

**Last Updated**: 2026-02-12  
**Version**: 2.0
