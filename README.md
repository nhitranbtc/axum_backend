# Axum Backend - Production-Ready Rust Web Service

> **A modern, scalable backend service built with Rust and Axum, following Domain-Driven Design (DDD) and Clean Architecture principles.**

![Rust](https://img.shields.io/badge/rust-1.75%2B-orange) ![Axum](https://img.shields.io/badge/axum-0.7-blue) ![Diesel](https://img.shields.io/badge/diesel-2.1-green) ![Docker](https://img.shields.io/badge/docker-ready-blue)

## 🏗️ Architecture

This project implements a **layered architecture** designed for maintainability and scalability.

- **Domain Layer**: Pure business logic, entities, and value objects. Zero external dependencies.
- **Application Layer**: Use cases, DTOs, and business orchestration.
- **Infrastructure Layer**: Database repositories (Diesel), email services (Lettre), configuration.
- **Presentation Layer**: HTTP REST API (Axum), routing, middleware, and documentation (Swagger).

📖 **[Read the Full Architecture Guide](docs/ARCHITECTURE.md)**

---

## 🚀 Quick Start

### Prerequisites

- **Rust**: 1.75+ (`rustup update`)
- **Docker**: For running the database and local development environment.
- **Cargo Make** (Optional): For running workflows.

### 🛠️ Setup & Run (The Easy Way)

We provide a standalone script to bootstrap the entire environment (Database + App + Monitoring):

```bash
./docker/backend/run_container.sh
```

This will:

1. Start PostgreSQL (with `axum` user and `axum_backend` db).
2. Run `cargo check` and `cargo fmt`.
3. Build the release binary.
4. Launch the application container on `host` network.

**Access Points:**

- **API**: `http://localhost:3000`
- **Swagger UI**: `http://localhost:3000/swagger-ui/`
- **Example Endpoint**: `GET http://localhost:3000/api/health`

### 📦 Build & Development

We use distinct Cargo profiles for different stages:

| Profile     | Command                         | Use Case                                        |
| ----------- | ------------------------------- | ----------------------------------------------- |
| **Dev**     | `cargo run`                     | Fast compilation, full debug info.              |
| **Staging** | `cargo build --profile staging` | Optimized but with debug symbols for profiling. |
| **Release** | `cargo build --release`         | Max performance, LTO, stripped symbols.         |

---

## 🧪 Testing

We have a robust testing suite covering unit, integration, and flow tests.

```bash
# Run all tests
./tests/run_tests.sh all

# Run specific category
./tests/run_tests.sh authentication
./tests/run_tests.sh users
```

---

## 📁 Project Structure

```
axum_backend/
├── src/
│   ├── domain/             # Entities, Value Objects, Repository Traits
│   ├── application/        # Use Cases, DTOs, Commands, Queries
│   ├── infrastructure/     # Database (Diesel), Email, Config
│   ├── presentation/       # Routes, Handlers, Middleware
│   └── shared/             # Errors, Utils, Telemetry
├── tests/                  # Integration tests
├── migrations/             # Diesel SQL migrations
├── docker/                 # Dockerfiles and scripts
│   ├── backend/            # App container setup
│   └── postgres-docker/    # DB container setup with init scripts
├── docs/                   # Detailed documentation
└── Cargo.toml              # Dependencies & Build Profiles
```

---

## 🛠️ Technology Stack

- **Core**: Rust, Tokio
- **Web Framework**: Axum 0.7
- **Database**: PostgreSQL 16, Diesel 2.1 (ORM & Migrations)
- **Auth**: JWT (Access + Refresh Tokens), Argon2 hashing
- **Observability**: Tracing, Prometheus Metrics
- **Documentation**: Utoipa (OpenAPI/Swagger)
- **Email**: Lettre (SMTP)

## 🤝 Contributing

1. **Follow the Architecture**: Keep logic in the correct layer.
2. **Run Tests**: Ensure `./tests/run_tests.sh all` passes.
3. **Format**: `cargo fmt` is enforced by the build pipeline.

## 📝 License

MIT License. See [LICENSE](LICENSE) for details.
