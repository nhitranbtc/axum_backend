---
name: create-second-brain-prd
description: Generate a personalized Second Brain PRD from a completed requirements template. Use when the user has filled out my-second-brain-requirements.md and wants to generate their build plan. Triggers on "create my second brain PRD", "generate my PRD", "build my second brain plan", "/create-second-brain-prd", or after completing the requirements template.
argument-hint: <path-to-requirements> [output-path]
---

# Second Brain PRD Generator

Generate a personalized Product Requirements Document for building an AI Second Brain, based on the user's completed requirements template.

**A blank template is bundled with this skill at `${CLAUDE_SKILL_DIR}/my-second-brain-requirements.md`.** Copy it to your workspace and fill it out before running this skill.

## Parameters

- **`$0`** (required) — Path to the filled-out requirements file (e.g., `./my-second-brain-requirements.md`)
- **`$1`** (optional) — Output path for the PRD. Defaults to `.agent/plans/second-brain-prd.md`

## Workflow

1. **Read the requirements** — Read the filled-out requirements file at `$0`. If no argument was provided, ask the user for the path. If they haven't filled one out yet, tell them a blank template is available at `${CLAUDE_SKILL_DIR}/my-second-brain-requirements.md` — they should copy it to their workspace and fill it out first.

2. **Load the architecture reference** — Read `${CLAUDE_SKILL_DIR}/references/architecture-reference.md` for the blueprint.

3. **Research ALL tools and APIs** — Do not assume familiarity with any platform or library. Even common APIs like Gmail or Slack have nuances, rate limits, and SDK-specific patterns that matter for implementation. For every tool in the user's stack, do web research to ensure the PRD contains accurate, specific guidance.

   **Always research these core dependencies:**
   - **Claude Agent SDK** — How to create conversations, system_prompt presets, setting_sources, allowed_tools, streaming responses, credential handling
   - **FastEmbed** — ONNX model loading, batch embedding API, model cache configuration, supported models
   - **Hook system** — Claude Code hook types (PreToolUse, PostToolUse, etc.), callback signatures, settings.json configuration

   **For every platform the user selected (Gmail, Slack, Linear, HubSpot, etc.):**
   - Authentication method (OAuth2 flow, API tokens, bot tokens, etc.)
   - Key SDK/library to use (e.g., google-api-python-client, slack_sdk, etc.)
   - The specific API endpoints needed for the user's top tasks
   - Platform-specific setup requirements (e.g., Slack Socket Mode needs an App Token + Bot Token, Gmail needs OAuth consent screen published to Production for custom domains)
   - Rate limits, pagination patterns, and common gotchas

   **The goal:** Every phase in the PRD should contain enough technical specificity that a coding agent can implement it without guessing. Don't bloat the PRD with raw research - distill it into actionable implementation notes per phase.

4. **Generate the PRD** — Create a phased build plan at the output path ($1, or `.agent/plans/second-brain-prd.md` if not specified) with these sections:

### PRD Structure

The output PRD should have:

**Header:**
- Project name (personalized: "[User's Name]'s Second Brain")
- Date generated
- Summary: 1-2 sentences based on their top tasks

**Phase 1: Foundation (Memory Layer)**
- Set up Obsidian vault (or their preferred notes app integration)
- Create SOUL.md, USER.md, MEMORY.md, daily/ structure
- Customize each file based on their "About You" and "Memory Categories" answers
- Key files to create, estimated complexity: Low

**Phase 2: Hooks (Context Persistence)**
- SessionStart hook (inject memory into every conversation)
- PreCompact hook (save context before auto-compaction)
- SessionEnd hook (save context on exit)
- Key files, estimated complexity: Medium

**Phase 3: Memory Search (Hybrid RAG)**
- Set up chunking + embedding pipeline
- SQLite + sqlite-vec + FTS5 (local) or Postgres + pgvector (VPS)
- Hybrid search: 0.7 vector + 0.3 keyword
- Key files, estimated complexity: Medium

**Phase 4: Integrations (Their Top 3 First)**
- Use their "Integration Priority" rankings
- For each: auth setup, API module, registry entry, query.py subcommand
- Reference the integration_template.py pattern
- Key files per integration, estimated complexity: Medium per integration

**Phase 5: Skills (Starter Pack)**
- Vault structure skill (teach agent their file organization)
- At least one custom skill based on their "Top Tasks"
- Skill anatomy: SKILL.md + scripts/ + references/
- Key files, estimated complexity: Low-Medium

**Phase 6: Proactive Systems (Heartbeat + Reflection)**
- Heartbeat: gather data from integrations → Claude reasons → notify
- Set schedule based on their proactivity level
- Daily reflection: promote important daily log items to MEMORY.md
- Map their proactivity level choice to specific heartbeat behaviors
- **Draft management**: Heartbeat scans platforms for messages needing a reply, generates drafts in the user's voice (stored in `drafts/active/`), tracks lifecycle (active → sent → expired after 24h), uses RAG on `drafts/sent/` for voice-matching
- **Habits tracking**: HABITS.md with customizable pillars, auto-detection rules for objective achievements, daily reset by heartbeat, late-day nudges for unchecked pillars
- Key files, estimated complexity: High

**Phase 7: Chat Interface (Optional)**
- Only include if they checked Chat/Messaging in platforms
- Slack/Discord bot with persistent conversations
- Platform adapter pattern for extensibility
- Key files, estimated complexity: High

**Phase 8: Security Hardening**
- Sanitize all external data (3-layer defense)
- Command guardrails based on their Security Boundaries answers
- API key isolation (Python CLI wrapper pattern)
- Key files, estimated complexity: Medium

**Phase 9: Deployment**
- Based on their Infrastructure answers
- Local: OS scheduler setup (Windows Task Scheduler / cron / launchd)
- If VPS: server setup, vault sync, SSH tunnel
- Cost estimate based on their choices

**Each phase includes:**
- What to build (1-2 sentences)
- Key files to create (with paths)
- Dependencies (which phases must come first)
- Estimated complexity (Low / Medium / High)
- Personalization notes (how their specific answers shape this phase)

**Footer:**
- Recommended build order (phases are mostly sequential but some can parallel)
- "This PRD was generated from your requirements. Revisit and update as your system evolves."

5. **Confirm output** — Tell the user where the PRD was saved (the output path) and suggest they start with Phase 1.

## Personalization Rules

- Use THEIR platform names everywhere (not generic "email" — use "Gmail" if that's what they chose)
- Map their proactivity level to concrete behaviors:
  - Observer → heartbeat notifications only, no drafting, no habit tracking
  - Advisor → heartbeat + draft emails/messages for review, habit tracking with suggestions but no auto-check
  - Assistant → heartbeat + drafts + auto-organize files + auto-log, habit auto-detection for objective pillars
  - Partner → all of the above + send low-risk messages + auto-complete routine tasks + full habit auto-detection
- Map their security boundaries directly into Phase 8 guardrail rules
- Use their memory categories to structure the vault folders in Phase 1
- Their integration priority determines Phase 4 order
