#!/usr/bin/env python3
"""
memory_search.py — Hybrid search CLI.

Usage:
    python memory_search.py "query text" [--path-prefix <path>] [--top-k 5]
    python memory_search.py "substrate runtime storage" --top-k 5
    python memory_search.py "draft tone" --path-prefix drafts/sent  # voice matching
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from db import init_db, search_hybrid
from embeddings import get_embedding, get_embeddings_batch

VAULT_ROOT = Path("Dynamous/Memory")


def search(query: str, path_prefix: str = "", top_k: int = 5) -> list[dict]:
    """
    Execute hybrid search across vault (or a sub-path).

    Returns top-k results with file_path, content, and combined_score.
    """
    init_db()

    # Generate query embedding
    query_bytes = get_embedding(query)

    # Execute hybrid search
    results = search_hybrid(query_bytes, query, top_k=top_k * 2)  # over-fetch for filtering

    # Filter by path_prefix if given
    if path_prefix:
        results = [r for r in results if path_prefix in r["file_path"]]

    return results[:top_k]


def format_result(result: dict) -> str:
    """Format a single result for terminal output."""
    score = result["combined_score"]
    path = result["file_path"]
    idx = result["chunk_idx"]
    content = result["content"].strip()
    # Truncate content for display
    preview = content[:300].replace("\n", " ")
    if len(content) > 300:
        preview += "..."

    return f"\n{'=' * 60}\n  [{score:.3f}] {path} (chunk {idx})\n  {preview}\n"


def main():
    parser = argparse.ArgumentParser(description="Hybrid memory search")
    parser.add_argument("query", type=str)
    parser.add_argument("--path-prefix", type=str, default="",
                        help="Filter to path prefix (e.g., drafts/sent)")
    parser.add_argument("--top-k", type=int, default=5)
    parser.add_argument("--json", action="store_true", help="Output raw JSON")
    args = parser.parse_args()

    try:
        results = search(args.query, path_prefix=args.path_prefix, top_k=args.top_k)
    except Exception as e:
        print(f"Search error: {e}", file=sys.stderr)
        sys.exit(1)

    if not results:
        print("No results found.")
        sys.exit(0)

    if args.json:
        import json
        print(json.dumps(results, indent=2, ensure_ascii=False))
    else:
        header = f"Top {len(results)} results for: {args.query}"
        if args.path_prefix:
            header += f" (prefix: {args.path_prefix})"
        print(f"\n{header}\n")
        for r in results:
            print(format_result(r))


if __name__ == "__main__":
    main()
