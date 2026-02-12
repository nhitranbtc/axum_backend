# gRPC Stress Tests (Rust)

Comprehensive Rust-based stress tests for gRPC services with performance analysis and reporting.

## Overview

The Rust stress tests provide native integration testing with the same features as the bash scripts:

- ✅ Performance metrics collection (latency percentiles, throughput)
- ✅ CSV export for detailed analysis
- ✅ Summary report generation
- ✅ Multiple test scenarios
- ✅ Automatic performance target validation

## Available Tests

### Smoke Test (50 users)

Quick validation test with low load.

```bash
cargo test --test grpc_tests test_smoke_50_users -- --ignored --nocapture
```

**Targets**:

- Success Rate: ≥ 99%
- P95 Latency: < 5000ms

### Baseline Test (100 users)

Establishes performance baseline at 10% capacity.

```bash
cargo test --test grpc_tests test_baseline_100_users -- --ignored --nocapture
```

**Targets**:

- Success Rate: ≥ 95%
- P95 Latency: < 5000ms

### Load Test (1000 users)

Tests at 100% target capacity.

```bash
cargo test --test grpc_tests test_load_1000_users -- --ignored --nocapture
```

**Targets**:

- Success Rate: ≥ 95%
- Throughput: ≥ 50 req/s
- P95 Latency: < 5000ms

### Stress Test (5000 users)

High concurrency stress test.

```bash
cargo test --test grpc_tests test_stress_5000_users -- --ignored --nocapture
```

**Targets**:

- Success Rate: ≥ 80%

### Spike Test (2000 users)

Tests sudden high load.

```bash
cargo test --test grpc_tests test_spike_2000_users -- --ignored --nocapture
```

**Targets**:

- Success Rate: ≥ 90%

### Endurance Test (10000 users)

Long-duration test for memory leaks and resource exhaustion.

```bash
cargo test --test grpc_tests test_endurance_10000_users -- --ignored --nocapture
```

**Targets**:

- Success Rate: ≥ 95%

## Test Output

### Console Output

```
╔════════════════════════════════════════════════════════╗
║     gRPC User Registration Stress Test (Rust)         ║
╚════════════════════════════════════════════════════════╝

Configuration:
  • Total Users: 50
  • Concurrent Requests: 10
  • Results Directory: /tmp/grpc-stress-test/smoke_test_1770871737
  • Test ID: 1770871737

✅ Progress: 50/50 users created

╔════════════════════════════════════════════════════════╗
║                    TEST RESULTS                        ║
╚════════════════════════════════════════════════════════╝

📊 Summary:
   ✅ Successful: 50 (100.0%)
   ❌ Failed: 0
   ⏱️  Duration: 17s
   🚀 Throughput: 2.90 req/s

⏱️  Latency (ms):
   Min: 690ms | Avg: 2056ms | Max: 3468ms
   P50: 2407ms | P95: 3120ms | P99: 3468ms

🎯 Performance Targets:
   Success Rate: ✅ PASS
   P95 Latency: ✅ PASS
   Throughput: ⚠️  WARN

📁 Results saved to: /tmp/grpc-stress-test/smoke_test_1770871737
   • results.csv - Detailed per-request data
   • summary.txt - Test summary report
```

### Results Directory

```
/tmp/grpc-stress-test/smoke_test_1770871737/
├── results.csv      # status,user_id,latency_ms
└── summary.txt      # Test summary report
```

## Features

### Performance Metrics

The `PerformanceMetrics` struct tracks:

- Total requests
- Successful/failed requests
- Individual request latencies
- Total test duration

**Calculated Metrics**:

- Success rate (%)
- Throughput (req/s)
- Min/Max/Average latency
- Latency percentiles (P50, P95, P99)

### CSV Export

Detailed per-request data:

```csv
status,user_id,latency_ms
SUCCESS,0,690
SUCCESS,1,1234
SUCCESS,2,987
...
```

### Summary Report

Text summary with all metrics:

```
gRPC User Registration Stress Test Results
===========================================
Timestamp: 1770871737
Host: localhost:50051
Total Users: 50

Results:
--------
Total Requests: 50
Successful: 50 (100.0%)
Failed: 0
Duration: 17s
Throughput: 2.90 req/s

Latency Statistics (ms):
-------------------------
Min: 690ms
Average: 2056ms
P50: 2407ms
P95: 3120ms
P99: 3468ms
Max: 3468ms
```

### Automatic Validation

Each test includes assertions for performance targets:

```rust
assert!(
    metrics.success_rate() >= 95.0,
    "Success rate too low: {:.1}%",
    metrics.success_rate()
);
```

## Advantages Over Bash Scripts

1. **Type Safety** - Compile-time guarantees
2. **Integration** - Direct access to Rust types and APIs
3. **Precision** - Nanosecond-level timing
4. **Debugging** - Full Rust debugging capabilities
5. **CI/CD** - Easy integration with `cargo test`

## Running All Tests

```bash
# Run all stress tests
cargo test --test grpc_tests stress -- --ignored --nocapture

# Run specific test
cargo test --test grpc_tests test_load_1000_users -- --ignored --nocapture

# Run without output capture (see progress)
cargo test --test grpc_tests -- --ignored --nocapture
```

## Customization

The `run_stress_test` function can be called with custom parameters:

```rust
let metrics = run_stress_test(
    1000,           // num_users
    100,            // concurrent
    "custom_test"   // test_name
).await;
```

## Comparison: Bash vs Rust

| Feature         | Bash Script               | Rust Test           |
| --------------- | ------------------------- | ------------------- |
| **Setup**       | Requires grpcurl          | Native Rust         |
| **Speed**       | External process overhead | Direct gRPC calls   |
| **Precision**   | Millisecond timing        | Nanosecond timing   |
| **Integration** | Separate tool             | Part of test suite  |
| **CI/CD**       | Manual setup              | `cargo test`        |
| **Debugging**   | Limited                   | Full Rust debugging |
| **Type Safety** | None                      | Compile-time        |

## Best Practices

1. **Run in Order** - Start with smoke test, then baseline, then load
2. **Monitor Resources** - Watch CPU, memory, database connections
3. **Clean Between Runs** - Tests use unique emails (timestamp-based)
4. **Analyze Results** - Review CSV and summary files
5. **Adjust Targets** - Modify assertions based on your requirements

## Troubleshooting

### Test Fails with Low Success Rate

**Check**:

- Database connection pool size
- Actor pool size
- Password hashing parameters
- Network latency

### High Latency (P95 > 5s)

**Check**:

- Database query performance
- Lock contention
- Resource exhaustion (CPU/Memory)

### Test Hangs

**Check**:

- Database connection pool exhaustion
- Deadlocks
- Network timeouts

## Next Steps

1. Run smoke test to verify setup
2. Run baseline test to establish performance baseline
3. Run load test to validate target capacity
4. Analyze results and optimize
5. Re-run tests to measure improvements

---

**Last Updated**: 2026-02-12  
**Version**: 1.0
