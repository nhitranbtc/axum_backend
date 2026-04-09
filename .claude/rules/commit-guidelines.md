# Commit Guidelines — axum_backend

Rules, checklists, and issue resolution process for committing code to this repository.
Every contributor (human or AI) must follow these before creating a commit.

> **Relationship to other rules:** This file governs the commit and issue lifecycle.
> For code-level rules (error handling, async safety, architecture layers), see
> [code-style.md](code-style.md). For project-level conventions (DDD layers, Diesel,
> naming), see [CLAUDE.md](../../CLAUDE.md). CI enforces a subset of these rules
> automatically via `.github/workflows/claude-code-review.yml`.

---

## 1. Commit Message Format

Use [Conventional Commits](https://www.conventionalcommits.org/).

### Structure

```text
<type>(<scope>): <description>

<body>

<footer>
```

### Types

| Type       | When to use                                        |
|------------|----------------------------------------------------|
| `feat`     | New feature or capability                          |
| `fix`      | Bug fix                                            |
| `refactor` | Code restructuring with no behavior change         |
| `docs`     | Documentation only                                 |
| `test`     | Adding or updating tests                           |
| `chore`    | Tooling, deps, config (no production code change)  |
| `perf`     | Performance improvement                            |
| `ci`       | CI/CD pipeline changes                             |

### Scope

Optional but encouraged. Use the domain area that best describes the change:

`auth` | `users` | `config` | `ci` | `security` | `middleware` | `db` | `domain` | `infra` | `shared`

### Rules

| Element     | Rule                                                              |
|-------------|-------------------------------------------------------------------|
| Description | Imperative mood ("add", not "added"), lowercase, no trailing `.`  |
| Length      | Header under 72 characters                                        |
| Body        | Explain **why**, not what — the diff shows what                   |
| Footer      | Reference issues: `Closes #12`, `Fixes #14`                      |
| Line wrap   | Body and footer wrapped at 100 characters                         |

### Examples

```text
feat(auth): add rate limiting to all auth endpoints

Apply tower-governor middleware with configurable per-second and burst
limits via RATE_LIMIT_PER_SECOND and RATE_LIMIT_BURST_SIZE env vars.
Returns HTTP 429 with Retry-After header when exceeded.

Closes #14
```

```text
fix(security): replace thread_rng with OsRng for confirmation codes

Confirmation codes used ~41 bits of entropy (8-char alphanumeric).
Upgraded to 32-byte hex tokens (256 bits) via OsRng.fill_bytes() +
hex::encode(). Widened DB column VARCHAR(20) → VARCHAR(64).

Closes #13
```

```text
refactor(db): migrate LoginUseCase to LoginCommand under CQRS

Move login logic from application/use_cases/auth/login.rs to
application/commands/auth/login.rs. No behavior change — same
error handling and repository trait bounds.
```

---

## 2. Pre-Commit Checklist

Complete every applicable item before committing. No exceptions.

> **CI enforcement:** Items marked with **(CI)** are also enforced by the
> `build-and-test` job in `claude-code-review.yml`. Local verification
> catches failures before pushing.

### 2.1 Compilation and Formatting

- [ ] `cargo check` compiles without errors
- [ ] `cargo fmt -- --check` passes — zero formatting issues **(CI)**
- [ ] `cargo clippy -- -D warnings` passes — zero warnings **(CI)**

### 2.2 Tests

- [ ] `cargo test --lib` passes — unit tests **(CI)**
- [ ] `cargo test` passes with Docker running — API and integration tests **(CI)**
- [ ] New or changed code has corresponding tests
- [ ] Test coverage >= 80% for changed modules
- [ ] Test names are descriptive without `test_` prefix (see [code-style.md](code-style.md))

### 2.3 Code Quality

These rules are defined in [code-style.md](code-style.md) and [CLAUDE.md](../../CLAUDE.md).
Verify them before each commit:

- [ ] No `unwrap()`, `expect()`, or `panic!()` in production code paths
- [ ] Error propagation uses `From` impls or explicit `match` — no `.to_string()` erasure
- [ ] Functions under 50 lines, files under 800 lines **(CI: file size only)**
- [ ] Nesting depth 4 levels max — use early returns and `?` to flatten
- [ ] No blocking I/O in async functions without `spawn_blocking` **(CI)**

### 2.4 Architecture — DDD Layer Boundaries

These are enforced by CI grep checks. Verify locally to avoid failed pushes:

- [ ] `domain/` imports only `uuid`, `chrono`, `serde`, `thiserror`, and `crate::domain::*` **(CI)**
- [ ] `application/` depends on traits (`Arc<R: Trait>`), not concrete infra types **(CI)**
- [ ] `application/` does not import `diesel::*`, `axum::*`, or `crate::infrastructure::database` **(CI)**
- [ ] New write operations go in `application/commands/`, not `use_cases/` (CQRS migration)
- [ ] New read operations go in `application/queries/`, not `use_cases/`
- [ ] Handlers call use cases/commands/queries — never call repository methods directly

### 2.5 Security

Cross-reference with [CLAUDE.md Security Rules](../../CLAUDE.md) and
[code-style.md Security Invariants](code-style.md):

- [ ] No hardcoded secrets (API keys, passwords, tokens, connection strings)
- [ ] No secrets in `.env.example`, `docker-compose.yml`, or scripts — use `${VAR}` references
- [ ] `JWT_SECRET` and `DATABASE_URL` have no fallback defaults — fail at startup if missing
- [ ] All user input validated at system boundaries (DTOs with `validator`, value objects at construction)
- [ ] SQL queries use parameterized statements (Diesel enforces this by default)
- [ ] Error responses do not leak internal details (DB errors, file paths, stack traces)
- [ ] Auth endpoints have rate limiting applied
- [ ] Every privileged handler checks `claims.role` (RBAC) — authentication alone is not authorization
- [ ] Confirmation codes use `OsRng` (CSPRNG), minimum 32-byte hex tokens
- [ ] Refresh tokens hashed with SHA-256 before DB storage
- [ ] Cookie `secure` flag driven by `COOKIE_SECURE` env var / runtime config
- [ ] Swagger UI and `/metrics` gated behind auth or feature flags in production builds
- [ ] Run `cargo audit` if dependencies were added or updated

---

## 3. Staging and Committing

### What to Stage

- Stage specific files by name — avoid `git add -A` or `git add .`
- Review staged changes with `git diff --cached` before committing
- One logical change per commit — separate refactors from features from fixes

### What NOT to Commit

The project `.gitignore` covers most of these, but verify manually:

| Category              | Examples                                         |
|-----------------------|--------------------------------------------------|
| Secrets               | `.env`, `*.pem`, `*.key`, `credentials.json`     |
| Build artifacts       | `target/`                                        |
| IDE config            | `.idea/`, `.vscode/`, `*.iml`                    |
| OS files              | `.DS_Store`, `Thumbs.db`                         |
| Large binaries        | Images, videos, compiled libs                    |
| Temporary files       | `*.swp`, `*.swo`, `*~`                           |

> Use `.env.example` to document required environment variables with placeholder values.
> Never put real secrets in `.env.example`.

---

## 4. Database Migrations

When committing migration files, verify all of the following:

- [ ] Migration created with `diesel migration generate <name>`
- [ ] Migration name follows format: `YYYY-MM-DD-HHMMSS_description`
- [ ] Both `up.sql` and `down.sql` are present and correct
- [ ] `diesel migration run` succeeds on a clean database
- [ ] `diesel migration redo` succeeds (verifies `down.sql` + `up.sql` round-trip)
- [ ] `src/infrastructure/database/schema.rs` is regenerated and included in the commit
- [ ] Schema annotation attributes (e.g., `#[max_length = N]`) match the migration
- [ ] Non-destructive changes preferred (e.g., `ALTER COLUMN TYPE` over drop-and-recreate)
- [ ] `down.sql` includes a warning comment if the rollback is lossy (e.g., truncating data)

---

## 5. Branch and PR Workflow

### Branch Naming

```text
<type>/<short-description>
```

Examples: `feat/user-import`, `fix/critical-security-issues`, `refactor/cqrs-migration`

### Before Creating a PR

- [ ] All pre-commit checks pass (Section 2)
- [ ] Full test suite passes (`cargo test` with Docker running)
- [ ] Commit history is clean and logical — one concern per commit
- [ ] All referenced issues have resolution comments (Section 6)
- [ ] Branch is rebased on latest `main` (no merge commits)

### PR Description Format

```markdown
## Summary
- What changed and why (1-3 bullet points per issue resolved)
- Reference each issue: `Closes #N`

## Changes
- List key files and what changed in each

## Test Plan
- [ ] `cargo test --lib` passes (unit tests)
- [ ] `cargo test` passes with Docker (API + integration tests)
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Manual verification steps (if applicable)
```

### PR with Multiple Issues

When a single PR resolves multiple issues (e.g., a security hardening branch):

- List each issue in the Summary with a one-line description
- Use `Closes #N` for every issue — either in PR body or individual commit footers
- Each issue must still have its own resolution comment (Section 6)

### CI Gate

PRs targeting `main` must pass the `build-and-test` job before the Claude code review
runs. The CI pipeline enforces: formatting, clippy, file size limits, async safety checks,
DDD layer boundaries, unit tests, and API/integration tests. See
`.github/workflows/claude-code-review.yml` for the full pipeline.

---

## 6. Issue Resolution Process

Every issue follows a structured lifecycle: pick up, plan, implement, document, close.
The resolution comment on the issue is the permanent record of what was done and why.

### Step 1: Pick Up the Issue

- Read the issue description, acceptance criteria, and any prior comments
- Identify affected files and the scope of the fix
- Create or switch to the appropriate branch (`<type>/<short-description>`)

### Step 2: Plan the Implementation

Break the work into ordered phases before writing code:

1. **Dependencies** — add/update crates in `Cargo.toml`
2. **Migrations** — schema changes with `up.sql` and `down.sql`
3. **Schema regeneration** — `diesel print-schema` if migrations were added
4. **Core implementation** — the code change itself
5. **Tests** — new or updated tests covering the change
6. **Verification** — `cargo clippy`, `cargo test --lib`, manual checks

### Step 3: Implement and Commit

- Follow the Pre-Commit Checklist (Section 2)
- Reference the issue in the commit footer: `Closes #N`
- One commit per logical change — do not bundle unrelated fixes

### Step 4: Write a Resolution Comment

Post a structured comment on the GitHub issue **before closing it**. This is the
permanent record of the decision for traceability, knowledge transfer, and audit.

The comment must include these sections:

| Section                      | Purpose                                                                   |
|------------------------------|---------------------------------------------------------------------------|
| `## Resolution: <title>`     | One-line summary of the fix                                               |
| `### Problem Description`    | What was wrong, why it matters, affected files, severity                  |
| `### Tasks`                  | Checklist of all acceptance criteria, marked `[x]`                        |
| `### Solution`               | What was implemented — include before/after `rust` code blocks            |
| `### Comparison: Old vs New` | Markdown table comparing key aspects (entropy, performance, format, etc)  |
| `### Why This Approach`      | Technical rationale: alternatives considered, trade-offs, standards cited  |
| `### Reference Projects`     | How established projects/standards solve the same problem (when relevant) |
| `### Implementation Plan`    | Ordered phases as executed (Phase 1, Phase 2, ...)                        |
| `### Files Changed`          | Each file path with a short description of what changed                   |

**Section applicability:**

- **All issues** require: Resolution title, Problem Description, Tasks, Solution, Files Changed
- **Security and architecture issues** additionally require: Comparison table, Why This Approach, Reference Projects
- **Simple bug fixes** may omit: Comparison table, Reference Projects, Implementation Plan (if trivial)

See [issue #13](https://github.com/nhitranbtc/axum_backend/issues/13) for a complete
example of this format applied to a security issue.

### Step 5: Close the Issue

- Verify the resolution comment is complete and accurate
- Close: `gh issue close <N> --reason completed`

### Why This Process Matters

| Benefit                | What it enables                                                |
|------------------------|----------------------------------------------------------------|
| **Traceability**       | Every closed issue has a self-contained explanation of the fix |
| **Knowledge transfer** | Future contributors understand *why*, not just *what*          |
| **Review quality**     | Structured comments surface gaps before merge                  |
| **Audit trail**        | Security issues require documented rationale for compliance    |
| **Onboarding**         | New team members learn patterns by reading resolution comments |
