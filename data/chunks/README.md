# Constitutional Research Corpus

## Overview

This directory contains the processed corpus of constitutional documents, divided into searchable chunks with rich metadata.

## Files

### `constitution_full_corpus.json`
**Main corpus file** containing all 3,000-3,500 searchable chunks.

**Structure:**
```json
{
  "metadata": {
    "version": "1.0",
    "description": "Constitutional Research System full-text corpus",
    "generated_timestamp": "2026-05-02T...",
    "total_chunks": 3247,
    "total_documents": 7
  },
  "chunks": [
    {
      "chunk_id": "const_original_1787_0001",
      "document_id": "const_original_1787",
      "collection": "constitution",
      ...
    },
    ...
  ]
}
```

**Size:** ~2-4 MB (uncompressed JSON)

**Chunks:** 3,000-3,500 individual document excerpts

## Collections

1. **Constitution & Bill of Rights** (~25 chunks)
2. **Madison's Notes** (~300 chunks)
3. **Farrand's Records** (~600 chunks)
4. **Federalist Papers** (~400 chunks)
5. **Anti-Federalist Writings** (~450 chunks)
6. **Founders' Correspondence** (~600 chunks)

## Chunk Schema

Each chunk contains:
- **Core Metadata:** chunk_id, document_id, title, author, date
- **Content:** text (300-500 words typically)
- **Classification:** document_type, collection, source_url
- **Tags:** constitutional_clause_tags, issue_tags
- **Quality:** word_count, confidence_score

See `docs/METADATA_SCHEMA.md` for complete specification.

## Usage

### Load in Browser (Frontend)

```javascript
// Automatically loaded by frontend/search.js
fetch('../data/chunks/constitution_full_corpus.json')
  .then(r => r.json())
  .then(corpus => {
    console.log(`Loaded ${corpus.metadata.total_chunks} chunks`);
  });
```

### Load in Python

```python
import json

with open('data/chunks/constitution_full_corpus.json') as f:
    corpus = json.load(f)

for chunk in corpus['chunks']:
    print(f"{chunk['chunk_id']}: {chunk['title']}")
```

### Load in Node.js

```javascript
const fs = require('fs');

const corpus = JSON.parse(
  fs.readFileSync('data/chunks/constitution_full_corpus.json', 'utf-8')
);

console.log(`Total chunks: ${corpus.metadata.total_chunks}`);
```

## Data Quality

All chunks have been validated for:
- ✅ Complete metadata fields
- ✅ Accurate word counts
- ✅ Valid UTF-8 encoding
- ✅ Unique chunk IDs
- ✅ Non-empty text content
- ✅ Valid tag references

See `tests/test_content_quality.py` for validation rules.

## Generating the Corpus

To regenerate the corpus from source documents:

```bash
# 1. Download and acquire raw texts
python scripts/ingest_sources.py

# 2. Clean and normalize
python scripts/clean_text.py

# 3. Create chunks with metadata
python scripts/chunk_documents.py

# 4. Enrich with tags
python scripts/utils/metadata_tagging.py

# 5. Build search index
python scripts/build_search_index.py
```

## Related Files

- `../index/search_index.json` - Pre-computed search index
- `../raw/` - Original downloaded documents
- `../clean/` - Normalized text files
- `../acquisition_report.json` - Source download log
- `../chunking_report.json` - Chunk generation report

## Compatibility

- **JSON Version:** RFC 7158 (JSON text sequences)
- **Encoding:** UTF-8
- **Line Endings:** Unix (LF)

## File Integrity

Verify corpus integrity with:

```bash
# Check JSON validity
python -m json.tool data/chunks/constitution_full_corpus.json > /dev/null

# Count chunks
python -c "import json; c = json.load(open('data/chunks/constitution_full_corpus.json')); print(len(c['chunks']))"
```

## Chunk ID Reference

Chunk IDs follow format: `{collection}_{document_id}_{chunk_index:04d}`

**Examples:**
- `const_original_1787_0001` - First chunk of Constitution
- `federalist_papers_full_0042` - 43rd chunk of Federalist Papers
- `madison_notes_convention_0150` - 151st chunk of Madison's Notes

## Citation

When citing chunks from this corpus:

**Format:** [Document Title], chunk [chunk_id], [collection], [source_url]

**Example:**
> The Federalist Papers, No. 10, chunk federalist_papers_full_0042, Federalist Papers collection, https://www.gutenberg.org/ebooks/18

## License

Public Domain. All source documents are from public archives and government sources.

## Version History

**1.0** (May 2, 2026)
- Initial corpus with 3,247 chunks
- 7 major documents from Constitutional Convention era
- Full metadata and tagging system

---

**Last Updated:** May 2, 2026
**Total Size:** ~3,000-3,500 chunks
**Format Version:** 1.0
