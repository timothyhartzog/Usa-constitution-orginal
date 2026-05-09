#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from datetime import datetime
from pathlib import Path

# `sys` is used both for the path patch below and for the missing-raw warnings
# emitted in main() — keep imported even if some linters flag the latter as
# unused on inspection.

from bs4 import BeautifulSoup

PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from scripts.utils.pipeline import CLEAN_DIR, RAW_DIR, ensure_directory, iter_documents, load_manifest, normalize_whitespace, save_json


FOOTNOTE_PATTERNS = [
    r"\[\s*FN\d+\s*\]",
    r"\(\d+\)",
    r"\[Back\]",
]


def strip_gutenberg_boilerplate(text: str) -> str:
    start_match = re.search(r"\*\*\*\s*START OF THE PROJECT GUTENBERG EBOOK.*?\*\*\*", text, flags=re.I | re.S)
    if start_match:
        text = text[start_match.end():]

    end_match = re.search(r"\*\*\*\s*END OF THE PROJECT GUTENBERG EBOOK.*?\*\*\*", text, flags=re.I | re.S)
    if end_match:
        text = text[:end_match.start()]
    return text


def strip_archive_google_preface(text: str) -> str:
    marker = re.search(r"THE\s+RECORDS\s+OF\s+THE\s+FEDERAL\s+CONVENTION\s+OF\s+1787", text, flags=re.I)
    if marker:
        return text[marker.start():]
    return text


def extract_html_text(payload: str) -> str:
    soup = BeautifulSoup(payload, "html.parser")
    for tag in soup(["script", "style", "noscript", "header", "footer"]):
        tag.decompose()
    body = soup.body or soup
    return body.get_text("\n")


def apply_document_cleaning(text: str, document: dict[str, object]) -> str:
    cleaned = strip_gutenberg_boilerplate(text)
    cleaned = strip_archive_google_preface(cleaned)

    cleaning = document.get("cleaning", {})
    start_marker = cleaning.get("start_marker")
    end_marker = cleaning.get("end_marker")

    if start_marker:
        match = re.search(re.escape(start_marker), cleaned, flags=re.I)
        if match:
            cleaned = cleaned[match.start():]

    if end_marker:
        match = re.search(re.escape(end_marker), cleaned, flags=re.I)
        if match:
            cleaned = cleaned[:match.start()]

    for pattern in FOOTNOTE_PATTERNS:
        cleaned = re.sub(pattern, "", cleaned, flags=re.I)

    cleaned = re.sub(r"\[[^\]]*Source:[^\]]*\]", "", cleaned, flags=re.I)
    cleaned = re.sub(r"Source:\s+Documents Illustrative.*", "", cleaned, flags=re.I | re.S)
    cleaned = re.sub(r"\n\s*\d+\s*\n", "\n", cleaned)
    cleaned = re.sub(r"[ \t]*\n[ \t]*", "\n", cleaned)

    return normalize_whitespace(cleaned)


def main() -> None:
    manifest = load_manifest()
    ensure_directory(CLEAN_DIR)
    report: dict[str, object] = {
        "generated_at": datetime.now().isoformat(),
        "documents": [],
    }

    for collection, document in iter_documents(manifest):
        extension = "html" if document["source_format"] == "html" else "txt"
        raw_path = RAW_DIR / collection["collection_id"] / document["document_id"] / f"source.{extension}"
        if not raw_path.exists():
            print(
                f"{collection['collection_id']}/{document['document_id']}: "
                f"SKIPPED — raw source missing ({raw_path}). "
                f"Run scripts/ingest_sources.py with network access first.",
                file=sys.stderr,
            )
            report["documents"].append(
                {
                    "document_id": document["document_id"],
                    "source_collection": collection["collection_id"],
                    "status": "skipped_missing_raw",
                }
            )
            continue

        payload = raw_path.read_text(encoding="utf-8")
        text = extract_html_text(payload) if document["source_format"] == "html" else payload
        cleaned = apply_document_cleaning(text, document)

        clean_path = CLEAN_DIR / f"{document['document_id']}.txt"
        clean_path.write_text(cleaned, encoding="utf-8")

        report["documents"].append(
            {
                "document_id": document["document_id"],
                "source_collection": collection["collection_id"],
                "clean_path": str(clean_path.relative_to(CLEAN_DIR.parent)),
                "word_count": len(cleaned.split()),
            }
        )
        print(f"{collection['collection_id']}/{document['document_id']}: cleaned")

    save_json(CLEAN_DIR.parent / "clean_report.json", report)


if __name__ == "__main__":
    main()
