# Docker Configurations

This directory contains Docker configurations for running the Axum Backend application and its dependencies.

## 📁 Directory Structure

- **backend/**: Contains configuration to run the full stack (API + Database + Monitoring).
- **postgres/**: Contains configuration to run only the PostgreSQL database (useful for local development).

## 🚀 Quick Start Guides

### Option 1: Full Stack (API + DB + Monitoring)

Use this option if you want to run the entire system in containers, including the backend API, database, and monitoring tools.

1. **Navigate to the backend directory:**

   ```bash
   cd backend
   ```

2. **Start the services:**

   ```bash
   docker compose up --build -d
   ```

   This will start the following containers:
   - **axum_backend**: The Rust API server running on `http://localhost:3000`
   - **axum_db**: PostgreSQL database on port `5432`
   - **axum_prometheus**: Prometheus metrics on `http://localhost:9090`
   - **axum_grafana**: Grafana dashboards on `http://localhost:3001`

### Option 2: Database Only (For Local Development)

Use this option if you are running the backend locally (e.g., via `cargo run`) and need a PostgreSQL database instance.

1. **Navigate to the postgres directory:**

   ```bash
   cd postgres
   ```

2. **Start the database:**

   ```bash
   docker compose up -d
   ```

   This will start:
   - **postgres_db**: PostgreSQL database on port `5432`

   **Default Credentials:**
   These match the default `.env` configuration for local development.
   - **Host**: `localhost`
   - **Port**: `5432`
   - **User**: `axum`
   - **Password**: `axum123`
   - **Database**: `axum_backend`

## 🛠️ Common Commands

### View Logs

To view logs for the backend service (when running full stack):

```bash
docker compose logs -f backend
```

### Stop Services

To stop the running containers:

```bash
docker compose down
```

### Reset Database

To stop containers and delete the database volume (⚠️ **Warning: All data will be lost**):

```bash
docker compose down -v
```
