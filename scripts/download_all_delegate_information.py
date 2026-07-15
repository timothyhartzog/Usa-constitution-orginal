#!/usr/bin/env python3
"""Download and bundle available writings for all 55 Convention delegates.

This coordinator turns the delegate acquisition map into a repeatable workflow:

1. Download public-domain Internet Archive text targets listed in
   config/delegate_acquisition_targets.json.
2. Optionally download the Library of Congress "Letters of Delegates to
   Congress" OCR volumes.
3. Optionally fetch Founders Online documents authored by the delegates when
   Founders metadata/API access is available.
4. Build per-delegate bundle indexes and combined text files for downstream
   tools such as text-to-speech readers.

It does not pretend that every delegate has a complete surviving archive. The
report explicitly marks sparse, missing, and non-bulk-downloadable collections.
"""
from __future__ import annotations

import argparse
import csv
import json
import re
import subprocess
import sys
import time
from dataclasses import asdict, dataclass
from datetime import datetime
from pathlib import Path
from typing import Any, Iterable
from urllib.parse import quote

import requests

PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from scripts.fetch_founders_online import (  # noqa: E402
    CLEAN_DIR as FOUNDERS_CLEAN_DIR,
    FoundersOnlineError,
    FoundersOnlineWafChallenge,
    MetadataRecord,
    build_chunks,
    download_metadata,
    fetch_document as fetch_founders_document,
    load_metadata_records,
    local_doc_path as founders_local_doc_path,
    read_cookie,
    write_document as write_founders_document,
)
from scripts.utils.pipeline import (  # noqa: E402
    CLEAN_DIR,
    DATA_DIR,
    PROJECT_ROOT as PIPELINE_PROJECT_ROOT,
    RAW_DIR,
    ensure_directory,
    load_json,
    normalize_whitespace,
    sanitize_identifier,
    save_json,
)

assert PROJECT_ROOT == PIPELINE_PROJECT_ROOT

DELEGATES_PATH = DATA_DIR / "delegates" / "federal_convention_delegates.json"
TARGETS_PATH = PROJECT_ROOT / "config" / "delegate_acquisition_targets.json"
BUNDLE_DIR = DATA_DIR / "delegates" / "all_delegate_information"
REPORT_DIR = DATA_DIR / "delegates" / "reports"
IA_COLLECTION_ID = "delegate_writings"
IA_URL = "https://archive.org/download/{ident}/{ident}_djvu.txt"
IA_METADATA_URL = "https://archive.org/metadata/{ident}"
IA_SEARCH_URL = "https://archive.org/advancedsearch.php"
USER_AGENT = "Constitutional Research System/2.0 (55 delegate local archival build)"
TIMEOUT = 60
IA_TIMEOUT = 15


@dataclass(frozen=True)
class SourceItem:
    source_kind: str
    document_id: str
    title: str
    source_url: str
    raw_path: str
    clean_path: str
    status: str
    word_count: int
    error: str = ""


def slug(value: str) -> str:
    return re.sub(r"[^a-z0-9]+", "_", value.lower()).strip("_")


def load_delegates() -> list[dict[str, Any]]:
    return load_json(DELEGATES_PATH)["delegates"]


def load_targets_by_name() -> dict[str, dict[str, Any]]:
    return {target["name"]: target for target in load_json(TARGETS_PATH)["targets"]}


def ia_document_id(delegate_name: str, ident_count: int, index: int) -> str:
    suffix = f"_vol_{index}" if ident_count > 1 else ""
    return f"{slug(delegate_name)}_writings{suffix}"


def title_search_phrase(delegate_name: str, target: dict[str, Any]) -> str:
    source = str(target.get("pd_source", "")).strip()
    source = re.split(r"\s*\(|;", source, maxsplit=1)[0].strip()
    source = source.replace(" / ", " ")
    source = re.sub(r"\bPD\b", "", source, flags=re.I).strip(" ,")
    if len(source.split()) >= 4:
        return source
    return delegate_name


def archive_djvu_url_from_metadata(metadata: dict[str, Any]) -> str:
    ident = str(metadata.get("metadata", {}).get("identifier") or metadata.get("item") or "")
    for file_info in metadata.get("files", []):
        name = str(file_info.get("name", ""))
        if ident and name.endswith("_djvu.txt"):
            return f"https://archive.org/download/{ident}/{quote(name)}"
    return ""


def archive_metadata(session: requests.Session, ident: str) -> dict[str, Any]:
    response = session.get(
        IA_METADATA_URL.format(ident=quote(ident)),
        headers={"User-Agent": USER_AGENT},
        timeout=IA_TIMEOUT,
    )
    response.raise_for_status()
    data = response.json()
    return data if isinstance(data, dict) else {}


def archive_search_candidates(
    session: requests.Session,
    delegate_name: str,
    target: dict[str, Any],
    tried: set[str],
) -> Iterable[tuple[str, str]]:
    phrase = title_search_phrase(delegate_name, target)
    params = {
        "q": f'title:("{phrase}") AND mediatype:texts',
        "fl[]": ["identifier", "title"],
        "rows": "25",
        "output": "json",
    }
    response = session.get(IA_SEARCH_URL, params=params, headers={"User-Agent": USER_AGENT}, timeout=IA_TIMEOUT)
    response.raise_for_status()
    docs = response.json().get("response", {}).get("docs", [])
    delegate_tokens = [token for token in re.findall(r"[a-z]+", delegate_name.lower()) if len(token) > 2]
    for doc in docs:
        ident = str(doc.get("identifier", ""))
        title = str(doc.get("title", ""))
        haystack = f"{ident} {title}".lower()
        if not ident or ident in tried:
            continue
        if delegate_tokens and not any(token in haystack for token in delegate_tokens):
            continue
        yield ident, title


def resolve_archive_text_url(
    session: requests.Session,
    ident: str,
    delegate_name: str,
    target: dict[str, Any],
    tried: set[str],
) -> tuple[str, str]:
    tried.add(ident)
    try:
        url = archive_djvu_url_from_metadata(archive_metadata(session, ident))
        if url:
            return ident, url
    except (requests.RequestException, ValueError):
        pass

    try:
        candidates = list(archive_search_candidates(session, delegate_name, target, tried))
    except (requests.RequestException, ValueError):
        candidates = []
    for candidate_ident, _title in candidates:
        tried.add(candidate_ident)
        try:
            url = archive_djvu_url_from_metadata(archive_metadata(session, candidate_ident))
            if url:
                return candidate_ident, url
        except (requests.RequestException, ValueError):
            continue
    return ident, IA_URL.format(ident=ident)


def write_raw_and_clean(
    *,
    collection_id: str,
    document_id: str,
    title: str,
    source_url: str,
    text: str,
    metadata: dict[str, Any],
) -> SourceItem:
    raw_dir = ensure_directory(RAW_DIR / collection_id / document_id)
    raw_path = raw_dir / "source.txt"
    metadata_path = raw_dir / "metadata.json"
    clean_path = CLEAN_DIR / f"{document_id}.txt"
    normalized = normalize_whitespace(text)

    raw_path.write_text(text, encoding="utf-8")
    clean_path.write_text(normalized + "\n", encoding="utf-8")
    save_json(
        metadata_path,
        {
            "document_id": document_id,
            "title": title,
            "source_url": source_url,
            "source_format": "text",
            "downloaded_at": datetime.now().isoformat(),
            **metadata,
        },
    )
    return SourceItem(
        source_kind=collection_id,
        document_id=document_id,
        title=title,
        source_url=source_url,
        raw_path=str(raw_path.relative_to(PROJECT_ROOT)),
        clean_path=str(clean_path.relative_to(PROJECT_ROOT)),
        status="downloaded",
        word_count=len(normalized.split()),
    )


def cached_source_item(
    *,
    collection_id: str,
    document_id: str,
    title: str,
    source_url: str,
) -> SourceItem | None:
    raw_path = RAW_DIR / collection_id / document_id / "source.txt"
    clean_path = CLEAN_DIR / f"{document_id}.txt"
    if not raw_path.exists() or not clean_path.exists():
        return None
    text = clean_path.read_text(encoding="utf-8", errors="ignore")
    return SourceItem(
        source_kind=collection_id,
        document_id=document_id,
        title=title,
        source_url=source_url,
        raw_path=str(raw_path.relative_to(PROJECT_ROOT)),
        clean_path=str(clean_path.relative_to(PROJECT_ROOT)),
        status="cached",
        word_count=len(text.split()),
    )


def download_ia_targets(
    delegates: list[dict[str, Any]],
    targets: dict[str, dict[str, Any]],
    *,
    force: bool,
    delay: float,
) -> dict[str, list[SourceItem]]:
    by_delegate: dict[str, list[SourceItem]] = {delegate["name"]: [] for delegate in delegates}
    headers = {"User-Agent": USER_AGENT}

    with requests.Session() as session:
        for delegate in delegates:
            target = targets.get(delegate["name"], {})
            idents = list(target.get("internet_archive", []))
            for index, ident in enumerate(idents, start=1):
                document_id = ia_document_id(delegate["name"], len(idents), index)
                title = (
                    f"{delegate['name']} - {target.get('pd_source', 'Public-domain writings')}"
                    + (f" (vol. {index})" if len(idents) > 1 else "")
                )
                resolved_ident, source_url = resolve_archive_text_url(
                    session,
                    ident,
                    delegate["name"],
                    target,
                    tried=set(),
                )

                if not force:
                    cached = cached_source_item(
                        collection_id=IA_COLLECTION_ID,
                        document_id=document_id,
                        title=title,
                        source_url=source_url,
                    )
                    if cached:
                        by_delegate[delegate["name"]].append(cached)
                        print(f"{delegate['name']}: {document_id} cached", flush=True)
                        continue

                try:
                    response = session.get(source_url, headers=headers, timeout=IA_TIMEOUT)
                    response.raise_for_status()
                    item = write_raw_and_clean(
                        collection_id=IA_COLLECTION_ID,
                        document_id=document_id,
                        title=title,
                        source_url=source_url,
                        text=response.text,
                        metadata={
                            "delegate": delegate["name"],
                            "internet_archive_id": resolved_ident,
                            "configured_internet_archive_id": ident,
                            "source_note": target.get("pd_source", ""),
                        },
                    )
                    by_delegate[delegate["name"]].append(item)
                    print(f"{delegate['name']}: {document_id} downloaded", flush=True)
                except requests.RequestException as exc:
                    by_delegate[delegate["name"]].append(
                        SourceItem(
                            source_kind=IA_COLLECTION_ID,
                            document_id=document_id,
                            title=title,
                            source_url=source_url,
                            raw_path="",
                            clean_path="",
                            status="failed",
                            word_count=0,
                            error=str(exc),
                        )
                    )
                    print(f"{delegate['name']}: {document_id} failed", flush=True)
                time.sleep(delay)

    return by_delegate


def delegate_author_match(record: MetadataRecord, delegate_name: str) -> bool:
    target = delegate_name.casefold()
    return any(author.casefold() == target for author in record.authors)


def select_founders_records(
    metadata_path: Path,
    delegates: list[dict[str, Any]],
    *,
    primary_only: bool,
    limit_per_delegate: int | None,
) -> dict[str, list[MetadataRecord]]:
    records = load_metadata_records(metadata_path, primary_only=primary_only)
    by_delegate: dict[str, list[MetadataRecord]] = {delegate["name"]: [] for delegate in delegates}
    for delegate in delegates:
        matches = [record for record in records if delegate_author_match(record, delegate["name"])]
        matches.sort(key=lambda record: (record.date_from or record.date_to or "", record.identifier))
        if limit_per_delegate is not None:
            matches = matches[:limit_per_delegate]
        by_delegate[delegate["name"]] = matches
    return by_delegate


def download_founders_targets(
    by_delegate_records: dict[str, list[MetadataRecord]],
    *,
    force: bool,
    delay: float,
    retries: int,
    cookie_file: Path | None,
    stop_on_waf: bool,
) -> dict[str, list[SourceItem]]:
    by_delegate: dict[str, list[SourceItem]] = {name: [] for name in by_delegate_records}
    cookie = read_cookie(cookie_file)
    with requests.Session() as session:
        if cookie:
            session.headers.update({"Cookie": cookie})
        for delegate_name, records in by_delegate_records.items():
            for record in records:
                raw_path = founders_local_doc_path(
                    PROJECT_ROOT / "data" / "founders_online" / "raw",
                    record.identifier,
                    "json",
                )
                clean_path = founders_local_doc_path(FOUNDERS_CLEAN_DIR, record.identifier, "txt")

                if raw_path.exists() and clean_path.exists() and not force:
                    text = clean_path.read_text(encoding="utf-8", errors="ignore")
                    by_delegate[delegate_name].append(
                        SourceItem(
                            source_kind="founders_online",
                            document_id=record.identifier,
                            title=record.title,
                            source_url=record.permalink,
                            raw_path=str(raw_path.relative_to(PROJECT_ROOT)),
                            clean_path=str(clean_path.relative_to(PROJECT_ROOT)),
                            status="cached",
                            word_count=len(text.split()),
                        )
                    )
                    continue

                last_error = ""
                for attempt in range(retries + 1):
                    try:
                        data = fetch_founders_document(
                            session,
                            record,
                            requests_per_second=0,
                            retries=0,
                            user_agent=USER_AGENT,
                        )
                        raw_path, clean_path = write_founders_document(record, data)
                        text = clean_path.read_text(encoding="utf-8", errors="ignore")
                        by_delegate[delegate_name].append(
                            SourceItem(
                                source_kind="founders_online",
                                document_id=record.identifier,
                                title=record.title,
                                source_url=record.permalink,
                                raw_path=str(raw_path.relative_to(PROJECT_ROOT)),
                                clean_path=str(clean_path.relative_to(PROJECT_ROOT)),
                                status="downloaded",
                                word_count=len(text.split()),
                            )
                        )
                        print(f"{delegate_name}: founders {record.identifier} downloaded")
                        last_error = ""
                        break
                    except FoundersOnlineWafChallenge as exc:
                        last_error = str(exc)
                        print(f"{delegate_name}: founders WAF challenge")
                        if stop_on_waf:
                            return by_delegate
                        break
                    except FoundersOnlineError as exc:
                        last_error = str(exc)
                        if attempt < retries:
                            time.sleep(min(30, 2**attempt))

                if last_error:
                    by_delegate[delegate_name].append(
                        SourceItem(
                            source_kind="founders_online",
                            document_id=record.identifier,
                            title=record.title,
                            source_url=record.permalink,
                            raw_path="",
                            clean_path="",
                            status="failed",
                            word_count=0,
                            error=last_error,
                        )
                    )
                time.sleep(delay)

    return by_delegate


def run_loc_download(delay: float, force: bool) -> None:
    cmd = [sys.executable, "scripts/fetch_letters_delegates_loc.py", "--delay", str(delay)]
    if force:
        cmd.append("--force")
    subprocess.run(cmd, cwd=PROJECT_ROOT, check=True)


def existing_loc_source_item() -> SourceItem | None:
    report_path = DATA_DIR / "letters_delegates_congress_acquisition_report.json"
    if not report_path.exists():
        return None
    report = load_json(report_path)
    documents = report.get("documents", [])
    total_words = sum(int(doc.get("word_count") or 0) for doc in documents)
    return SourceItem(
        source_kind="letters_delegates_congress",
        document_id="letters_delegates_congress_all_volumes",
        title="Letters of Delegates to Congress, 1774-1789",
        source_url=str(report.get("source", "https://www.loc.gov/item/76002592/")),
        raw_path="data/raw/letters_delegates_congress",
        clean_path="data/clean/letters_delegates_congress_vol_*.txt",
        status="available",
        word_count=total_words,
    )


def merge_source_maps(*maps: dict[str, list[SourceItem]]) -> dict[str, list[SourceItem]]:
    merged: dict[str, list[SourceItem]] = {}
    for source_map in maps:
        for delegate, items in source_map.items():
            merged.setdefault(delegate, []).extend(items)
    return merged


def read_clean_path(path_string: str) -> str:
    if not path_string:
        return ""
    path = PROJECT_ROOT / path_string
    if not path.exists() or "*" in path_string:
        return ""
    return path.read_text(encoding="utf-8", errors="ignore").strip()


def bundle_delegate(
    delegate: dict[str, Any],
    target: dict[str, Any],
    source_items: list[SourceItem],
    loc_item: SourceItem | None,
) -> dict[str, Any]:
    delegate_slug = slug(delegate["name"])
    delegate_dir = ensure_directory(BUNDLE_DIR / delegate_slug)
    combined_path = delegate_dir / "combined.txt"
    manifest_path = delegate_dir / "manifest.json"

    usable_items = [item for item in source_items if item.status in {"cached", "downloaded"}]
    text_parts = [
        delegate["name"],
        f"State: {delegate['state']}",
        f"Signed Constitution: {delegate['signed_constitution']}",
        f"Surviving papers tier: {target.get('survival', 'unknown')}",
        f"Best public-domain source note: {target.get('pd_source', '')}",
        "",
    ]
    for item in usable_items:
        text = read_clean_path(item.clean_path)
        if not text:
            continue
        text_parts.extend(
            [
                f"\n\n===== {item.title} =====",
                f"Source: {item.source_url}",
                "",
                text,
            ]
        )

    combined_text = "\n".join(text_parts).strip() + "\n"
    combined_path.write_text(combined_text, encoding="utf-8")

    manifest = {
        "generated_at": datetime.now().isoformat(),
        "delegate": delegate,
        "survival": target.get("survival", "unknown"),
        "pd_source": target.get("pd_source", ""),
        "internet_archive_targets": target.get("internet_archive", []),
        "founders_online_search": f"https://founders.archives.gov/?q=Author%3A%22{delegate['name'].replace(' ', '+')}%22&s=1111311111&r=1",
        "shared_loc_collection": asdict(loc_item) if loc_item else None,
        "sources": [asdict(item) for item in source_items],
        "combined_text_path": str(combined_path.relative_to(PROJECT_ROOT)),
        "combined_word_count": len(combined_text.split()),
    }
    save_json(manifest_path, manifest)

    successful = [item for item in source_items if item.status in {"cached", "downloaded"}]
    failed = [item for item in source_items if item.status == "failed"]
    return {
        "name": delegate["name"],
        "state": delegate["state"],
        "survival": target.get("survival", "unknown"),
        "downloaded_or_cached_sources": len(successful),
        "failed_sources": len(failed),
        "combined_word_count": len(combined_text.split()),
        "bundle_manifest": str(manifest_path.relative_to(PROJECT_ROOT)),
        "combined_text": str(combined_path.relative_to(PROJECT_ROOT)),
        "internet_archive_targets": len(target.get("internet_archive", [])),
        "has_loc_collection": bool(loc_item),
        "pd_source": target.get("pd_source", ""),
    }


def write_reports(rows: list[dict[str, Any]], source_maps: dict[str, list[SourceItem]]) -> None:
    ensure_directory(REPORT_DIR)
    csv_path = REPORT_DIR / "all_delegate_information_download_report.csv"
    json_path = REPORT_DIR / "all_delegate_information_download_report.json"
    fieldnames = [
        "name",
        "state",
        "survival",
        "downloaded_or_cached_sources",
        "failed_sources",
        "combined_word_count",
        "bundle_manifest",
        "combined_text",
        "internet_archive_targets",
        "has_loc_collection",
        "pd_source",
    ]
    with csv_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)

    save_json(
        json_path,
        {
            "generated_at": datetime.now().isoformat(),
            "delegate_count": len(rows),
            "summary": {
                "with_downloaded_or_cached_sources": sum(
                    1 for row in rows if int(row["downloaded_or_cached_sources"]) > 0
                ),
                "with_failed_sources": sum(1 for row in rows if int(row["failed_sources"]) > 0),
                "total_combined_words": sum(int(row["combined_word_count"]) for row in rows),
            },
            "delegates": rows,
            "sources_by_delegate": {
                delegate: [asdict(item) for item in items] for delegate, items in source_maps.items()
            },
        },
    )
    print(f"Wrote {csv_path.relative_to(PROJECT_ROOT)}")
    print(f"Wrote {json_path.relative_to(PROJECT_ROOT)}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Download and bundle available public information for all 55 Federal Convention delegates."
    )
    parser.add_argument("--force", action="store_true", help="Re-download cached source text")
    parser.add_argument("--delay", type=float, default=0.5, help="Delay between remote requests")
    parser.add_argument(
        "--skip-internet-archive",
        action="store_true",
        help="Skip Internet Archive delegate-writing downloads",
    )
    parser.add_argument(
        "--include-loc-letters",
        action="store_true",
        help="Download LOC Letters of Delegates to Congress volumes if not already cached",
    )
    parser.add_argument(
        "--include-founders-online",
        action="store_true",
        help="Fetch delegate-authored Founders Online documents using metadata/API access",
    )
    parser.add_argument(
        "--founders-metadata-file",
        type=Path,
        help="Use an existing Founders Online metadata JSON instead of downloading metadata",
    )
    parser.add_argument(
        "--founders-limit-per-delegate",
        type=int,
        help="Limit Founders Online fetches per delegate for staged runs",
    )
    parser.add_argument("--founders-cookie-file", type=Path, help="Cookie header file for Founders Online WAF sessions")
    parser.add_argument("--founders-retries", type=int, default=2)
    parser.add_argument("--no-primary-only", action="store_true", help="Include editorial/no-date Founders records")
    parser.add_argument(
        "--chunk-founders",
        action="store_true",
        help="After Founders downloads, rebuild data/founders_online/chunks/founders_online_corpus.json",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    delegates = load_delegates()
    targets = load_targets_by_name()
    ensure_directory(BUNDLE_DIR)
    ensure_directory(CLEAN_DIR)

    source_maps: list[dict[str, list[SourceItem]]] = []

    if not args.skip_internet_archive:
        source_maps.append(
            download_ia_targets(
                delegates,
                targets,
                force=args.force,
                delay=args.delay,
            )
        )

    if args.include_loc_letters:
        run_loc_download(delay=args.delay, force=args.force)
    loc_item = existing_loc_source_item()

    if args.include_founders_online:
        metadata_path = args.founders_metadata_file or download_metadata(
            force=args.force,
            cookie=read_cookie(args.founders_cookie_file),
            user_agent=USER_AGENT,
        )
        founders_records = select_founders_records(
            metadata_path,
            delegates,
            primary_only=not args.no_primary_only,
            limit_per_delegate=args.founders_limit_per_delegate,
        )
        source_maps.append(
            download_founders_targets(
                founders_records,
                force=args.force,
                delay=args.delay,
                retries=args.founders_retries,
                cookie_file=args.founders_cookie_file,
                stop_on_waf=True,
            )
        )
        if args.chunk_founders:
            all_records = [record for records in founders_records.values() for record in records]
            build_chunks(all_records)

    merged = merge_source_maps(*source_maps)
    rows = []
    for delegate in delegates:
        rows.append(
            bundle_delegate(
                delegate,
                targets.get(delegate["name"], {}),
                merged.get(delegate["name"], []),
                loc_item,
            )
        )
    write_reports(rows, merged)


if __name__ == "__main__":
    main()
