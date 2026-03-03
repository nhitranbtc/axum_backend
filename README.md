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
- **Docker**: For running the database (PostgreSQL and ScyllaDB) and local development environment.
- **Cargo Make** (Optional): For running workflows.

### 🛠️ Setup & Run

#### Option 1: Docker (Recommended)

Bootstrap the entire environment (PostgreSQL + ScyllaDB + Redis + App + Monitoring):

```bash
./docker/backend/run_container.sh
```

This will:

1. Start PostgreSQL & ScyllaDB containers
2. Run `cargo check` and `cargo fmt`
3. Build the release binary
4. Launch the application container on `host` network

#### Option 2: Local Development

#### Start REST API Server (Port 3000)

```bash
cargo run
```

#### Start gRPC Server (Port 50051)

```bash
cargo run --bin grpc_server
```

#### Start gRPC Client (Example)

```bash
cargo run --bin grpc_client
```

**Access Points:**

- **REST API**: `http://localhost:3000`
- **gRPC Server**: `http://localhost:50051`
- **Swagger UI**: `http://localhost:3000/swagger-ui/`
- **Health Check**: `GET http://localhost:3000/api/health`

### 📦 Build & Development

We use distinct Cargo profiles for different stages:

| Profile     | Command                         | Use Case                                        |
| ----------- | ------------------------------- | ----------------------------------------------- |
| **Dev**     | `cargo run`                     | Fast compilation, full debug info.              |
| **Staging** | `cargo build --profile staging` | Optimized but with debug symbols for profiling. |
| **Release** | `cargo build --release`         | Max performance, LTO, stripped symbols.         |

---

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

### Integration Tests

```bash
# Run all tests
./tests/run_tests.sh all

# Run specific category
./tests/run_tests.sh authentication
./tests/run_tests.sh users
./tests/run_tests.sh redis

# Run gRPC tests
cargo test --test grpc_tests

# Run Redis tests directly
cargo test --test redis_tests
```

### Stress Testing

#### Bash Scripts (Quick & Easy)

```bash
# gRPC CreateUser stress test
./scripts/stress_test_create_user.sh

# REST API Register stress test
./scripts/stress_test_register_user.sh

# Cleanup test users
./scripts/cleanup_stress_test_users.sh
```

#### Rust Integration Tests

```bash
# Run stress tests (ignored by default)
cargo test --test grpc_tests stress -- --ignored --nocapture
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
