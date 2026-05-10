#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
import math
import sys
from collections import Counter, defaultdict
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from scripts.utils.pipeline import CHUNKS_DIR, DATA_DIR, STOP_WORDS, ensure_directory, load_json, save_json, tokenize


OUT_DIR = DATA_DIR / "comparative_analysis"


def token_counts(chunk: dict[str, object]) -> Counter[str]:
    text = f"{chunk['title']} {chunk['issue_tags']} {chunk['constitutional_clause_tags']} {chunk['text']}"
    return Counter(token for token in tokenize(text) if len(token) > 2 and token not in STOP_WORDS)


def cosine_sparse(a: dict[str, float], b: dict[str, float]) -> float:
    if len(a) > len(b):
        a, b = b, a
    dot = sum(weight * b.get(term, 0.0) for term, weight in a.items())
    return dot


def country_label(chunk: dict[str, object]) -> str:
    title = str(chunk["title"])
    if "'s Constitution" in title:
        return title.split("'s Constitution", 1)[0]
    if " Constitution" in title:
        return title.split(" Constitution", 1)[0]
    doc = str(chunk["document_id"]).removeprefix("world_constitution_")
    return doc.rsplit("_", 1)[0].replace("_", " ").title()


def build_vectors(chunks: list[dict[str, object]], max_features: int) -> tuple[dict[str, dict[str, float]], dict[str, float]]:
    counts_by_id = {chunk["chunk_id"]: token_counts(chunk) for chunk in chunks}
    df: Counter[str] = Counter()
    for counts in counts_by_id.values():
        df.update(counts.keys())
    vocab = {term for term, _ in df.most_common(max_features)}
    total = len(chunks)
    idf = {term: math.log((1 + total) / (1 + df[term])) + 1.0 for term in vocab}

    vectors: dict[str, dict[str, float]] = {}
    for chunk_id, counts in counts_by_id.items():
        weights: dict[str, float] = {}
        total_terms = sum(counts.values()) or 1
        for term, count in counts.items():
            if term not in vocab:
                continue
            weights[term] = (count / total_terms) * idf[term]
        norm = math.sqrt(sum(weight * weight for weight in weights.values())) or 1.0
        vectors[chunk_id] = {term: weight / norm for term, weight in weights.items()}
    return vectors, idf


def main() -> None:
    parser = argparse.ArgumentParser(description="Compare original U.S. Constitution chunks to current world constitutions.")
    parser.add_argument("--top", type=int, default=25, help="Top world matches to keep per U.S. chunk")
    parser.add_argument("--max-features", type=int, default=60000)
    args = parser.parse_args()

    corpus = load_json(CHUNKS_DIR / "constitution_full_corpus.json")
    chunks = corpus["chunks"]
    us_chunks = [
        chunk
        for chunk in chunks
        if chunk["source_collection"] == "constitution" and str(chunk["document_id"]).startswith("us_constitution_1787")
    ]
    world_chunks = [chunk for chunk in chunks if chunk["source_collection"] == "comparative_constitutions_world"]
    if not us_chunks:
        raise SystemExit("No original U.S. Constitution chunks found.")
    if not world_chunks:
        raise SystemExit("No world constitution chunks found.")

    vectors, _idf = build_vectors(us_chunks + world_chunks, args.max_features)
    by_chunk = {chunk["chunk_id"]: chunk for chunk in us_chunks + world_chunks}
    matches: list[dict[str, object]] = []
    country_scores: dict[tuple[str, str], list[float]] = defaultdict(list)

    for us in us_chunks:
        scored: list[tuple[float, dict[str, object]]] = []
        us_vector = vectors[us["chunk_id"]]
        for world in world_chunks:
            score = cosine_sparse(us_vector, vectors[world["chunk_id"]])
            if score <= 0:
                continue
            scored.append((score, world))
        scored.sort(key=lambda item: (-item[0], str(item[1]["title"])))
        for rank, (score, world) in enumerate(scored[: args.top], start=1):
            country = country_label(world)
            country_scores[(us["chunk_id"], country)].append(score)
            matches.append(
                {
                    "us_chunk_id": us["chunk_id"],
                    "us_title": us["title"],
                    "rank": rank,
                    "score": round(score, 8),
                    "world_chunk_id": world["chunk_id"],
                    "world_document_id": world["document_id"],
                    "country": country,
                    "world_title": world["title"],
                    "world_preview": world.get("preview", ""),
                    "world_source_url": world["source_url"],
                }
            )

    ensure_directory(OUT_DIR)
    with (OUT_DIR / "us_to_world_matches.csv").open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(matches[0].keys()))
        writer.writeheader()
        writer.writerows(matches)
    with (OUT_DIR / "us_to_world_matches.jsonl").open("w", encoding="utf-8") as handle:
        for row in matches:
            handle.write(json.dumps(row, ensure_ascii=False, sort_keys=True) + "\n")

    country_rows: list[dict[str, object]] = []
    for (us_chunk_id, country), scores in country_scores.items():
        us = by_chunk[us_chunk_id]
        country_rows.append(
            {
                "us_chunk_id": us_chunk_id,
                "us_title": us["title"],
                "country": country,
                "match_count": len(scores),
                "best_score": round(max(scores), 8),
                "mean_score": round(sum(scores) / len(scores), 8),
            }
        )
    country_rows.sort(key=lambda row: (str(row["us_title"]), -float(row["best_score"]), str(row["country"])))
    with (OUT_DIR / "us_to_country_summary.csv").open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(country_rows[0].keys()))
        writer.writeheader()
        writer.writerows(country_rows)

    save_json(
        OUT_DIR / "comparison_summary.json",
        {
            "metadata": {
                "us_chunks": len(us_chunks),
                "world_chunks": len(world_chunks),
                "top_matches_per_us_chunk": args.top,
                "model": "local_tfidf_cosine_v1",
            },
            "outputs": {
                "matches_csv": str((OUT_DIR / "us_to_world_matches.csv").relative_to(DATA_DIR.parent)),
                "matches_jsonl": str((OUT_DIR / "us_to_world_matches.jsonl").relative_to(DATA_DIR.parent)),
                "country_summary_csv": str((OUT_DIR / "us_to_country_summary.csv").relative_to(DATA_DIR.parent)),
            },
        },
    )
    print(f"Compared {len(us_chunks)} U.S. chunks to {len(world_chunks)} world chunks")
    print(f"Saved {OUT_DIR / 'us_to_world_matches.csv'}")
    print(f"Saved {OUT_DIR / 'us_to_country_summary.csv'}")


if __name__ == "__main__":
    main()
