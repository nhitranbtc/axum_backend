# Axum Backend - Production-Ready Rust Web Service

> **A modern, scalable backend service built with Rust and Axum, following Domain-Driven Design (DDD) and Clean Architecture principles.**

![Rust](https://img.shields.io/badge/rust-1.75%2B-orange) ![Axum](https://img.shields.io/badge/axum-0.7-blue) ![Diesel](https://img.shields.io/badge/diesel-2.1-green) ![ScyllaDB](https://img.shields.io/badge/scylladb-6.2-blueviolet) ![Docker](https://img.shields.io/badge/docker-ready-blue)

## 🏗️ Architecture

This project implements a **layered architecture** designed for maintainability and scalability.

- **Domain Layer**: Pure business logic, entities, and value objects. Zero external dependencies.
- **Application Layer**: Use cases, DTOs, and business orchestration.
- **Infrastructure Layer**: Database repositories (Diesel), email services (Lettre), Redis caching, configuration.
- **Presentation Layer**: HTTP REST API (Axum), routing, middleware, and documentation (Swagger).

📖 **[Read the Full Architecture Guide](docs/ARCHITECTURE.md)**

---

## 🚀 Quick Start

### Prerequisites

- **Rust**: 1.75+ (`rustup update`)
- **Docker** + **Docker Compose**

---

### 🐳 Docker Guide

All container management is handled by a single script:

```bash
./docker/backend/run_container.sh [options]
```

#### Options

| Flag        | Description                                                                  |
| ----------- | ---------------------------------------------------------------------------- |
| `--single`  | Use a **single-node** ScyllaDB (docker/scylla)                               |
| `--cluster` | Use a **3-node** ScyllaDB cluster _(default)_ (docker/scylla-cluster)        |
| `--build`   | Force rebuild of the backend image _(implies `--clean`)_                     |
| `--clean`   | Drop the ScyllaDB keyspace and flush Redis before starting                   |
| `--stop`    | Stop all containers (without removing them)                                  |
| `--restart` | Stop all containers then redeploy _(combinable with `--single`/`--cluster`)_ |
| `--remove`  | Stop containers, remove volumes, and delete locally built images             |
| `--test`    | Run the full API smoke test (register → verify → login)                      |
| `--help`    | Show usage                                                                   |

---

#### ▶️ Run — Single-Node ScyllaDB (local dev)

```bash
# First run: build image + start all services
./docker/backend/run_container.sh --single --build

# Subsequent runs: reuse existing image
./docker/backend/run_container.sh --single
```

#### ▶️ Run — 3-Node ScyllaDB Cluster (staging / production-like)

```bash
# First run: build image + start cluster
./docker/backend/run_container.sh --cluster --build

# Subsequent runs
./docker/backend/run_container.sh --cluster
```

> **Note:** `--cluster` is the default so you can also just run `./docker/backend/run_container.sh --build`.

---

#### 🧹 Clean & Rebuild

```bash
# Drop DB data and rebuild the image from scratch
./docker/backend/run_container.sh --single --build
./docker/backend/run_container.sh --cluster --build

# Drop DB data only (no rebuild)
./docker/backend/run_container.sh --single --clean
```

#### Stop / Remove

```bash
# Stop all containers without removing them (containers and data preserved)
./docker/backend/run_container.sh --stop

# Full teardown: removes containers, volumes, and locally built images
./docker/backend/run_container.sh --remove
```

---

#### Restart (`--restart`)

Stops all running containers, then performs a fresh deploy — equivalent to `--stop` followed by a full startup.

```bash
# Restart with the default cluster mode
./docker/backend/run_container.sh --restart

# Restart single-node setup
./docker/backend/run_container.sh --single --restart

# Restart, drop DB data, and rebuild the image in one shot
./docker/backend/run_container.sh --restart --clean --build

# Restart and run smoke test afterwards
./docker/backend/run_container.sh --restart --test
```

---

#### 🔍 Smoke Test (register → verify → login)

```bash
# Run E2E smoke test against the already-running stack
./docker/backend/run_container.sh --test

# Deploy single-node, build fresh, then smoke-test in one command
./docker/backend/run_container.sh --single --build --test
```

---

#### 📋 View Live Logs

```bash
# Follow backend application logs
docker logs -f axum_backend

# Tail the last 50 lines, then follow
docker logs --tail 50 -f axum_backend

# View ScyllaDB logs
docker logs -f axum_scylla1

# View Redis logs
docker logs -f axum_redis

# View NATS logs
docker logs -f axum_nats
```

---

#### 🌐 Service Access Points

| Service            | URL / Address                       |
| ------------------ | ----------------------------------- |
| **REST API**       | `http://localhost:3000`             |
| **Swagger UI**     | `http://localhost:3000/swagger-ui/` |
| **Health Check**   | `http://localhost:3000/health`      |
| **ScyllaDB CQL**   | `localhost:9042`                    |
| **Redis**          | `localhost:6379`                    |
| **NATS**           | `localhost:4222`                    |
| **NATS Monitor**   | `http://localhost:8222`             |
| **Scylla Manager** | `http://localhost:5080`             |

---

### Local Development

First, ensure the infrastructure (ScyllaDB, Redis, NATS) is running:

```bash
./docker/backend/run_container.sh --single
```

Then, you can run the binaries locally.

```bash
# REST API server (port 3000)
cargo run --bin axum_backend

# Release profile (max performance, LTO)
cargo run --release --bin axum_backend

# gRPC server (port 50051)
cargo run --bin grpc_server

# NATS event subscriber
cargo run --bin nats_client
```

### 📦 Build Profiles

| Profile     | Command                         | Use Case                                   |
| ----------- | ------------------------------- | ------------------------------------------ |
| **Dev**     | `cargo run`                     | Fast compilation, full debug info          |
| **Staging** | `cargo build --profile staging` | Optimised with debug symbols for profiling |
| **Release** | `cargo build --release`         | Max performance, LTO, stripped symbols     |

## 🔧 Available Binaries

The project provides multiple binary executables:

### REST API Server (Default)

```bash
cargo run
# Runs on http://localhost:3000
```

### gRPC Server

```bash
cargo run --bin grpc_server
# Runs on http://localhost:50051
```

### gRPC Client (Example)

```bash
cargo run --bin grpc_client
# Demonstrates gRPC client usage
```

### Testing gRPC with grpcurl

```bash
# List available services
grpcurl -plaintext localhost:50051 list

# Create a user
grpcurl -plaintext -d '{
  "email": "test@example.com",
  "name": "Test User",
  "password": "SecurePassword123!"
}' localhost:50051 user.UserService/CreateUser
```

### NATS Client (Event Subscriber)

```bash
cargo run --bin nats_client
# Subscribes to NATS events for testing and monitoring
```

📖 **[gRPC Development Guide](docs/grpc_guide.md)** | **[NATS Messaging Guide](docs/NATS_MESSAGING.md)**

---

## 🧪 Testing

The test suite is split into five Cargo test binaries, each covering a distinct layer.  
All tests use an **ephemeral in-process server** backed by a fresh ScyllaDB keyspace — no shared state, fully parallel-safe.

---

### 🔍 Smoke Test (Docker — Full E2E)

Verifies the entire **register → verify → login** workflow against the running container:

```bash
./docker/backend/run_container.sh --test
```

> Spins up a unique test user, reads the confirmation code directly from ScyllaDB,
> verifies the email, then asserts a JWT is returned at login.

---

### 🧪 API Tests (`tests/api_tests.rs`)

HTTP-level integration tests — auth, users, cookies, health, preflight:

```bash
# Run all API tests
cargo test --test api_tests

# Run all Post feature API tests
cargo test --test api_tests post

# Run only the post suite
cargo test --test api_tests api::post

# Run only the auth suite
cargo test --test api_tests api::auth

# Run a single named test
cargo test --test api_tests test_register_success
cargo test --test api_tests test_get_post_success
cargo test --test api_tests test_login_success
cargo test --test api_tests test_login_wrong_credentials
cargo test --test api_tests test_register_duplicate_email
cargo test --test api_tests test_register_invalid_email
cargo test --test api_tests test_set_password_too_short
cargo test --test api_tests test_list_users_access_control
cargo test --test api_tests test_concurrent_registrations
cargo test --test api_tests test_forgot_password_flow
cargo test --test api_tests test_resend_code_flow
cargo test --test api_tests test_full_auth_flow
cargo test --test api_tests test_login_with_code_flow

# Run with output visible
cargo test --test api_tests -- --nocapture

# Run Post tests with output visible
cargo test --test api_tests post -- --nocapture
```

---

### ⚙️ Integration Tests (`tests/integration_tests.rs`)

Infrastructure-level tests — ScyllaDB, NATS, email:

```bash
# Run all integration tests
cargo test --test integration_tests

# Run by sub-module
cargo test --test integration_tests integration::scylla
cargo test --test integration_tests integration::nats
cargo test --test integration_tests integration::email_tests

# Run with output
cargo test --test integration_tests -- --nocapture
```

---

### 🗄️ Redis Tests (`tests/redis_tests.rs`)

Cache-aside, distributed locking, and cluster behaviour:

```bash
# Run all Redis tests
cargo test --test redis_tests

# Run by sub-module
cargo test --test redis_tests redis::cache
cargo test --test redis_tests redis::locking
cargo test --test redis_tests redis::cluster

# Run with output
cargo test --test redis_tests -- --nocapture
```

---

### 📡 gRPC Tests (`tests/grpc_tests.rs`)

gRPC service integration and stress tests:

```bash
# Run all gRPC tests
cargo test --test grpc_tests

# Run standard (non-ignored) tests only
cargo test --test grpc_tests grpc

# Run stress tests (marked #[ignore] — opt-in)
cargo test --test grpc_tests -- --ignored --nocapture
```

---

### 📈 Load Tests (`tests/load_tests.rs`)

High-throughput load scenarios (marked `#[ignore]` by default):

```bash
# Run load tests explicitly
cargo test --test load_tests -- --ignored --nocapture
```

---

### 🔄 Run All Test Suites At Once

```bash
cargo test --tests
```

> **Tip:** Run with `-- --nocapture` appended to see tracing output:
>
> ```bash
> cargo test --tests -- --nocapture
> ```

---

### Stress Testing (Bash Scripts)

```bash
# gRPC CreateUser stress test
./scripts/stress_test_create_user.sh

# REST API Register stress test
./scripts/stress_test_register_user.sh

# Cleanup stress-test users
./scripts/cleanup_stress_test_users.sh
```

📖 **[Stress Testing Guide](scripts/GRPC-README.md)**

---

## ⚡ Redis & ScyllaDB (High Performance Data)

The project leverages Redis for caching and ScyllaDB for high-throughput, low-latency persistence:

### ScyllaDB Features

1. **Native Rust Driver**: High-performance asynchronous driver support.
2. **Event Sourcing & Auditing**: Tracks all domain events at scale.
3. **High-Velocity Reads/Writes**: Optimized for high-throughput user operations.

📖 **[ScyllaDB Integration Guide](docs/SCYLLA_INTEGRATION.md)** | **[ScyllaDB Test Suite](docs/SCYLLA_TEST_SUITE.md)**

---

### Redis Features

1. **Caching (Cache-Aside Pattern)**:
   - Optimizes read-heavy operations (e.g., `GetUser`, `ListUsers`).
   - Automatic cache invalidation on write operations (`UpdateUser`, `DeleteUser`).

2. **Distributed Locking**:
   - Ensures mutual exclusion for critical sections (e.g., User Registration).
   - Prevents race conditions in a distributed environment.

3. **Rate Limiting**:
   - Protects APIs from abuse using Sliding Window algorithm.

📖 **[Full Redis Implementation Guide](docs/REDIS_GUIDE.md)**

---

## NATS Messaging & Event-Driven Architecture

The application uses **NATS** for asynchronous, event-driven communication between services.

### Key Features

1. **Event Publishing**:
   - Automatic event publishing when domain entities change (e.g., User Updated)
   - Versioned events (v1, v2) for backward compatibility
   - Field change tracking for audit trails

2. **Subject Hierarchy**:
   - Pattern: `{env}.{domain}.{version}.{event_type}`
   - Example: `dev.users.v2.updated`
   - Supports wildcard subscriptions

3. **Graceful Degradation**:
   - Event publishing failures don't break main operations
   - Comprehensive logging for debugging

### Event Types

- **UserCreatedEventV2**: Published when users are created
- **UserUpdatedEventV2**: Published when users are updated (with field change tracking)
- **UserDeletedEventV2**: Published when users are deleted

### Quick Start

```bash
# Start NATS server
docker run -d --name nats -p 4222:4222 nats:latest

# Subscribe to events
nats sub "dev.users.v2.*"

# Or use the built-in client
cargo run --bin nats_client
```

📖 **[NATS Integration Guide](docs/nats_guide.md)** | **[NATS Messaging Guide](docs/NATS_MESSAGING.md)**

---

## 📁 Project Structure

```text
axum_backend/
├── src/
│   ├── domain/             # Entities, Value Objects, Repository Traits
│   ├── application/        # Use Cases, DTOs, Commands, Queries
│   ├── infrastructure/     # Database (Diesel), Email, Cache (Redis), Messaging (NATS), Config
│   ├── presentation/       # REST API Routes, Handlers, Middleware
│   ├── grpc/               # gRPC Services, Handlers, Proto implementations
│   ├── bin/                # Binary executables (grpc_server, grpc_client, nats_client)
│   ├── config/             # Configuration management
│   ├── shared/             # Errors, Utils, Telemetry
│   ├── main.rs             # REST API server entry point
│   └── lib.rs              # Library exports
├── proto/                  # Protocol Buffer definitions (.proto files)
├── examples/               # Example code (grpc_load_test)
├── scripts/                # Utility scripts (stress tests, cleanup)
├── tests/                  # Integration and stress tests
│   ├── grpc/               # gRPC integration tests
│   ├── rest/               # REST API tests
│   ├── redis/              # Redis modular tests (cache, locking)
│   ├── redis_tests.rs      # Redis test entry point
│   └── grpc_tests.rs       # gRPC test entry point
├── migrations/             # Diesel SQL migrations
├── templates/              # Email templates (Handlebars)
├── docker/                 # Dockerfiles and deployment scripts
│   ├── backend/            # App container setup
│   └── postgres-docker/    # DB container with init scripts
├── docs/                   # Detailed documentation
│   ├── ARCHITECTURE.md     # Architecture guide
│   ├── GOOSE_LOAD_TESTING.md  # Load testing guide
│   ├── grpc_guide.md       # gRPC development guide
│   ├── REDIS_GUIDE.md      # Redis caching and locking guide
│   ├── SCYLLA_INTEGRATION.md # ScyllaDB architecture and setup guide
│   ├── SCYLLA_TEST_SUITE.md  # ScyllaDB test isolation and infrastructure
│   ├── nats_guide.md       # NATS integration guide
│   ├── NATS_MESSAGING.md   # NATS messaging detailed guide
│   └── ...                 # Additional documentation
└── Cargo.toml              # Dependencies & Build Profiles
```

---

## 🛠️ Technology Stack

### Core

- **Language**: Rust 1.75+
- **Async Runtime**: Tokio
- **Web Framework**: Axum 0.7
- **gRPC Framework**: Tonic 0.12
- **Protocol Buffers**: Prost

### Data & Persistence

- **Databases**: PostgreSQL 16 (Relational) & ScyllaDB 6.2 (NoSQL/High-throughput)
- **ORM**: Diesel 2.1 (for PostgreSQL), scylla-rust-driver (for ScyllaDB)
- **Migrations**: Diesel CLI
- **Caching**: Redis 7.2 (Cache-Aside, Distributed Locking)

### Authentication & Security

- **JWT**: jsonwebtoken (Access + Refresh Tokens)
- **Password Hashing**: Argon2
- **Validation**: validator

### Observability

- **Logging**: Tracing, tracing-subscriber
- **Metrics**: Prometheus
- **API Documentation**: Utoipa (OpenAPI/Swagger)

### Communication

- **Email**: Lettre (SMTP)
- **Templates**: Handlebars
- **Messaging**: NATS (Event-driven architecture)

### Testing & Load Testing

- **Load Testing**: Goose 0.17
- **Integration Tests**: Tokio test
- **Stress Testing**: Custom bash scripts with grpcurl
- **Test Containers**: testcontainers

---

## 🤝 Contributing

1. **Follow the Architecture**: Keep logic in the correct layer.
2. **Run Tests**: Ensure `./tests/run_tests.sh all` passes.
3. **Format**: `cargo fmt` is enforced by the build pipeline.

## 📝 License

MIT License. See [LICENSE](LICENSE) for details.
