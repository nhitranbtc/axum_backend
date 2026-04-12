# Second Brain — Usage Guide

> For Tran Thi Ai Nhi's AI Second Brain built with Claude Code + local vault.
> Last updated: 2026-04-12

---

## Table of Contents

1. [Quick Start](#1-quick-start)
2. [Daily Workflow](#2-daily-workflow)
3. [Vault Files Explained](#3-vault-files-explained)
4. [GitHub Integration](#4-github-integration)
5. [Memory Search](#5-memory-search)
6. [Daily Sessions](#6-daily-sessions)
7. [Draft System](#7-draft-system)
8. [Habit Tracking](#8-habit-tracking)
9. [Maintenance](#9-maintenance)
10. [Troubleshooting](#10-troubleshooting)

---

## 1. Quick Start

### 1.1 Set Up Environment Variables

Add these to your shell profile (`~/.zshrc`, `~/.bashrc`) or `.env` in the project root:

```bash
# Required for GitHub integration
export GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxx

# Optional: for future integrations
export LINEAR_API_KEY=lin_api_xxxxxxxxxxxx
export SLACK_BOT_TOKEN=xoxb-xxxxxxxxxxxx
export SLACK_APP_TOKEN=xapp-xxxxxxxxxxxx
```

Reload your shell:
```bash
source ~/.zshrc  # or source ~/.bashrc
```

### 1.2 Index the Vault (One-Time Setup)

```bash
# Install dependencies
pip install fastembed sqlite-vec requests

# Index all vault files (run once, then incrementally)
python .claude/scripts/memory_index.py
```

You should see output like:
```
Indexed 12 files, 87 chunks
```

### 1.3 Test the Setup

```bash
# Check integrations
python .claude/scripts/query.py registry status

# Search your memory
python .claude/scripts/memory_search.py "rust ownership patterns"

# Test GitHub integration
python .claude/scripts/query.py github prs --repo nhitranbtc/axum_backend --state open
```

---

## 2. Daily Workflow

### Morning (08:00 UTC+7)

The `memory_reflect.py` script runs automatically. It:
- Reads yesterday's daily log
- Promotes important items to `MEMORY.md`
- Archives yesterday's habits to `HABITS.md` History

**You can also run it manually:**
```bash
python .claude/scripts/memory_reflect.py
```

### Throughout the Day

1. **Start a Claude Code session** — hooks automatically inject SOUL.md + USER.md + MEMORY.md + recent daily logs into context
2. **Work normally** — every session is logged to `daily/YYYY-MM-DD.md`
3. **Ask Claude to update MEMORY.md** when you make important decisions:
   ```
   Update MEMORY.md with the architecture decision we made: using spawn_blocking
   for all CPU-heavy operations in async handlers.
   ```
4. **Ask questions about your notes/code:**
   ```
   Search my vault for notes about substrate storage optimization
   ```

### End of Day

Session ends → `session-end-flush.py` automatically saves a summary to today's daily log.

---

## 3. Vault Files Explained

### SOUL.md — Agent Personality
Loaded on every session start. Defines how Claude behaves:
- Technical, precise, no fluff
- Rust/Substrate-first explanations
- Advisor mode (drafts for review, never sends)

**You can edit this** to change Claude's voice or rules.

### USER.md — Your Profile
Loaded on every session start. Contains:
- Your name, GitHub, timezone, email
- Technical stack and platforms
- Security boundaries (what Claude must NEVER do)
- Integration config

**Update this** when your role, team, or platforms change.

### MEMORY.md — Persistent Memory
Cross-conversation memory index. Updated by:
- `PreCompact` hook (automatic during sessions)
- Manual additions by you or Claude

**Keep it concise** — under 500 words. Detailed items go in daily logs.

Sections:
- Active Projects
- Key Decisions
- Important Patterns (Rust, Substrate, architecture)
- Lessons Learned
- Integration State
- Recent Context

### daily/YYYY-MM-DD.md — Daily Logs
Append-only session logs. Format:
```
## Session: 14:30 UTC+7
Session at 2026-04-12 14:30 UTC+7: 3 user messages, 2 assistant responses...

## Notes
- Investigating substrate runtime storage issue

## Decisions
- Using spawn_blocking for RocksDB access in async handlers

## Tasks
- [x] Fix storage race condition
- [ ] Test parachain upgrade

## Research
- New Polkadot SDK release: v1.2.0 — runtime API changes
```

### HEARTBEAT.md — Heartbeat Checklist
Defines what the heartbeat monitors. Edit to add/remove checks.

### HABITS.md — Habit Pillars
5 pillars: Main Project, Learning, Health, Community, Side Project.
- Auto-detected items checked by heartbeat
- Subjective items (Health, Community) require self-reporting

---

## 4. GitHub Integration

### Setup

1. Create a GitHub Personal Access Token:
   - Go to: https://github.com/settings/tokens
   - Scopes needed: `repo` (full control), `read:user`
   - Copy the token

2. Add to your environment:
   ```bash
   export GITHUB_TOKEN=ghp_your_token_here
   ```

3. Test:
   ```bash
   python .claude/scripts/query.py github prs --repo nhitranbtc/axum_backend --state open
   ```

### Available Commands

#### List PRs
```bash
python .claude/scripts/query.py github prs --repo owner/repo
python .claude/scripts/query.py github prs --repo owner/repo --state closed
python .claude/scripts/query.py github prs --repo owner/repo --sort created
```

#### PRs Requesting Your Review
```bash
python .claude/scripts/query.py github review-requested --repo owner/repo --user nhitranbtc
```

#### List Issues
```bash
python .claude/scripts/query.py github issues --repo owner/repo
python .claude/scripts/query.py github issues --repo owner/repo --labels bug
python .claude/scripts/query.py github issues --repo owner/repo --assignee nhitranbtc
```

#### Single PR Detail + Changed Files
```bash
python .claude/scripts/query.py github pr-detail --repo owner/repo --number 42
```

#### CI Status for a Branch/Commit
```bash
python .claude/scripts/query.py github status --repo owner/repo --ref develop
python .claude/scripts/query.py github status --repo owner/repo --ref main
python .claude/scripts/query.py github status --repo owner/repo --ref abc1234
```

#### Search Repositories
```bash
python .claude/scripts/query.py github search "substrate runtime"
python .claude/scripts/query.py github search "rust async tokio" --top-k 20
```

### Using with Claude Code

During a Claude Code session, you can ask:
```
Check the status of PR #27 on nhitranbtc/axum_backend
```
Claude will run the command and explain the result.

---

## 5. Memory Search

### Basic Search
```bash
python .claude/scripts/memory_search.py "substrate storage optimization"
```

Output:
```
Top 5 results for: substrate storage optimization

============================================================
  [0.847] Dynamous/Memory/daily/2026-04-10.md (chunk 2)
  Storage optimization strategies for Substrate runtimes...
============================================================
  [0.812] Dynamous/Memory/MEMORY.md (chunk 0)
  ## Key Decisions: Using spawn_blocking for RocksDB...
```

### Voice Matching (Drafts)
Search only in `drafts/sent/` to find similar past replies for voice matching:
```bash
python .claude/scripts/memory_search.py "technical review feedback" --path-prefix drafts/sent
```

### JSON Output
```bash
python .claude/scripts/memory_search.py "rust async patterns" --json
```

### Re-index After Changes
```bash
# Re-index entire vault
python .claude/scripts/memory_index.py

# Re-index single file
python .claude/scripts/memory_index.py --file Dynamous/Memory/daily/2026-04-12.md
```

---

## 6. Daily Sessions

### What Happens Automatically

| Event | What Runs | Effect |
|-------|-----------|--------|
| Session starts | `session-start-context.py` | SOUL + USER + MEMORY + recent dailies injected |
| Before compaction | `pre-compact-flush.py` | Decisions/facts extracted → daily log |
| Session ends | `session-end-flush.py` | Session summary → daily log |
| Stop command | `code-review-graph detect-changes` | Change report printed |

### Manual Memory Updates

Ask Claude to update MEMORY.md:
```
/mem add: Decision — using Actor model for Solana anchor programs, not CQRS
/mem add: Lesson — std::sync::Mutex blocks tokio runtime, use tokio::sync::Mutex instead
/mem add: Project — Polkadot Runtime migration v1.2.0 in progress, ETA 2 weeks
```

Format for MEMORY.md updates:
```
- **2026-04-12: Using Actor model for Solana anchor programs** — cleaner state isolation per program account
```

### Session Start Cache

Every session, `.claude/hooks/session_start_cache.txt` is written with the injected context. This is used by `pre-compact-flush.py` to track what was in context before compaction.

---

## 7. Draft System

> Draft system is active once Phase 6 (Heartbeat) is built.

### How It Works

1. Heartbeat scans GitHub PRs, Slack messages, community posts
2. Messages needing a reply are detected
3. Claude generates a draft reply, saved to `Dynamous/Memory/drafts/active/`
4. You review the draft (NEVER auto-sent — Advisor mode)
5. You manually send the reply on the platform
6. The draft is moved to `drafts/sent/`

### Draft File Format

```
---
name: 2026-04-12_slack_ai-nhi-channel_dear-team
type: slack
source_id: U1234567890
recipient: "#ai-nhi-channel"
subject: Re: Runtime upgrade timeline
context: "We're on track to ship v1.2.0 by end of month..."
created: 2026-04-12T14:30:00+07:00
status: active
---

## Original Message

We're on track to ship v1.2.0 by end of month. Any blockers on your end?

## Draft Reply

We're tracking well. No blockers from my side — runtime storage migration is complete, CI passes on all targets. On track for end of month.
```

### Voice Matching

When drafting, the system searches `drafts/sent/` for similar past replies using `memory_search.py --path-prefix drafts/sent` to match your tone.

---

## 8. Habit Tracking

### Pillars

| Pillar | Auto-Detect | Self-Report |
|--------|-------------|-------------|
| Main Project | Linear issue completed, PR merged | — |
| Learning | Code committed, notes added | — |
| Health | — | Self-report |
| Community | PR review, community post | Self-report |
| Side Project | — | Self-report |

### Daily Flow

1. **Morning (08:00 UTC+7):** Heartbeat creates fresh checklist in `HABITS.md`
2. **Throughout day:** Auto-detected items are checked automatically
3. **18:00 UTC+7:** If pillar unchecked, notification nudge sent
4. **Next morning:** Previous checklist archived to History

### Manual Check

You can manually check items in `HABITS.md`:
```
### Main Project
- [x] Complete substrate storage migration PR
```

---

## 9. Maintenance

### Re-index After Adding Files
```bash
python .claude/scripts/memory_index.py
```

### Update GitHub Token
Edit your shell profile or `.env` with the new token, then:
```bash
source ~/.zshrc
python .claude/scripts/query.py github prs --repo test/test --state open
```
(If this fails with 401, token is invalid)

### Check Integration Status
```bash
python .claude/scripts/query.py registry status
```

### Backup the Vault
```bash
cd Dynamous/Memory
git add .
git commit -m "Session: $(date +%Y-%m-%d)"
git push  # if backed to git
```

### Clean Expired Drafts
```bash
# Move drafts older than 24h to expired/
python .claude/scripts/heartbeat.py --cleanup
```

---

## 10. Troubleshooting

### "No module named 'requests'"
```bash
pip install requests
```

### "fastembed not installed"
```bash
pip install fastembed
```

### "GITHUB_TOKEN not set"
```bash
export GITHUB_TOKEN=ghp_your_token
# Add to ~/.zshrc for persistence
```

### GitHub rate limit hit
Wait ~1 hour. The rate limit resets automatically. Or create a new token with higher limits.

### Memory search returns no results
Re-index the vault:
```bash
python .claude/scripts/memory_index.py
```

### Hooks not running
Check `.claude/settings.json` has the hook entries. Restart Claude Code session.

### Session start context feels stale
Run PreCompact manually by ending and restarting the session, or ask Claude:
```
Compact my context now
```

---

## Next Phases to Build

| Phase | What's Left |
|-------|-------------|
| 4B | Linear integration |
| 4C | Slack integration |
| 5 | Skills (vault-structure + codebase-comprehension) |
| 6 | Heartbeat + reflection + drafts + habits |
| 8 | Security hardening (sanitize, guardrails) |
| 9 | Deployment (cron setup, optional VPS) |

Run `/create-second-brain-prd my-second-brain-requirements.md` to regenerate the full PRD, or say "continue with Phase 4B" to build Linear next.
