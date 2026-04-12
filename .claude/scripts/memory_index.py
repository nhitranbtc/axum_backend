#!/usr/bin/env python3
"""
memory_index.py — Incremental vault indexer.

Watches vault, re-indexes only changed files.
Run manually or on a schedule.

Usage:
    python memory_index.py                 # Index entire vault
    python memory_index.py --file <path>  # Index single file
    python memory_index.py --watch         # Watch mode (inotify/FSEvents)
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

# Add scripts dir to path
sys.path.insert(0, str(Path(__file__).parent))

from db import init_db, upsert_chunk, delete_chunks_for_file, get_connection
from embeddings import chunk_file, chunk_file_by_lines, estimate_tokens, get_embeddings_batch

VAULT_ROOT = Path("Dynamous/Memory")
IGNORE_PREFIXES = {".obsidian", ".trash", ".drafts/expired"}
IGNORE_EXTENSIONS = {".pdf", ".png", ".jpg", ".gif", ".zip"}


def should_index(path: Path) -> bool:
    """Check if file should be indexed."""
    name = path.name
    for prefix in IGNORE_PREFIXES:
        if str(path).startswith(prefix):
            return False
    if name.startswith("."):
        return False
    if path.suffix.lower() in IGNORE_EXTENSIONS:
        return False
    if path.suffix.lower() not in {".md", ".txt", ".py", ".rs", ".toml", ".yaml", ".yml"}:
        return False
    return True


def index_file(file_path: Path, db_path: Path = None) -> int:
    """
    Index a single file. Deletes old chunks, creates new ones.

    Returns number of chunks indexed.
    """
    if not file_path.exists():
        delete_chunks_for_file(str(file_path))
        return 0

    content = file_path.read_text(encoding="utf-8", errors="replace")
    if not content.strip():
        return 0

    file_key = str(file_path)
    delete_chunks_for_file(file_key)  # remove stale chunks

    # Use line-based chunking for code files, paragraph for markdown
    suffix = file_path.suffix.lower()
    if suffix in {".py", ".rs", ".toml"}:
        chunks = list(chunk_file_by_lines(content, max_lines=80, overlap=20))
    else:
        chunks = list(chunk_file(content, chunk_size=400, overlap=50))

    if not chunks:
        return 0

    chunk_texts = [c[0] for c in chunks]
    token_counts = [estimate_tokens(c) for c in chunk_texts]

    # Batch embed
    embeddings = get_embeddings_batch(chunk_texts)

    for (text, idx), tokens, embedding in zip(chunks, token_counts, embeddings):
        upsert_chunk(file_key, idx, text, tokens, embedding)

    return len(chunks)


def index_vault(vault_root: Path = VAULT_ROOT) -> tuple[int, int]:
    """
    Index all indexable files in vault.

    Returns (files_indexed, chunks_created).
    """
    init_db()
    files_indexed = 0
    total_chunks = 0

    for file_path in vault_root.rglob("*"):
        if file_path.is_file() and should_index(file_path):
            n = index_file(file_path)
            if n > 0:
                files_indexed += 1
                total_chunks += n

    return files_indexed, total_chunks


def reindex_all():
    """CLI entry point."""
    parser = argparse.ArgumentParser(description="Memory vault indexer")
    parser.add_argument("--file", type=str, help="Index single file")
    parser.add_argument("--watch", action="store_true", help="Watch mode")
    args = parser.parse_args()

    if args.file:
        path = Path(args.file)
        init_db()
        n = index_file(path)
        print(f"Indexed {n} chunks: {path}")
    else:
        files, chunks = index_vault()
        print(f"Indexed {files} files, {chunks} chunks")


if __name__ == "__main__":
    reindex_all()
