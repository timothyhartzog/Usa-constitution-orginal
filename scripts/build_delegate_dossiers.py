#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
import re
import sys
from collections import Counter
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from scripts.delegate_coverage_report import DELEGATES_PATH, pattern_for_terms
from scripts.utils.pipeline import CHUNKS_DIR, DATA_DIR, ensure_directory, load_json, save_json


DOSSIER_DIR = DATA_DIR / "delegates" / "dossiers"
REPORT_DIR = DATA_DIR / "delegates" / "reports"


def text_for_match(chunk: dict[str, object]) -> str:
    return f"{chunk['title']}\n{chunk['author']}\n{chunk['text']}"


def author_matches(delegate: dict[str, object], author: str) -> bool:
    names = [delegate["name"], *delegate.get("aliases", [])]
    return any(str(name).lower() in author.lower() for name in names if len(str(name)) > 4)


def main() -> None:
    parser = argparse.ArgumentParser(description="Build per-delegate research dossiers from the chunk corpus.")
    parser.add_argument("--top", type=int, default=50, help="Top mention chunks to include in each dossier")
    parser.add_argument(
        "--include-comparative",
        action="store_true",
        help="Include comparative constitution collections in mention matching.",
    )
    args = parser.parse_args()

    delegates = load_json(DELEGATES_PATH)["delegates"]
    corpus = load_json(CHUNKS_DIR / "constitution_full_corpus.json")
    chunks = [
        chunk
        for chunk in corpus["chunks"]
        if args.include_comparative or not str(chunk["source_collection"]).startswith("comparative_constitutions_")
    ]

    ensure_directory(DOSSIER_DIR)
    ensure_directory(REPORT_DIR)
    index_rows: list[dict[str, object]] = []

    for delegate in delegates:
        regex = pattern_for_terms([delegate["name"], *delegate.get("aliases", [])])
        mentions: list[dict[str, object]] = []
        authored: list[dict[str, object]] = []
        sources: Counter[str] = Counter()
        documents: Counter[str] = Counter()
        total_mentions = 0

        for chunk in chunks:
            if author_matches(delegate, str(chunk["author"])):
                authored.append(
                    {
                        "chunk_id": chunk["chunk_id"],
                        "document_id": chunk["document_id"],
                        "title": chunk["title"],
                        "source_collection": chunk["source_collection"],
                        "word_count": chunk["word_count"],
                        "preview": chunk.get("preview", ""),
                    }
                )

            found = regex.findall(text_for_match(chunk))
            if not found:
                continue
            count = len(found)
            total_mentions += count
            sources[str(chunk["source_collection"])] += count
            documents[str(chunk["document_id"])] += count
            mentions.append(
                {
                    "chunk_id": chunk["chunk_id"],
                    "document_id": chunk["document_id"],
                    "title": chunk["title"],
                    "author": chunk["author"],
                    "date": chunk["date"],
                    "source_collection": chunk["source_collection"],
                    "document_type": chunk["document_type"],
                    "mention_count": count,
                    "word_count": chunk["word_count"],
                    "preview": chunk.get("preview", ""),
                    "text": chunk["text"],
                }
            )

        mentions.sort(key=lambda row: (-int(row["mention_count"]), str(row["source_collection"]), str(row["title"])))
        authored.sort(key=lambda row: (str(row["source_collection"]), str(row["title"]), str(row["chunk_id"])))
        dossier = {
            "delegate": delegate,
            "coverage": {
                "matching_chunks": len(mentions),
                "total_mentions": total_mentions,
                "authored_chunks": len(authored),
                "source_collections": dict(sources.most_common()),
                "documents": dict(documents.most_common()),
            },
            "authored_chunks": authored[: args.top],
            "top_mentions": mentions[: args.top],
            "all_mention_chunk_ids": [row["chunk_id"] for row in mentions],
        }
        slug = re.sub(r"[^a-z0-9]+", "_", delegate["name"].lower()).strip("_")
        save_json(DOSSIER_DIR / f"{slug}.json", dossier)
        index_rows.append(
            {
                "name": delegate["name"],
                "state": delegate["state"],
                "signed_constitution": delegate["signed_constitution"],
                "matching_chunks": len(mentions),
                "total_mentions": total_mentions,
                "authored_chunks": len(authored),
                "dossier_path": str((DOSSIER_DIR / f"{slug}.json").relative_to(DATA_DIR.parent)),
            }
        )

    with (REPORT_DIR / "delegate_dossier_index.csv").open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(index_rows[0].keys()))
        writer.writeheader()
        writer.writerows(index_rows)

    save_json(
        REPORT_DIR / "delegate_dossier_index.json",
        {
            "metadata": {
                "delegate_count": len(delegates),
                "chunk_count_scanned": len(chunks),
                "include_comparative": args.include_comparative,
                "top_records_per_dossier": args.top,
            },
            "delegates": index_rows,
        },
    )
    print(f"Built {len(index_rows)} delegate dossiers in {DOSSIER_DIR}")


if __name__ == "__main__":
    main()
