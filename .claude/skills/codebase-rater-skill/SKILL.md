---
name: codebase-rater
description: >
  Analyze and rate this Axum backend codebase across multiple dimensions: code structure,
  architecture design, security risks, and overall code quality. Produces a scored
  report (0–10 per category) with findings, risk flags, and actionable recommendations.
  Trigger this skill whenever the user asks to "rate", "audit", "review", "score",
  "analyze", or "assess" the codebase — even if they only mention one dimension like
  "check for security issues" or "how good is the structure". Also trigger for phrases
  like "how healthy is this code", "what's wrong with this project", "give me a code
  review", or "roast my codebase".
---

# Codebase Rater Skill

Performs a structured multi-dimensional audit of this Axum backend codebase and produces a scored report.

---

## Step 1 — Recon

Before scoring, build a project map. Run these in order:

```bash
# 1. Directory tree (depth 3)
find . -type f -name "*.rs" | grep -v target | head -200

# 2. Dependency manifest
cat Cargo.toml

# 3. Lock file for pinned versions
cat Cargo.lock 2>/dev/null | grep "^name" | head -50

# 4. Entry points
find . -name "lib.rs" -o -name "main.rs" | grep -v target

# 5. Test coverage signal
find . -name "*.rs" | xargs grep -l "#\[test\]" 2>/dev/null | wc -l
find . -path "*/tests/*" -name "*.rs" | wc -l

# 6. CI/CD presence
ls .github/workflows/ 2>/dev/null || echo "No CI found"

# 7. Docs presence
ls README* CHANGELOG* CLAUDE.md MEMORY.md docs/ 2>/dev/null
```

Read key files identified in recon before scoring. For this DDD/CQRS Axum project, also read:
- `src/main.rs` — startup, router composition, dependency wiring
- `src/domain/mod.rs` — entity and trait re-exports
- `src/presentation/routes/mod.rs` — route tree and OpenAPI config
- `src/config/` — AppConfig, DatabaseConfig
- `CLAUDE.md` — project conventions and known tech debt

---

## Step 2 — Score Each Dimension

Rate each dimension 0–10. Use half-points (e.g., 7.5). Read `references/rubrics.md` for detailed scoring criteria per dimension.

### Dimensions

| # | Dimension | What it covers |
|---|---|---|
| 1 | **Structure** | Directory layout, DDD layer separation, modularity, naming |
| 2 | **Architecture & Design** | DDD/CQRS patterns, coupling, abstraction quality, scalability |
| 3 | **Security** | Auth/authz, input validation, secret management, async safety, SQL injection |
| 4 | **Code Quality** | Readability, consistency, error handling, complexity, Rust idioms |
| 5 | **Testing** | Coverage signal, test quality, edge cases, integration tests |
| 6 | **Dependencies** | Pinned versions, known-bad deps, supply chain hygiene |
| 7 | **Documentation** | README, CLAUDE.md, inline docs, API docs (Swagger), changelogs |

---

## Step 3 — Output Format

ALWAYS produce the report in this exact structure:

---

```
# Codebase Audit Report
**Project:** <name>
**Stack:** <detected stack + versions>
**Date:** <YYYY-MM-DD>
**Files Scanned:** <N>

---

## Overall Score: X.X / 10

| Dimension | Score | Signal |
|---|---|---|
| Structure | X.X / 10 | 🟢 / 🟡 / 🔴 |
| Architecture & Design | X.X / 10 | 🟢 / 🟡 / 🔴 |
| Security | X.X / 10 | 🟢 / 🟡 / 🔴 |
| Code Quality | X.X / 10 | 🟢 / 🟡 / 🔴 |
| Testing | X.X / 10 | 🟢 / 🟡 / 🔴 |
| Dependencies | X.X / 10 | 🟢 / 🟡 / 🔴 |
| Documentation | X.X / 10 | 🟢 / 🟡 / 🔴 |

Signal key: 🟢 8–10  🟡 5–7.9  🔴 0–4.9

---

## 🔴 Critical Findings
<!-- Security risks, data loss risk, correctness bugs. Must fix. -->
- [CRITICAL] <finding> → `file:line`

## 🟡 Warnings
<!-- Design issues, tech debt, missing tests. Should fix. -->
- [WARN] <finding> → `file:line`

## 🟢 Strengths
<!-- What is genuinely well done. Be specific, not generic. -->
- <strength> → `file:line`

---

## Dimension Breakdown

### 1. Structure (X.X / 10)
<2–4 sentences on what was found. Specific file references.>

**Issues:**
- <specific issue> → `file`

**Recommendations:**
- <actionable fix>

---

### 2. Architecture & Design (X.X / 10)
...

### 3. Security (X.X / 10)
...

### 4. Code Quality (X.X / 10)
...

### 5. Testing (X.X / 10)
...

### 6. Dependencies (X.X / 10)
...

### 7. Documentation (X.X / 10)
...

---

## Top 5 Priority Actions

1. **[CRITICAL/HIGH/MED]** <action> — <why> → `file`
2. ...
3. ...
4. ...
5. ...

---

## Verdict

<2–3 sentence honest summary. State if the codebase is production-ready, needs work,
or is in early/prototype state. No sugarcoating.>
```

---

## Stack-Specific Security Checks

### Rust / Axum Backend
Read `references/rust-security.md` for full checklist. Key flags:
- `unsafe` blocks — list every occurrence with justification check
- `unwrap()` / `expect()` in production paths (not tests)
- Hardcoded secrets or fallback defaults for JWT_SECRET, DATABASE_URL
- Missing auth middleware on privileged endpoints
- RBAC not enforced in handlers (authentication ≠ authorization)
- Blocking I/O in async context (std::fs, sync Mutex, Argon2 without spawn_blocking)
- SQL injection risk (should be mitigated by Diesel's parameterized queries)
- Error messages leaking internal details in API responses
- Missing rate limiting on auth endpoints
- Cookie `secure` flag hardcoded to `false`
- Refresh tokens stored unhashed

---

## Scoring Notes

- Be honest. A score of 6/10 is not a bad score for active development.
- Do not inflate scores. 9–10 means near-production-grade with evidence.
- Always cite specific files/lines for critical and warning findings.
- If the project is clearly a prototype/POC, note this in Verdict but still score accurately.

---

## Reference Files

- `references/rubrics.md` — Detailed scoring rubrics per dimension (read when unsure how to score)
- `references/rust-security.md` — Full Rust/Axum backend security checklist
