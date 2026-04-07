---
name: claude-memory
description: >
  Implement, manage, and maintain a structured memory system (MEMORY.md + topic files) for
  any Claude Code project. Use this skill whenever the user wants to set up persistent memory,
  initialize MEMORY.md, update memory after a work session, prune stale context, query what
  Claude remembers about a project, or add a specific architectural decision to memory.
  Also trigger when the user says things like "remember this", "update memory", "what do you
  know about this project", "set up memory for this project", "save this decision", or
  "clean up memory". This skill applies to Axum backend services with DDD architecture,
  CQRS pattern, and Rust-specific conventions.
---

# Claude Memory Skill

Implements a **two-file persistent memory architecture** for Claude Code projects:
- `CLAUDE.md` — Static rules written by the developer. Committed to git. Loaded every session.
- `MEMORY.md` — Dynamic index written/maintained by Claude. Optionally gitignored. ≤200 lines.

Topic files (e.g., `memory/architecture.md`, `memory/decisions.md`) are loaded on demand.

---

## Operations

Detect which operation the user wants and execute accordingly. See `references/prompts.md` for the exact CLI prompts for each operation.

| User Intent | Operation |
|---|---|
| New project, no MEMORY.md exists | `init` |
| Analyze codebase and populate | `populate` |
| End of a work session | `update` |
| Remove stale/noisy entries | `prune` |
| Ask what Claude knows | `query` |
| Save a specific decision now | `append` |
| Full consolidation (manual autoDream) | `consolidate` |
| Set up CLAUDE.md rules | `claude-md` |

---

## MEMORY.md Format Rules

Always enforce these when writing or updating MEMORY.md:

```markdown
# MEMORY.md
<!-- index only — max 200 lines — pointers not explanations -->

## Project
- type: <axum-backend|api-service|...>
- stack: <Rust, Axum 0.7, Diesel 2.1, diesel-async, deadpool, PostgreSQL>
- entry: <src/main.rs>
- architecture: <DDD with CQRS>

## Build & Test
- build: `cargo build --release`
- test: `cargo test`
- unit: `cargo test --lib`
- api: `cargo test --test api_tests`
- lint: `cargo clippy -- -D warnings`
- fmt: `cargo fmt -- --check`

## Architecture
- domain: entities, value objects, repository traits → src/domain/
- application: use cases, commands, queries, DTOs → src/application/
- infrastructure: DB repos, email, cache → src/infrastructure/
- presentation: Axum handlers, middleware, routes → src/presentation/
- shared: cross-cutting utilities → src/shared/
- config: AppConfig, pool config → src/config/

## Decisions
- [YYYY-MM-DD] <decision made> → <affected files>
- ...

## Known Issues / Workarounds
- <issue>: <workaround> → <file:line>

## Topic Files
- memory/architecture.md — deep dive on DDD layers and CQRS migration
- memory/api.md — endpoint inventory and auth flows
- <add as needed>
```

---

## CLAUDE.md Format Rules

CLAUDE.md holds **developer-defined static rules**. It should cover:

```markdown
# CLAUDE.md

## Project Overview
<1-2 sentences>

## Stack
<versions, key crates>

## Architecture
<DDD layers, CQRS pattern, dependency rules>

## Coding Standards
- <error handling patterns>
- <async safety rules>
- <naming conventions>
- <layer dependency rules>

## Build & Test
- <how to build, test, lint>

## Security Rules
- <auth, secrets, RBAC>

## Do Not
- <things Claude should never do in this repo>
```

---

## Topic File Guidelines

Create topic files under `memory/` when a subject exceeds 10 lines in MEMORY.md:

- Max 4KB per file
- Named clearly: `memory/architecture.md`, `memory/api.md`, `memory/migrations.md`
- Always add a pointer in MEMORY.md under `## Topic Files`
- Load on demand — do not preload all topic files

---

## Axum / DDD / Rust-Specific Memory Entries

When working on Axum backend projects with DDD architecture, always capture:

```
## Architecture
- domain: value objects (Email, UserId, UserRole) with parse constructors → src/domain/
- application: CQRS migration in progress — commands/ + queries/ (legacy: use_cases/) → src/application/
- infrastructure: Diesel repos implementing domain traits, diesel-async + deadpool → src/infrastructure/
- presentation: Axum handlers with State extraction, auth middleware → src/presentation/
- shared: JWT utils, password hashing, error types → src/shared/

## Build & Test
- docker: `cd docker/postgres && docker compose up -d` (PostgreSQL for tests)
- test: `cargo test` (requires Docker), `cargo test --lib` (unit only)
- api tests: `cargo test --test api_tests` (uses testcontainers)
- watch: `cargo watch -x run`

## Known Issues
- CQRS migration incomplete — use_cases/ and commands/ coexist
- UserService::get_user_count() returns hardcoded 0
- delete_all on UserRepository trait — remove or restrict to test-only
```

---

## Gitignore Recommendation

Add to `.gitignore` if memory contains environment-specific or private context:
```
MEMORY.md
memory/
```

Or commit if the team should share it — project-specific.

---

## Reference Files

- `references/prompts.md` — All CLI prompts to copy-paste into Claude Code
