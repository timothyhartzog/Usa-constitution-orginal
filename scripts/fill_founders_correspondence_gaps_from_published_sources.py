#!/usr/bin/env python3
from __future__ import annotations

import html
import re
import shutil
import zipfile
from pathlib import Path


PROJECT_ROOT = Path(__file__).resolve().parent.parent
CLEAN_DIR = PROJECT_ROOT / "data" / "clean"
RAW_DIR = PROJECT_ROOT / "data" / "raw" / "founders_correspondence"


TARGETS = {
    "hamilton_correspondence_1787_1788": {
        "title": "Alexander Hamilton constitutional-period writings and correspondence, 1787-1788",
        "sources": [
            "The Works of Alexander Hamilton, Volume 2 (New York ratification convention, 1788)",
            "The Works of Alexander Hamilton, Volume 8 (New York Assembly speeches, 1787)",
            "The Works of Alexander Hamilton, Volume 11 (The Federalist, 1787-1788)",
        ],
    },
    "washington_correspondence_1787_1789": {
        "title": "George Washington selected correspondence, 1787-1789",
        "sources": ["The Writings of George Washington, Volume 11, 1785-1790"],
    },
    "adams_correspondence_1787_1789": {
        "title": "John Adams selected correspondence, 1787-1789",
        "sources": ["The Works of John Adams, Volume 8 (Letters and State Papers 1782-1799), OLL ePub"],
    },
    "jay_correspondence_1787_1789": {
        "title": "John Jay selected correspondence, 1787-1789",
        "sources": ["The Correspondence and Public Papers of John Jay, Volume 3 (1782-1793), OLL ePub"],
    },
}


def words(text: str) -> int:
    return len(text.split())


def normalize_text(text: str) -> str:
    text = text.replace("\r\n", "\n").replace("\r", "\n")
    text = re.sub(r"[ \t]+", " ", text)
    text = re.sub(r"\n{3,}", "\n\n", text)
    return text.strip() + "\n"


def strip_html(fragment: str) -> str:
    fragment = re.sub(r"<\s*br\s*/?\s*>", "\n", fragment, flags=re.I)
    fragment = re.sub(r"</\s*(p|div|h[1-6]|li|tr)\s*>", "\n", fragment, flags=re.I)
    fragment = re.sub(r"<[^>]+>", "", fragment)
    fragment = html.unescape(fragment)
    fragment = fragment.replace("\u2014", "--").replace("\u2013", "-")
    fragment = fragment.replace("\u2018", "'").replace("\u2019", "'")
    fragment = fragment.replace("\u201c", '"').replace("\u201d", '"')
    return normalize_text(fragment)


def read_epub_member(epub_path: Path, member_name: str) -> str:
    with zipfile.ZipFile(epub_path) as archive:
        return archive.read(member_name).decode("utf-8", errors="replace")


def extract_oll_sections(epub_path: Path, member_name: str, start_year: int, end_year: int) -> list[str]:
    raw_html = read_epub_member(epub_path, member_name)
    sections = re.split(r"(?=<h2\b)", raw_html, flags=re.I)
    selected: list[str] = []
    for section in sections:
        if "<h2" not in section.lower():
            continue
        date_match = re.search(r'class="date\s+((?:17|18)\d{2})(?:-[^"]*)?"', section)
        if not date_match:
            continue
        year = int(date_match.group(1))
        if start_year <= year <= end_year:
            text = strip_html(section)
            if words(text) >= 20:
                selected.append(text)
    return selected


def extract_between_lines(path: Path, start_pattern: str, end_pattern: str, backtrack_header: bool = True) -> str:
    lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    start = next(i for i, line in enumerate(lines) if re.search(start_pattern, line))
    if backtrack_header:
        for i in range(start, -1, -1):
            if re.match(r"^(TO|FROM|[A-Z][A-Z .,'-]+ TO)\b", lines[i].strip()):
                start = i
                break
    end = len(lines)
    for i in range(start + 1, len(lines)):
        if re.search(end_pattern, lines[i]):
            end = i
            break
    return normalize_text("\n".join(lines[start:end]))


def header(target_id: str) -> str:
    target = TARGETS[target_id]
    source_lines = "\n".join(f"Source: {source}" for source in target["sources"])
    return normalize_text(
        f"{target['title']}\n"
        f"Corpus gap file: {target_id}\n"
        f"{source_lines}\n"
        "Note: This file fills a Founders Online API gap from public-domain published editions because the API fetch was blocked by WAF.\n"
    )


def write_target(target_id: str, body_parts: list[str]) -> dict[str, object]:
    raw_target = RAW_DIR / target_id
    raw_target.mkdir(parents=True, exist_ok=True)
    text = normalize_text(header(target_id) + "\n\n" + "\n\n".join(part.strip() for part in body_parts if part.strip()))
    clean_path = CLEAN_DIR / f"{target_id}.txt"
    clean_path.write_text(text, encoding="utf-8")
    (raw_target / "source_note.txt").write_text(header(target_id), encoding="utf-8")
    return {"document_id": target_id, "words": words(text), "path": str(clean_path.relative_to(PROJECT_ROOT))}


def fill_hamilton() -> dict[str, object]:
    vol2 = CLEAN_DIR / "hamilton_works_vol_2.txt"
    vol8 = CLEAN_DIR / "hamilton_works_vol_8.txt"
    vol11 = CLEAN_DIR / "hamilton_works_vol_11.txt"
    parts = [
        "TO READERS.\nThe following selections come from already-cached public-domain Hamilton volumes and emphasize the constitutional and ratification period.",
        extract_between_lines(vol8, r"SPEECHES IN THE NEW YORK ASSEMBLY, 1787", r"SPEECH ON ACCEDING TO THE INDEPENDENCE OF\s*$", backtrack_header=False),
        extract_between_lines(vol2, r"CONVENTION OF NEW YORK\s*$", r"BETTERS OF H\. G|LETTERS OF H\. G", backtrack_header=False),
        extract_between_lines(vol11, r"THE FEDERALIST\. No\. I\b", r"WILL BE ASSESSED FOR FAILURE", backtrack_header=False),
    ]
    return write_target("hamilton_correspondence_1787_1788", parts)


def fill_washington() -> dict[str, object]:
    vol11 = CLEAN_DIR / "washington_writings_vol_11.txt"
    body = extract_between_lines(vol11, r"MOUNT VERNON,\s*10 January,\s*1787", r"SPEECH TO BOTH HOUSES OF CONGRESS,\s*JANUARY 8TH", backtrack_header=True)
    return write_target("washington_correspondence_1787_1789", [body])


def fill_adams() -> dict[str, object]:
    epub_path = Path("/private/tmp/adams_vol8.epub")
    if not epub_path.exists():
        raise FileNotFoundError(f"Missing {epub_path}; download the OLL Adams vol. 8 ePub first.")
    raw_target = RAW_DIR / "adams_correspondence_1787_1789"
    raw_target.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(epub_path, raw_target / "adams_vol8_oll.epub")
    sections = extract_oll_sections(epub_path, "Adams_1431-08.html", 1787, 1789)
    return write_target("adams_correspondence_1787_1789", sections)


def fill_jay() -> dict[str, object]:
    epub_path = Path("/private/tmp/jay_vol3.epub")
    if not epub_path.exists():
        raise FileNotFoundError(f"Missing {epub_path}; download the OLL Jay vol. 3 ePub first.")
    raw_target = RAW_DIR / "jay_correspondence_1787_1789"
    raw_target.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(epub_path, raw_target / "jay_vol3_oll.epub")
    sections = extract_oll_sections(epub_path, "Jay_1530-03.html", 1787, 1789)
    return write_target("jay_correspondence_1787_1789", sections)


def main() -> None:
    CLEAN_DIR.mkdir(parents=True, exist_ok=True)
    RAW_DIR.mkdir(parents=True, exist_ok=True)
    results = [fill_hamilton(), fill_washington(), fill_adams(), fill_jay()]
    for result in results:
        print(f"{result['document_id']}: {result['words']} words -> {result['path']}")


if __name__ == "__main__":
    main()
