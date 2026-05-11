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

import requests

PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from scripts.utils.pipeline import CLEAN_DIR, DATA_DIR, RAW_DIR, ensure_directory, normalize_whitespace, save_json


COLLECTION_ID = "elliots_debates"
ITEM_URL = "https://www.loc.gov/item/17007172/"
RAW_COLLECTION_DIR = RAW_DIR / COLLECTION_ID
REPORT_PATH = DATA_DIR / "elliots_debates_acquisition_report.json"
LOG_PATH = DATA_DIR / "research_logs" / "elliots_debates.log"

VOLUMES = [
    {
        "volume": 1,
        "document_id": "elliots_debates_vol_1",
        "title": "Elliot's Debates, Volume 1",
        "caption": "volume 1",
        "fulltext_url": "https://tile.loc.gov/storage-services/service/ll/llscd/lled001/lled001.xml",
        "pdf_url": "https://tile.loc.gov/storage-services/service/ll/llscd/lled001/lled001.pdf",
    },
    {
        "volume": 2,
        "document_id": "elliots_debates_vol_2",
        "title": "Elliot's Debates, Volume 2",
        "caption": "volume 2",
        "fulltext_url": "https://tile.loc.gov/storage-services/service/ll/llscd/lled002/lled002.xml",
        "pdf_url": "https://tile.loc.gov/storage-services/service/ll/llscd/lled002/lled002.pdf",
    },
    {
        "volume": 3,
        "document_id": "elliots_debates_vol_3",
        "title": "Elliot's Debates, Volume 3",
        "caption": "volume 3",
        "fulltext_url": "https://tile.loc.gov/storage-services/service/ll/llscd/lled003/lled003.xml",
        "pdf_url": "https://tile.loc.gov/storage-services/service/ll/llscd/lled003/lled003.pdf",
    },
    {
        "volume": 4,
        "document_id": "elliots_debates_vol_4",
        "title": "Elliot's Debates, Volume 4",
        "caption": "volume 4",
        "fulltext_url": "https://tile.loc.gov/storage-services/service/ll/llscd/lled004/lled004.xml",
        "pdf_url": "https://tile.loc.gov/storage-services/service/ll/llscd/lled004/lled004.pdf",
    },
    {
        "volume": 5,
        "document_id": "elliots_debates_vol_5",
        "title": "Elliot's Debates, Volume 5",
        "caption": "volume 5",
        "fulltext_url": "https://tile.loc.gov/storage-services/service/ll/llscd/lled005/lled005.xml",
        "pdf_url": "https://tile.loc.gov/storage-services/service/ll/llscd/lled005/lled005.pdf",
    },
]


def log(message: str) -> None:
    ensure_directory(LOG_PATH.parent)
    line = f"{datetime.now().isoformat(timespec='seconds')} {message}"
    print(line, flush=True)
    with LOG_PATH.open("a", encoding="utf-8") as handle:
        handle.write(line + "\n")


def fetch_text(session: requests.Session, url: str, timeout: int) -> str:
    response = session.get(
        url,
        timeout=timeout,
        headers={"User-Agent": "Constitutional Research Corpus/2.0"},
    )
    response.raise_for_status()
    return response.text


def element_text(element: ET.Element) -> str:
    parts: list[str] = []
    for text in element.itertext():
        if text:
            parts.append(text)
    return normalize_whitespace(" ".join(parts))


def extract_clean_text(xml_text: str, volume: dict[str, Any]) -> str:
    root = ET.fromstring(xml_text)
    paragraphs: list[str] = []
    for tag in ("head", "p"):
        for element in root.iter(tag):
            text = element_text(element)
            if text:
                paragraphs.append(text)

    text = "\n\n".join(paragraphs)
    text = re.sub(r"\s+([,.;:?!])", r"\1", text)
    text = re.sub(r"\(\s+", "(", text)
    text = re.sub(r"\s+\)", ")", text)
    text = normalize_whitespace(text)
    header = "\n".join(
        [
            str(volume["title"]),
            "The Debates in the Several State Conventions on the Adoption of the Federal Constitution",
            "Editor: Jonathan Elliot",
            f"Source: {ITEM_URL}",
            f"Library of Congress full text: {volume['fulltext_url']}",
            "",
        ]
    )
    return f"{header}{text}\n"


def fetch_volume(session: requests.Session, volume: dict[str, Any], *, force: bool, timeout: int) -> dict[str, Any]:
    document_id = str(volume["document_id"])
    raw_dir = ensure_directory(RAW_COLLECTION_DIR / document_id)
    xml_path = raw_dir / "source.xml"
    raw_text_path = raw_dir / "source.txt"
    clean_path = CLEAN_DIR / f"{document_id}.txt"
    metadata_path = raw_dir / "metadata.json"

    status = "cached"
    if force or not xml_path.exists() or xml_path.stat().st_size == 0:
        xml_text = fetch_text(session, str(volume["fulltext_url"]), timeout=timeout)
        xml_path.write_text(xml_text, encoding="utf-8")
        status = "downloaded"
    else:
        xml_text = xml_path.read_text(encoding="utf-8", errors="ignore")

    clean_text = extract_clean_text(xml_text, volume)
    raw_text_path.write_text(clean_text, encoding="utf-8")
    clean_path.write_text(clean_text, encoding="utf-8")

    metadata = {
        "document_id": document_id,
        "title": volume["title"],
        "caption": volume["caption"],
        "volume": volume["volume"],
        "source_url": ITEM_URL,
        "fulltext_url": volume["fulltext_url"],
        "pdf_url": volume["pdf_url"],
        "source_format": "loc_fulltext_xml",
        "status": status,
        "generated_at": datetime.now().isoformat(),
        "raw_xml_path": str(xml_path.relative_to(PROJECT_ROOT)),
        "raw_text_path": str(raw_text_path.relative_to(PROJECT_ROOT)),
        "clean_path": str(clean_path.relative_to(PROJECT_ROOT)),
        "byte_count": xml_path.stat().st_size,
        "word_count": len(clean_text.split()),
    }
    metadata_path.write_text(json.dumps(metadata, indent=2, ensure_ascii=False), encoding="utf-8")
    return metadata


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Download LOC full text for Elliot's Debates volumes 1-5.")
    parser.add_argument("--force", action="store_true", help="Re-fetch LOC XML even when cached")
    parser.add_argument("--delay", type=float, default=0.25)
    parser.add_argument("--timeout", type=int, default=90)
    parser.add_argument("--volumes", nargs="*", type=int, choices=range(1, 6), help="Specific volume numbers to fetch")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    ensure_directory(CLEAN_DIR)
    ensure_directory(RAW_COLLECTION_DIR)
    wanted = set(args.volumes or [1, 2, 3, 4, 5])
    selected = [volume for volume in VOLUMES if int(volume["volume"]) in wanted]
    documents: list[dict[str, Any]] = []

    with requests.Session() as session:
        for volume in selected:
            log(f"fetching {volume['document_id']}")
            result = fetch_volume(session, volume, force=args.force, timeout=args.timeout)
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
            log(f"{result['document_id']}: {result['status']} words={result['word_count']}")
            time.sleep(args.delay)

    log(f"finished volumes={len(documents)} words={sum(int(row['word_count']) for row in documents)}")


if __name__ == "__main__":
    main()
