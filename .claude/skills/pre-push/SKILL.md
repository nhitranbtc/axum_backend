---
name: pre-push
description: Pre-push validation — runs CI checks, commits changes with issue refs, updates CHANGELOG.md, verifies resolved GitHub issues, and closes them. Use before pushing code.
---

# Pre-Push Validation — axum_backend

Validate, commit, update changelog, and reconcile GitHub issues before pushing code.

## When to Use

- When the user says `/pre-push`, "before push", "confirm changes", "ready to push"
- After completing a feature, bug fix, or refactoring session
- Before creating a PR

## Workflow

Execute these steps in order. Stop on any failure and report to the user.

### Step 1: Run CI Checks

Run all checks that the CI pipeline enforces. **All must pass before proceeding.**

```bash
# 1a. Formatting
cargo fmt -- --check

# 1b. Linting (warnings = errors)
cargo clippy -- -D warnings

# 1c. Unit tests (no Docker needed)
cargo test --lib

# 1d. API + integration tests (Docker required)
# Check Docker first
docker info > /dev/null 2>&1
cargo test --test api_tests --test integration_tests
```

If any step fails:

- Report the failure clearly
- Attempt to fix if trivial (unused imports, formatting)
- Re-run the failing step after fix
- Do NOT proceed to commit until all checks pass

### Step 2: Review Changes

Analyze what changed before committing:

```bash
git status
git diff --stat
git diff          # staged + unstaged
```

**Group changes into logical commits.** Each commit should be a single concern:

- Test fixes separate from production code fixes
- CI/config changes separate from source changes
- Refactoring separate from new features

### Step 3: Stage and Commit

For each logical group:

1. **Stage** the related files by name (never `git add -A`)

2. **Draft** a commit message following the project convention:

   ```text
   <type>(<scope>): <description> (#<issue>)
   ```

   Types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `perf`, `ci`

3. **Find the matching issue number:**

   ```bash
   gh issue list --state open --limit 50
   ```

   - Match changes to open issues by reading issue titles and descriptions
   - If no issue exists for a change, ask the user whether to create one
   - Every commit MUST reference an issue number

4. **Commit** using HEREDOC format:

   ```bash
   git commit -m "$(cat <<'EOF'
   <type>(<scope>): <description> (#<issue>)

   <optional body explaining why>

   EOF
   )"
   ```

### Step 4: Update CHANGELOG.md

After committing, update `CHANGELOG.md` in the repository root.

**Changelog format** follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/):

```markdown
## [Unreleased]

### Added
- <description> (<scope>) [#issue] — <short-hash>

### Changed
- <description> (<scope>) [#issue] �� <short-hash>

### Fixed
- <description> (<scope>) [#issue] — <short-hash>

### Security
- <description> (<scope>) [#issue] — <short-hash>
```

**Rules for changelog entries:**

- Add new entries under `[Unreleased]` in the correct category
- Each entry: `- <what changed> (<scope>) [#issue] — <commit-hash>`
- Map commit types to changelog categories:

  | Commit type | Changelog category |
  | ----------- | ------------------ |
  | `feat`      | Added              |
  | `fix`       | Fixed              |
  | `refactor`  | Changed            |
  | `perf`      | Changed            |
  | `ci`        | Changed            |
  | `docs`      | Added or Changed   |
  | `chore`     | Changed            |
  | `security`  | Security           |

- Do NOT duplicate entries — check if the commit is already logged
- Do NOT remove existing entries
- When a release is tagged, move `[Unreleased]` entries into a versioned section:

  ```markdown
  ## [0.6.0] — 2026-04-08
  ```

- Update the comparison links at the bottom of the file

**Stage and commit the changelog update:**

```bash
git add CHANGELOG.md
git commit -m "$(cat <<'EOF'
docs: update CHANGELOG.md

EOF
)"
```

### Step 5: Verify Resolved Issues

Check which open issues are now resolved:

```bash
gh issue list --state open --limit 50
```

For each open issue:

1. **Read** the issue details: `gh issue view <number> --json title,body`
2. **Verify** against the codebase — grep for the problem described:
   - If the issue describes a code pattern (e.g., `unwrap()` in production), grep for it
   - If the issue describes a missing feature, check if it exists now
   - If the issue describes a test failure, run the tests
3. **Classify** the issue:
   - **Resolved** — all acceptance criteria met
   - **Partially resolved** — some criteria met, some remain
   - **Not resolved** — the problem still exists

### Step 6: Close Resolved Issues

Present a table to the user for confirmation:

```markdown
| Issue | Title  | Status                    | Evidence         |
| ----- | ------ | ------------------------- | ---------------- |
| #N    | ...    | Resolved / Partial / Open | What was checked |
```

After user confirms:

- **Resolved**: Close with a comment citing the commit(s)

  ```bash
  gh issue close <N> --comment "Resolved in <commit>. <what was fixed>."
  ```

- **Partially resolved**: Add a comment explaining what remains

  ```bash
  gh issue comment <N> --body "Partially resolved in <commit>. Remaining: <what's left>."
  ```

- **Not resolved**: Skip silently

### Step 7: Final Summary

Report to the user:

```markdown
## Pre-Push Summary

### CI Checks
- cargo fmt:    PASS
- cargo clippy: PASS
- Unit tests:   PASS (N passed)
- API tests:    PASS (N passed)

### Commits (branch: <branch>)
- <hash> <message>
- <hash> <message>

### Changelog
- N new entries added to [Unreleased]

### Issues
- Closed: #N, #N
- Commented: #N
- Still open: #N, #N

Ready to push.
```

## Rules

1. **Never push automatically** — always stop after summary and wait for user
2. **Never commit without an issue reference** — ask user to create one if missing
3. **Never close an issue without verifying** — grep/test to confirm the fix exists
4. **Never use `git add -A`** — stage files by name
5. **Stop on CI failure** — do not commit broken code
6. **Separate concerns** — one logical change per commit
7. **Check for secrets** �� scan staged files for API keys, passwords, tokens before committing
8. **Always update CHANGELOG.md** — every commit must have a changelog entry

## Troubleshooting

| Problem                  | Action                                                |
| ------------------------ | ----------------------------------------------------- |
| `cargo fmt` fails        | Run `cargo fmt` to auto-fix, then re-check            |
| `cargo clippy` warnings  | Fix the warnings, re-run                              |
| Tests fail               | Diagnose and fix; invoke `/test` skill if needed      |
| Docker not running       | Run `sudo systemctl start docker`, retry              |
| No matching issue        | Ask user: create a new issue or skip reference         |
| Issue partially resolved | Comment with progress, keep open                      |
| CHANGELOG.md missing     | Create from template (see Step 4)                     |
| Duplicate changelog entry| Skip — do not add entries already present              |
