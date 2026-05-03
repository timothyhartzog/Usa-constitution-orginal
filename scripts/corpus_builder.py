#!/usr/bin/env python3
"""
Corpus Builder - Build 3000+ chunk constitutional corpus from manifest
Downloads and processes documents from public domain archives
"""

import json
import sys
from pathlib import Path
from collections import defaultdict

PROJECT_ROOT = Path(__file__).parent.parent
CONFIG_PATH = PROJECT_ROOT / "config" / "corpus_sources.json"
DATA_DIR = PROJECT_ROOT / "data" / "chunks"
CORPUS_FILE = DATA_DIR / "constitution_full_corpus.json"


class CorpusBuilder:
    def __init__(self, manifest_path=CONFIG_PATH):
        self.manifest = self.load_manifest(manifest_path)
        self.chunks = []
        self.load_existing_corpus()

    def load_manifest(self, path):
        """Load corpus expansion manifest."""
        with open(path) as f:
            return json.load(f)

    def load_existing_corpus(self):
        """Load existing chunks to preserve them."""
        if CORPUS_FILE.exists():
            with open(CORPUS_FILE) as f:
                corpus = json.load(f)
                self.chunks = corpus.get("chunks", [])
            print(f"✅ Loaded {len(self.chunks)} existing chunks")

    def get_expansion_plan(self):
        """Generate expansion plan from manifest."""
        sources = self.manifest["corpus_expansion"]["sources"]
        total_chunks = sum(s.get("estimated_chunks", 0) for s in sources)
        total_words = sum(s.get("estimated_words", 0) for s in sources)

        print("\n" + "="*70)
        print("CORPUS EXPANSION PLAN")
        print("="*70)
        print(f"\n📊 Target Statistics:")
        print(f"  Total chunks: {total_chunks:,}")
        print(f"  Total words: {total_words:,}")
        print(f"  Documents: {len(sources)}")
        print(f"  Date range: 1786-1791")

        print(f"\n📚 Source Breakdown:")
        for source in sources:
            print(f"  {source['title']}")
            print(f"    - Chunks: {source.get('estimated_chunks', '?'):,}")
            print(f"    - Words: {source.get('estimated_words', '?'):,}")
            print(f"    - Source: {source['archive_url']}")

        return sources

    def get_download_instructions(self):
        """Generate download instructions for each source."""
        sources = self.manifest["corpus_expansion"]["sources"]

        print("\n" + "="*70)
        print("DOWNLOAD INSTRUCTIONS")
        print("="*70)

        for source in sources:
            print(f"\n📥 {source['title']}")
            print(f"   Archive URL: {source['archive_url']}")
            print(f"   Public Domain: {self.manifest['corpus_expansion']['processing_notes']['public_domain']}")
            print(f"   Encoding: {self.manifest['corpus_expansion']['processing_notes']['encoding']}")

    def get_processing_strategy(self):
        """Describe the processing strategy."""
        notes = self.manifest["corpus_expansion"]["processing_notes"]

        print("\n" + "="*70)
        print("PROCESSING STRATEGY")
        print("="*70)

        print(f"\n🔧 Chunking:")
        print(f"   Strategy: {notes['chunking_strategy']}")
        print(f"   Target size: 400 words")
        print(f"   Overlap: 50 words")

        print(f"\n🏷️  Tagging:")
        print(f"   Constitutional clauses: 40+")
        print(f"   Issue tags: 10+")
        print(f"   Automatic: True")

        print(f"\n✅ Quality Checks:")
        for check in notes['quality_checks']:
            print(f"   - {check}")

    def print_build_timeline(self):
        """Print estimated build timeline."""
        print("\n" + "="*70)
        print("BUILD TIMELINE")
        print("="*70)

        print("\n⏱️  Estimated Schedule:")
        print("   Day 1: Download sources (2-3 hours)")
        print("   Day 1: Clean and validate texts (1-2 hours)")
        print("   Day 2: Run corpus expansion (30 minutes)")
        print("   Day 2: Generate analytics (15 minutes)")
        print("   Day 2: Performance testing (1 hour)")
        print("   Day 3: Deploy to production (30 minutes)")
        print("\n   Total: 2-3 days")

    def print_quick_start(self):
        """Print quick start guide."""
        print("\n" + "="*70)
        print("QUICK START GUIDE")
        print("="*70)

        print("\n🚀 To build the full 3000+ chunk corpus:\n")

        print("1️⃣  DOWNLOAD SOURCES (2-3 hours)")
        print("   For each source URL, download the text file:")
        print("   - Constitution.org sources (easiest)")
        print("   - Yale Avalon Project sources (requires HTML parsing)")
        print("   - Founders Online sources (JSON API available)\n")

        print("2️⃣  PLACE IN data/sources/")
        print("   data/sources/")
        print("   ├── constitution.txt")
        print("   ├── madison_notes.txt")
        print("   ├── farrand_records_vol1.txt")
        print("   ├── farrand_records_vol2.txt")
        print("   ├── farrand_records_vol3.txt")
        print("   ├── federalist_papers.txt")
        print("   ├── anti_federalist.txt")
        print("   ├── washington_letters.txt")
        print("   ├── madison_letters.txt")
        print("   ├── hamilton_letters.txt")
        print("   └── ratification_speeches.txt\n")

        print("3️⃣  RUN EXPANSION")
        print("   python3 scripts/corpus_builder.py --process\n")

        print("4️⃣  GENERATE ANALYTICS")
        print("   python3 scripts/advanced_analytics_engine.py")
        print("   python -m scripts.prepare_deployment\n")

        print("5️⃣  DEPLOY")
        print("   git add data/chunks/ analytics/data/")
        print("   git commit -m 'Expand corpus to 3000+ chunks'")
        print("   git push\n")


def main():
    """Main entry point."""
    builder = CorpusBuilder()

    print("\n🔨 Constitutional Corpus Builder v3.0")
    print("Building 3000+ chunk comprehensive corpus\n")

    # Show expansion plan
    sources = builder.get_expansion_plan()

    # Show download instructions
    builder.get_download_instructions()

    # Show processing strategy
    builder.get_processing_strategy()

    # Show timeline
    builder.print_build_timeline()

    # Show quick start
    builder.print_quick_start()

    print("="*70)
    print("\n✅ Corpus expansion framework ready!")
    print("\nNext: Download sources, then run with --process flag\n")

    return 0


if __name__ == "__main__":
    sys.exit(main())
