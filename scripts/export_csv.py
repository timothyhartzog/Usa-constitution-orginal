#!/usr/bin/env python3
"""
Export corpus chunks to CSV format.
"""

import json
import csv
from pathlib import Path

PROJECT_ROOT = Path(__file__).parent.parent
DATA_CHUNKS = PROJECT_ROOT / "data" / "chunks"
EXPORT_DIR = PROJECT_ROOT / "exports"

def export_to_csv():
    """Export corpus to CSV."""
    # Load corpus
    corpus_path = DATA_CHUNKS / "constitution_full_corpus.json"
    if not corpus_path.exists():
        print("Error: Corpus file not found. Run chunking script first.")
        return

    with open(corpus_path) as f:
        corpus = json.load(f)

    # Create export directory
    EXPORT_DIR.mkdir(exist_ok=True)

    # Define CSV columns
    fieldnames = [
        'chunk_id',
        'document_id',
        'collection',
        'title',
        'author',
        'date',
        'document_type',
        'source_url',
        'word_count',
        'constitutional_clause_tags',
        'issue_tags',
        'text_preview'
    ]

    # Write CSV
    csv_path = EXPORT_DIR / "constitution_corpus.csv"

    with open(csv_path, 'w', newline='', encoding='utf-8') as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()

        for chunk in corpus['chunks']:
            # Create text preview (first 100 chars)
            text_preview = chunk['text'][:100].replace('\n', ' ')

            writer.writerow({
                'chunk_id': chunk['chunk_id'],
                'document_id': chunk['document_id'],
                'collection': chunk['collection'],
                'title': chunk['title'],
                'author': chunk['author'],
                'date': chunk['date'],
                'document_type': chunk['document_type'],
                'source_url': chunk['source_url'],
                'word_count': chunk['word_count'],
                'constitutional_clause_tags': '|'.join(chunk.get('constitutional_clause_tags', [])),
                'issue_tags': '|'.join(chunk.get('issue_tags', [])),
                'text_preview': text_preview
            })

    print(f"Exported {len(corpus['chunks'])} chunks to {csv_path}")

if __name__ == "__main__":
    export_to_csv()
