#!/usr/bin/env python3
"""
SessionEnd Hook — Save conversation summary to daily log on exit.

Reads the session start cache (injected context) and current transcript,
generates a session summary, and appends it to today's daily log.

Usage: Called automatically by Claude Code when the session ends.
"""

from __future__ import annotations

import sys
import json
from pathlib import Path
from datetime import datetime

VAULT_ROOT = Path("Dynamous/Memory")
SESSION_START_CACHE = Path(".claude/hooks/session_start_cache.txt")
CURRENT_TRANSCRIPT = Path(".claude/hooks/current_transcript.jsonl")


def read_file(path: Path) -> str:
    if not path.exists():
        return ""
    return path.read_text(encoding="utf-8")


def read_transcript() -> list[dict]:
    """Read the JSONL transcript file."""
    if not CURRENT_TRANSCRIPT.exists():
        return []
    lines = CURRENT_TRANSCRIPT.read_text(encoding="utf-8").splitlines()
    return [json.loads(line) for line in lines if line.strip()]


def estimate_token_count(text: str) -> int:
    """Rough token estimate: ~4 chars per token."""
    return len(text) // 4


def generate_session_summary(transcript: list[dict], session_start_text: str) -> str:
    """
    Generate a one-paragraph session summary from the transcript.

    Summarizes:
    - What was discussed / worked on
    - Key outcomes
    - Any open items or next steps
    """
    if not transcript:
        return "_Session ended with no transcript recorded._"

    # Collect all user and assistant messages
    user_msgs = []
    assistant_msgs = []

    for msg in transcript:
        role = msg.get("role", "")
        text = msg.get("text", "") or msg.get("message", {}).get("content", "")
        if not text:
            continue
        if role == "user":
            user_msgs.append(text)
        elif role == "assistant":
            assistant_msgs.append(text)

    # Count file reads/writes/edits from tool calls
    tool_counts = {"Read": 0, "Edit": 0, "Write": 0, "Bash": 0, "Grep": 0, "Glob": 0}
    for msg in transcript:
        for tool in tool_counts:
            if tool.lower() in str(msg.get("text", "")).lower():
                tool_counts[tool] = tool_counts.get(tool, 0) + 1

    total_tokens = estimate_token_count(" ".join(user_msgs + assistant_msgs))
    num_user_msgs = len(user_msgs)
    num_assistant_msgs = len(assistant_msgs)

    # Last assistant message often contains the closing summary
    last_assistant = assistant_msgs[-1] if assistant_msgs else ""

    # Build summary
    summary_parts = [
        f"Session at {datetime.now().strftime('%Y-%m-%d %H:%M UTC+7')}:",
        f"{num_user_msgs} user messages, {num_assistant_msgs} assistant responses,",
        f"~{total_tokens} tokens of conversation.",
    ]

    tool_summary = ", ".join(
        f"{count} {tool}" for tool, count in tool_counts.items() if count > 0
    )
    if tool_summary:
        summary_parts.append(f"Tools used: {tool_summary}.")

    if last_assistant:
        # Take first 200 chars of last response as outcome indicator
        outcome = last_assistant[:200].replace("\n", " ").strip()
        summary_parts.append(f"Last response: {outcome}...")

    return " ".join(summary_parts)


def append_to_daily_log(summary: str):
    """Append session summary to today's daily log."""
    today = datetime.now().strftime("%Y-%m-%d")
    daily_file = VAULT_ROOT / "daily" / f"{today}.md"

    section = [
        f"\n## Session: {datetime.now().strftime('%H:%M UTC+7')}\n",
        summary,
        "\n",
    ]

    content = "\n".join(section)

    if daily_file.exists():
        existing = daily_file.read_text(encoding="utf-8")
        daily_file.write_text(existing + content, encoding="utf-8")
    else:
        daily_file.write_text(
            f"# {today} — Daily Log\n\n---\n\n## Session Log\n\n## Notes\n\n## Decisions\n\n## Tasks\n\n## Research\n\n{content}\n",
            encoding="utf-8",
        )


def main():
    transcript = read_transcript()
    session_start_text = read_file(SESSION_START_CACHE)
    summary = generate_session_summary(transcript, session_start_text)
    append_to_daily_log(summary)
    print(f"SessionEnd: appended summary to {datetime.now().strftime('%Y-%m-%d')} daily log", file=sys.stdout)


if __name__ == "__main__":
    main()
