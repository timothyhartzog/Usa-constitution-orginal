from __future__ import annotations

import json
from pathlib import Path

from .pipeline import unique_preserve


PROJECT_ROOT = Path(__file__).resolve().parents[2]
CLAUSES_PATH = PROJECT_ROOT / "config" / "constitutional_clauses.json"


class MetadataTagger:
    """Attach issue and constitutional clause tags from the project taxonomy."""

    def __init__(self, taxonomy_path: Path = CLAUSES_PATH) -> None:
        with taxonomy_path.open(encoding="utf-8") as handle:
            taxonomy = json.load(handle)
        self.clauses = taxonomy["constitutional_clauses"]
        self.issues = taxonomy["issue_tags"]

    @staticmethod
    def _matches_keywords(text: str, keywords: list[str], minimum_matches: int = 1) -> bool:
        matches = 0
        for keyword in keywords:
            if keyword.lower() in text:
                matches += 1
            if matches >= minimum_matches:
                return True
        return False

    def tag_text(
        self,
        text: str,
        seed_issue_tags: list[str] | None = None,
        seed_clause_tags: list[str] | None = None,
    ) -> tuple[list[str], list[str]]:
        lowered = text.lower()
        issue_tags = list(seed_issue_tags or [])
        clause_tags = list(seed_clause_tags or [])

        for issue in self.issues:
            if self._matches_keywords(lowered, issue.get("keywords", [])):
                issue_tags.append(issue["tag_id"])

        for clause in self.clauses:
            minimum = 1
            if clause["clause_id"] in {"I.8", "II.1", "III.1"}:
                minimum = 2
            if self._matches_keywords(lowered, clause.get("keywords", []), minimum):
                clause_tags.append(clause["clause_id"])

        return unique_preserve(issue_tags), unique_preserve(clause_tags)
