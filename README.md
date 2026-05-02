# Constitutional Research System

A comprehensive full-text corpus and search system for U.S. Constitutional Convention primary sources, featuring the original Constitution, Madison's Notes, Farrand's Records, Federalist Papers, Anti-Federalist writings, and Founders' correspondence (1786-1791).

## Features

- **Unified Corpus**: 3,000+ searchable document chunks across 6 primary collections
- **Rich Metadata**: Constitutional clause tagging, thematic issue classification, source provenance
- **Client-Side Search**: Full-text search engine in pure JavaScript (no backend required)
- **Faceted Navigation**: Filter by collection, constitutional clause, or thematic issue
- **Passage Viewer**: Read chunks in context with surrounding document passages
- **Source Links**: Direct links to original archival sources
- **Data Export**: Export selected passages as JSON or CSV

## Collections

1. **U.S. Constitution & Bill of Rights** (~25 chunks)
   - Original 1787 text + 1791 Bill of Rights
   - Source: National Archives, Wikisource

2. **Madison's Notes of the Constitutional Convention** (~300 chunks)
   - Day-by-day debate record from May 25 - September 17, 1787
   - Source: Yale Avalon Project, Archive.org

3. **Farrand's Records of the Federal Convention** (~600 chunks)
   - Complete 3-volume reference work by Max Farrand
   - Source: Yale Avalon Project, Archive.org

4. **Federalist Papers** (~400 chunks)
   - All 85 essays by Hamilton, Madison, and Jay (1787-1788)
   - Source: Project Gutenberg, Yale Avalon Project

5. **Anti-Federalist Writings** (~450 chunks)
   - Selected essays and speeches by Mason, Henry, Gerry, Clinton, and others
   - Source: Constitution.org, Yale Avalon Project

6. **Founders' Correspondence** (~600 chunks)
   - Letters and documents among Washington, Madison, Hamilton, Jefferson, Monroe, Jay, et al. (1786-1789)
   - Source: Founders Online (UVA), Archive.org

**Total: 3,000-3,500 chunks, ~2-4MB corpus**

## Quick Start

### Installation

```bash
# Clone repository
git clone <repository-url>
cd Usa-constitution-orginal

# Install Python dependencies
pip install -r requirements.txt

# Create empty data directories (auto-created by scripts)
mkdir -p data/{raw,clean,chunks,index}
```

### Building the Corpus

```bash
# Phase 1: Acquire sources from public archives
python scripts/ingest_sources.py

# Phase 2: Clean and normalize texts
python scripts/clean_text.py

# Phase 3: Chunk documents semantically
python scripts/chunk_documents.py

# Phase 4: Enrich metadata with constitutional clause and issue tags
python scripts/metadata_tagging.py

# Phase 5: Generate search index for frontend
python scripts/build_search_index.py

# Phase 6: Run validation tests
pytest tests/

# Phase 7: Export to CSV (optional)
python scripts/export_csv.py
```

### Using the Search Interface

1. Open `frontend/index.html` in a modern web browser
2. Enter search terms in the search bar
3. Use faceted filters to narrow results by collection, clause, or issue
4. Click a result to view the full passage in context
5. Navigate through surrounding chunks with Previous/Next buttons
6. Export selected passages using the Export button

## Data Structure

### Main Corpus File: `data/chunks/constitution_full_corpus.json`

Each chunk contains:
- `chunk_id`: Unique identifier (e.g., `const_original_1787_0001`)
- `document_id`: Document reference
- `title`, `author`, `date`: Metadata
- `text`: 300-500 word content block
- `word_count`: Character count
- `source_url`: Link to original archive
- `constitutional_clause_tags`: Article/Section references (e.g., `I.1.legislative_power`)
- `issue_tags`: Thematic categories (federalism, separation_of_powers, etc.)

### Search Index: `data/index/search_index.json`

Pre-computed inverted index for fast client-side search:
- Word → chunk_ids
- Constitutional clause → chunk_ids
- Filters for collections, authors, document types

## Testing

Run the full test suite to validate data quality:

```bash
pytest tests/ -v
pytest tests/ --cov=scripts --cov-report=html
```

Tests validate:
- All chunks have required metadata fields
- Source URLs are accessible
- No duplicate chunk_ids
- Word counts match actual text
- Constitutional clause tags are valid
- All text is valid UTF-8

## Documentation

- `docs/SOURCES.md` - Detailed source information and acquisition methods
- `docs/METADATA_SCHEMA.md` - Full specification of chunk structure
- `docs/API.md` - Frontend JavaScript API documentation
- `docs/TAGGING_GUIDE.md` - Convention for constitutional clause and issue tagging
- `config/sources_manifest.json` - Complete source URLs and fallback chains
- `config/constitutional_clauses.json` - Constitutional clause taxonomy and issue tag definitions

## Architecture

### Data Pipeline

```
Raw Sources (Archive.org, Project Gutenberg, Yale Avalon)
    ↓
[ingest_sources.py] → data/raw/{collection}/
    ↓
[clean_text.py] → data/clean/
    ↓
[chunk_documents.py] → data/chunks/constitution_full_corpus.json
    ↓
[metadata_tagging.py] → enriched chunks with tags
    ↓
[build_search_index.py] → data/index/search_index.json
    ↓
Frontend [search.js] → Client-side full-text search
```

### Frontend (No Backend)

- Single-page application (`frontend/index.html`)
- Pure JavaScript search engine (`frontend/search.js`)
- Loads full corpus and index on page load (~5MB total)
- All operations client-side (search, filtering, export)

## Requirements

- Python 3.8+
- Modern web browser (Chrome, Firefox, Safari, Edge)
- No backend server required

## License

Public Domain. All source documents are from public archives and government sources. See `SOURCES.md` for specific source attributions.

## Contributing

Contributions welcome! Areas for expansion:
- Additional amendments (13th-26th)
- Scholarly commentary and annotations
- Additional founder correspondence
- Interactive timeline and debate visualizations
- Advanced search features (phrase search, boolean operators)

## Acknowledgments

Sources:
- National Archives: Constitution and Bill of Rights
- Project Gutenberg: Federalist Papers, Madison's Notes
- Yale Law School Avalon Project: Foundational documents and Farrand's Records
- Constitution.org: Anti-Federalist Papers collection
- University of Virginia Founders Online: Extensive correspondence database
- Internet Archive: Historical document preservation

---

**Last Updated:** May 2, 2026
**Status:** In Development
**Estimated Completion:** May 2026
