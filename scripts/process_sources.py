#!/usr/bin/env python3
"""
Process all source documents into expanded corpus
"""

import json
import sys
from pathlib import Path

PROJECT_ROOT = Path(__file__).parent.parent
sys.path.insert(0, str(PROJECT_ROOT))

from expand_corpus import CorpusExpander, DataSeeder

SOURCES_DIR = PROJECT_ROOT / "data" / "sources"

def main():
    expander = CorpusExpander()

    # Source configuration
    sources = [
        {
            "file": "constitution.txt",
            "collection": "constitution",
            "document_id": "1787",
            "title": "Constitution of the United States (1787)",
            "author": "Constitutional Convention",
            "date": "1787-09-17",
            "source_url": "https://constitution.org/us/us.txt"
        },
        {
            "file": "madison_notes.txt",
            "collection": "madison_notes",
            "document_id": "madison_1787",
            "title": "Madison's Notes of Debates in the Federal Convention",
            "author": "James Madison",
            "date": "1787-05-25",
            "source_url": "https://constitution.org/dh/madison/"
        },
        {
            "file": "farrand_vol1.txt",
            "collection": "farrand_records",
            "document_id": "farrand_vol1",
            "title": "Farrand's Records Volume 1",
            "author": "Max Farrand",
            "date": "1787-05-25",
            "source_url": "https://avalon.law.yale.edu/18th_century/debates_001.asp"
        },
        {
            "file": "farrand_vol2.txt",
            "collection": "farrand_records",
            "document_id": "farrand_vol2",
            "title": "Farrand's Records Volume 2",
            "author": "Max Farrand",
            "date": "1787-07-01",
            "source_url": "https://avalon.law.yale.edu/18th_century/debates_002.asp"
        },
        {
            "file": "farrand_vol3.txt",
            "collection": "farrand_records",
            "document_id": "farrand_vol3",
            "title": "Farrand's Records Volume 3",
            "author": "Max Farrand",
            "date": "1787-09-17",
            "source_url": "https://avalon.law.yale.edu/18th_century/debates_003.asp"
        },
        {
            "file": "federalist_papers.txt",
            "collection": "federalist",
            "document_id": "federalist_essays",
            "title": "The Federalist Papers",
            "author": "Hamilton, Madison, Jay",
            "date": "1787-10-27",
            "source_url": "https://constitution.org/fed/federa.txt"
        },
        {
            "file": "anti_federalist.txt",
            "collection": "anti_federalist",
            "document_id": "anti_federalist_essays",
            "title": "Anti-Federalist Papers and Essays",
            "author": "Mason, Henry, Gerry, and others",
            "date": "1787-09-17",
            "source_url": "https://constitution.org/afp/afp.txt"
        },
        {
            "file": "washington_letters.txt",
            "collection": "correspondence",
            "document_id": "washington_letters",
            "title": "George Washington Letters (1786-1789)",
            "author": "George Washington",
            "date": "1786-01-01",
            "source_url": "https://founders.archives.gov/"
        },
        {
            "file": "hamilton_letters.txt",
            "collection": "correspondence",
            "document_id": "hamilton_letters",
            "title": "Alexander Hamilton Letters (1786-1789)",
            "author": "Alexander Hamilton",
            "date": "1786-01-01",
            "source_url": "https://founders.archives.gov/"
        },
        {
            "file": "ratification_speeches.txt",
            "collection": "ratification_debates",
            "document_id": "ratification_speeches",
            "title": "State Ratification Convention Speeches",
            "author": "Various delegates",
            "date": "1787-12-01",
            "source_url": "https://avalon.law.yale.edu/18th_century/"
        }
    ]

    print("\n" + "="*70)
    print("PROCESSING SOURCE DOCUMENTS")
    print("="*70)

    # Process each source
    for source in sources:
        source_file = SOURCES_DIR / source["file"]

        if not source_file.exists():
            print(f"\n⚠️  Skipped: {source['file']} (not found)")
            continue

        try:
            with open(source_file, 'r', encoding='utf-8') as f:
                text = f.read()

            expander.add_document(
                collection=source["collection"],
                document_id=source["document_id"],
                title=source["title"],
                author=source["author"],
                date=source["date"],
                text=text,
                source_url=source["source_url"]
            )
        except Exception as e:
            print(f"\n❌ Error processing {source['file']}: {e}")

    print("\n" + "="*70)
    print("EXPANSION COMPLETE")
    print("="*70)

    total_words = sum(c["word_count"] for c in expander.chunks)
    avg_chunk_size = total_words / len(expander.chunks) if expander.chunks else 0

    print(f"\nTotal chunks: {len(expander.chunks)}")
    print(f"Total words: {total_words:,}")
    print(f"Avg chunk size: {avg_chunk_size:.0f} words")
    print(f"Collections: {len(set(c['document_id'].split('_')[0] for c in expander.chunks))}")

    all_clauses = set()
    for chunk in expander.chunks:
        all_clauses.update(chunk["constitutional_clause_tags"])
    print(f"Constitutional clauses covered: {len(all_clauses)}")

    all_issues = set()
    for chunk in expander.chunks:
        all_issues.update(chunk["issue_tags"])
    print(f"Issue tags covered: {len(all_issues)}")

    print("\n✅ Saving expanded corpus...")
    expander.save_corpus()

    return 0

if __name__ == "__main__":
    import sys
    sys.exit(main())
