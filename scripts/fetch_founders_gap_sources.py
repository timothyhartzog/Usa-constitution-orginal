#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
import time
from datetime import datetime, timedelta
from pathlib import Path
from typing import Any

import requests

PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from scripts.fetch_founders_online import (
    FoundersOnlineError,
    FoundersOnlineWafChallenge,
    MetadataRecord,
    api_url,
    fetch_document,
    load_metadata_records,
    read_cookie,
)
from scripts.utils.pipeline import CLEAN_DIR, DATA_DIR, ensure_directory, normalize_whitespace, save_json


GAP_DIR = DATA_DIR / "founders_online" / "gap_fetch"
LOG_DIR = DATA_DIR / "research_logs"

TARGETS = [
    {
        "document_id": "hamilton_correspondence_1787_1788",
        "author": "Hamilton, Alexander",
        "date_start": "1787-01-01",
        "date_end": "1788-12-31",
        "title": "Alexander Hamilton correspondence and writings, 1787-1788",
    },
    {
        "document_id": "washington_correspondence_1787_1789",
        "author": "Washington, George",
        "date_start": "1787-01-01",
        "date_end": "1789-12-31",
        "title": "George Washington correspondence and writings, 1787-1789",
    },
    {
        "document_id": "adams_correspondence_1787_1789",
        "author": "Adams, John",
        "date_start": "1787-01-01",
        "date_end": "1789-12-31",
        "title": "John Adams correspondence and writings, 1787-1789",
    },
    {
        "document_id": "jay_correspondence_1787_1789",
        "author": "Jay, John",
        "date_start": "1787-01-01",
        "date_end": "1789-12-31",
        "title": "John Jay correspondence and writings, 1787-1789",
    },
]

FRANKLIN_DOCUMENT_ID = "franklin_speech_constitutional_convention"
FRANKLIN_START = "I confess that there are several parts of this constitution which I do not at present approve"
FRANKLIN_END = "On the whole, Sir, I can not help expressing a wish"


def log(message: str) -> None:
    ensure_directory(LOG_DIR)
    line = f"{datetime.now().isoformat(timespec='seconds')} {message}"
    print(line, flush=True)
    with (LOG_DIR / "founders_gap_fetch.log").open("a", encoding="utf-8") as handle:
        handle.write(line + "\n")


def target_records(records: list[MetadataRecord]) -> dict[str, list[MetadataRecord]]:
    selected: dict[str, list[MetadataRecord]] = {}
    for target in TARGETS:
        rows = [
            record
            for record in records
            if target["author"] in record.authors
            and target["date_start"] <= (record.date_from or "") <= target["date_end"]
        ]
        rows.sort(key=lambda record: ((record.date_from or ""), record.identifier))
        selected[target["document_id"]] = rows
    return selected


def raw_path_for(document_id: str, record: MetadataRecord) -> Path:
    identifier = record.identifier.replace("/", "__")
    return GAP_DIR / document_id / "raw" / f"{identifier}.json"


def content_from_api(data: dict[str, Any]) -> str:
    content = data.get("content") or ""
    if not isinstance(content, str):
        content = str(content)
    return normalize_whitespace(content)


def cached_content(document_id: str, record: MetadataRecord) -> str:
    path = raw_path_for(document_id, record)
    if not path.exists():
        return ""
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return ""
    api = payload.get("api")
    if not isinstance(api, dict):
        return ""
    return content_from_api(api)


def write_raw(document_id: str, record: MetadataRecord, data: dict[str, Any]) -> None:
    path = raw_path_for(document_id, record)
    ensure_directory(path.parent)
    payload = {
        "identifier": record.identifier,
        "metadata": record.__dict__,
        "api": data,
        "fetched_at": datetime.now().isoformat(),
        "api_url": api_url(record.identifier),
    }
    path.write_text(json.dumps(payload, indent=2, ensure_ascii=False), encoding="utf-8")


def rebuild_combined_clean(document_id: str, records: list[MetadataRecord]) -> dict[str, Any]:
    parts: list[str] = []
    included = 0
    words = 0
    for record in records:
        text = cached_content(document_id, record)
        if not text:
            continue
        included += 1
        words += len(text.split())
        heading = [
            "=" * 80,
            record.title,
            f"Date: {record.date_from or record.date_to}",
            f"Founders Online ID: {record.identifier}",
            f"Source: {record.permalink}",
            "=" * 80,
            "",
        ]
        parts.append("\n".join(heading) + text)

    clean_path = CLEAN_DIR / f"{document_id}.txt"
    clean_path.write_text("\n\n".join(parts).strip() + ("\n" if parts else ""), encoding="utf-8")
    return {"document_id": document_id, "included_records": included, "words": words, "clean_path": str(clean_path.relative_to(PROJECT_ROOT))}


def fill_franklin_from_local_avalon() -> dict[str, Any]:
    source = CLEAN_DIR / "madison_debates_complete_avalon.txt"
    clean_path = CLEAN_DIR / f"{FRANKLIN_DOCUMENT_ID}.txt"
    if not source.exists():
        return {"document_id": FRANKLIN_DOCUMENT_ID, "status": "missing_source", "clean_path": str(clean_path.relative_to(PROJECT_ROOT))}
    text = source.read_text(encoding="utf-8")
    start = text.find(FRANKLIN_START)
    end = text.find(FRANKLIN_END, start)
    if start < 0 or end < 0:
        return {"document_id": FRANKLIN_DOCUMENT_ID, "status": "markers_not_found", "clean_path": str(clean_path.relative_to(PROJECT_ROOT))}
    paragraph_end = text.find("\n\n", end)
    if paragraph_end < 0:
        paragraph_end = end + 1200
    speech = normalize_whitespace(text[start:paragraph_end])
    output = "\n".join(
        [
            "Benjamin Franklin — Speech in the Convention at the Conclusion of Its Deliberations",
            "Date: 1787-09-17",
            "Source used: Madison's Debates, Avalon Project copy already cached in madison_debates_complete_avalon.txt",
            "",
            speech,
            "",
        ]
    )
    clean_path.write_text(output, encoding="utf-8")
    return {
        "document_id": FRANKLIN_DOCUMENT_ID,
        "status": "filled_from_local_avalon",
        "words": len(output.split()),
        "clean_path": str(clean_path.relative_to(PROJECT_ROOT)),
    }


def save_status(status: dict[str, Any]) -> None:
    ensure_directory(GAP_DIR)
    save_json(GAP_DIR / "status.json", status)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Fetch and combine the remaining Founders Online gap sources.")
    parser.add_argument("--metadata-file", type=Path, required=True)
    parser.add_argument("--cookie-file", type=Path)
    parser.add_argument("--hours", type=float, default=12)
    parser.add_argument("--requests-per-second", type=float, default=0.25)
    parser.add_argument("--user-agent", default="Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36")
    parser.add_argument("--waf-sleep-seconds", type=int, default=600)
    parser.add_argument("--max-waf-events", type=int, default=500)
    parser.add_argument("--retries", type=int, default=2)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    ensure_directory(GAP_DIR)
    records = load_metadata_records(args.metadata_file, primary_only=True)
    selected = target_records(records)
    deadline = datetime.now() + timedelta(hours=args.hours)
    delay = 1.0 / args.requests_per_second if args.requests_per_second > 0 else 0

    selection_summary = {
        document_id: [record.__dict__ for record in rows]
        for document_id, rows in selected.items()
    }
    save_json(GAP_DIR / "selected_records.json", selection_summary)

    status: dict[str, Any] = {
        "started_at": datetime.now().isoformat(),
        "deadline": deadline.isoformat(),
        "metadata_records": len(records),
        "targets": {document_id: len(rows) for document_id, rows in selected.items()},
        "franklin": fill_franklin_from_local_avalon(),
        "downloaded": 0,
        "cached": 0,
        "failed": 0,
        "waf_events": 0,
        "last_error": "",
        "combined": {},
    }
    save_status(status)
    log(f"loaded metadata={len(records)} targets={status['targets']}")
    log(f"franklin {status['franklin']}")

    cookie = read_cookie(args.cookie_file)
    with requests.Session() as session:
        if cookie:
            session.headers.update({"Cookie": cookie})
        waf_events = 0
        while datetime.now() < deadline:
            made_progress = False
            for document_id, rows in selected.items():
                for record in rows:
                    if datetime.now() >= deadline:
                        break
                    if cached_content(document_id, record):
                        status["cached"] += 1
                        continue
                    try:
                        data = fetch_document(
                            session=session,
                            record=record,
                            requests_per_second=0,
                            retries=args.retries,
                            user_agent=args.user_agent,
                        )
                    except FoundersOnlineWafChallenge as exc:
                        waf_events += 1
                        status["waf_events"] = waf_events
                        status["last_error"] = str(exc)
                        save_status(status)
                        log(f"waf challenge; sleeping {args.waf_sleep_seconds}s before retry")
                        if waf_events >= args.max_waf_events:
                            log("max waf events reached; stopping")
                            break
                        time.sleep(args.waf_sleep_seconds)
                        continue
                    except FoundersOnlineError as exc:
                        status["failed"] += 1
                        status["last_error"] = f"{record.identifier}: {exc}"
                        log(f"failed {record.identifier}: {exc}")
                        save_status(status)
                        time.sleep(delay)
                        continue

                    text = content_from_api(data)
                    if text:
                        write_raw(document_id, record, data)
                        status["downloaded"] += 1
                        made_progress = True
                        if status["downloaded"] == 1 or status["downloaded"] % 25 == 0:
                            log(f"downloaded {status['downloaded']} latest={record.identifier}")
                    else:
                        status["failed"] += 1
                        status["last_error"] = f"{record.identifier}: empty content"
                    time.sleep(delay)
                    save_status(status)

                status["combined"][document_id] = rebuild_combined_clean(document_id, rows)
                save_status(status)
                if waf_events >= args.max_waf_events or datetime.now() >= deadline:
                    break

            if waf_events >= args.max_waf_events or datetime.now() >= deadline:
                break
            if not made_progress:
                log(f"no new documents this cycle; sleeping {args.waf_sleep_seconds}s")
                time.sleep(args.waf_sleep_seconds)

    for document_id, rows in selected.items():
        status["combined"][document_id] = rebuild_combined_clean(document_id, rows)
    status["finished_at"] = datetime.now().isoformat()
    save_status(status)
    log(f"finished downloaded={status['downloaded']} waf_events={status['waf_events']} failed={status['failed']}")


if __name__ == "__main__":
    main()
