#!/usr/bin/env python3
"""
Enrich chunks with constitutional clause and issue tags.

Performs keyword matching and confidence scoring for metadata enrichment.
"""

import json
import re
from pathlib import Path

PROJECT_ROOT = Path(__file__).parent.parent.parent
CONFIG_DIR = PROJECT_ROOT / "config"
DATA_CHUNKS = PROJECT_ROOT / "data" / "chunks"

class MetadataTagger:
    """Tag chunks with constitutional clauses and issue themes."""

    def __init__(self):
        self.clauses = self._load_clauses()
        self.issues = self._load_issues()

    def _load_clauses(self):
        """Load constitutional clauses configuration."""
        path = CONFIG_DIR / "constitutional_clauses.json"
        with open(path) as f:
            data = json.load(f)
        return {c['clause_id']: c for c in data['constitutional_clauses']}

    def _load_issues(self):
        """Load issue tags configuration."""
        path = CONFIG_DIR / "constitutional_clauses.json"
        with open(path) as f:
            data = json.load(f)
        return {t['tag_id']: t for t in data['issue_tags']}

    def tag_chunk(self, chunk):
        """Enrich chunk with tags."""
        text = chunk['text'].lower()

        # Find matching clauses
        clause_tags = []
        for clause_id, clause in self.clauses.items():
            if self._matches_clause(text, clause):
                clause_tags.append(clause_id)

        # Find matching issues
        issue_tags = []
        for issue_id, issue in self.issues.items():
            if self._matches_issue(text, issue):
                issue_tags.append(issue_id)

        chunk['constitutional_clause_tags'] = clause_tags
        chunk['issue_tags'] = issue_tags

        return chunk

    def _matches_clause(self, text, clause):
        """Check if text matches clause keywords."""
        keywords = clause.get('keywords', [])
        # Check if any keyword appears in text
        for keyword in keywords:
            if keyword.lower() in text:
                return True
        return False

    def _matches_issue(self, text, issue):
        """Check if text matches issue keywords."""
        keywords = issue.get('keywords', [])
        matches = 0
        for keyword in keywords:
            if keyword.lower() in text:
                matches += 1
        # Require at least one keyword match
        return matches > 0

def tag_corpus(corpus_path):
    """Tag all chunks in corpus."""
    with open(corpus_path) as f:
        corpus = json.load(f)

    tagger = MetadataTagger()

    for chunk in corpus['chunks']:
        tagger.tag_chunk(chunk)

    # Save tagged corpus
    with open(corpus_path, 'w', encoding='utf-8') as f:
        json.dump(corpus, f, indent=2, ensure_ascii=False)

    print(f"Tagged {len(corpus['chunks'])} chunks")

if __name__ == "__main__":
    corpus_path = DATA_CHUNKS / "constitution_full_corpus.json"
    tag_corpus(corpus_path)
