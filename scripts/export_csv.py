#!/usr/bin/env python3
from __future__ import annotations

import csv
import sys
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from scripts.utils.pipeline import CHUNKS_DIR, PROJECT_ROOT, ensure_directory, load_json


EXPORT_DIR = PROJECT_ROOT / "exports"


def main() -> None:
    corpus = load_json(CHUNKS_DIR / "constitution_full_corpus.json")
    ensure_directory(EXPORT_DIR)
    output_path = EXPORT_DIR / "constitution_full_corpus.csv"

    fieldnames = [
        "chunk_id",
        "document_id",
        "title",
        "author",
        "date",
        "source_collection",
        "source_url",
        "document_type",
        "issue_tags",
        "constitutional_clause_tags",
        "text",
        "word_count",
    ]

    with output_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=fieldnames)
        writer.writeheader()
        for chunk in corpus["chunks"]:
            writer.writerow(
                {
                    "chunk_id": chunk["chunk_id"],
                    "document_id": chunk["document_id"],
                    "title": chunk["title"],
                    "author": chunk["author"],
                    "date": chunk["date"],
                    "source_collection": chunk["source_collection"],
                    "source_url": chunk["source_url"],
                    "document_type": chunk["document_type"],
                    "issue_tags": "|".join(chunk["issue_tags"]),
                    "constitutional_clause_tags": "|".join(chunk["constitutional_clause_tags"]),
                    "text": chunk["text"],
                    "word_count": chunk["word_count"],
                }
            )

    print(f"Exported {len(corpus['chunks'])} chunks to {output_path}")


if __name__ == "__main__":
    main()
