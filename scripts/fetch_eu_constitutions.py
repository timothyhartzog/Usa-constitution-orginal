#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
import time
from datetime import datetime
from html.parser import HTMLParser
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.parse import urlencode
from urllib.request import Request, urlopen

PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from scripts.utils.pipeline import CLEAN_DIR, MANIFEST_PATH, RAW_DIR, ensure_directory, load_manifest, normalize_whitespace, save_json, sanitize_identifier


API_BASE = "https://www.constituteproject.org/service"
COLLECTION_ID = "comparative_constitutions_eu"
HEADERS = {
    "User-Agent": "Constitutional Research System/2.0 (comparative analysis; contact local researcher)",
    "Accept": "application/json,text/html;q=0.9,*/*;q=0.8",
}
EU_COUNTRY_IDS = {
    "Austria",
    "Belgium",
    "Bulgaria",
    "Croatia",
    "Cyprus",
    "Czech_Republic",
    "Czech_Republic_the",
    "Czechia",
    "Denmark",
    "Estonia",
    "Finland",
    "France",
    "Germany",
    "Greece",
    "Hungary",
    "Ireland",
    "Italy",
    "Latvia",
    "Lithuania",
    "Luxembourg",
    "Malta",
    "Netherlands",
    "Netherlands_the",
    "Poland",
    "Portugal",
    "Romania",
    "Slovakia",
    "Slovenia",
    "Spain",
    "Sweden",
}
EXPECTED_EU_MEMBERS = 27


class ConstitutionTextParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.parts: list[str] = []
        self.skip_depth = 0

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        if tag in {"script", "style", "noscript"}:
            self.skip_depth += 1
            return
        if tag in {"h1", "h2", "h3", "h4", "h5", "p", "div", "section", "article", "li", "br"}:
            self.parts.append("\n")

    def handle_endtag(self, tag: str) -> None:
        if tag in {"script", "style", "noscript"} and self.skip_depth:
            self.skip_depth -= 1
            return
        if tag in {"h1", "h2", "h3", "h4", "h5", "p", "div", "section", "article", "li"}:
            self.parts.append("\n")

    def handle_data(self, data: str) -> None:
        if self.skip_depth:
            return
        stripped = data.strip()
        if stripped:
            self.parts.append(stripped)

    def text(self) -> str:
        return "\n".join(self.parts)


def request_json(url: str, timeout: int) -> object:
    request = Request(url, headers=HEADERS)
    with urlopen(request, timeout=timeout) as response:
        return json.loads(response.read().decode("utf-8"))


def constitution_html_url(cons_id: str, lang: str) -> str:
    return f"{API_BASE}/html?{urlencode({'cons_id': cons_id, 'lang': lang})}"


def normalize_country_id(country_id: str) -> str:
    return country_id.replace(" ", "_")


def choose_current_eu_constitutions(records: list[dict[str, object]]) -> list[dict[str, object]]:
    by_country: dict[str, dict[str, object]] = {}
    for record in records:
        if not record.get("public", False):
            continue
        if not record.get("in_force", False):
            continue
        country_id = normalize_country_id(str(record.get("country_id", "")))
        if country_id not in EU_COUNTRY_IDS:
            continue

        current = by_country.get(country_id)
        if current is None:
            by_country[country_id] = record
            continue

        # Prefer the newest revised/updated/enacted text if Constitute returns
        # more than one in-force entry for a country.
        record_year = max(int(record.get(key) or 0) for key in ("year_updated", "year_revised", "year_enacted"))
        current_year = max(int(current.get(key) or 0) for key in ("year_updated", "year_revised", "year_enacted"))
        if record_year > current_year:
            by_country[country_id] = record

    return sorted(by_country.values(), key=lambda item: str(item.get("country_id", "")))


def clean_constitution_html(html: str) -> str:
    parser = ConstitutionTextParser()
    parser.feed(html)
    text = parser.text()
    text = "\n".join(line.strip() for line in text.splitlines())
    return normalize_whitespace(text)


def update_manifest(documents: list[dict[str, object]]) -> None:
    manifest = load_manifest()
    collections = manifest.setdefault("collections", [])
    collection = {
        "collection_id": COLLECTION_ID,
        "title": "European Union Member State Constitutions from Constitute Project",
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
    parser = argparse.ArgumentParser(description="Download current EU member-state constitutions from the Constitute API.")
    parser.add_argument("--lang", default="en", choices=("en", "es", "ar"), help="Constitute language to fetch")
    parser.add_argument("--delay", type=float, default=0.35, help="Delay between constitution HTML requests")
    parser.add_argument("--timeout", type=int, default=45, help="HTTP timeout in seconds")
    parser.add_argument("--force", action="store_true", help="Re-fetch HTML even when cached")
    args = parser.parse_args()

    listing_url = f"{API_BASE}/constitutions?{urlencode({'lang': args.lang})}"
    records = request_json(listing_url, args.timeout)
    if not isinstance(records, list):
        raise SystemExit("Constitute API did not return a constitution list.")

    selected = choose_current_eu_constitutions(records)
    if len(selected) != EXPECTED_EU_MEMBERS:
        print(
            f"Warning: expected {EXPECTED_EU_MEMBERS} EU member constitutions, selected {len(selected)}.",
            file=sys.stderr,
        )

    raw_root = ensure_directory(RAW_DIR / COLLECTION_ID)
    ensure_directory(CLEAN_DIR)
    manifest_documents: list[dict[str, object]] = []
    report: dict[str, object] = {
        "generated_at": datetime.now().isoformat(),
        "source": "Constitute Project API",
        "listing_url": listing_url,
        "language": args.lang,
        "documents": [],
    }

    for record in selected:
        cons_id = str(record["id"])
        country_id = normalize_country_id(str(record["country_id"]))
        country_slug = sanitize_identifier(country_id)
        doc_id = f"eu_constitution_{country_slug}"
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
                print(f"{doc_id}: failed ({exc})", file=sys.stderr)
                report["documents"].append({"document_id": doc_id, "status": "failed", "error": str(exc)})
                continue

        html = raw_path.read_text(encoding="utf-8")
        cleaned = clean_constitution_html(html)
        clean_path.write_text(cleaned, encoding="utf-8")

        metadata = {
            "document_id": doc_id,
            "constitute_id": cons_id,
            "country": record.get("country"),
            "country_id": record.get("country_id"),
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
                "author": record.get("country") or record.get("country_id"),
                "date": str(record.get("year_updated") or record.get("year_revised") or record.get("year_enacted") or ""),
                "document_type": "comparative_constitution",
                "source_url": source_url,
                "source_format": "html",
                "chunk_strategy": "constitution_sections",
                "default_issue_tags": [
                    "comparative_constitutional_law",
                    "european_union",
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
                "country_id": record.get("country_id"),
                "status": status,
                "raw_path": str(raw_path.relative_to(PROJECT_ROOT)),
                "clean_path": str(clean_path.relative_to(PROJECT_ROOT)),
                "word_count": word_count,
            }
        )
        print(f"{doc_id}: {status}, {word_count} words")

    update_manifest(manifest_documents)
    save_json(RAW_DIR.parent / "eu_constitutions_acquisition_report.json", report)
    print(f"Saved {len(manifest_documents)} EU constitutions from Constitute.")


if __name__ == "__main__":
    main()
