#!/usr/bin/env python3
"""
Semantically chunk documents with metadata.

Creates chunks respecting document structure (articles, essays, sections).
Generates unique chunk IDs and metadata.
"""

import json
import re
from pathlib import Path
from datetime import datetime

# Project directories
PROJECT_ROOT = Path(__file__).parent.parent
DATA_CLEAN = PROJECT_ROOT / "data" / "clean"
DATA_CHUNKS = PROJECT_ROOT / "data" / "chunks"
MANIFEST_PATH = PROJECT_ROOT / "config" / "sources_manifest.json"

# Chunking configuration
CHUNK_TARGET_SIZE = 400  # Target words per chunk
CHUNK_MIN_SIZE = 50  # Minimum words
CHUNK_MAX_SIZE = 1000  # Maximum words
CHUNK_OVERLAP = 50  # Sliding window overlap for continuous prose

class DocumentChunker:
    """Chunk documents semantically into searchable units."""

    def __init__(self):
        self.manifest = self._load_manifest()
        self.all_chunks = []
        self.chunk_report = {
            "timestamp": datetime.now().isoformat(),
            "total_documents": 0,
            "total_chunks": 0,
            "documents": []
        }

    def _load_manifest(self):
        """Load sources manifest."""
        with open(MANIFEST_PATH, 'r') as f:
            return json.load(f)

    def _read_cleaned_file(self, doc_id):
        """Read cleaned text file."""
        filepath = DATA_CLEAN / f"{doc_id}.txt"
        if not filepath.exists():
            return None
        try:
            with open(filepath, 'r', encoding='utf-8') as f:
                return f.read()
        except Exception as e:
            print(f"    Error reading file: {e}")
            return None

    def _split_by_articles(self, text):
        """Split Constitution and Bill of Rights by Article/Amendment."""
        sections = []

        # Pattern: "Article X" or "Amendment X"
        pattern = r'((?:Article|Amendment)\s+[IVX]+|ARTICLE|AMENDMENT)'

        # Find all matches
        parts = re.split(f'({pattern}.*)', text, flags=re.IGNORECASE)

        for i in range(1, len(parts), 2):
            if i + 1 < len(parts):
                header = parts[i].strip()
                content = parts[i + 1].strip()
                if content:
                    sections.append((header, content))

        return sections if sections else [(None, text)]

    def _split_by_essays(self, text):
        """Split Federalist/Anti-Federalist by essay numbers."""
        sections = []

        # Pattern: "No. X" or "Essay X" or "No. X:"
        pattern = r'(?:No\.|Essay|NUMBER)\s*\d+[:\s]'

        parts = re.split(f'({pattern})', text, flags=re.IGNORECASE)

        for i in range(1, len(parts), 2):
            if i + 1 < len(parts):
                header = parts[i].strip()
                content = parts[i + 1].strip()
                if content:
                    sections.append((header, content))

        return sections if sections else [(None, text)]

    def _split_by_date(self, text):
        """Split notes/correspondence by date entries."""
        sections = []

        # Pattern: Date entries like "May 25, 1787" or similar
        pattern = r'[A-Z][a-z]+\s+\d{1,2},\s+1787'

        parts = re.split(f'({pattern})', text)

        for i in range(1, len(parts), 2):
            if i + 1 < len(parts):
                header = parts[i].strip()
                content = parts[i + 1].strip()
                if content:
                    sections.append((header, content))

        return sections if sections else [(None, text)]

    def _create_sliding_chunks(self, text, target_size=CHUNK_TARGET_SIZE):
        """Create chunks with sliding window for continuous prose."""
        words = text.split()

        if len(words) <= target_size:
            return [' '.join(words)]

        chunks = []
        start = 0

        while start < len(words):
            end = min(start + target_size, len(words))
            chunk_words = words[start:end]
            chunks.append(' '.join(chunk_words))

            # Slide window by (target_size - overlap)
            start += target_size - CHUNK_OVERLAP

        return chunks

    def _chunk_document(self, collection, doc_spec, source_url):
        """Chunk a single document."""
        doc_id = doc_spec['id']
        title = doc_spec['title']
        author = doc_spec.get('author', 'Unknown')
        date = doc_spec.get('date', 'Unknown')
        document_type = doc_spec.get('document_type', 'document')

        print(f"\n  Chunking: {title}")

        # Read cleaned text
        text = self._read_cleaned_file(doc_id)
        if text is None:
            print(f"    Failed to read cleaned text")
            return False, 0

        # Choose chunking strategy based on document type
        if 'constitution' in collection.lower() or 'bill' in title.lower():
            sections = self._split_by_articles(text)
        elif 'federalist' in collection.lower() or 'anti_federalist' in collection.lower():
            sections = self._split_by_essays(text)
        elif 'madison' in collection.lower() or 'farrand' in collection.lower():
            sections = self._split_by_date(text)
        else:
            sections = [(None, text)]

        # Create chunks from sections
        chunk_count = 0

        for section_idx, (section_header, section_text) in enumerate(sections):
            section_text = section_text.strip()

            if not section_text:
                continue

            # Create chunks from section
            section_chunks = self._create_sliding_chunks(
                section_text,
                target_size=CHUNK_TARGET_SIZE
            )

            for chunk_idx, chunk_text in enumerate(section_chunks):
                chunk_text = chunk_text.strip()

                # Validate chunk size
                word_count = len(chunk_text.split())
                if word_count < CHUNK_MIN_SIZE:
                    continue

                # Create chunk object
                chunk_id = f"{collection}_{doc_id}_{chunk_count:04d}"

                chunk = {
                    "chunk_id": chunk_id,
                    "document_id": doc_id,
                    "collection": collection,
                    "title": title,
                    "author": author,
                    "date": date,
                    "document_type": document_type,
                    "source_collection": collection,
                    "source_url": source_url,
                    "text": chunk_text,
                    "word_count": word_count,
                    "chunk_index": chunk_count,
                    "section_number": section_idx,
                    "chunk_size_category": self._categorize_chunk_size(word_count),
                    "position_in_document": {
                        "section_number": section_idx,
                        "chunk_in_section": chunk_idx
                    },
                    "metadata": {
                        "cleaned": True,
                        "manually_reviewed": False,
                        "confidence_score": 0.85,
                        "created_timestamp": datetime.now().isoformat()
                    }
                }

                self.all_chunks.append(chunk)
                chunk_count += 1

        print(f"    Created {chunk_count} chunks")

        return True, chunk_count

    def _categorize_chunk_size(self, word_count):
        """Categorize chunk by word count."""
        if word_count < 200:
            return "small"
        elif word_count < 500:
            return "medium"
        elif word_count < 750:
            return "large"
        else:
            return "xlarge"

    def chunk_all(self):
        """Chunk all documents."""
        print(f"\n{'='*70}")
        print("Constitutional Research System - Document Chunking")
        print(f"{'='*70}")

        for source in self.manifest['sources']:
            collection = source['collection']
            documents = source['documents']

            print(f"\nCollection: {collection.replace('_', ' ').title()}")

            for doc in documents:
                # Get source URL (first successful source)
                source_url = doc.get('sources', [{}])[0].get('url', 'Unknown')

                success, chunk_count = self._chunk_document(
                    collection, doc, source_url
                )

                if success:
                    self.chunk_report['documents'].append({
                        "document_id": doc['id'],
                        "title": doc['title'],
                        "collection": collection,
                        "status": "success",
                        "chunk_count": chunk_count
                    })
                    self.chunk_report['total_chunks'] += chunk_count

                self.chunk_report['total_documents'] += 1

        # Save corpus
        self._save_corpus()
        self._save_report()

        # Print summary
        print(f"\n{'='*70}")
        print("CHUNKING SUMMARY")
        print(f"{'='*70}")
        print(f"Total documents: {self.chunk_report['total_documents']}")
        print(f"Total chunks: {self.chunk_report['total_chunks']}")
        print(f"\nCorpus saved to: {DATA_CHUNKS / 'constitution_full_corpus.json'}")
        print(f"Report saved to: {PROJECT_ROOT / 'data' / 'chunking_report.json'}")
        print(f"{'='*70}\n")

    def _save_corpus(self):
        """Save chunked corpus to JSON."""
        DATA_CHUNKS.mkdir(parents=True, exist_ok=True)
        corpus_file = DATA_CHUNKS / "constitution_full_corpus.json"

        corpus_obj = {
            "metadata": {
                "version": "1.0",
                "description": "Constitutional Research System full-text corpus",
                "generated_timestamp": datetime.now().isoformat(),
                "total_chunks": len(self.all_chunks),
                "total_documents": self.chunk_report['total_documents']
            },
            "chunks": self.all_chunks
        }

        with open(corpus_file, 'w', encoding='utf-8') as f:
            json.dump(corpus_obj, f, indent=2, ensure_ascii=False)

    def _save_report(self):
        """Save chunking report."""
        report_path = PROJECT_ROOT / "data" / "chunking_report.json"
        with open(report_path, 'w') as f:
            json.dump(self.chunk_report, f, indent=2)

def main():
    """Main entry point."""
    chunker = DocumentChunker()
    chunker.chunk_all()

if __name__ == "__main__":
    main()
