#!/usr/bin/env python3
from __future__ import annotations

from collections import Counter, defaultdict
from datetime import datetime
from pathlib import Path
import sys

PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from scripts.utils.pipeline import CHUNKS_DIR, INDEX_DIR, STOP_WORDS, ensure_directory, load_json, save_json, tokenize, unique_preserve


def build_index() -> dict[str, object]:
    corpus = load_json(CHUNKS_DIR / "constitution_full_corpus.json")
    postings: dict[str, list[str]] = defaultdict(list)
    issue_postings: dict[str, list[str]] = defaultdict(list)
    clause_postings: dict[str, list[str]] = defaultdict(list)
    document_map: dict[str, dict[str, str]] = {}

    authors: list[str] = []
    source_collections: list[str] = []
    document_types: list[str] = []
    issues: list[str] = []

    for chunk in corpus["chunks"]:
        chunk_id = chunk["chunk_id"]
        tokens = [token for token in tokenize(f"{chunk['title']} {chunk['text']}") if len(token) > 2 and token not in STOP_WORDS]
        frequencies = Counter(tokens)
        for token in frequencies:
            postings[token].append(chunk_id)

        for issue in chunk["issue_tags"]:
            issue_postings[issue].append(chunk_id)
            issues.append(issue)

        for clause in chunk["constitutional_clause_tags"]:
            clause_postings[clause].append(chunk_id)

        document_map[chunk["document_id"]] = {
            "document_id": chunk["document_id"],
            "title": chunk["title"],
            "author": chunk["author"],
            "source_collection": chunk["source_collection"],
        }

        authors.append(chunk["author"])
        source_collections.append(chunk["source_collection"])
        document_types.append(chunk["document_type"])

    return {
        "metadata": {
            "version": "2.0",
            "generated_at": datetime.now().isoformat(),
            "total_chunks": len(corpus["chunks"]),
            "total_terms": len(postings),
        },
        "inverted_index": {token: sorted(unique_preserve(ids)) for token, ids in postings.items()},
        "issue_index": {issue: sorted(unique_preserve(ids)) for issue, ids in issue_postings.items()},
        "clause_index": {clause: sorted(unique_preserve(ids)) for clause, ids in clause_postings.items()},
        "filters": {
            "source_collections": sorted(unique_preserve(source_collections)),
            "authors": sorted(unique_preserve(authors)),
            "document_types": sorted(unique_preserve(document_types)),
            "issues": sorted(unique_preserve(issues)),
            "documents": sorted(document_map.values(), key=lambda item: (item["source_collection"], item["title"])),
        },
    }


def main() -> None:
    ensure_directory(INDEX_DIR)
    save_json(INDEX_DIR / "search_index.json", build_index())
    print(f"Saved {INDEX_DIR / 'search_index.json'}")


if __name__ == "__main__":
    main()
