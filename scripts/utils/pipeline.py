from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any, Iterable


PROJECT_ROOT = Path(__file__).resolve().parents[2]
DATA_DIR = PROJECT_ROOT / "data"
RAW_DIR = DATA_DIR / "raw"
CLEAN_DIR = DATA_DIR / "clean"
CHUNKS_DIR = DATA_DIR / "chunks"
INDEX_DIR = DATA_DIR / "index"
MANIFEST_PATH = PROJECT_ROOT / "config" / "sources_manifest.json"

STOP_WORDS = {
    "a",
    "an",
    "and",
    "are",
    "as",
    "at",
    "be",
    "been",
    "but",
    "by",
    "for",
    "from",
    "had",
    "has",
    "have",
    "he",
    "her",
    "his",
    "if",
    "in",
    "into",
    "is",
    "it",
    "its",
    "of",
    "on",
    "or",
    "that",
    "the",
    "their",
    "them",
    "there",
    "they",
    "this",
    "to",
    "was",
    "were",
    "which",
    "who",
    "with",
}


def ensure_directory(path: Path) -> Path:
    path.mkdir(parents=True, exist_ok=True)
    return path


def load_json(path: Path) -> Any:
    with path.open(encoding="utf-8") as handle:
        return json.load(handle)


def save_json(path: Path, payload: Any) -> None:
    ensure_directory(path.parent)
    with path.open("w", encoding="utf-8") as handle:
        json.dump(payload, handle, indent=2, ensure_ascii=False)


def load_manifest() -> dict[str, Any]:
    return load_json(MANIFEST_PATH)


def iter_documents(manifest: dict[str, Any]) -> Iterable[tuple[dict[str, Any], dict[str, Any]]]:
    for collection in manifest["collections"]:
        for document in collection["documents"]:
            yield collection, document


def unique_preserve(items: Iterable[str]) -> list[str]:
    seen: set[str] = set()
    ordered: list[str] = []
    for item in items:
        if item and item not in seen:
            seen.add(item)
            ordered.append(item)
    return ordered


def normalize_whitespace(text: str) -> str:
    text = text.replace("\r\n", "\n").replace("\r", "\n").replace("\ufeff", "")
    text = re.sub(r"[ \t]+", " ", text)
    text = re.sub(r"\n{3,}", "\n\n", text)
    return text.strip()


def sanitize_identifier(value: str) -> str:
    return re.sub(r"[^a-z0-9]+", "_", value.lower()).strip("_")


def preview_text(text: str, words: int = 40) -> str:
    word_list = text.split()
    snippet = " ".join(word_list[:words])
    if len(word_list) > words:
        return f"{snippet} ..."
    return snippet


def tokenize(text: str) -> list[str]:
    return re.findall(r"[a-z0-9']+", text.lower())
