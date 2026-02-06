# Axum Backend - Production-Ready Rust Web Service

A modern, scalable backend service built with Rust and Axum, following **Domain-Driven Design (DDD)** and **Clean Architecture** principles.

## 🏗️ Architecture

This project implements a **layered architecture** with clear separation of concerns:

- **Domain Layer**: Pure business logic (framework-agnostic)
- **Application Layer**: Use cases and business orchestration
- **Infrastructure Layer**: Database, cache, external services
- **Presentation Layer**: HTTP handlers, routing, middleware

See [ARCHITECTURE.md](./ARCHITECTURE.md) for detailed documentation.

## 📋 Features

- ✅ **Clean Architecture**: Testable, maintainable, scalable
- ✅ **Type-Safe Database**: SQLx with compile-time verification
- ✅ **Async-First**: Tokio runtime for high performance
- ✅ **Error Handling**: Comprehensive error types and handling
- ✅ **Logging & Tracing**: Structured logging with tracing
- ✅ **Authentication**: JWT-based auth (planned)
- ✅ **Validation**: Input validation with validator
- ✅ **Testing**: Unit, integration, and E2E tests
- ✅ **Docker Ready**: Containerized deployment

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+ (`rustup update`)
- PostgreSQL 14+
- Docker & Docker Compose (optional)

### Setup

1. **Clone and navigate to the project**:
   ```bash
   cd /home/nhitran/RustApps/axum_backend
   ```

2. **Copy environment configuration**:
   ```bash
   cp .env.example .env
   # Edit .env with your database credentials
   ```

3. **Install SQLx CLI** (for database migrations):
   ```bash
   cargo install sqlx-cli --no-default-features --features postgres
   ```

4. **Create database**:
   ```bash
   createdb axum_db
   # Or using psql:
   # psql -U postgres -c "CREATE DATABASE axum_db;"
   ```

5. **Run migrations**:
   ```bash
   sqlx migrate run
   ```

6. **Build and run**:
   ```bash
   cargo run
   ```

The server will start at `http://127.0.0.1:3000`

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run integration tests only
cargo test --test '*'
```

## 📁 Project Structure

```
axum_backend/
├── src/
│   ├── config/              # Configuration management
│   ├── domain/              # Core business logic
│   │   ├── entities/        # Domain entities
│   │   ├── value_objects/   # Value objects
│   │   ├── repositories/    # Repository traits
│   │   └── errors/          # Domain errors
│   ├── application/         # Use cases & orchestration
│   │   ├── dto/             # Data Transfer Objects
│   │   ├── services/        # Application services
│   │   └── use_cases/       # Business use cases
│   ├── infrastructure/      # External concerns
│   │   ├── database/        # Database implementation
│   │   ├── cache/           # Caching layer
│   │   └── external_apis/   # Third-party APIs
│   ├── presentation/        # HTTP layer
│   │   ├── routes/          # Route definitions
│   │   ├── handlers/        # HTTP handlers
│   │   ├── middleware/      # Custom middleware
│   │   └── responses/       # Response types
│   └── shared/              # Shared utilities
│       ├── errors/          # Application errors
│       ├── utils/           # Helper functions
│       └── telemetry/       # Logging & tracing
├── migrations/              # Database migrations
└── tests/                   # Integration tests
```

## 🔧 Development

### Database Migrations

```bash
# Create a new migration
sqlx migrate add migration_name

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Check without building
cargo check
```

## 🐳 Docker

### Development

```bash
docker-compose up -d
```

### Production Build

```bash
docker build -t axum_backend .
docker run -p 3000:3000 --env-file .env axum_backend
```

## 📚 API Documentation

Once running, visit:
- Health Check: `http://localhost:3000/health`
- API Docs: `http://localhost:3000/docs` (planned)

## 🛠️ Technology Stack

- **Web Framework**: [Axum](https://github.com/tokio-rs/axum) 0.7
- **Runtime**: [Tokio](https://tokio.rs/)
- **Database**: PostgreSQL with [SQLx](https://github.com/launchbadge/sqlx)
- **Serialization**: [Serde](https://serde.rs/)
- **Validation**: [Validator](https://github.com/Keats/validator)
- **Logging**: [Tracing](https://github.com/tokio-rs/tracing)
- **Authentication**: [jsonwebtoken](https://github.com/Keats/jsonwebtoken)

## 📖 Documentation

- [Architecture Guide](./ARCHITECTURE.md) - Detailed architecture documentation
- [Implementation Plan](./IMPLEMENTATION_PLAN.md) - Development roadmap

## 🤝 Contributing

1. Follow the architecture patterns defined in ARCHITECTURE.md
2. Write tests for new features
3. Run `cargo fmt` and `cargo clippy` before committing
4. Update documentation as needed

## 📝 License

MIT License - see LICENSE file for details

## 🎯 Roadmap

See [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md) for the complete development roadmap.

### Current Status: Phase 1 - Foundation Setup ✅

### Next Steps:
- [ ] Implement configuration module
- [ ] Set up database connection
- [ ] Create first domain entity (User)
- [ ] Implement user CRUD operations
- [ ] Add authentication

---

**Built with ❤️ using Rust and Axum**
