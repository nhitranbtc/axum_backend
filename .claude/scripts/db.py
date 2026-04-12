#!/usr/bin/env python3
"""
db.py — Database abstraction for memory search.

Supports:
  - SQLite + FTS5 (local) — vector similarity computed in Python
  - Postgres + pgvector + tsvector+GIN (VPS)

Environment:
  DATABASE_URL=postgresql://...  (VPS, optional)
  LOCAL_DB=.claude/data/memory.db (local, default)
"""

from __future__ import annotations

import os
import sqlite3
from pathlib import Path
from typing import Any
from contextlib import contextmanager

LOCAL_DB = Path(".claude/data/memory.db")
POSTGRES_URL = os.environ.get("DATABASE_URL")

# ---------------------------------------------------------------------------
# Connection factory
# ---------------------------------------------------------------------------


def _local_conn() -> sqlite3.Connection:
    LOCAL_DB.parent.mkdir(parents=True, exist_ok=True)
    # isolation_level=None = autocommit mode (every statement commits immediately)
    # This ensures triggers, FTS updates, and upserts all commit correctly
    conn = sqlite3.connect(str(LOCAL_DB), isolation_level=None)
    conn.execute("PRAGMA journal_mode = WAL")
    conn.execute("PRAGMA foreign_keys = ON")
    return conn


@contextmanager
def get_connection():
    """Yield a DB connection. SQLite for local, Postgres for VPS."""
    if POSTGRES_URL:
        import psycopg2

        conn = psycopg2.connect(POSTGRES_URL)
        conn.autocommit = True
        try:
            yield conn
        finally:
            conn.close()
    else:
        conn = _local_conn()
        try:
            yield conn
        finally:
            conn.close()


# ---------------------------------------------------------------------------
# Schema init
# ---------------------------------------------------------------------------

SQLITE_SCHEMA = """
CREATE TABLE IF NOT EXISTS chunks (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path  TEXT    NOT NULL,
    chunk_idx  INTEGER NOT NULL,
    content    TEXT    NOT NULL,
    token_count INTEGER,
    updated_at TEXT    NOT NULL,
    UNIQUE(file_path, chunk_idx)
);

CREATE TABLE IF NOT EXISTS chunk_vectors (
    chunk_id   INTEGER PRIMARY KEY,
    embedding  BLOB    NOT NULL,
    FOREIGN KEY (chunk_id) REFERENCES chunks(id) ON DELETE CASCADE
);

CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(
    content,
    content='chunks',
    content_rowid='id'
);

CREATE TRIGGER IF NOT EXISTS chunks_fts_insert
AFTER INSERT ON chunks FOR EACH ROW BEGIN
    INSERT INTO chunks_fts(rowid, content) VALUES (new.id, new.content);
END;

CREATE TRIGGER IF NOT EXISTS chunks_fts_delete
AFTER DELETE ON chunks FOR EACH ROW BEGIN
    INSERT INTO chunks_fts(chunks_fts, rowid, content)
        VALUES('delete', old.id, old.content);
END;

CREATE TRIGGER IF NOT EXISTS chunks_fts_update
AFTER UPDATE ON chunks FOR EACH ROW BEGIN
    INSERT INTO chunks_fts(chunks_fts, rowid, content)
        VALUES('delete', old.id, old.content);
    INSERT INTO chunks_fts(rowid, content) VALUES (new.id, new.content);
END;
"""


def init_db():
    """Initialize schema. Run once."""
    with get_connection() as conn:
        if POSTGRES_URL:
            _init_postgres(conn)
        else:
            _init_sqlite(conn)


def _init_sqlite(conn: sqlite3.Connection):
    conn.executescript(SQLITE_SCHEMA)


def _init_postgres(conn):
    cur = conn.cursor()
    cur.execute("CREATE EXTENSION IF NOT EXISTS pgvector")
    cur.execute("""
        CREATE TABLE IF NOT EXISTS chunks (
            id          SERIAL PRIMARY KEY,
            file_path   TEXT    NOT NULL,
            chunk_idx   INTEGER NOT NULL,
            content     TEXT    NOT NULL,
            token_count INTEGER,
            updated_at  TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(file_path, chunk_idx)
        )
    """)
    cur.execute("""
        CREATE TABLE IF NOT EXISTS chunk_vectors (
            chunk_id  INTEGER PRIMARY KEY REFERENCES chunks(id) ON DELETE CASCADE,
            embedding vector(384) NOT NULL
        )
    """)
    cur.execute("""
        CREATE INDEX IF NOT EXISTS chunks_fts_idx ON chunks USING GIN(to_tsvector('english', content))
    """)
    cur.execute("""
        CREATE INDEX IF NOT EXISTS chunk_vectors_idx ON chunk_vectors USING ivfflat(embedding vector_cosine_ops)
    """)


# ---------------------------------------------------------------------------
# Upsert chunk
# ---------------------------------------------------------------------------

SQLITE_UPSERT = """
    INSERT INTO chunks (file_path, chunk_idx, content, token_count, updated_at)
    VALUES (?, ?, ?, ?, ?)
    ON CONFLICT(file_path, chunk_idx)
        DO UPDATE SET content=excluded.content,
                     token_count=excluded.token_count,
                     updated_at=excluded.updated_at;
"""

SQLITE_VEC_UPSERT = """
    INSERT INTO chunk_vectors (chunk_id, embedding)
    VALUES (last_insert_rowid(), ?)
    ON CONFLICT(chunk_id)
        DO UPDATE SET embedding=excluded.embedding;
"""

POSTGRES_UPSERT = """
    INSERT INTO chunks (file_path, chunk_idx, content, token_count, updated_at)
    VALUES (%s, %s, %s, %s, NOW())
    ON CONFLICT(file_path, chunk_idx)
        DO UPDATE SET content=excluded.content,
                     token_count=excluded.token_count,
                     updated_at=NOW()
    RETURNING id
"""

POSTGRES_VEC_UPSERT = """
    INSERT INTO chunk_vectors (chunk_id, embedding)
    VALUES (%s, %s::vector)
    ON CONFLICT(chunk_id)
        DO UPDATE SET embedding=excluded.embedding
"""


def upsert_chunk(file_path: str, chunk_idx: int, content: str, token_count: int, embedding: bytes):
    """Insert or update a chunk and its vector."""
    with get_connection() as conn:
        if POSTGRES_URL:
            cur = conn.cursor()
            cur.execute(POSTGRES_UPSERT, (file_path, chunk_idx, content, token_count))
            chunk_id = cur.fetchone()[0]
            cur.execute(POSTGRES_VEC_UPSERT, (chunk_id, embedding))
        else:
            cur = conn.cursor()
            cur.execute(SQLITE_UPSERT, (file_path, chunk_idx, content, token_count, _now()))
            chunk_id = cur.lastrowid
            cur.execute(SQLITE_VEC_UPSERT, (embedding,))


def _now() -> str:
    from datetime import datetime
    return datetime.utcnow().strftime("%Y-%m-%d %H:%M:%S")


# ---------------------------------------------------------------------------
# Hybrid search
# ---------------------------------------------------------------------------

SQLITE_FTS_ONLY = """
    SELECT c.id, c.file_path, c.chunk_idx, c.content, c.token_count,
           cv.embedding,
           bm25(chunks_fts) AS fts_rank
    FROM chunks c
    JOIN chunks_fts ON c.id = chunks_fts.rowid
    LEFT JOIN chunk_vectors cv ON c.id = cv.chunk_id
    WHERE chunks_fts MATCH ?
    ORDER BY bm25(chunks_fts) ASC
    LIMIT 50
"""


def _cosine_sim(a: bytes, b: bytes) -> float:
    """Compute cosine similarity between two raw float32 vectors."""
    import numpy as np
    va = np.frombuffer(a, dtype=np.float32)
    vb = np.frombuffer(b, dtype=np.float32)
    norm_a = np.linalg.norm(va)
    norm_b = np.linalg.norm(vb)
    if norm_a == 0 or norm_b == 0:
        return 0.0
    return float(np.dot(va, vb) / (norm_a * norm_b))


def search_hybrid(query_embedding: bytes, fts_query: str, top_k: int = 5) -> list[dict[str, Any]]:
    """
    Execute hybrid search: FTS5 for keyword matching, cosine similarity in Python for vectors.

    Args:
        query_embedding: raw bytes of query vector
        fts_query: plain-text query for FTS
        top_k: number of results to return

    Returns:
        list of dicts with file_path, chunk_idx, content, combined_score
    """
    with get_connection() as conn:
        if POSTGRES_URL:
            cur = conn.cursor()
            # Postgres: use tsvector for keyword, vector for similarity
            cur.execute("""
                SELECT c.id, c.file_path, c.chunk_idx, c.content,
                       1 - (cv.embedding <=> %s::vector) AS vec_score,
                       ts_rank(to_tsvector('english', c.content), plainto_tsquery('english', %s)) AS fts_rank
                FROM chunks c
                LEFT JOIN chunk_vectors cv ON c.id = cv.chunk_id
                WHERE to_tsvector('english', c.content) @@ plainto_tsquery('english', %s)
                   OR cv.embedding IS NOT NULL
                ORDER BY vec_score DESC NULLS LAST
                LIMIT 50
            """, (fts_query, fts_query, fts_query))
            rows = cur.fetchall()
        else:
            cur = conn.cursor()
            cur.execute(SQLITE_FTS_ONLY, (fts_query,))
            rows = cur.fetchall()

    results = []
    for row in rows:
        id_, file_path, chunk_idx, content, token_count, embedding_bytes, fts_rank = row
        fts_score = 1.0 / (abs(fts_rank) + 1) if fts_rank else 0.0

        vec_score = 0.0
        if embedding_bytes:
            vec_score = _cosine_sim(query_embedding, embedding_bytes)

        combined = 0.7 * vec_score + 0.3 * fts_score
        results.append({
            "id": id_,
            "file_path": file_path,
            "chunk_idx": chunk_idx,
            "content": content,
            "vec_score": vec_score,
            "fts_score": fts_score,
            "combined_score": combined,
        })

    results.sort(key=lambda x: x["combined_score"], reverse=True)
    return results[:top_k]


# ---------------------------------------------------------------------------
# Delete stale chunks
# ---------------------------------------------------------------------------

def delete_chunks_for_file(file_path: str):
    """Remove all chunks for a file."""
    with get_connection() as conn:
        cur = conn.cursor()
        cur.execute("DELETE FROM chunks WHERE file_path = ?", (file_path,))
