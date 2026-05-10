#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import hashlib
import json
import math
import re
import sqlite3
from collections import Counter, defaultdict
from datetime import datetime
from pathlib import Path
from typing import Iterable

PROJECT_ROOT = Path(__file__).resolve().parent.parent
import sys

if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from scripts.utils.pipeline import CHUNKS_DIR, DATA_DIR, STOP_WORDS, ensure_directory, load_json, save_json, tokenize


GRAPH_DIR = DATA_DIR / "research_graph"
VECTOR_DIR = DATA_DIR / "research_vectors"
DEFAULT_MAX_FEATURES = 50000
DEFAULT_MAX_TERMS_PER_CHUNK = 192

PERSON_PATTERNS = {
    "George Washington": r"\bGeorge\s+Washington\b|\bWashington\b",
    "James Madison": r"\bJames\s+Madison\b|\bMadison\b",
    "Alexander Hamilton": r"\bAlexander\s+Hamilton\b|\bHamilton\b",
    "Thomas Jefferson": r"\bThomas\s+Jefferson\b|\bJefferson\b",
    "John Adams": r"\bJohn\s+Adams\b|\bAdams\b",
    "Benjamin Franklin": r"\bBenjamin\s+Franklin\b|\bFranklin\b",
    "John Jay": r"\bJohn\s+Jay\b|\bJay\b",
    "George Mason": r"\bGeorge\s+Mason\b|\bMason\b",
    "Patrick Henry": r"\bPatrick\s+Henry\b|\bHenry\b",
    "Gouverneur Morris": r"\bGouverneur\s+Morris\b",
    "James Wilson": r"\bJames\s+Wilson\b",
    "Roger Sherman": r"\bRoger\s+Sherman\b",
    "Edmund Randolph": r"\bEdmund\s+Randolph\b|\bRandolph\b",
}
FEDERALIST_PATTERN = re.compile(r"\bFederalist\s+(?:No\.?\s*)?(\d{1,2}|[IVXLCDM]+)\b", re.I)
ARTICLE_PATTERN = re.compile(r"\bArticle\s+([IVX]+|\d+)\b", re.I)
AMENDMENT_PATTERN = re.compile(r"\b(?:Amendment|Amend\.)\s+([IVX]+|\d+|First|Second|Third|Fourth|Fifth|Sixth|Seventh|Eighth|Ninth|Tenth)\b", re.I)


def stable_id(prefix: str, *parts: str) -> str:
    digest = hashlib.sha1("|".join(parts).encode("utf-8")).hexdigest()[:16]
    return f"{prefix}:{digest}"


def write_jsonl(path: Path, rows: Iterable[dict[str, object]]) -> int:
    ensure_directory(path.parent)
    count = 0
    with path.open("w", encoding="utf-8") as handle:
        for row in rows:
            handle.write(json.dumps(row, ensure_ascii=False, sort_keys=True))
            handle.write("\n")
            count += 1
    return count


def clean_token_counts(text: str) -> Counter[str]:
    return Counter(token for token in tokenize(text) if len(token) > 2 and token not in STOP_WORDS)


def build_vocabulary(chunks: list[dict[str, object]], max_features: int) -> tuple[dict[str, int], dict[str, float]]:
    document_frequency: Counter[str] = Counter()
    for chunk in chunks:
        terms = set(clean_token_counts(f"{chunk['title']} {chunk['text']}"))
        document_frequency.update(terms)

    ranked_terms = sorted(document_frequency.items(), key=lambda item: (-item[1], item[0]))[:max_features]
    vocabulary = {term: index for index, (term, _) in enumerate(ranked_terms)}
    total_docs = len(chunks)
    idf = {
        term: math.log((1 + total_docs) / (1 + document_frequency[term])) + 1.0
        for term in vocabulary
    }
    return vocabulary, idf


def vectorize_chunk(chunk: dict[str, object], vocabulary: dict[str, int], idf: dict[str, float], max_terms: int) -> list[dict[str, object]]:
    counts = clean_token_counts(f"{chunk['title']} {chunk['text']}")
    total = sum(counts.values()) or 1
    weights: list[tuple[str, float]] = []
    for term, count in counts.items():
        if term not in vocabulary:
            continue
        tf = count / total
        weights.append((term, tf * idf[term]))

    weights.sort(key=lambda item: (-item[1], item[0]))
    weights = weights[:max_terms]
    norm = math.sqrt(sum(weight * weight for _, weight in weights)) or 1.0
    return [
        {
            "term": term,
            "index": vocabulary[term],
            "weight": round(weight / norm, 8),
        }
        for term, weight in weights
    ]


def build_vector_outputs(chunks: list[dict[str, object]], max_features: int, max_terms: int) -> dict[str, object]:
    vocabulary, idf = build_vocabulary(chunks, max_features)

    vector_rows: list[dict[str, object]] = []
    sqlite_path = VECTOR_DIR / "chunk_vectors.sqlite"
    if sqlite_path.exists():
        sqlite_path.unlink()
    conn = sqlite3.connect(sqlite_path)
    conn.execute(
        """
        CREATE TABLE chunks (
            chunk_id TEXT PRIMARY KEY,
            document_id TEXT,
            title TEXT,
            author TEXT,
            date TEXT,
            source_collection TEXT,
            document_type TEXT,
            word_count INTEGER,
            text TEXT,
            vector_json TEXT
        )
        """
    )
    conn.execute("CREATE INDEX idx_chunks_document_id ON chunks(document_id)")
    conn.execute("CREATE INDEX idx_chunks_source_collection ON chunks(source_collection)")
    conn.execute("CREATE INDEX idx_chunks_author ON chunks(author)")

    for index, chunk in enumerate(chunks, start=1):
        vector = vectorize_chunk(chunk, vocabulary, idf, max_terms)
        row = {
            "chunk_id": chunk["chunk_id"],
            "document_id": chunk["document_id"],
            "source_collection": chunk["source_collection"],
            "vector_model": "local_tfidf_sparse_v1",
            "dimensions": len(vocabulary),
            "nonzero": len(vector),
            "vector": vector,
        }
        vector_rows.append(row)
        conn.execute(
            """
            INSERT INTO chunks VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            (
                chunk["chunk_id"],
                chunk["document_id"],
                chunk["title"],
                chunk["author"],
                chunk["date"],
                chunk["source_collection"],
                chunk["document_type"],
                int(chunk["word_count"]),
                chunk["text"],
                json.dumps(vector, ensure_ascii=False, sort_keys=True),
            ),
        )
        if index % 2500 == 0:
            conn.commit()
            print(f"vectorized {index}/{len(chunks)} chunks")

    conn.commit()
    conn.close()

    vocabulary_rows = [
        {
            "term": term,
            "index": index,
            "idf": round(idf[term], 8),
        }
        for term, index in sorted(vocabulary.items(), key=lambda item: item[1])
    ]
    vector_count = write_jsonl(VECTOR_DIR / "chunk_vectors.jsonl", vector_rows)
    vocab_count = write_jsonl(VECTOR_DIR / "vocabulary.jsonl", vocabulary_rows)

    return {
        "vector_rows": vector_count,
        "vocabulary_terms": vocab_count,
        "sqlite_path": str(sqlite_path.relative_to(DATA_DIR.parent)),
        "jsonl_path": str((VECTOR_DIR / "chunk_vectors.jsonl").relative_to(DATA_DIR.parent)),
        "vocabulary_path": str((VECTOR_DIR / "vocabulary.jsonl").relative_to(DATA_DIR.parent)),
        "model": "local_tfidf_sparse_v1",
        "max_terms_per_chunk": max_terms,
    }


def add_node(nodes: dict[str, dict[str, object]], node_id: str, label: str, node_type: str, **properties: object) -> None:
    nodes.setdefault(node_id, {"id": node_id, "label": label, "type": node_type, **properties})


def add_edge(edges: dict[tuple[str, str, str], dict[str, object]], source: str, target: str, edge_type: str, **properties: object) -> None:
    key = (source, target, edge_type)
    if key in edges:
        edges[key]["weight"] = int(edges[key].get("weight", 1)) + int(properties.pop("weight", 1))
        return
    edges[key] = {"source": source, "target": target, "type": edge_type, "weight": properties.pop("weight", 1), **properties}


def extract_reference_nodes(text: str) -> list[tuple[str, str, str]]:
    refs: list[tuple[str, str, str]] = []
    for person, pattern in PERSON_PATTERNS.items():
        if re.search(pattern, text, flags=re.I):
            refs.append((stable_id("person", person), person, "person"))
    for match in FEDERALIST_PATTERN.finditer(text):
        label = f"Federalist No. {match.group(1).upper()}"
        refs.append((stable_id("reference", label), label, "reference"))
    for match in ARTICLE_PATTERN.finditer(text):
        label = f"Article {match.group(1).upper()}"
        refs.append((stable_id("reference", label), label, "reference"))
    for match in AMENDMENT_PATTERN.finditer(text):
        label = f"Amendment {match.group(1).title()}"
        refs.append((stable_id("reference", label), label, "reference"))
    return sorted(set(refs))


def build_graph_outputs(chunks: list[dict[str, object]]) -> dict[str, object]:
    nodes: dict[str, dict[str, object]] = {}
    edges: dict[tuple[str, str, str], dict[str, object]] = {}

    for chunk in chunks:
        chunk_id = f"chunk:{chunk['chunk_id']}"
        document_id = f"document:{chunk['document_id']}"
        collection_id = f"collection:{chunk['source_collection']}"
        author_id = stable_id("author", str(chunk["author"]))

        add_node(
            nodes,
            chunk_id,
            str(chunk["title"]),
            "chunk",
            chunk_id_raw=chunk["chunk_id"],
            document_id=chunk["document_id"],
            source_collection=chunk["source_collection"],
            date=chunk["date"],
            word_count=chunk["word_count"],
        )
        add_node(nodes, document_id, str(chunk["title"]), "document", document_id_raw=chunk["document_id"])
        add_node(nodes, collection_id, str(chunk["source_collection"]), "collection")
        add_node(nodes, author_id, str(chunk["author"]), "author")

        add_edge(edges, chunk_id, document_id, "belongs_to_document")
        add_edge(edges, document_id, collection_id, "belongs_to_collection")
        add_edge(edges, document_id, author_id, "authored_by")

        for issue in chunk.get("issue_tags", []):
            issue_id = stable_id("issue", str(issue))
            add_node(nodes, issue_id, str(issue), "issue")
            add_edge(edges, chunk_id, issue_id, "has_issue")

        for clause in chunk.get("constitutional_clause_tags", []):
            clause_id = stable_id("clause", str(clause))
            add_node(nodes, clause_id, str(clause), "clause")
            add_edge(edges, chunk_id, clause_id, "mentions_clause")

        for ref_id, label, ref_type in extract_reference_nodes(str(chunk["text"])):
            add_node(nodes, ref_id, label, ref_type)
            add_edge(edges, chunk_id, ref_id, "mentions")

    node_count = write_jsonl(GRAPH_DIR / "nodes.jsonl", nodes.values())
    edge_count = write_jsonl(GRAPH_DIR / "edges.jsonl", edges.values())

    with (GRAPH_DIR / "edges.csv").open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=["source", "target", "type", "weight"])
        writer.writeheader()
        for edge in edges.values():
            writer.writerow({key: edge.get(key, "") for key in ("source", "target", "type", "weight")})

    return {
        "nodes": node_count,
        "edges": edge_count,
        "nodes_path": str((GRAPH_DIR / "nodes.jsonl").relative_to(DATA_DIR.parent)),
        "edges_path": str((GRAPH_DIR / "edges.jsonl").relative_to(DATA_DIR.parent)),
        "edges_csv_path": str((GRAPH_DIR / "edges.csv").relative_to(DATA_DIR.parent)),
    }


def main() -> None:
    parser = argparse.ArgumentParser(description="Build graph and local sparse-vector research artifacts from the chunk corpus.")
    parser.add_argument("--max-features", type=int, default=DEFAULT_MAX_FEATURES)
    parser.add_argument("--max-terms-per-chunk", type=int, default=DEFAULT_MAX_TERMS_PER_CHUNK)
    args = parser.parse_args()

    ensure_directory(GRAPH_DIR)
    ensure_directory(VECTOR_DIR)
    corpus = load_json(CHUNKS_DIR / "constitution_full_corpus.json")
    chunks = corpus["chunks"]
    started_at = datetime.now().isoformat()

    print(f"building graph artifacts for {len(chunks)} chunks")
    graph_summary = build_graph_outputs(chunks)
    print(f"graph complete: {graph_summary['nodes']} nodes, {graph_summary['edges']} edges")

    print(f"building sparse vectors for {len(chunks)} chunks")
    vector_summary = build_vector_outputs(chunks, args.max_features, args.max_terms_per_chunk)
    print(f"vectors complete: {vector_summary['vector_rows']} rows, {vector_summary['vocabulary_terms']} terms")

    summary = {
        "generated_at": datetime.now().isoformat(),
        "started_at": started_at,
        "corpus_path": str((CHUNKS_DIR / "constitution_full_corpus.json").relative_to(DATA_DIR.parent)),
        "total_chunks": len(chunks),
        "graph": graph_summary,
        "vectors": vector_summary,
        "notes": [
            "Vectors are local sparse TF-IDF vectors, not neural embeddings.",
            "Graph edges are deterministic metadata/tag/reference relationships extracted from the corpus.",
        ],
    }
    save_json(DATA_DIR / "research_ingestion_summary.json", summary)
    print("saved data/research_ingestion_summary.json")


if __name__ == "__main__":
    main()
