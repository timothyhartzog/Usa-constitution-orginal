#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
import math
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from scripts.utils.pipeline import CHUNKS_DIR, CLEAN_DIR, DATA_DIR, STOP_WORDS, ensure_directory, load_json, save_json, tokenize


REPORT_DIR = DATA_DIR / "bill_of_rights" / "reports"

BILL_OF_RIGHTS_DOCUMENTS = [
    "bill_of_rights_original_text_1791",
    "madison_speech_bill_of_rights_1789_06_08",
    "jefferson_madison_correspondence_bill_of_rights",
    "congressional_debates_bill_of_rights_1789",
    "anti_federalist_positions_bill_of_rights",
    "bill_of_rights_key_figures",
    "bill_of_rights_index",
]

AMENDMENTS = [
    {
        "number": 1,
        "label": "First Amendment",
        "topics": ["religion", "speech", "press", "assembly", "petition"],
        "terms": [
            "establishment of religion",
            "free exercise",
            "freedom of speech",
            "freedom of the press",
            "assemble",
            "petition",
            "redress of grievances",
        ],
    },
    {
        "number": 2,
        "label": "Second Amendment",
        "topics": ["militia", "arms"],
        "terms": ["militia", "keep and bear arms", "bear arms", "arms", "security of a free state"],
    },
    {
        "number": 3,
        "label": "Third Amendment",
        "topics": ["quartering soldiers"],
        "terms": ["quarter", "quartered", "soldier", "soldiers", "consent of the owner"],
    },
    {
        "number": 4,
        "label": "Fourth Amendment",
        "topics": ["search", "seizure", "warrants"],
        "terms": ["searches", "seizures", "warrants", "probable cause", "oath or affirmation", "persons houses papers effects"],
    },
    {
        "number": 5,
        "label": "Fifth Amendment",
        "topics": ["grand jury", "double jeopardy", "self-incrimination", "due process", "takings"],
        "terms": [
            "grand jury",
            "double jeopardy",
            "witness against himself",
            "due process",
            "life liberty or property",
            "just compensation",
            "private property",
        ],
    },
    {
        "number": 6,
        "label": "Sixth Amendment",
        "topics": ["criminal trial", "jury", "confrontation", "counsel"],
        "terms": [
            "speedy and public trial",
            "impartial jury",
            "nature and cause",
            "confronted with the witnesses",
            "compulsory process",
            "assistance of counsel",
        ],
    },
    {
        "number": 7,
        "label": "Seventh Amendment",
        "topics": ["civil jury"],
        "terms": ["suits at common law", "trial by jury", "twenty dollars", "re-examined", "common law"],
    },
    {
        "number": 8,
        "label": "Eighth Amendment",
        "topics": ["bail", "fines", "punishment"],
        "terms": ["excessive bail", "excessive fines", "cruel and unusual", "punishments"],
    },
    {
        "number": 9,
        "label": "Ninth Amendment",
        "topics": ["unenumerated rights"],
        "terms": ["enumeration", "deny or disparage", "others retained by the people", "retained by the people"],
    },
    {
        "number": 10,
        "label": "Tenth Amendment",
        "topics": ["reserved powers", "states", "people"],
        "terms": ["powers not delegated", "reserved to the states", "reserved", "states respectively", "or to the people"],
    },
]


def clean_text(text: str) -> str:
    return re.sub(r"\s+", " ", text).strip()


def source_inventory(manifest: dict[str, object]) -> list[dict[str, object]]:
    documents = {
        doc["document_id"]: (collection, doc)
        for collection in manifest["collections"]
        for doc in collection.get("documents", [])
        if doc["document_id"] in BILL_OF_RIGHTS_DOCUMENTS
    }
    rows: list[dict[str, object]] = []
    for document_id in BILL_OF_RIGHTS_DOCUMENTS:
        path = CLEAN_DIR / f"{document_id}.txt"
        text = path.read_text(encoding="utf-8") if path.exists() else ""
        collection, doc = documents.get(document_id, ({}, {}))
        rows.append(
            {
                "document_id": document_id,
                "title": doc.get("title", document_id),
                "collection_id": collection.get("collection_id", ""),
                "document_type": doc.get("document_type", ""),
                "author": doc.get("author", ""),
                "date": doc.get("date", ""),
                "source_url": doc.get("source_url", ""),
                "clean_path": str(path.relative_to(PROJECT_ROOT)) if path.exists() else "",
                "exists": path.exists(),
                "bytes": path.stat().st_size if path.exists() else 0,
                "words": len(text.split()),
            }
        )
    return rows


def amendment_texts() -> dict[int, str]:
    path = CLEAN_DIR / "bill_of_rights_original_text_1791.txt"
    text = path.read_text(encoding="utf-8")
    parts = re.split(r"(?=AMENDMENT\s+[IVX]+)", text)
    roman_to_number = {"I": 1, "II": 2, "III": 3, "IV": 4, "V": 5, "VI": 6, "VII": 7, "VIII": 8, "IX": 9, "X": 10}
    amendments: dict[int, str] = {}
    for part in parts:
        match = re.match(r"AMENDMENT\s+([IVX]+)\s+(.*)", clean_text(part), flags=re.S)
        if not match:
            continue
        number = roman_to_number.get(match.group(1))
        if number:
            body = match.group(2).split("---", 1)[0].strip()
            amendments[number] = body
    return amendments


def term_score(text: str, terms: list[str]) -> int:
    lowered = text.lower()
    score = 0
    for term in terms:
        pattern = re.escape(term.lower()).replace(r"\ ", r"\s+")
        score += len(re.findall(pattern, lowered))
    return score


def weighted_counts(chunk: dict[str, object]) -> Counter[str]:
    text = f"{chunk['title']} {chunk.get('issue_tags', [])} {chunk.get('constitutional_clause_tags', [])} {chunk['text']}"
    return Counter(token for token in tokenize(text) if len(token) > 2 and token not in STOP_WORDS)


def build_vectors(chunks: list[dict[str, object]], amendment_text_by_number: dict[int, str], max_features: int) -> tuple[dict[str, dict[str, float]], dict[str, dict[str, float]]]:
    chunk_counts = {str(chunk["chunk_id"]): weighted_counts(chunk) for chunk in chunks}
    amendment_counts = {
        str(number): Counter(token for token in tokenize(text) if len(token) > 2 and token not in STOP_WORDS)
        for number, text in amendment_text_by_number.items()
    }
    df: Counter[str] = Counter()
    for counts in list(chunk_counts.values()) + list(amendment_counts.values()):
        df.update(counts.keys())
    vocab = {term for term, _count in df.most_common(max_features)}
    total_docs = len(chunk_counts) + len(amendment_counts)
    idf = {term: math.log((1 + total_docs) / (1 + df[term])) + 1.0 for term in vocab}

    def vectorize(counts: Counter[str]) -> dict[str, float]:
        total = sum(counts.values()) or 1
        weights = {term: (count / total) * idf[term] for term, count in counts.items() if term in vocab}
        norm = math.sqrt(sum(weight * weight for weight in weights.values())) or 1.0
        return {term: weight / norm for term, weight in weights.items()}

    return (
        {chunk_id: vectorize(counts) for chunk_id, counts in chunk_counts.items()},
        {number: vectorize(counts) for number, counts in amendment_counts.items()},
    )


def cosine_sparse(a: dict[str, float], b: dict[str, float]) -> float:
    if len(a) > len(b):
        a, b = b, a
    return sum(weight * b.get(term, 0.0) for term, weight in a.items())


def country_label(chunk: dict[str, object]) -> str:
    title = str(chunk["title"])
    if "'s Constitution" in title:
        return title.split("'s Constitution", 1)[0]
    doc = str(chunk["document_id"]).removeprefix("world_constitution_")
    return doc.rsplit("_", 1)[0].replace("_", " ").title()


def write_csv(path: Path, rows: list[dict[str, object]]) -> None:
    if not rows:
        return
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def main() -> None:
    parser = argparse.ArgumentParser(description="Build Bill of Rights research reports from the local corpus.")
    parser.add_argument("--top-source-chunks", type=int, default=30)
    parser.add_argument("--top-world-matches", type=int, default=25)
    parser.add_argument("--max-features", type=int, default=50000)
    args = parser.parse_args()

    ensure_directory(REPORT_DIR)
    manifest = load_json(PROJECT_ROOT / "config" / "sources_manifest.json")
    corpus = load_json(CHUNKS_DIR / "constitution_full_corpus.json")
    chunks = corpus["chunks"]
    amendment_text_by_number = amendment_texts()

    inventory_rows = source_inventory(manifest)
    write_csv(REPORT_DIR / "source_inventory.csv", inventory_rows)

    bor_chunks = [chunk for chunk in chunks if chunk["source_collection"] == "bill_of_rights"]
    founding_chunks = [
        chunk
        for chunk in chunks
        if not str(chunk["source_collection"]).startswith("comparative_constitutions_")
    ]
    world_chunks = [chunk for chunk in chunks if chunk["source_collection"] == "comparative_constitutions_world"]

    amendment_dossiers: list[dict[str, object]] = []
    source_match_rows: list[dict[str, object]] = []
    for amendment in AMENDMENTS:
        scored: list[tuple[int, dict[str, object]]] = []
        for chunk in founding_chunks:
            score = term_score(f"{chunk['title']} {chunk['text']}", amendment["terms"])
            if score:
                scored.append((score, chunk))
        scored.sort(key=lambda item: (-item[0], str(item[1]["source_collection"]), str(item[1]["title"])))
        top_chunks = []
        for rank, (score, chunk) in enumerate(scored[: args.top_source_chunks], start=1):
            row = {
                "amendment": amendment["number"],
                "amendment_label": amendment["label"],
                "rank": rank,
                "score": score,
                "chunk_id": chunk["chunk_id"],
                "document_id": chunk["document_id"],
                "title": chunk["title"],
                "author": chunk["author"],
                "date": chunk["date"],
                "source_collection": chunk["source_collection"],
                "source_url": chunk["source_url"],
                "preview": chunk.get("preview", ""),
            }
            source_match_rows.append(row)
            top_chunks.append(row)
        amendment_dossiers.append(
            {
                "amendment": amendment,
                "text": amendment_text_by_number.get(int(amendment["number"]), ""),
                "founding_source_match_count": len(scored),
                "top_founding_source_chunks": top_chunks,
            }
        )

    write_csv(REPORT_DIR / "amendment_source_matches.csv", source_match_rows)
    save_json(REPORT_DIR / "amendment_dossiers.json", {"amendments": amendment_dossiers})

    world_match_rows: list[dict[str, object]] = []
    if world_chunks:
        chunk_vectors, amendment_vectors = build_vectors(world_chunks, amendment_text_by_number, args.max_features)
        for amendment in AMENDMENTS:
            amendment_number = str(amendment["number"])
            scored_world = [
                (cosine_sparse(amendment_vectors[amendment_number], chunk_vectors[str(chunk["chunk_id"])]), chunk)
                for chunk in world_chunks
            ]
            scored_world = [(score, chunk) for score, chunk in scored_world if score > 0]
            scored_world.sort(key=lambda item: (-item[0], country_label(item[1]), str(item[1]["title"])))
            seen_country_counts: defaultdict[str, int] = defaultdict(int)
            rank = 0
            for score, chunk in scored_world:
                country = country_label(chunk)
                if seen_country_counts[country] >= 3:
                    continue
                seen_country_counts[country] += 1
                rank += 1
                world_match_rows.append(
                    {
                        "amendment": amendment["number"],
                        "amendment_label": amendment["label"],
                        "rank": rank,
                        "score": round(score, 8),
                        "country": country,
                        "world_chunk_id": chunk["chunk_id"],
                        "world_document_id": chunk["document_id"],
                        "world_title": chunk["title"],
                        "source_url": chunk["source_url"],
                        "preview": chunk.get("preview", ""),
                    }
                )
                if rank >= args.top_world_matches:
                    break
    write_csv(REPORT_DIR / "amendment_world_matches.csv", world_match_rows)

    save_json(
        REPORT_DIR / "summary.json",
        {
            "metadata": {
                "model": "bill_of_rights_local_keyword_and_tfidf_v1",
                "bill_of_rights_documents": len(BILL_OF_RIGHTS_DOCUMENTS),
                "bill_of_rights_chunks_in_corpus": len(bor_chunks),
                "founding_chunks_scanned": len(founding_chunks),
                "world_chunks_scanned": len(world_chunks),
                "amendments": len(AMENDMENTS),
            },
            "outputs": {
                "source_inventory_csv": str((REPORT_DIR / "source_inventory.csv").relative_to(PROJECT_ROOT)),
                "amendment_dossiers_json": str((REPORT_DIR / "amendment_dossiers.json").relative_to(PROJECT_ROOT)),
                "amendment_source_matches_csv": str((REPORT_DIR / "amendment_source_matches.csv").relative_to(PROJECT_ROOT)),
                "amendment_world_matches_csv": str((REPORT_DIR / "amendment_world_matches.csv").relative_to(PROJECT_ROOT)),
            },
        },
    )
    print(f"Bill of Rights chunks in corpus: {len(bor_chunks)}")
    print(f"Saved reports in {REPORT_DIR}")


if __name__ == "__main__":
    main()
