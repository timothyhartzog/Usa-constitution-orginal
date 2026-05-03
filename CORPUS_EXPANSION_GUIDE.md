# Corpus Expansion Guide: 3000+ Chunks

This guide explains how to expand the constitutional corpus from 16 chunks to 3000+ chunks with comprehensive founding documents.

## Current Status

| Component | Count | Size |
|-----------|-------|------|
| Chunks | 16 | ~50 KB |
| Documents | 7 | Multiple |
| Words | 1,415 | ~50 words/chunk |
| Collections | 7 | Constitution, Federalist, Anti-Fed, etc. |

## Target Expansion

| Source | Est. Chunks | Est. Words |
|--------|------------|-----------|
| Constitution & Bill of Rights | 50 | 13,000 |
| Madison's Convention Notes | 800 | 160,000 |
| Farrand's Records (3 vols) | 1,200 | 240,000 |
| Federalist Papers (85 essays) | 600 | 120,000 |
| Anti-Federalist Writings | 300 | 60,000 |
| Founders' Correspondence (1786-1789) | 50 | 100,000 |
| **TOTAL** | **~3,000** | **~700,000** |

## Document Sources

### 1. Constitution & Bill of Rights

**Source**: Library of Congress
- **URL**: https://www.loc.gov/exhibits/creating-the-united-states
- **Format**: HTML/Text
- **Chunks**: ~50 (original + amendments)

**Integration**:
```python
expander.add_document(
    collection="constitution",
    document_id="original_1787",
    title="Constitution of the United States",
    author="Constitutional Convention",
    date="1787-09-17",
    text=constitution_text,
    source_url="https://www.loc.gov/..."
)
```

### 2. Madison's Convention Notes

**Source**: Constitution.org
- **URL**: https://constitution.org/dh/madison/
- **Format**: Plain text
- **Chunks**: ~800 (50 days of debates)
- **Size**: ~150 KB raw

**Integration**:
```python
# Split by date (each day = multiple chunks)
for date, daily_notes in madison_notes_by_date.items():
    expander.add_document(
        collection="madison_notes",
        document_id=f"madison_{date}",
        title=f"Madison's Notes: {date}",
        author="James Madison",
        date=date,
        text=daily_notes,
        source_url="https://constitution.org/dh/madison/"
    )
```

### 3. Farrand's Records

**Source**: Yale Avalon Project (requires parsing)
- **URL**: https://avalon.law.yale.edu/18th_century/debates.asp
- **Format**: HTML (needs parsing)
- **Chunks**: ~1,200 (convention daily records)
- **Note**: More detailed than Madison's Notes

**Integration**:
```python
# Parse HTML sessions
for session in farrand_sessions:
    expander.add_document(
        collection="farrand_records",
        document_id=f"farrand_{session['date']}",
        title=f"Farrand's Records: {session['date']}",
        author="Max Farrand (Ed.)",
        date=session['date'],
        text=session['text'],
        source_url="https://avalon.law.yale.edu/..."
    )
```

### 4. Federalist Papers

**Source**: Constitution.org + Project Gutenberg
- **URL**: https://www.constitution.org/fed/federa.txt
- **Format**: Plain text (85 essays)
- **Chunks**: ~600 (variable length essays)
- **Authors**: Hamilton, Madison, Jay

**Integration**:
```python
# Each essay = 1-2 chunks
for essay_num, essay_text in federalist_papers.items():
    expander.add_document(
        collection="federalist",
        document_id=f"essay_{essay_num}",
        title=f"Federalist Paper No. {essay_num}",
        author=essay_authors[essay_num],
        date="1787-10-27",  # Publication start
        text=essay_text,
        source_url=f"https://constitution.org/fed/federa{essay_num}.txt"
    )
```

### 5. Anti-Federalist Writings

**Source**: Constitution.org Anti-Federalist Collection
- **URL**: https://www.constitution.org/afp/afp.txt
- **Format**: Plain text (comprehensive collection)
- **Chunks**: ~300 (essays, letters, speeches)
- **Note**: Over 80 documents compiled

**Integration**:
```python
for doc in anti_federalist_docs:
    expander.add_document(
        collection="anti_federalist",
        document_id=doc['id'],
        title=doc['title'],
        author=doc['author'],  # Mason, Henry, Gerry, etc.
        date=doc['date'],
        text=doc['text'],
        source_url=doc['url']
    )
```

### 6. Founders' Correspondence

**Source**: Founders Online (Library of Congress)
- **URL**: https://founders.archives.gov
- **Format**: JSON API or HTML
- **Chunks**: ~50 selected letters (1786-1789)
- **Note**: Comprehensive, requires filtering

**Integration**:
```python
# Focus on convention period (1786-1789)
for letter in founders_correspondence:
    if 1786 <= letter['year'] <= 1789:
        expander.add_document(
            collection="correspondence",
            document_id=f"letter_{letter['id']}",
            title=letter['subject'],
            author=letter['from'],
            date=letter['date'],
            text=letter['text'],
            source_url=letter['url']
        )
```

## Implementation Steps

### Step 1: Prepare Source Data

```bash
# Create source directory
mkdir data/sources

# Download sources (or copy prepared text files)
# - constitution.txt
# - madison_notes.txt
# - farrand_records.txt
# - federalist_papers.txt
# - anti_federalist.txt
# - founders_letters.txt
```

### Step 2: Create Ingestion Script

```python
#!/usr/bin/env python3
from scripts.expand_corpus import CorpusExpander

expander = CorpusExpander()

# Load and process each source
with open("data/sources/constitution.txt") as f:
    expander.add_document(
        collection="constitution",
        document_id="1787",
        title="Constitution of the United States",
        author="Constitutional Convention",
        date="1787-09-17",
        text=f.read(),
        source_url="https://www.loc.gov/"
    )

# ... repeat for other sources

expander.save_corpus()
```

### Step 3: Generate Analytics

```bash
# Generate all analytics
python scripts/advanced_analytics_engine.py
python -m scripts.prepare_deployment
```

### Step 4: Verify & Deploy

```bash
# Check corpus stats
python -c "import json; c=json.load(open('data/chunks/constitution_full_corpus.json')); print(f'Chunks: {len(c[\"chunks\"])}, Words: {sum(x[\"word_count\"] for x in c[\"chunks\"])}')"

# Commit and push
git add data/chunks/ analytics/data/
git commit -m "Expand corpus to 3000+ chunks"
git push
```

## Performance Expectations

### Search Performance with WASM

| Corpus Size | Query Time | Index Size |
|------------|-----------|-----------|
| 16 chunks | 2ms | 50 KB |
| 500 chunks | 5ms | 500 KB |
| 3000 chunks | 15ms | 3 MB |

**Note**: WASM shows 5-10x speedup over JavaScript at scale.

### File Sizes

| Format | 3000 Chunks |
|--------|------------|
| JSON corpus | ~10-15 MB |
| Gzipped | ~2-3 MB |
| Search index | ~5 MB |
| Analytics | ~200 KB |

## Public Domain Sources

All sources are public domain and can be freely used:

- **Constitution**: Public domain (1787)
- **Madison's Notes**: Public domain (published 1840s)
- **Farrand's Records**: Public domain (1911)
- **Federalist Papers**: Public domain (1787-1788)
- **Anti-Federalist Papers**: Public domain (1787-1788)
- **Founders' Letters**: Public domain (Library of Congress)

## Advanced Tagging

The expansion script automatically tags all chunks with:

**Constitutional Clauses** (40+ tags):
- Articles (I-VII): 27 clauses
- Amendments (I-X): 13+ amendment-specific clauses
- Examples: `I.1.legislative_power`, `II.2.commander_in_chief`, `amendment_I.rights`

**Issue Tags** (10+ categories):
- Federalism, Separation of Powers, Representation
- Commerce, Rights, Judicial Review, Amendment Process
- Ratification, Executive Power, Political Theory

## Next Steps

1. **Acquire Source Data** (1-2 days)
   - Download from public archives
   - Clean OCR artifacts
   - Validate completeness

2. **Process & Tag** (1 hour with script)
   - Run expansion script
   - Verify chunk count and tags
   - Check analytics generation

3. **Deploy** (30 minutes)
   - Commit to git
   - Push to branch
   - Enable on Cloudflare/GitHub Pages

4. **Test & Optimize** (1 hour)
   - Measure search performance with WASM
   - Verify analytics visualizations
   - Monitor bundle sizes

## Resources

- **Constitutional Convention**: https://constitution.org/
- **Yale Avalon Project**: https://avalon.law.yale.edu/
- **Founders Online**: https://founders.archives.gov/
- **Library of Congress**: https://www.loc.gov/
- **Project Gutenberg**: https://www.gutenberg.org/

---

**Status**: Ready to expand to 3000+ chunks
**Estimated Time**: 2-3 days for full expansion
**Expected Result**: Comprehensive constitutional research corpus
