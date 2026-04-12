#!/usr/bin/env python3
"""
PreCompact Hook — Flush context before auto-compaction.

Reads the JSONL transcript of the current session, extracts key decisions
and facts, and appends them to the current daily log.

Usage: Called automatically by Claude Code before context compaction.
"""

from __future__ import annotations

import sys
import json
import re
from pathlib import Path
from datetime import datetime

VAULT_ROOT = Path("Dynamous/Memory")
CURRENT_TRANSCRIPT = Path(".claude/hooks/current_transcript.jsonl")
SESSION_START_CACHE = Path(".claude/hooks/session_start_cache.txt")


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


def extract_decisions_and_facts(transcript: list[dict]) -> list[str]:
    """
    Extract decision and fact patterns from transcript messages.

    Looks for:
    - Explicit decisions: "we decided", "the approach is", "going with X"
    - Code patterns discovered: Rust async patterns, Substrate macros, etc.
    - Architecture choices
    - Lessons learned
    - New information about the user or project
    """
    decisions = []
    decision_patterns = [
        r"(?i)decided\s+(to|that|on)",
        r"(?i)the\s+approach\s+is",
        r"(?i)going\s+with",
        r"(?i)we\s+will\s+",
        r"(?i)use\s+(.*?)\s+instead",
        r"(?i)chose\s+",
        r"(?i)pattern\s+discovered",
        r"(?i)lesson\s+learned",
        r"(?i)key\s+decision",
        r"(?i)new\s+information\s+about",
        r"(?i)found\s+that\s+",
        r"(?i)turns\s+out\s+",
    ]

    code_pattern_indicators = [
        r"(?i)async\s+pattern",
        r"(?i)ownership\s+pattern",
        r"(?i)FRAME\s+macro",
        r"(?i)substrate\s+(pallet|runtime|storage)",
        r"(?i)tokio\s+",
        r"(?i)Arc<dyn\s+",
        r"(?i)spawn_blocking",
        r"(?i)decl_module",
        r"(?i)decl_storage",
        r"(?i)\[#[\w_]+\]",
    ]

    all_text = " ".join(
        msg.get("text", "")
        for msg in transcript
        if isinstance(msg, dict) and "text" in msg
    )

    for pattern in decision_patterns + code_pattern_indicators:
        # Strip embedded (?i) flags — re.IGNORECASE is already set at call site
        clean_pattern = re.sub(r"^\(\?i\)", "", pattern)
        matches = re.findall(
            r".{0,120}" + clean_pattern + r".{0,120}", all_text, re.IGNORECASE
        )
        for match in matches:
            # Clean up the match
            cleaned = re.sub(r"\s+", " ", match).strip()
            if len(cleaned) > 30 and len(cleaned) < 300:
                decisions.append(cleaned)

    # Deduplicate while preserving order
    seen = set()
    unique = []
    for d in decisions:
        normalized = d.lower()[:80]
        if normalized not in seen:
            seen.add(normalized)
            unique.append(d)

    return unique[:10]  # Cap at 10 items


def append_to_daily_log(entries: list[str]):
    """Append extracted entries to today's daily log."""
    today = datetime.now().strftime("%Y-%m-%d")
    daily_file = VAULT_ROOT / "daily" / f"{today}.md"

    if not entries:
        return

    section = ["\n## PreCompact Flush\n"]
    section.append(f"_Extracted at {datetime.now().strftime('%Y-%m-%d %H:%M UTC+7')}_")
    section.append("")
    for entry in entries:
        section.append(f"- {entry}")
    section.append("")

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
    entries = extract_decisions_and_facts(transcript)
    if entries:
        append_to_daily_log(entries)
        print(f"PreCompact flushed {len(entries)} entries to daily log", file=sys.stdout)
    else:
        print("PreCompact: no significant decisions or facts found", file=sys.stdout)


if __name__ == "__main__":
    main()
