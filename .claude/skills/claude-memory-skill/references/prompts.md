# Claude Memory — CLI Prompts

Copy-paste these directly into Claude Code CLI (`claude` command).
Replace `<placeholders>` with your actual values.

---

## init — First-time setup for a new project

```
Read the project structure and create two files:

1. MEMORY.md in the project root:
   - Project type, stack, and entry points
   - Build and test commands (check Cargo.toml, Makefile, docker/)
   - Key architectural components with file pointers (DDD layers, CQRS status)
   - Keep under 200 lines, use pointers not explanations
   - Format: sections for Project, Build & Test, Architecture, Decisions, Known Issues, Topic Files

2. CLAUDE.md in the project root (if it doesn't exist):
   - Project overview (2 sentences)
   - Stack and versions
   - Architecture (DDD layers, CQRS pattern, dependency rules)
   - Coding standards and conventions you observe in the code
   - Build, test, lint commands
   - Security rules
   - A "Do Not" section for things that would break the project

Do not invent anything — only write what you can derive from the actual files.
```

---

## populate — Deep scan to fill MEMORY.md from codebase

```
Analyze this entire codebase and populate MEMORY.md with accurate, specific entries:

- Scan Cargo.toml / Cargo.lock for stack and versions
- Identify all DDD layers (domain, application, infrastructure, presentation, shared)
- Map the CQRS migration status: which operations are in commands/ vs use_cases/
- Find all repository traits in domain/ and their implementations in infrastructure/
- List all API route groups and their handlers in presentation/
- Find build, test, benchmark, and deployment commands in Cargo.toml / docker/ / README
- Identify middleware: auth, RBAC, rate limiting
- Find any TODO/FIXME/HACK comments that indicate known issues or workarounds
- Note database migrations in migrations/ directory
- Identify value objects and their parse constructors in domain/

Format as a scannable index under these sections:
Project | Build & Test | Architecture | Decisions | Known Issues | Topic Files

Create topic files under memory/ for any section that would exceed 10 lines.
Add pointers to topic files in MEMORY.md under ## Topic Files.
Keep MEMORY.md itself under 200 lines total.
```

---

## update — After a work session

```
We just finished a work session. Update MEMORY.md to reflect what changed:

1. Review our conversation and the files we modified
2. Add any architectural decisions we made (with today's date YYYY-MM-DD)
3. Add any new patterns or conventions we introduced
4. Add any bugs we hit and the workarounds we used, with file:line pointers
5. Update build/test commands if they changed
6. Track CQRS migration progress (what moved from use_cases/ to commands/ or queries/)
7. Remove or correct anything that is now outdated

Do not rewrite the whole file unless it needs restructuring.
Keep MEMORY.md under 200 lines — move overflow to topic files under memory/.
```

---

## prune — Remove stale entries (manual autoDream)

```
Review MEMORY.md and all files under memory/ and consolidate:

- Remove facts that are derivable directly from source code (no value in storing)
- Remove stale entries that no longer match the codebase (verify against actual files)
- Merge duplicate entries
- Convert vague references ("recently", "we decided to") to factual statements with dates
- Convert all file references to absolute paths from project root
- Ensure every entry is either a pointer or an actionable fact — no narrative prose
- If MEMORY.md exceeds 200 lines, extract sections to topic files under memory/

After pruning, show a summary: N entries removed, N merged, N moved to topic files.
```

---

## query — Ask Claude what it knows

```
Based on MEMORY.md and the project files, tell me:

1. What type of project this is and the full tech stack
2. How to build, test, and run the service
3. The DDD layer structure and where key components live
4. CQRS migration status — what's in commands/queries/ vs legacy use_cases/
5. API endpoint inventory and auth flow
6. Any known issues or workarounds I should be aware of
7. Any decisions made recently that affect how I should work

Also flag: anything in MEMORY.md that looks stale, missing, or inconsistent with the actual code.
```

---

## append — Save a specific decision immediately

```
Add the following to MEMORY.md under the ## Decisions section:

Decision: <your decision here>
Reason: <brief reason>
Date: <YYYY-MM-DD>
Affected files: <list files>

Do not rewrite the whole file. Append only. If the Decisions section doesn't exist, create it.
Keep the entry to 1-2 lines max using pointer format.
```

---

## consolidate — Full autoDream-style consolidation

```
Run a full memory consolidation pass on this project:

Phase 1 — Audit: List all facts in MEMORY.md and memory/ topic files. Flag: stale, duplicate, derivable, vague.
Phase 2 — Distill: For each flagged entry, either rewrite as a hard factual pointer or mark for removal.
Phase 3 — Conflict resolution: Identify contradictions between MEMORY.md entries and actual source files. Resolve in favor of source truth.
Phase 4 — Prune: Remove low-signal facts. Keep only entries that save future context reconstruction time.
Phase 5 — Index sync: Rewrite MEMORY.md with clean entries. Update ## Topic Files pointers. Verify all file:line references exist.

Output a consolidation report: total entries before/after, what was removed and why.
Keep MEMORY.md under 200 lines.
```

---

## claude-md — Set up CLAUDE.md with project standards

```
Create CLAUDE.md in the project root with static rules for this project.

Derive from the codebase:
- Project overview (2 sentences max)
- Stack with pinned versions (from Cargo.toml)
- Architecture: DDD layers, CQRS pattern, dependency rules between layers
- Coding standards: error handling (no unwrap), async safety (no blocking), naming conventions
- Build & test: how to run tests, lint (clippy -D warnings), format (cargo fmt)
- Security rules: no hardcoded secrets, RBAC enforcement, token hashing
- Do Not section: things that would break this project

CLAUDE.md should be committed to git and reviewed by the team.
Keep it under 200 lines. Factual and prescriptive only — no narrative.
```

---

## axum-init — Axum/DDD backend specific init

```
Initialize memory for this Axum backend project with DDD architecture.

Create MEMORY.md with these sections populated from the actual codebase:

## Project
- type: axum-backend
- stack: Rust, Axum version, Diesel version, diesel-async, deadpool
- entry: src/main.rs
- architecture: DDD with CQRS (migration in progress)

## Build & Test
- build: cargo build --release
- test: cargo test (requires Docker for testcontainers)
- unit: cargo test --lib
- api: cargo test --test api_tests
- lint: cargo clippy -- -D warnings
- fmt: cargo fmt -- --check
- docker: cd docker/postgres && docker compose up -d

## Architecture
- List all domain entities and value objects in src/domain/
- List all repository traits and their implementations
- Map CQRS status: commands/ vs queries/ vs legacy use_cases/
- List all API route groups in src/presentation/routes/
- Identify middleware chain: auth, RBAC, rate limiting
- List shared utilities: JWT, password hashing, error types

## Decisions
- (empty, ready for entries)

## Known Issues
- Any TODO/FIXME/HACK found in the codebase with file:line
- Technical debt items from CLAUDE.md

## Topic Files
- (create memory/architecture.md if DDD layer details exceed 10 lines)
- (create memory/api.md if endpoint inventory exceeds 10 lines)
- (create memory/migrations.md if migration history is complex)

Keep MEMORY.md under 200 lines. Move overflow to memory/ topic files.
```
