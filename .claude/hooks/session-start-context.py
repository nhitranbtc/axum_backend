#!/usr/bin/env python3
"""
SessionStart Hook — Inject memory into every conversation.

Reads SOUL.md + USER.md + MEMORY.md + 3 most recent daily logs.
Injects a concise context summary into the conversation.

Usage: Called automatically by Claude Code on session start.
"""

from __future__ import annotations

import sys
import json
import re
from pathlib import Path
from datetime import datetime, timedelta

VAULT_ROOT = Path("Dynamous/Memory")
HOOKS_DIR = Path(".claude/hooks")
MAX_DAILY_LOGS = 3
MAX_TOTAL_TOKENS = 500
TOKEN_ESTIMATE = 4  # rough chars-per-token estimate


def read_file(path: Path) -> str:
    """Read file contents, return empty string if missing."""
    if not path.exists():
        return ""
    return path.read_text(encoding="utf-8")


def truncate(text: str, max_tokens: int) -> str:
    """Truncate text to approximate token budget."""
    max_chars = max_tokens * TOKEN_ESTIMATE
    if len(text) <= max_chars:
        return text
    return text[:max_chars] + "\n\n[...truncated by SessionStart hook]"


def get_recent_daily_logs() -> list[Path]:
    """Return N most recent daily log files, sorted by name descending."""
    daily_dir = VAULT_ROOT / "daily"
    if not daily_dir.exists():
        return []
    logs = sorted(daily_dir.glob("*.md"), reverse=True)
    return logs[:MAX_DAILY_LOGS]


def build_inject_text() -> str:
    """Build the context injection text for the session."""
    soul = read_file(VAULT_ROOT / "SOUL.md")
    user = read_file(VAULT_ROOT / "USER.md")
    memory = read_file(VAULT_ROOT / "MEMORY.md")
    recent_logs = get_recent_daily_logs()

    # Truncate MEMORY.md to ~100 words (~150 tokens)
    memory_summary = truncate(memory, 150)
    # Daily logs truncated collectively to ~350 tokens
    daily_content = ""
    for log in recent_logs:
        content = read_file(log)
        if content:
            daily_content += f"\n\n--- {log.name} ---\n{content}"

    daily_content = truncate(daily_content, 350) if daily_content else ""

    parts = [
        "=== SESSION START HOOK — INJECTED CONTEXT ===",
        f"Injected at: {datetime.now().strftime('%Y-%m-%d %H:%M UTC+7')}",
        "\n## SOUL.md (Agent Personality)",
        soul,
        "\n## USER.md (User Profile)",
        user,
        "\n## MEMORY.md (Persistent Memory Index — truncated)",
        memory_summary,
        "\n## Recent Daily Logs",
        daily_content,
        "=== END INJECTED CONTEXT ===",
    ]

    return "\n".join(part for part in parts if part)


def main():
    inject_text = build_inject_text()
    # Output as a JSON blob so Claude Code can parse it if needed
    # but also print to stdout for logging
    print(inject_text, file=sys.stdout)
    # Write to a cache file so PreCompact can reference it
    cache_path = HOOKS_DIR / "session_start_cache.txt"
    cache_path.parent.mkdir(parents=True, exist_ok=True)
    cache_path.write_text(inject_text, encoding="utf-8")


if __name__ == "__main__":
    main()
