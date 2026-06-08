#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import sys
from collections import Counter
from datetime import datetime
from pathlib import Path

import requests

PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from scripts.utils.pipeline import RAW_DIR, ensure_directory, iter_documents, load_manifest, save_json


HEADERS = {
    "User-Agent": "Constitutional Research System/2.0 (local archival build)"
}
TIMEOUT = 45


def fetch_document(url: str) -> str:
    response = requests.get(url, headers=HEADERS, timeout=TIMEOUT)
    response.raise_for_status()
    return response.text


def write_summary(report: dict[str, object], summary_path: Path) -> None:
    """Emit a Markdown summary suitable for $GITHUB_STEP_SUMMARY."""
    docs = report["documents"]
    counts = Counter(d["status"] for d in docs)
    lines = [
        "## Constitution corpus ingest",
        "",
        f"Generated at `{report['generated_at']}` "
        f"({'force-refresh' if report['force_refresh'] else 'incremental'}).",
        "",
        "| Status | Count |",
        "|---|---:|",
    ]
    for status in ("downloaded", "cached", "failed"):
        lines.append(f"| {status} | {counts.get(status, 0)} |")
    lines.append(f"| **total** | **{len(docs)}** |")

    failures = [d for d in docs if d["status"] == "failed"]
    if failures:
        lines += ["", "### Failed", "", "| Document | URL | Error |", "|---|---|---|"]
        for d in failures:
            lines.append(
                f"| `{d['document_id']}` | <{d['source_url']}> | {d['error']} |"
            )

    downloaded = [d for d in docs if d["status"] == "downloaded"]
    if downloaded:
        lines += ["", "### Newly downloaded", "", "| Document | Collection |", "|---|---|"]
        for d in downloaded:
            lines.append(f"| `{d['document_id']}` | {d['source_collection']} |")

    summary_path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description="Download and cache configured public-domain sources.")
    parser.add_argument("--force", action="store_true", help="Re-fetch raw sources even when cached")
    parser.add_argument(
        "--summary",
        type=Path,
        default=None,
        help="Write a Markdown step-summary to this path (defaults to "
        "$GITHUB_STEP_SUMMARY when set, otherwise none).",
    )
    args = parser.parse_args()

    manifest = load_manifest()
    ensure_directory(RAW_DIR)

    report: dict[str, object] = {
        "generated_at": datetime.now().isoformat(),
        "force_refresh": args.force,
        "documents": [],
    }

    for collection, document in iter_documents(manifest):
        doc_dir = ensure_directory(RAW_DIR / collection["collection_id"] / document["document_id"])
        extension = "html" if document["source_format"] == "html" else "txt"
        source_path = doc_dir / f"source.{extension}"
        status = "cached"
        error_message = None

        if args.force or not source_path.exists():
            try:
                payload = fetch_document(document["source_url"])
                source_path.write_text(payload, encoding="utf-8")
                status = "downloaded"
            except requests.RequestException as exc:
                error_message = str(exc)
                status = "failed"

        save_json(
            doc_dir / "metadata.json",
            {
                "document_id": document["document_id"],
                "title": document["title"],
                "source_url": document["source_url"],
                "source_format": document["source_format"],
                "downloaded_at": datetime.now().isoformat(),
            },
        )

        report["documents"].append(
            {
                "document_id": document["document_id"],
                "source_collection": collection["collection_id"],
                "status": status,
                "source_url": document["source_url"],
                "raw_path": str(source_path.relative_to(RAW_DIR.parent)),
                "error": error_message,
            }
        )

        print(f"{collection['collection_id']}/{document['document_id']}: {status}")

    save_json(RAW_DIR.parent / "acquisition_report.json", report)

    summary_path = args.summary or (
        Path(os.environ["GITHUB_STEP_SUMMARY"]) if os.environ.get("GITHUB_STEP_SUMMARY") else None
    )
    if summary_path is not None:
        write_summary(report, summary_path)


if __name__ == "__main__":
    main()
