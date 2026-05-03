#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from datetime import datetime
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from scripts.utils.metadata_tagging import MetadataTagger
from scripts.utils.pipeline import CHUNKS_DIR, CLEAN_DIR, ensure_directory, iter_documents, load_manifest, preview_text, sanitize_identifier, save_json, unique_preserve


DEFAULT_TARGET_WORDS = 320
MIN_WORDS = 80
MAX_WORDS = 650


def word_count(text: str) -> int:
    return len(text.split())


def split_constitution_sections(document: dict[str, object], text: str) -> list[dict[str, str]]:
    normalized = text.replace("ARTICLE.", "ARTICLE ").replace("Article.", "Article ")
    parts = re.split(r"(?=(?:ARTICLE|Article)\s+(?:[IVX]+|\d+))", normalized)
    records: list[dict[str, str]] = []

    preamble = parts[0].strip()
    if preamble:
        records.append(
            {
                "document_id": f"{document['document_id']}_preamble",
                "title": f"{document['title']} - Preamble",
                "date": str(document["date"]),
                "author": str(document["author"]),
                "document_type": str(document["document_type"]),
                "source_collection": "constitution",
                "source_url": str(document["source_url"]),
                "text": preamble,
                "seed_clause_tags": ["preamble"],
            }
        )

    article_map = {
        "I": [f"I.{index}" for index in range(1, 11)],
        "1": [f"I.{index}" for index in range(1, 11)],
        "II": [f"II.{index}" for index in range(1, 5)],
        "2": [f"II.{index}" for index in range(1, 5)],
        "III": [f"III.{index}" for index in range(1, 4)],
        "3": [f"III.{index}" for index in range(1, 4)],
        "IV": [f"IV.{index}" for index in range(1, 5)],
        "4": [f"IV.{index}" for index in range(1, 5)],
        "V": ["V"],
        "5": ["V"],
        "VI": ["VI.1", "VI.2"],
        "6": ["VI.1", "VI.2"],
        "VII": ["VII"],
        "7": ["VII"],
    }

    for section in parts[1:]:
        lines = [line.strip() for line in section.splitlines() if line.strip()]
        if not lines:
            continue
        header = lines[0]
        numeral_match = re.search(r"((?:I{1,3}|IV|V|VI|VII|VIII|IX|X)|\d+)", header)
        numeral = numeral_match.group(1) if numeral_match else "section"
        records.append(
            {
                "document_id": f"{document['document_id']}_article_{sanitize_identifier(numeral)}",
                "title": f"{document['title']} - Article {numeral}",
                "date": str(document["date"]),
                "author": str(document["author"]),
                "document_type": str(document["document_type"]),
                "source_collection": "constitution",
                "source_url": str(document["source_url"]),
                "text": "\n".join(lines),
                "seed_clause_tags": article_map.get(numeral, []),
            }
        )
    return records


def split_federalist_essays(document: dict[str, object], text: str) -> list[dict[str, str]]:
    body_match = re.search(r"FEDERALIST No\. I\.", text)
    body = text[body_match.start():] if body_match else text
    sections = re.split(r"(?=FEDERALIST No\.\s+[IVXLCDM0-9]+\.?)", body)
    records: list[dict[str, str]] = []

    author_map = {"HAMILTON": "Alexander Hamilton", "MADISON": "James Madison", "JAY": "John Jay"}
    for section in sections:
        cleaned = section.strip()
        if not cleaned:
            continue
        lines = [line.strip() for line in cleaned.splitlines() if line.strip()]
        if not lines:
            continue
        title = lines[0].title().replace("No.", "No.")
        author = str(document["author"])
        for candidate in lines[:8]:
            if candidate.upper() in author_map:
                author = author_map[candidate.upper()]
                break
        number_match = re.search(r"No\.\s+([IVXLCDM0-9]+)", lines[0], flags=re.I)
        essay_id = sanitize_identifier(number_match.group(1) if number_match else title)
        records.append(
            {
                "document_id": f"federalist_{essay_id}",
                "title": title,
                "date": str(document["date"]),
                "author": author,
                "document_type": str(document["document_type"]),
                "source_collection": str(document["source_collection"]),
                "source_url": str(document["source_url"]),
                "text": cleaned,
                "seed_clause_tags": [],
            }
        )
    return records


def split_jefferson_letters(document: dict[str, object], text: str) -> list[dict[str, str]]:
    sections = re.split(r"(?=\nTO\s+[A-Z][A-Z .,&'\-]+?\.\n)", f"\n{text}")
    records: list[dict[str, str]] = []
    for section in sections:
        cleaned = section.strip()
        if not cleaned.startswith("TO "):
            continue
        lines = [line.strip() for line in cleaned.splitlines() if line.strip()]
        if len(lines) < 2:
            continue
        year_match = re.search(r"\b(178[6-9])\b", cleaned)
        if not year_match:
            continue
        recipient = lines[0][3:].rstrip(".").title()
        date_line = next((line for line in lines[1:6] if re.search(r"\b178[6-9]\b", line)), str(document["date"]))
        month_match = re.search(r"([A-Z][a-z]+ \d{1,2}, 178[6-9])", date_line)
        date_value = month_match.group(1) if month_match else date_line
        records.append(
            {
                "document_id": f"jefferson_to_{sanitize_identifier(recipient)}_{year_match.group(1)}_{len(records)+1:03d}",
                "title": f"Thomas Jefferson to {recipient}",
                "date": date_value,
                "author": "Thomas Jefferson",
                "document_type": str(document["document_type"]),
                "source_collection": str(document["source_collection"]),
                "source_url": str(document["source_url"]),
                "text": cleaned,
                "seed_clause_tags": [],
            }
        )
    if records:
        return records
    return [
        {
            "document_id": str(document["document_id"]),
            "title": str(document["title"]),
            "date": str(document["date"]),
            "author": str(document["author"]),
            "document_type": str(document["document_type"]),
            "source_collection": str(document["source_collection"]),
            "source_url": str(document["source_url"]),
            "text": text,
            "seed_clause_tags": [],
        }
    ]


def build_records(collection: dict[str, object], document: dict[str, object], text: str) -> list[dict[str, str]]:
    document = {
        **document,
        "source_collection": collection["collection_id"],
    }
    strategy = document["chunk_strategy"]
    if strategy == "constitution_sections":
        return split_constitution_sections(document, text)
    if strategy == "federalist_essays":
        return split_federalist_essays(document, text)
    if strategy == "jefferson_letters":
        return split_jefferson_letters(document, text)
    return [
        {
            "document_id": str(document["document_id"]),
            "title": str(document["title"]),
            "date": str(document["date"]),
            "author": str(document["author"]),
            "document_type": str(document["document_type"]),
            "source_collection": str(collection["collection_id"]),
            "source_url": str(document["source_url"]),
            "text": text,
            "seed_clause_tags": [],
        }
    ]


def paragraph_chunk(text: str, target_words: int = DEFAULT_TARGET_WORDS) -> list[str]:
    paragraphs = [paragraph.strip() for paragraph in re.split(r"\n\s*\n", text) if paragraph.strip()]
    if not paragraphs:
        return []

    chunks: list[str] = []
    current: list[str] = []
    current_words = 0

    for paragraph in paragraphs:
        paragraph_words = word_count(paragraph)
        if paragraph_words > MAX_WORDS:
            if current:
                chunks.append("\n\n".join(current))
                current = []
                current_words = 0
            words = paragraph.split()
            start = 0
            while start < len(words):
                chunk_words = words[start:start + target_words]
                chunks.append(" ".join(chunk_words))
                start += target_words - 45
            continue

        if current and current_words + paragraph_words > target_words + 120:
            chunks.append("\n\n".join(current))
            current = [paragraph]
            current_words = paragraph_words
            continue

        current.append(paragraph)
        current_words += paragraph_words

    if current:
        chunks.append("\n\n".join(current))

    normalized: list[str] = []
    for chunk in chunks:
        cleaned = chunk.strip()
        if not cleaned:
            continue
        if normalized and word_count(cleaned) < MIN_WORDS:
            normalized[-1] = f"{normalized[-1]}\n\n{cleaned}"
        else:
            normalized.append(cleaned)
    return normalized


def main() -> None:
    manifest = load_manifest()
    ensure_directory(CHUNKS_DIR)
    tagger = MetadataTagger()
    all_chunks: list[dict[str, object]] = []
    total_documents = 0

    for collection, document in iter_documents(manifest):
        clean_path = CLEAN_DIR / f"{document['document_id']}.txt"
        if not clean_path.exists():
            raise FileNotFoundError(f"Missing cleaned source: {clean_path}")

        records = build_records(collection, document, clean_path.read_text(encoding="utf-8"))
        total_documents += len(records)

        for record in records:
            base_issue_tags = list(document.get("default_issue_tags", []))
            seed_clause_tags = list(record.get("seed_clause_tags", []))

            for index, chunk_text in enumerate(paragraph_chunk(record["text"])):
                if word_count(chunk_text) < MIN_WORDS:
                    continue
                issue_tags, clause_tags = tagger.tag_text(
                    chunk_text,
                    seed_issue_tags=base_issue_tags,
                    seed_clause_tags=seed_clause_tags,
                )
                all_chunks.append(
                    {
                        "chunk_id": f"{record['document_id']}_{index:04d}",
                        "document_id": record["document_id"],
                        "title": record["title"],
                        "author": record["author"],
                        "date": record["date"],
                        "source_collection": record["source_collection"],
                        "source_url": record["source_url"],
                        "document_type": record["document_type"],
                        "issue_tags": unique_preserve(issue_tags),
                        "constitutional_clause_tags": unique_preserve(clause_tags),
                        "text": chunk_text,
                        "word_count": word_count(chunk_text),
                        "preview": preview_text(chunk_text),
                    }
                )

        print(f"{collection['collection_id']}/{document['document_id']}: {len(records)} records")

    corpus = {
        "metadata": {
            "version": "2.0",
            "generated_at": datetime.now().isoformat(),
            "total_chunks": len(all_chunks),
            "total_documents": total_documents,
        },
        "chunks": all_chunks,
    }
    save_json(CHUNKS_DIR / "constitution_full_corpus.json", corpus)


if __name__ == "__main__":
    main()
