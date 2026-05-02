#!/usr/bin/env python3
"""
Build client-side search index from corpus.

Creates inverted index for full-text search without requiring a backend.
"""

import json
import re
from pathlib import Path
from collections import defaultdict
from datetime import datetime

PROJECT_ROOT = Path(__file__).parent.parent
DATA_CHUNKS = PROJECT_ROOT / "data" / "chunks"
DATA_INDEX = PROJECT_ROOT / "data" / "index"

# Stop words to exclude from index
STOP_WORDS = {
    'a', 'an', 'and', 'are', 'as', 'at', 'be', 'but', 'by', 'for', 'from',
    'had', 'has', 'he', 'her', 'here', 'hers', 'him', 'his', 'how', 'i',
    'if', 'in', 'into', 'is', 'it', 'its', 'me', 'my', 'myself', 'of', 'on',
    'or', 'other', 'our', 'ours', 'ourselves', 'out', 'over', 'own', 'she',
    'so', 'some', 'such', 'than', 'that', 'the', 'their', 'theirs', 'them',
    'themselves', 'then', 'there', 'these', 'they', 'this', 'those', 'to',
    'too', 'under', 'up', 'very', 'was', 'we', 'were', 'what', 'when',
    'which', 'while', 'who', 'whom', 'why', 'will', 'with', 'you', 'your',
    'yours', 'yourself', 'yourselves'
}

class SearchIndexBuilder:
    """Build search index for client-side search."""

    def __init__(self):
        self.corpus = self._load_corpus()
        self.index = {
            "words": defaultdict(list),
            "clauses": defaultdict(list),
            "collections": set(),
            "authors": set(),
            "document_types": set(),
        }

    def _load_corpus(self):
        """Load the chunked corpus."""
        path = DATA_CHUNKS / "constitution_full_corpus.json"
        with open(path) as f:
            return json.load(f)

    def _tokenize(self, text):
        """Tokenize text into words."""
        # Convert to lowercase and split on non-alphanumeric
        words = re.findall(r'\b[a-z]+\b', text.lower())
        return words

    def _build_word_index(self):
        """Build inverted index of words to chunks."""
        print("Building word index...")
        for chunk in self.corpus['chunks']:
            chunk_id = chunk['chunk_id']
            text = chunk['text']

            words = self._tokenize(text)

            for word in set(words):  # Use set to avoid duplicates
                if word not in STOP_WORDS and len(word) > 2:
                    self.index['words'][word].append(chunk_id)

    def _build_clause_index(self):
        """Build index of clauses to chunks."""
        print("Building clause index...")
        for chunk in self.corpus['chunks']:
            chunk_id = chunk['chunk_id']
            tags = chunk.get('constitutional_clause_tags', [])

            for tag in tags:
                self.index['clauses'][tag].append(chunk_id)

    def _build_metadata_index(self):
        """Build indexes for filtering."""
        print("Building metadata index...")
        for chunk in self.corpus['chunks']:
            self.index['collections'].add(chunk.get('collection'))
            self.index['authors'].add(chunk.get('author'))
            self.index['document_types'].add(chunk.get('document_type'))

    def build(self):
        """Build all indexes."""
        print(f"\n{'='*70}")
        print("Building Search Index")
        print(f"{'='*70}\n")

        self._build_word_index()
        self._build_clause_index()
        self._build_metadata_index()

        print(f"Unique words indexed: {len(self.index['words'])}")
        print(f"Constitutional clauses: {len(self.index['clauses'])}")
        print(f"Collections: {len(self.index['collections'])}")

        self._save_index()

    def _save_index(self):
        """Save index to JSON."""
        DATA_INDEX.mkdir(parents=True, exist_ok=True)

        # Convert defaultdicts and sets to regular dicts/lists for JSON
        index_to_save = {
            "metadata": {
                "version": "1.0",
                "generated_timestamp": datetime.now().isoformat(),
                "total_chunks": len(self.corpus['chunks']),
                "unique_words": len(self.index['words']),
                "total_clauses": len(self.index['clauses'])
            },
            "words": dict(self.index['words']),
            "clauses": dict(self.index['clauses']),
            "filters": {
                "collections": sorted(list(self.index['collections'])),
                "authors": sorted(list(self.index['authors'])),
                "document_types": sorted(list(self.index['document_types']))
            }
        }

        # Save main index
        index_path = DATA_INDEX / "search_index.json"
        with open(index_path, 'w', encoding='utf-8') as f:
            json.dump(index_to_save, f, indent=2, ensure_ascii=False)

        print(f"\nIndex saved to: {index_path}")

def main():
    """Main entry point."""
    builder = SearchIndexBuilder()
    builder.build()

if __name__ == "__main__":
    main()
