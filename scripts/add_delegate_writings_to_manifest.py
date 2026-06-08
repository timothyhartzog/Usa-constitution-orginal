#!/usr/bin/env python3
"""Inject a `delegate_writings` collection into config/sources_manifest.json.

Generates one manifest document per Internet Archive edition listed in
config/delegate_acquisition_targets.json, using the standard archive.org
djvu.txt download URL. Idempotent: re-running replaces the collection in place
rather than duplicating it. No network access required.

After running this, the existing pipeline fetches + integrates the new sources:
    python3 scripts/ingest_sources.py        # needs network
    python3 scripts/clean_text.py
    python3 scripts/chunk_documents.py
    python3 scripts/build_search_index.py
    python3 scripts/build_delegate_dossiers.py
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

MANIFEST_PATH = PROJECT_ROOT / "config" / "sources_manifest.json"
TARGETS_PATH = PROJECT_ROOT / "config" / "delegate_acquisition_targets.json"
COLLECTION_ID = "delegate_writings"

IA_URL = "https://archive.org/download/{ident}/{ident}_djvu.txt"


def slug(name: str) -> str:
    return re.sub(r"[^a-z0-9]+", "_", name.lower()).strip("_")


def build_documents(targets: list[dict]) -> list[dict]:
    documents: list[dict] = []
    for target in targets:
        if target.get("have_authored_corpus"):
            continue  # already ingested elsewhere
        idents = target.get("internet_archive", [])
        for index, ident in enumerate(idents, start=1):
            name = target["name"]
            suffix = f"_vol_{index}" if len(idents) > 1 else ""
            documents.append(
                {
                    "document_id": f"{slug(name)}_writings{suffix}",
                    "title": f"{name} — {target.get('pd_source', 'Writings')}"
                    + (f" (vol. {index})" if len(idents) > 1 else ""),
                    "author": name,
                    "date": "1770-1830",
                    "document_type": "correspondence",
                    "source_url": IA_URL.format(ident=ident),
                    "source_format": "text",
                    "chunk_strategy": "sliding_window",
                    "delegate": name,
                    "default_issue_tags": [
                        "delegate_writings",
                        "founding_era",
                        "constitutional_convention",
                    ],
                }
            )
    return documents


def main() -> None:
    with MANIFEST_PATH.open(encoding="utf-8") as fh:
        manifest = json.load(fh)
    targets = json.load(TARGETS_PATH.open(encoding="utf-8"))["targets"]

    documents = build_documents(targets)
    new_collection = {
        "collection_id": COLLECTION_ID,
        "title": "Delegate Writings — public-domain editions of Convention delegates' own papers",
        "documents": documents,
    }

    collections = manifest["collections"]
    for i, collection in enumerate(collections):
        if collection.get("collection_id") == COLLECTION_ID:
            collections[i] = new_collection
            break
    else:
        collections.append(new_collection)

    with MANIFEST_PATH.open("w", encoding="utf-8") as fh:
        json.dump(manifest, fh, indent=2, ensure_ascii=False)
        fh.write("\n")

    print(f"Wrote {len(documents)} delegate-writings documents to {COLLECTION_ID} collection.")
    for doc in documents:
        print(f"  {doc['document_id']:32} <- {doc['source_url']}")


if __name__ == "__main__":
    main()
