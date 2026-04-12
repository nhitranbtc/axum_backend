#!/usr/bin/env python3
"""
embeddings.py — FastEmbed ONNX embedding pipeline.

Models: all-MiniLM-L6-v2 (384-dim, ~80MB cache).
Caching: models cached at ~/.cache/fastembed/

Chunking: ~400 tokens per chunk, 50-token overlap.
Tokens estimated at 4 chars/token.
"""

from __future__ import annotations

import hashlib
import os
import re
from pathlib import Path
from typing import Generator

TOKEN_ESTIMATE = 4  # chars per token
CHUNK_SIZE = 400    # target tokens per chunk
OVERLAP = 50        # overlap tokens between chunks


# ---------------------------------------------------------------------------
# Chunking
# ---------------------------------------------------------------------------

def chunk_file(content: str, chunk_size: int = CHUNK_SIZE,
               overlap: int = OVERLAP) -> Generator[tuple[str, int], None, None]:
    """
    Split file content into overlapping chunks.

    Yields (chunk_text, chunk_index) tuples.
    """
    chars_per_chunk = chunk_size * TOKEN_ESTIMATE
    overlap_chars = overlap * TOKEN_ESTIMATE
    step = chars_per_chunk - overlap_chars

    if len(content) <= chars_per_chunk:
        yield content.strip(), 0
        return

    start = 0
    idx = 0
    while start < len(content):
        chunk = content[start:start + chars_per_chunk].strip()
        if chunk:
            yield chunk, idx
        start += step
        idx += 1


def chunk_file_by_lines(content: str, max_lines: int = 80,
                        overlap: int = 20) -> Generator[tuple[str, int], None, None]:
    """
    Split file content into line-based chunks with overlap.

    Better for code files where breaking mid-function is undesirable.
    """
    lines = content.splitlines(keepends=True)
    step = max_lines - overlap
    start = 0
    idx = 0
    while start < len(lines):
        chunk_lines = lines[start:start + max_lines]
        chunk_text = "".join(chunk_lines)
        if chunk_text.strip():
            yield chunk_text, idx
        start += step
        idx += 1


def estimate_tokens(text: str) -> int:
    """Rough token estimate."""
    return len(text) // TOKEN_ESTIMATE


# ---------------------------------------------------------------------------
# Embedding generation
# ---------------------------------------------------------------------------

def get_embedding(text: str) -> bytes:
    """
    Generate embedding for a single text string.

    Returns raw bytes (384-dimensional float32, BAAI/bge-small-en-v1.5).
    """
    import numpy as np

    try:
        from fastembed import TextEmbedding
    except ImportError:
        raise ImportError(
            "fastembed not installed. Run: pip install fastembed"
        )

    cache_dir = Path(os.path.expanduser("~/.cache/fastembed"))
    cache_dir.mkdir(parents=True, exist_ok=True)

    model = TextEmbedding(
        model_name="BAAI/bge-small-en-v1.5",
        cache_dir=str(cache_dir),
    )

    # fastembed returns a generator; consume one
    embedding = next(iter(model.embed([text])))
    # Convert to raw bytes for storage (fastembed returns ndarray directly in newer versions)
    import numpy as np
    arr = np.asarray(embedding).astype(np.float32)
    return arr.tobytes()


def get_embeddings_batch(texts: list[str]) -> list[bytes]:
    """
    Generate embeddings for a batch of texts.

    Returns list of raw bytes (one per input text).
    """
    import numpy as np

    try:
        from fastembed import TextEmbedding
    except ImportError:
        raise ImportError(
            "fastembed not installed. Run: pip install fastembed"
        )

    cache_dir = Path(os.path.expanduser("~/.cache/fastembed"))
    cache_dir.mkdir(parents=True, exist_ok=True)

    model = TextEmbedding(
        model_name="BAAI/bge-small-en-v1.5",
        cache_dir=str(cache_dir),
    )

    embeddings = list(model.embed(texts))
    import numpy as np
    return [emb.astype(np.float32).tobytes() for emb in embeddings]


def decode_embedding(bytes_: bytes) -> list[float]:
    """Decode raw bytes back to a list of floats."""
    import numpy as np
    arr = np.frombuffer(bytes_, dtype=np.float32)
    return arr.tolist()


# ---------------------------------------------------------------------------
# Normalize vector (for cosine similarity)
# ---------------------------------------------------------------------------

def normalize_bytes(bytes_: bytes) -> bytes:
    """L2-normalize a vector stored as bytes."""
    import numpy as np
    arr = np.frombuffer(bytes_, dtype=np.float32)
    norm = np.linalg.norm(arr)
    if norm == 0:
        return bytes_
    normalized = arr / norm
    return normalized.astype(np.float32).tobytes()
