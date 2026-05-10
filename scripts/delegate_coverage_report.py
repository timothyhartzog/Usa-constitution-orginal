#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from scripts.utils.pipeline import CHUNKS_DIR, DATA_DIR, ensure_directory, load_json, save_json


DELEGATES_PATH = DATA_DIR / "delegates" / "federal_convention_delegates.json"
REPORT_DIR = DATA_DIR / "delegates" / "reports"


def pattern_for_terms(terms: list[str]) -> re.Pattern[str]:
    alternatives = [re.escape(term) for term in terms if term]
    return re.compile(r"\b(?:" + "|".join(alternatives) + r")\b", re.I)


def main() -> None:
    parser = argparse.ArgumentParser(description="Report delegate coverage in the current chunk corpus.")
    parser.add_argument("--top", type=int, default=12, help="Number of top matching chunks to store per delegate")
    parser.add_argument(
        "--include-comparative",
        action="store_true",
        help="Include comparative constitution collections. By default these are excluded to avoid surname false positives.",
    )
    args = parser.parse_args()

    delegates = load_json(DELEGATES_PATH)["delegates"]
    corpus = load_json(CHUNKS_DIR / "constitution_full_corpus.json")
    chunks = [
        chunk
        for chunk in corpus["chunks"]
        if args.include_comparative or not str(chunk["source_collection"]).startswith("comparative_constitutions_")
    ]

    rows: list[dict[str, object]] = []
    detail: dict[str, object] = {
        "metadata": {
            "total_delegates": len(delegates),
            "total_chunks": len(chunks),
            "include_comparative": args.include_comparative,
            "top_matches_per_delegate": args.top,
        },
        "delegates": [],
    }

    for delegate in delegates:
        terms = [delegate["name"], *delegate.get("aliases", [])]
        regex = pattern_for_terms(terms)
        source_counts: Counter[str] = Counter()
        document_counts: Counter[str] = Counter()
        matches: list[dict[str, object]] = []
        total_mentions = 0

        for chunk in chunks:
            text = f"{chunk['title']}\n{chunk['author']}\n{chunk['text']}"
            found = regex.findall(text)
            if not found:
                continue
            mention_count = len(found)
            total_mentions += mention_count
            source_counts[chunk["source_collection"]] += mention_count
            document_counts[chunk["document_id"]] += mention_count
            matches.append(
                {
                    "chunk_id": chunk["chunk_id"],
                    "document_id": chunk["document_id"],
                    "title": chunk["title"],
                    "source_collection": chunk["source_collection"],
                    "mention_count": mention_count,
                    "preview": chunk.get("preview", ""),
                }
            )

        matches.sort(key=lambda item: (-int(item["mention_count"]), str(item["source_collection"]), str(item["title"])))
        row = {
            "name": delegate["name"],
            "state": delegate["state"],
            "signed_constitution": delegate["signed_constitution"],
            "matching_chunks": len(matches),
            "total_mentions": total_mentions,
            "top_source_collections": "; ".join(f"{source}:{count}" for source, count in source_counts.most_common(5)),
            "top_documents": "; ".join(f"{doc}:{count}" for doc, count in document_counts.most_common(5)),
        }
        rows.append(row)
        detail["delegates"].append(
            {
                **row,
                "aliases": delegate.get("aliases", []),
                "top_matches": matches[: args.top],
            }
        )

    ensure_directory(REPORT_DIR)
    save_json(REPORT_DIR / "delegate_coverage.json", detail)
    with (REPORT_DIR / "delegate_coverage.csv").open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)

    missing = [row["name"] for row in rows if int(row["matching_chunks"]) == 0]
    thin = [row["name"] for row in rows if 0 < int(row["matching_chunks"]) < 5]
    print(f"Delegates: {len(rows)}")
    print(f"No current corpus matches: {len(missing)}")
    if missing:
        print("  " + ", ".join(missing))
    print(f"Thin coverage (<5 chunks): {len(thin)}")
    if thin:
        print("  " + ", ".join(thin))
    print(f"Saved {REPORT_DIR / 'delegate_coverage.csv'}")
    print(f"Saved {REPORT_DIR / 'delegate_coverage.json'}")


if __name__ == "__main__":
    main()
