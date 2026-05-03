# Corpus Metadata Schema

The generated corpus file lives at `data/chunks/constitution_full_corpus.json`.

## Top-level structure

```json
{
  "metadata": {
    "version": "2.0",
    "generated_at": "2026-05-02T00:00:00",
    "total_chunks": 0,
    "total_documents": 0
  },
  "chunks": []
}
```

## Chunk schema

Every entry in `chunks` includes:

| Field | Type | Description |
| --- | --- | --- |
| `chunk_id` | string | Unique chunk identifier |
| `document_id` | string | Logical document identifier used for document-level filtering |
| `title` | string | Display title for the document or subdocument |
| `author` | string | Author, editor, or convention body |
| `date` | string | Source date or date range |
| `source_collection` | string | Top-level collection name |
| `source_url` | string | Public-domain source URL |
| `document_type` | string | One of `foundational_document`, `convention_notes`, `convention_records`, `political_essay`, or `correspondence` |
| `issue_tags` | array of strings | Thematic tags from `config/constitutional_clauses.json` |
| `constitutional_clause_tags` | array of strings | Constitutional clause tags from `config/constitutional_clauses.json` |
| `text` | string | Passage text used for search and display |
| `word_count` | integer | Exact word count for `text` |
| `preview` | string | Short display snippet used by the browser interface |

## Search index schema

The generated search index lives at `data/index/search_index.json` and contains:

- `inverted_index` - token to chunk-id postings
- `issue_index` - issue tag to chunk-id postings
- `clause_index` - clause tag to chunk-id postings
- `filters` - collection, document, author, issue, and document-type filter metadata
- `chunks` - lightweight per-chunk metadata used by the browser
