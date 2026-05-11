#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import sys
import time
import xml.etree.ElementTree as ET
from datetime import datetime
from pathlib import Path
from typing import Any
from urllib.parse import unquote

import requests

PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from scripts.utils.pipeline import CLEAN_DIR, DATA_DIR, RAW_DIR, ensure_directory, normalize_whitespace, save_json


ITEM_URL = "https://www.loc.gov/item/76002592/"
BASE_STORAGE = "https://tile.loc.gov/storage-services/service/ll/lldelegatesvol"
COLLECTION_ID = "letters_delegates_congress"
RAW_COLLECTION_DIR = RAW_DIR / COLLECTION_ID
REPORT_PATH = DATA_DIR / "letters_delegates_congress_acquisition_report.json"
LOG_PATH = DATA_DIR / "research_logs" / "letters_delegates_congress.log"


def log(message: str) -> None:
    ensure_directory(LOG_PATH.parent)
    line = f"{datetime.now().isoformat(timespec='seconds')} {message}"
    print(line, flush=True)
    with LOG_PATH.open("a", encoding="utf-8") as handle:
        handle.write(line + "\n")


def fetch_text(session: requests.Session, url: str, timeout: int = 60) -> str:
    response = session.get(url, timeout=timeout, headers={"User-Agent": "Constitutional Research Corpus/2.0"})
    response.raise_for_status()
    return response.text


def resources_from_item_html(html: str) -> list[dict[str, Any]]:
    match = re.search(r'"resources":\s*(\[.*?\])\s*,\s*"related_items"', html, flags=re.S)
    if not match:
        # Fallback: derive resources from repeated LOC volume URLs.
        resources: list[dict[str, Any]] = []
        for volume in range(1, 27):
            volume_id = f"76002592_{volume:02d}"
            page_count_match = re.search(rf'"caption":\s*"([^"]+)"[^{{}}]+?"files":\s*(\d+)[^{{}}]+?"url":\s*"https://www.loc.gov/resource/lldelegatesvol\.{volume_id}/"', html, flags=re.S)
            if page_count_match:
                resources.append(
                    {
                        "caption": page_count_match.group(1),
                        "files": int(page_count_match.group(2)),
                        "url": f"https://www.loc.gov/resource/lldelegatesvol.{volume_id}/",
                    }
                )
        if resources:
            return resources
        raise RuntimeError("Could not locate LOC resources in item page.")
    return json.loads(match.group(1))


def volume_id_from_resource(resource: dict[str, Any]) -> str:
    url = str(resource.get("url", ""))
    match = re.search(r"lldelegatesvol\.(76002592_\d{2})", url)
    if match:
        return match.group(1)
    pdf = str(resource.get("pdf", ""))
    match = re.search(r"(76002592_\d{2})", pdf)
    if match:
        return match.group(1)
    raise RuntimeError(f"Could not determine volume id from resource: {resource}")


def document_id_for_volume(volume_number: int) -> str:
    return f"letters_delegates_congress_vol_{volume_number:02d}"


def alto_url(volume_id: str, page_number: int) -> str:
    return f"{BASE_STORAGE}/{volume_id}/{page_number:04d}.alto.xml"


def extract_alto_text(xml_text: str) -> str:
    try:
        root = ET.fromstring(xml_text)
    except ET.ParseError:
        return ""
    words: list[str] = []
    for element in root.iter():
        if element.tag.endswith("String"):
            content = element.attrib.get("CONTENT", "")
            if content:
                words.append(content)
        elif element.tag.endswith("SP"):
            words.append(" ")
    text = " ".join(words)
    text = text.replace(" ,", ",").replace(" .", ".").replace(" ;", ";").replace(" :", ":")
    text = text.replace(" )", ")").replace("( ", "(")
    return normalize_whitespace(text)


def load_cached_page(path: Path) -> str:
    if not path.exists():
        return ""
    return path.read_text(encoding="utf-8", errors="ignore")


def fetch_volume(
    session: requests.Session,
    resource: dict[str, Any],
    *,
    delay: float,
    force: bool,
    max_pages: int | None,
) -> dict[str, Any]:
    volume_id = volume_id_from_resource(resource)
    volume_number = int(volume_id.rsplit("_", 1)[1])
    document_id = document_id_for_volume(volume_number)
    page_count = int(resource.get("files") or 0)
    if max_pages is not None:
        page_count = min(page_count, max_pages)

    raw_volume_dir = RAW_COLLECTION_DIR / document_id
    ensure_directory(raw_volume_dir)
    clean_path = CLEAN_DIR / f"{document_id}.txt"
    metadata_path = raw_volume_dir / "metadata.json"
    pages_path = raw_volume_dir / "pages.jsonl"

    if clean_path.exists() and clean_path.stat().st_size > 0 and not force:
        words = len(clean_path.read_text(encoding="utf-8", errors="ignore").split())
        return {
            "document_id": document_id,
            "volume_id": volume_id,
            "caption": resource.get("caption", ""),
            "status": "cached",
            "pages": page_count,
            "word_count": words,
            "clean_path": str(clean_path.relative_to(PROJECT_ROOT)),
            "source_url": resource.get("url", ""),
        }

    page_rows: list[dict[str, Any]] = []
    text_parts: list[str] = [
        f"Letters of Delegates to Congress, 1774-1789 - {resource.get('caption', f'Volume {volume_number}')}",
        f"Source: {resource.get('url', ITEM_URL)}",
        f"Library of Congress item: {ITEM_URL}",
        "",
    ]
    downloaded = 0
    failed = 0
    for page_number in range(1, page_count + 1):
        url = alto_url(volume_id, page_number)
        page_cache = raw_volume_dir / f"{page_number:04d}.txt"
        page_text = load_cached_page(page_cache) if not force else ""
        if not page_text:
            try:
                xml_text = fetch_text(session, url)
                page_text = extract_alto_text(xml_text)
                page_cache.write_text(page_text + "\n", encoding="utf-8")
                downloaded += 1
            except requests.RequestException as exc:
                failed += 1
                page_rows.append({"page": page_number, "url": url, "status": "failed", "error": str(exc)})
                time.sleep(max(delay, 1.0))
                continue
            time.sleep(delay)

        if page_text:
            text_parts.append(f"\n\n[Page {page_number}]\n{page_text}")
        page_rows.append({"page": page_number, "url": url, "status": "ok", "words": len(page_text.split())})
        if page_number == 1 or page_number % 100 == 0 or page_number == page_count:
            log(f"{document_id}: page {page_number}/{page_count} downloaded={downloaded} failed={failed}")

    combined = "\n".join(text_parts).strip() + "\n"
    clean_path.write_text(combined, encoding="utf-8")
    metadata = {
        "document_id": document_id,
        "volume_id": volume_id,
        "caption": resource.get("caption", ""),
        "files": resource.get("files"),
        "pdf": str(resource.get("pdf", "")).replace(" ", ""),
        "source_url": resource.get("url", ""),
        "item_url": ITEM_URL,
        "generated_at": datetime.now().isoformat(),
        "downloaded_pages": downloaded,
        "failed_pages": failed,
        "word_count": len(combined.split()),
        "clean_path": str(clean_path.relative_to(PROJECT_ROOT)),
    }
    metadata_path.write_text(json.dumps(metadata, indent=2, ensure_ascii=False), encoding="utf-8")
    with pages_path.open("w", encoding="utf-8") as handle:
        for row in page_rows:
            handle.write(json.dumps(row, ensure_ascii=False, sort_keys=True) + "\n")
    return {**metadata, "status": "downloaded"}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Download LOC Letters of Delegates to Congress page-level OCR into clean volume text files.")
    parser.add_argument("--delay", type=float, default=0.25)
    parser.add_argument("--force", action="store_true")
    parser.add_argument("--volumes", nargs="*", type=int, help="Specific volume numbers to fetch. Defaults to all 26.")
    parser.add_argument("--max-pages", type=int, help="Testing only: cap pages per volume.")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    ensure_directory(CLEAN_DIR)
    ensure_directory(RAW_COLLECTION_DIR)
    with requests.Session() as session:
        html = fetch_text(session, ITEM_URL)
        resources = resources_from_item_html(html)
        resources.sort(key=lambda resource: int(volume_id_from_resource(resource).rsplit("_", 1)[1]))
        if args.volumes:
            wanted = {int(volume) for volume in args.volumes}
            resources = [
                resource
                for resource in resources
                if int(volume_id_from_resource(resource).rsplit("_", 1)[1]) in wanted
            ]
        log(f"found {len(resources)} LOC volume resources")
        documents = []
        for resource in resources:
            result = fetch_volume(session, resource, delay=args.delay, force=args.force, max_pages=args.max_pages)
            documents.append(result)
            save_json(
                REPORT_PATH,
                {
                    "generated_at": datetime.now().isoformat(),
                    "source": ITEM_URL,
                    "collection": COLLECTION_ID,
                    "documents": documents,
                },
            )
        log(f"finished volumes={len(documents)} words={sum(int(row.get('word_count') or 0) for row in documents)}")


if __name__ == "__main__":
    main()
