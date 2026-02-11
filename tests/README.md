# Test Directory

This directory contains all tests for the Axum Backend.

## 📁 Structure

```
tests/
├── common/
│   └── mod.rs              # Shared test utilities and helpers
├── preflight_tests.rs      # Pre-flight system validation (8 tests)
├── integration_tests.rs    # API integration tests (14 tests)
├── stress_tests.rs         # Performance stress tests (5 tests)
├── run_tests.sh            # Convenient test runner script
└── README.md               # This file
```

## 🚀 Quick Start

### Using the Test Runner Script

```bash
# Run quick tests (health check + registration)
./tests/run_tests.sh quick

# Run pre-flight checks
./tests/run_tests.sh preflight

# Run all integration tests
./tests/run_tests.sh integration

# Run stress tests
./tests/run_tests.sh stress

# Run benchmarks
./tests/run_tests.sh bench

# Run all tests
./tests/run_tests.sh all

# Show help
./tests/run_tests.sh help
```

### Using Cargo Directly

```bash
# Run all tests
cargo test -- --nocapture

# Run specific test suite
cargo test --test preflight_tests -- --nocapture
cargo test --test integration_tests -- --nocapture
cargo test --test stress_tests -- --nocapture

# Run specific test
cargo test test_health_check -- --nocapture
cargo test test_user_registration_success -- --nocapture

# Run benchmarks
cargo bench
```

## 📋 Test Suites

### Pre-Flight Tests (`preflight_tests.rs`)

Validates system is ready before starting the server:

- ✅ Configuration loading
- ✅ Database connectivity
- ✅ Database migrations
- ✅ JWT functionality
- ✅ Password hashing
- ✅ Environment variables
- ✅ Database schema
- ✅ Connection pool configuration

**Run:** `./tests/run_tests.sh preflight`

### Integration Tests (`integration_tests.rs`)

Tests API endpoints with real HTTP server:

- ✅ Health check
- ✅ User registration (success, duplicate, invalid, weak password)
- ✅ User login (success, wrong password, nonexistent)
- ✅ Authentication & authorization
- ✅ Concurrent operations
- ✅ Complete user flows

**Run:** `./tests/run_tests.sh integration`

### Stress Tests (`stress_tests.rs`)

Performance and load testing:

- ✅ Health endpoint stress (250 requests)
- ✅ Concurrent registration (50 users)
- ✅ Login burst (250 requests)
- ✅ Authenticated endpoints (250 requests)
- ✅ Mixed workload (250 requests)

**Run:** `./tests/run_tests.sh stress`

## 🛠️ Test Utilities (`common/mod.rs`)

Shared utilities for all tests:

- **`TestServer`** - Helper for integration testing
- **`register_user()`** - Register a test user
- **`login_user()`** - Login and get access token
- **`health_check()`** - Check server health
- **`list_users()`** - List users with auth
- **`unique_email()`** - Generate unique email
- **`unique_name()`** - Generate unique name
- **`assert_success()`** - Assert response success
- **`assert_error()`** - Assert response error

## 📊 Example Output

```bash
$ ./tests/run_tests.sh quick

==================================
  Axum Backend Test Runner
==================================

Running quick integration tests...
Running: Quick Integration Tests
Command: cargo test --test integration_tests -- --nocapture test_health_check test_user_registration_success

running 2 tests
test test_health_check ... ok
test test_user_registration_success ... ok

test result: ok. 2 passed; 0 failed; 0 ignored

✅ Quick Integration Tests completed successfully!
```

## 🎯 Best Practices

1. **Run pre-flight checks before starting server**

   ```bash
   ./tests/run_tests.sh preflight
   ```

2. **Run quick tests during development**

   ```bash
   ./tests/run_tests.sh quick
   ```

3. **Run all tests before committing**

   ```bash
   ./tests/run_tests.sh all
   ```

4. **Use serial execution for database tests**
   ```bash
   cargo test -- --test-threads=1
   ```

## 📚 Documentation

- **[Testing Guide](../docs/TESTING_GUIDE.md)** - Comprehensive guide
- **[Rust Testing Summary](../docs/RUST_TESTING_SUMMARY.md)** - Executive summary
- **[Migration Complete](../docs/PURE_RUST_MIGRATION_COMPLETE.md)** - Migration docs

---

**All tests are written in pure Rust! 🦀**
