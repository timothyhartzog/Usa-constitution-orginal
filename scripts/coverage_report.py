#!/usr/bin/env python3
"""Per-delegate ingest-coverage report.

For every delegate listed in `data/delegates.json` (and every manifest
entry that does not correspond to a delegate), report whether the raw,
cleaned, and chunked artifacts exist on disk. Useful after running
`scripts/ingest_sources.py` to see what was actually pulled in.

Run:

    python3 scripts/coverage_report.py
    python3 scripts/coverage_report.py --json     # machine-readable
    python3 scripts/coverage_report.py --missing  # only delegates with no ingested texts
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
DELEGATES = PROJECT_ROOT / "data" / "delegates.json"
MANIFEST = PROJECT_ROOT / "config" / "sources_manifest.json"
RAW_DIR = PROJECT_ROOT / "data" / "raw"
CLEAN_DIR = PROJECT_ROOT / "data" / "clean"
CHUNKS_FILE = PROJECT_ROOT / "data" / "chunks" / "constitution_full_corpus.json"


def load_manifest() -> dict:
    return json.loads(MANIFEST.read_text(encoding="utf-8"))


def load_delegates() -> dict:
    return json.loads(DELEGATES.read_text(encoding="utf-8"))


def manifest_index(manifest: dict) -> dict[str, dict]:
    """Return `{document_id: {collection, document}}`."""
    out: dict[str, dict] = {}
    for collection in manifest["collections"]:
        for document in collection["documents"]:
            out[document["document_id"]] = {
                "collection_id": collection["collection_id"],
                "document": document,
            }
    return out


def chunk_doc_ids() -> set[str]:
    """Return the set of `document_id` values present in the chunks corpus."""
    if not CHUNKS_FILE.exists():
        return set()
    corpus = json.loads(CHUNKS_FILE.read_text(encoding="utf-8"))
    return {chunk["document_id"] for chunk in corpus.get("chunks", [])}


def coverage_for(document_id: str, manifest_idx: dict, doc_ids: set[str]) -> dict:
    """Return coverage flags for one manifest document_id."""
    entry = manifest_idx.get(document_id)
    if entry is None:
        return {"in_manifest": False, "raw": False, "cleaned": False, "chunked": False}
    document = entry["document"]
    collection_id = entry["collection_id"]
    extension = "html" if document["source_format"] == "html" else "txt"
    raw_path = RAW_DIR / collection_id / document_id / f"source.{extension}"
    clean_path = CLEAN_DIR / f"{document_id}.txt"
    # Chunks may have a derived document_id (e.g. essay splitter) — match the prefix.
    chunked = any(d == document_id or d.startswith(f"{document_id}_") for d in doc_ids)
    return {
        "in_manifest": True,
        "raw": raw_path.exists(),
        "cleaned": clean_path.exists(),
        "chunked": chunked,
    }


def render_text(report: list[dict]) -> str:
    cols = ["status", "delegate", "manifest_entry", "raw", "cleaned", "chunked"]
    width = {c: max(len(c), max((len(str(r.get(c, ""))) for r in report), default=0)) for c in cols}
    lines = ["  ".join(c.ljust(width[c]) for c in cols)]
    lines.append("  ".join("-" * width[c] for c in cols))
    for r in report:
        lines.append("  ".join(str(r.get(c, "")).ljust(width[c]) for c in cols))
    return "\n".join(lines)


def status_label(cov: dict) -> str:
    if not cov["in_manifest"]:
        return "no manifest"
    if cov["chunked"]:
        return "ingested"
    if cov["cleaned"]:
        return "cleaned"
    if cov["raw"]:
        return "raw only"
    return "pending"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="emit machine-readable JSON")
    parser.add_argument("--missing", action="store_true", help="only show entries without chunked output")
    args = parser.parse_args()

    delegates = load_delegates()["delegates"]
    manifest = load_manifest()
    manifest_idx = manifest_index(manifest)
    doc_ids = chunk_doc_ids()

    report: list[dict] = []
    for delegate in delegates:
        manifest_entry = delegate.get("manifest_entry")
        cov = coverage_for(manifest_entry, manifest_idx, doc_ids) if manifest_entry else {
            "in_manifest": False,
            "raw": False,
            "cleaned": False,
            "chunked": False,
        }
        row = {
            "delegate_id": delegate["id"],
            "delegate": delegate["name"],
            "state": delegate["state"],
            "manifest_entry": manifest_entry or "—",
            "in_manifest": cov["in_manifest"],
            "raw": cov["raw"],
            "cleaned": cov["cleaned"],
            "chunked": cov["chunked"],
            "status": status_label(cov),
        }
        report.append(row)

    if args.missing:
        report = [r for r in report if not r["chunked"]]

    if args.json:
        print(json.dumps(report, indent=2))
        return

    print(render_text(report))
    print()
    counts = {"ingested": 0, "cleaned": 0, "raw only": 0, "pending": 0, "no manifest": 0}
    for r in report:
        counts[r["status"]] = counts.get(r["status"], 0) + 1
    total = len(report)
    print(f"Summary ({total} delegates):")
    for k in ["ingested", "cleaned", "raw only", "pending", "no manifest"]:
        print(f"  {k:14} {counts[k]:>3}")
    if counts["no manifest"] > 0:
        print()
        print(f"Note: {counts['no manifest']} delegates have no associated manifest entry "
              f"(typically because their public-domain papers are too sparse to ingest).")


if __name__ == "__main__":
    sys.exit(main() or 0)
