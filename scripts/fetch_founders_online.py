#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
import time
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Any, Iterable
from urllib.parse import quote

import requests

PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from scripts.chunk_documents import paragraph_chunk, word_count
from scripts.utils.pipeline import ensure_directory, normalize_whitespace, preview_text, sanitize_identifier, save_json, unique_preserve


BASE_URL = "https://founders.archives.gov"
METADATA_URL = f"{BASE_URL}/Metadata/founders-online-metadata.json"
TIMESTAMP_URL = f"{BASE_URL}/Metadata/timestamp.txt"
API_PREFIX = f"{BASE_URL}/API/docdata"
DATASET_DIR = PROJECT_ROOT / "data" / "founders_online"
METADATA_DIR = DATASET_DIR / "metadata"
RAW_DIR = DATASET_DIR / "raw"
CLEAN_DIR = DATASET_DIR / "clean"
CHUNKS_DIR = DATASET_DIR / "chunks"
REPORTS_DIR = DATASET_DIR / "reports"
USER_AGENT = "Constitutional Research System/2.0 (bulk Founders Online import; local research build)"
TIMEOUT = 60


class FoundersOnlineError(RuntimeError):
    pass


class FoundersOnlineWafChallenge(FoundersOnlineError):
    pass


@dataclass(frozen=True)
class MetadataRecord:
    identifier: str
    title: str
    permalink: str
    project: str
    authors: list[str]
    recipients: list[str]
    date_from: str
    date_to: str


def request_headers(user_agent: str = USER_AGENT) -> dict[str, str]:
    return {
        "Accept": "application/json,text/plain;q=0.9,*/*;q=0.8",
        "User-Agent": user_agent,
    }


def detect_waf_challenge(response: requests.Response) -> bool:
    return (
        response.status_code == 202
        and response.headers.get("x-amzn-waf-action", "").lower() == "challenge"
    )


def get_url(session: requests.Session, url: str, user_agent: str = USER_AGENT) -> requests.Response:
    response = session.get(url, headers=request_headers(user_agent), timeout=TIMEOUT)
    if detect_waf_challenge(response):
        raise FoundersOnlineWafChallenge(
            "Founders Online returned a CloudFront WAF challenge for this command-line request. "
            "Download the metadata JSON in a browser and pass it with --metadata-file, or retry "
            "from a network/session that can access founders.archives.gov without the challenge."
        )
    response.raise_for_status()
    return response


def download_metadata(force: bool = False, cookie: str = "", user_agent: str = USER_AGENT) -> Path:
    ensure_directory(METADATA_DIR)
    metadata_path = METADATA_DIR / "founders-online-metadata.json"
    timestamp_path = METADATA_DIR / "timestamp.txt"

    if metadata_path.exists() and not force:
        return metadata_path

    with requests.Session() as session:
        if cookie:
            session.headers.update({"Cookie": cookie})
        timestamp = get_url(session, TIMESTAMP_URL, user_agent=user_agent).text.strip()
        payload = get_url(session, METADATA_URL, user_agent=user_agent).content

    metadata_path.write_bytes(payload)
    timestamp_path.write_text(f"{timestamp}\n", encoding="utf-8")
    return metadata_path


def load_metadata_payload(path: Path) -> Any:
    with path.open(encoding="utf-8") as handle:
        return json.load(handle)


def metadata_items(payload: Any) -> list[dict[str, Any]]:
    if isinstance(payload, list):
        return [item for item in payload if isinstance(item, dict)]
    if isinstance(payload, dict):
        for key in ("documents", "records", "items", "data"):
            value = payload.get(key)
            if isinstance(value, list):
                return [item for item in value if isinstance(item, dict)]
        for value in payload.values():
            if isinstance(value, list) and all(isinstance(item, dict) for item in value[:5]):
                return value
    raise FoundersOnlineError("Could not find a document list in the metadata JSON.")


def string_list(value: Any) -> list[str]:
    if value is None:
        return []
    if isinstance(value, str):
        return [value] if value else []
    if isinstance(value, list):
        values: list[str] = []
        for item in value:
            if isinstance(item, str) and item:
                values.append(item)
            elif isinstance(item, dict):
                name = item.get("name") or item.get("value") or item.get("display")
                if isinstance(name, str) and name:
                    values.append(name)
        return unique_preserve(values)
    return [str(value)]


def first_string(record: dict[str, Any], *keys: str) -> str:
    for key in keys:
        value = record.get(key)
        if isinstance(value, str) and value:
            return value
    return ""


def identifier_from_permalink(permalink: str) -> str:
    marker = f"{BASE_URL}/documents/"
    if marker in permalink:
        return permalink.split(marker, 1)[1].strip("/")
    if "/documents/" in permalink:
        return permalink.split("/documents/", 1)[1].strip("/")
    return ""


def normalize_metadata_record(record: dict[str, Any]) -> MetadataRecord | None:
    permalink = first_string(record, "permalink", "url", "uri", "link")
    identifier = first_string(record, "id", "identifier", "document-id", "document_id")
    if not identifier and permalink:
        identifier = identifier_from_permalink(permalink)
    if not identifier:
        return None

    if not permalink:
        permalink = f"{BASE_URL}/documents/{identifier}"

    return MetadataRecord(
        identifier=identifier,
        title=first_string(record, "title", "display-title", "document-title") or identifier,
        permalink=permalink,
        project=first_string(record, "project", "papers", "collection") or "Founders Online",
        authors=string_list(record.get("authors") or record.get("author")),
        recipients=string_list(record.get("recipients") or record.get("recipient")),
        date_from=first_string(record, "date-from", "date_from", "date"),
        date_to=first_string(record, "date-to", "date_to"),
    )


def load_metadata_records(path: Path, primary_only: bool) -> list[MetadataRecord]:
    payload = load_metadata_payload(path)
    records: list[MetadataRecord] = []
    for item in metadata_items(payload):
        record = normalize_metadata_record(item)
        if not record:
            continue
        if primary_only and not record.date_from:
            continue
        records.append(record)
    return records


def local_doc_path(root: Path, identifier: str, suffix: str) -> Path:
    parts = identifier.split("/")
    if len(parts) >= 2:
        subdir = root / sanitize_identifier(parts[0])
        filename = "__".join(sanitize_identifier(part) for part in parts[1:])
    else:
        subdir = root / "unknown"
        filename = sanitize_identifier(identifier)
    return subdir / f"{filename}.{suffix}"


def api_url(identifier: str) -> str:
    return f"{API_PREFIX}/{quote(identifier, safe='/')}"


def polite_sleep(requests_per_second: float) -> None:
    if requests_per_second <= 0:
        return
    time.sleep(1.0 / requests_per_second)


def fetch_document(
    session: requests.Session,
    record: MetadataRecord,
    requests_per_second: float,
    retries: int,
    user_agent: str,
) -> dict[str, Any]:
    last_error = ""
    for attempt in range(retries + 1):
        try:
            response = get_url(session, api_url(record.identifier), user_agent=user_agent)
            data = response.json()
            if not isinstance(data, dict):
                raise FoundersOnlineError("API response was not a JSON object")
            polite_sleep(requests_per_second)
            return data
        except FoundersOnlineWafChallenge:
            raise
        except (requests.RequestException, ValueError, FoundersOnlineError) as exc:
            last_error = str(exc)
            if attempt >= retries:
                break
            time.sleep(min(30, 2 ** attempt))
    raise FoundersOnlineError(last_error)


def write_document(record: MetadataRecord, data: dict[str, Any]) -> tuple[Path, Path]:
    raw_path = local_doc_path(RAW_DIR, record.identifier, "json")
    clean_path = local_doc_path(CLEAN_DIR, record.identifier, "txt")
    ensure_directory(raw_path.parent)
    ensure_directory(clean_path.parent)

    payload = {
        "identifier": record.identifier,
        "metadata": record.__dict__,
        "api": data,
        "fetched_at": datetime.now().isoformat(),
    }
    raw_path.write_text(json.dumps(payload, indent=2, ensure_ascii=False), encoding="utf-8")

    content = data.get("content") or ""
    if not isinstance(content, str):
        content = str(content)
    clean_path.write_text(normalize_whitespace(content), encoding="utf-8")
    return raw_path, clean_path


def fetch_all_documents(args: argparse.Namespace, records: list[MetadataRecord]) -> dict[str, Any]:
    ensure_directory(RAW_DIR)
    ensure_directory(CLEAN_DIR)
    selected = records[args.offset :]
    if args.limit is not None:
        selected = selected[: args.limit]

    report: dict[str, Any] = {
        "generated_at": datetime.now().isoformat(),
        "metadata_count": len(records),
        "selected_count": len(selected),
        "offset": args.offset,
        "limit": args.limit,
        "requests_per_second": args.requests_per_second,
        "documents": [],
    }
    consecutive_failures = 0

    cookie = read_cookie(args.cookie_file)
    with requests.Session() as session:
        if cookie:
            session.headers.update({"Cookie": cookie})
        for index, record in enumerate(selected, start=1):
            raw_path = local_doc_path(RAW_DIR, record.identifier, "json")
            clean_path = local_doc_path(CLEAN_DIR, record.identifier, "txt")
            if raw_path.exists() and clean_path.exists() and not args.force:
                status = "cached"
                error = ""
            else:
                try:
                    data = fetch_document(
                        session,
                        record,
                        args.requests_per_second,
                        args.retries,
                        args.user_agent,
                    )
                    raw_path, clean_path = write_document(record, data)
                    status = "downloaded"
                    error = ""
                    consecutive_failures = 0
                except FoundersOnlineWafChallenge as exc:
                    status = "failed"
                    error = str(exc)
                    consecutive_failures += 1
                    print(f"{index}/{len(selected)} {record.identifier}: failed - WAF challenge")
                    if args.stop_on_waf:
                        report["documents"].append(
                            {
                                "identifier": record.identifier,
                                "title": record.title,
                                "status": status,
                                "raw_path": "",
                                "clean_path": "",
                                "error": error,
                            }
                        )
                        break
                except FoundersOnlineError as exc:
                    status = "failed"
                    error = str(exc)
                    consecutive_failures += 1

            report["documents"].append(
                {
                    "identifier": record.identifier,
                    "title": record.title,
                    "status": status,
                    "raw_path": str(raw_path.relative_to(PROJECT_ROOT)) if raw_path.exists() else "",
                    "clean_path": str(clean_path.relative_to(PROJECT_ROOT)) if clean_path.exists() else "",
                    "error": error,
                }
            )
            if index == 1 or index % args.progress_every == 0 or index == len(selected):
                print(f"{index}/{len(selected)} {record.identifier}: {status}")
            if status == "failed" and args.stop_on_error:
                break
            if status == "failed" and consecutive_failures >= args.max_consecutive_failures:
                print(
                    f"Stopping after {consecutive_failures} consecutive failures. "
                    "Refresh credentials or inspect the latest report, then resume."
                )
                break

    return report


def iter_clean_documents(records: Iterable[MetadataRecord]) -> Iterable[tuple[MetadataRecord, Path]]:
    for record in records:
        clean_path = local_doc_path(CLEAN_DIR, record.identifier, "txt")
        if clean_path.exists():
            yield record, clean_path


def build_chunks(records: list[MetadataRecord], limit: int | None = None) -> dict[str, Any]:
    ensure_directory(CHUNKS_DIR)
    chunks: list[dict[str, Any]] = []
    total_documents = 0
    selected = list(iter_clean_documents(records))
    if limit is not None:
        selected = selected[:limit]

    for record, clean_path in selected:
        text = clean_path.read_text(encoding="utf-8")
        if not text.strip():
            continue
        total_documents += 1
        author = ", ".join(record.authors) if record.authors else "Unknown"
        date = record.date_from or record.date_to
        doc_id = "founders_online_" + sanitize_identifier(record.identifier)
        for index, chunk_text in enumerate(paragraph_chunk(text)):
            if word_count(chunk_text) < 20:
                continue
            chunks.append(
                {
                    "chunk_id": f"{doc_id}_{index:04d}",
                    "document_id": doc_id,
                    "founders_online_id": record.identifier,
                    "title": record.title,
                    "author": author,
                    "recipients": record.recipients,
                    "date": date,
                    "source_collection": "founders_online",
                    "source_url": record.permalink,
                    "document_type": "founders_online_document",
                    "project": record.project,
                    "issue_tags": [],
                    "constitutional_clause_tags": [],
                    "text": chunk_text,
                    "word_count": word_count(chunk_text),
                    "preview": preview_text(chunk_text),
                }
            )

    corpus = {
        "metadata": {
            "version": "1.0",
            "generated_at": datetime.now().isoformat(),
            "total_chunks": len(chunks),
            "total_documents": total_documents,
            "source": "Founders Online API",
            "license_note": "Check Founders Online/NARA metadata terms before redistribution.",
        },
        "chunks": chunks,
    }
    save_json(CHUNKS_DIR / "founders_online_corpus.json", corpus)
    return corpus


def write_inventory(records: list[MetadataRecord]) -> Path:
    ensure_directory(METADATA_DIR)
    inventory_path = METADATA_DIR / "inventory.jsonl"
    with inventory_path.open("w", encoding="utf-8") as handle:
        for record in records:
            handle.write(json.dumps(record.__dict__, ensure_ascii=False) + "\n")
    return inventory_path


def clamp_rate(value: str) -> float:
    try:
        rate = float(value)
    except ValueError as exc:
        raise argparse.ArgumentTypeError("requests per second must be a number") from exc
    if rate <= 0:
        raise argparse.ArgumentTypeError("requests per second must be greater than 0")
    if rate > 10:
        raise argparse.ArgumentTypeError("Founders Online asks automated clients to stay at or below 10 requests/second")
    return rate


def read_cookie(path: Path | None) -> str:
    if not path:
        return ""
    cookie = path.read_text(encoding="utf-8").strip()
    if cookie.startswith("Cookie:"):
        cookie = cookie.split(":", 1)[1].strip()
    return cookie


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Download the full Founders Online metadata and document-content corpus."
    )
    parser.add_argument(
        "command",
        choices=("metadata", "fetch", "chunk", "all"),
        help="metadata downloads/normalizes metadata; fetch gets document JSON/text; chunk builds a chunk corpus; all runs metadata+fetch+chunk",
    )
    parser.add_argument("--metadata-file", type=Path, help="Use an existing founders-online-metadata.json file")
    parser.add_argument("--primary-only", action=argparse.BooleanOptionalAction, default=True, help="Skip editorial records without date-from")
    parser.add_argument("--force", action="store_true", help="Re-download metadata/documents even when cached")
    parser.add_argument("--limit", type=int, help="Only process the first N selected documents")
    parser.add_argument("--offset", type=int, default=0, help="Skip N metadata records before fetching")
    parser.add_argument("--requests-per-second", type=clamp_rate, default=2.0)
    parser.add_argument("--retries", type=int, default=3)
    parser.add_argument("--progress-every", type=int, default=100)
    parser.add_argument("--stop-on-error", action="store_true")
    parser.add_argument("--stop-on-waf", action=argparse.BooleanOptionalAction, default=True)
    parser.add_argument("--max-consecutive-failures", type=int, default=3)
    parser.add_argument("--cookie-file", type=Path, help="Path to a local Cookie header value, for browser WAF sessions")
    parser.add_argument("--user-agent", default=USER_AGENT, help="User-Agent header for Founders Online requests")
    return parser.parse_args()


def resolve_metadata_path(args: argparse.Namespace) -> Path:
    if args.metadata_file:
        return args.metadata_file
    return download_metadata(
        force=args.force,
        cookie=read_cookie(args.cookie_file),
        user_agent=args.user_agent,
    )


def main() -> None:
    args = parse_args()
    ensure_directory(REPORTS_DIR)

    if args.command in {"metadata", "fetch", "chunk", "all"}:
        metadata_path = resolve_metadata_path(args)
        records = load_metadata_records(metadata_path, primary_only=args.primary_only)
        inventory_path = write_inventory(records)
        print(f"Loaded {len(records)} metadata records")
        print(f"Wrote {inventory_path.relative_to(PROJECT_ROOT)}")

    if args.command in {"fetch", "all"}:
        report = fetch_all_documents(args, records)
        report_path = REPORTS_DIR / f"fetch_report_{datetime.now().strftime('%Y%m%d_%H%M%S')}.json"
        save_json(report_path, report)
        print(f"Wrote {report_path.relative_to(PROJECT_ROOT)}")

    if args.command in {"chunk", "all"}:
        corpus = build_chunks(records, limit=args.limit)
        print(
            "Wrote "
            f"{(CHUNKS_DIR / 'founders_online_corpus.json').relative_to(PROJECT_ROOT)} "
            f"with {corpus['metadata']['total_chunks']} chunks"
        )


if __name__ == "__main__":
    try:
        main()
    except FoundersOnlineError as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        raise SystemExit(1)
