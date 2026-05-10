#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
import time
from datetime import datetime
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.parse import urlencode

PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from scripts.fetch_eu_constitutions import API_BASE, clean_constitution_html, constitution_html_url, request_json
from scripts.utils.pipeline import CLEAN_DIR, MANIFEST_PATH, RAW_DIR, ensure_directory, load_manifest, save_json, sanitize_identifier


COLLECTION_ID = "comparative_constitutions_world"


def current_public_constitutions(records: list[dict[str, object]]) -> list[dict[str, object]]:
    selected: list[dict[str, object]] = []
    for record in records:
        if not record.get("public", False):
            continue
        if not record.get("in_force", False):
            continue
        if record.get("is_draft", False) or record.get("is_historic", False):
            continue
        selected.append(record)
    return sorted(selected, key=lambda item: (str(item.get("region", "")), str(item.get("country_id", "")), str(item.get("id", ""))))


def update_manifest(documents: list[dict[str, object]]) -> None:
    manifest = load_manifest()
    collections = manifest.setdefault("collections", [])
    collection = {
        "collection_id": COLLECTION_ID,
        "title": "World Constitutions from Constitute Project",
        "documents": documents,
    }

    for index, existing in enumerate(collections):
        if existing.get("collection_id") == COLLECTION_ID:
            collections[index] = collection
            break
    else:
        collections.append(collection)

    metadata = manifest.setdefault("metadata", {})
    metadata["last_updated"] = datetime.now().date().isoformat()
    save_json(MANIFEST_PATH, manifest)


def main() -> None:
    parser = argparse.ArgumentParser(description="Download all public, in-force constitutions from the Constitute API.")
    parser.add_argument("--lang", default="en", choices=("en", "es", "ar"), help="Constitute language to fetch")
    parser.add_argument("--delay", type=float, default=0.25, help="Delay between constitution HTML requests")
    parser.add_argument("--timeout", type=int, default=45, help="HTTP timeout in seconds")
    parser.add_argument("--force", action="store_true", help="Re-fetch HTML even when cached")
    args = parser.parse_args()

    listing_url = f"{API_BASE}/constitutions?{urlencode({'lang': args.lang})}"
    records = request_json(listing_url, args.timeout)
    if not isinstance(records, list):
        raise SystemExit("Constitute API did not return a constitution list.")

    selected = current_public_constitutions(records)
    raw_root = ensure_directory(RAW_DIR / COLLECTION_ID)
    ensure_directory(CLEAN_DIR)
    manifest_documents: list[dict[str, object]] = []
    report: dict[str, object] = {
        "generated_at": datetime.now().isoformat(),
        "source": "Constitute Project API",
        "listing_url": listing_url,
        "language": args.lang,
        "selection": "public in-force non-draft non-historic constitutions",
        "record_count": len(selected),
        "documents": [],
    }

    for index, record in enumerate(selected, start=1):
        cons_id = str(record["id"])
        country_id = str(record.get("country_id") or cons_id)
        doc_id = f"world_constitution_{sanitize_identifier(cons_id)}"
        source_url = constitution_html_url(cons_id, args.lang)
        doc_dir = ensure_directory(raw_root / doc_id)
        raw_path = doc_dir / "source.html"
        metadata_path = doc_dir / "metadata.json"
        clean_path = CLEAN_DIR / f"{doc_id}.txt"

        status = "cached"
        if args.force or not raw_path.exists():
            try:
                payload = request_json(source_url, args.timeout)
                if not isinstance(payload, dict) or "html" not in payload:
                    raise RuntimeError("Constitute API response did not include an html field.")
                raw_path.write_text(str(payload["html"]), encoding="utf-8")
                status = "downloaded"
                time.sleep(args.delay)
            except (HTTPError, URLError, RuntimeError) as exc:
                print(f"{index}/{len(selected)} {doc_id}: failed ({exc})", file=sys.stderr)
                report["documents"].append({"document_id": doc_id, "constitute_id": cons_id, "status": "failed", "error": str(exc)})
                continue

        html = raw_path.read_text(encoding="utf-8")
        cleaned = clean_constitution_html(html)
        clean_path.write_text(cleaned, encoding="utf-8")

        metadata = {
            "document_id": doc_id,
            "constitute_id": cons_id,
            "country": record.get("country"),
            "country_id": country_id,
            "region": record.get("region"),
            "title": record.get("title_long") or record.get("title"),
            "source_url": source_url,
            "source_format": "html",
            "language": args.lang,
            "year_enacted": record.get("year_enacted"),
            "year_revised": record.get("year_revised"),
            "year_updated": record.get("year_updated"),
            "translator": record.get("translator"),
            "copyright": record.get("copyright"),
            "downloaded_at": datetime.now().isoformat(),
        }
        save_json(metadata_path, metadata)

        manifest_documents.append(
            {
                "document_id": doc_id,
                "title": metadata["title"],
                "author": record.get("country") or country_id,
                "date": str(record.get("year_updated") or record.get("year_revised") or record.get("year_enacted") or ""),
                "document_type": "comparative_constitution",
                "source_url": source_url,
                "source_format": "html",
                "chunk_strategy": "constitution_sections",
                "default_issue_tags": [
                    "comparative_constitutional_law",
                    "global_constitutions",
                    "rights",
                    "separation_of_powers",
                ],
            }
        )

        word_count = len(cleaned.split())
        report["documents"].append(
            {
                "document_id": doc_id,
                "constitute_id": cons_id,
                "country_id": country_id,
                "country": record.get("country"),
                "region": record.get("region"),
                "status": status,
                "raw_path": str(raw_path.relative_to(PROJECT_ROOT)),
                "clean_path": str(clean_path.relative_to(PROJECT_ROOT)),
                "word_count": word_count,
            }
        )
        print(f"{index}/{len(selected)} {doc_id}: {status}, {word_count} words")

    update_manifest(manifest_documents)
    save_json(RAW_DIR.parent / "world_constitutions_acquisition_report.json", report)
    print(f"Saved {len(manifest_documents)} world constitutions from Constitute.")


if __name__ == "__main__":
    main()
