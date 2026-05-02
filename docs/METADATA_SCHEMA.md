# Metadata Schema Specification

## Chunk Object Structure

Each chunk in `data/chunks/constitution_full_corpus.json` follows this schema:

```json
{
  "chunk_id": "string",
  "document_id": "string",
  "collection": "string",
  "title": "string",
  "author": "string",
  "date": "string (ISO 8601 or description)",
  "document_type": "string",
  "source_collection": "string",
  "source_url": "string",
  "text": "string",
  "word_count": "integer",
  "chunk_index": "integer",
  "section_number": "integer",
  "chunk_size_category": "string",
  "position_in_document": {
    "section_number": "integer",
    "chunk_in_section": "integer"
  },
  "constitutional_clause_tags": ["string"],
  "issue_tags": ["string"],
  "metadata": {
    "cleaned": "boolean",
    "manually_reviewed": "boolean",
    "confidence_score": "float (0-1)",
    "created_timestamp": "string (ISO 8601)"
  }
}
```

## Field Definitions

### Required Fields

#### `chunk_id` (string)
Unique identifier for the chunk.
- **Format:** `{collection}_{document_id}_{chunk_index:04d}`
- **Example:** `federalist_papers_full_0042`
- **Uniqueness:** Globally unique across entire corpus
- **Immutable:** Yes

#### `document_id` (string)
Identifier for the source document this chunk belongs to.
- **Format:** Snake_case identifier matching config/sources_manifest.json
- **Example:** `federalist_papers_full`, `madison_notes_convention`
- **Relationship:** Links to document metadata in manifest

#### `collection` (string)
High-level collection this chunk belongs to.
- **Valid Values:**
  - `constitution`
  - `madisonsnotes`
  - `farrands_records`
  - `federalist_papers`
  - `anti_federalist`
  - `founders_correspondence`

#### `title` (string)
Full title of the source document.
- **Example:** "Notes of Debates in the Federal Convention"
- **Inherited:** From document metadata, same for all chunks in document

#### `author` (string)
Primary author or attribution.
- **Examples:** "James Madison", "Alexander Hamilton", "Constitutional Convention"
- **For collections:** May list multiple authors (e.g., "Hamilton, Madison, Jay")
- **Inherited:** From document metadata

#### `date` (string)
Date or date range of the document.
- **Format Options:**
  - Single date: `"1787-09-17"` (ISO 8601)
  - Date range: `"1787-05-25 to 1787-09-17"`
  - Period: `"1787"` (year only)
  - Description: `"1786-01-01 to 1789-12-31"` (for correspondence collections)
- **Timezone:** UTC implicitly assumed
- **Inherited:** From document metadata

#### `document_type` (string)
Classification of document type.
- **Valid Values:**
  - `foundational_document` - Constitution, Bill of Rights
  - `convention_notes` - Madison's Notes
  - `convention_records` - Farrand's Records
  - `political_essay` - Federalist Papers, Anti-Federalist essays
  - `correspondence` - Founders' letters

#### `source_url` (string)
URL to the original source document.
- **Format:** Full URL starting with `http://` or `https://`
- **Example:** `"https://www.archives.gov/founding-docs/constitution-transcript"`
- **Accessibility:** Should be accessible (or archived equivalent)
- **Non-null:** Always required

#### `text` (string)
The actual text content of the chunk.
- **Encoding:** UTF-8
- **Line endings:** Unix-style (\n)
- **Whitespace:** Normalized (no excessive blank lines)
- **Length:** Typically 300-500 words, range 50-1000 words
- **Non-empty:** Always has content

#### `word_count` (integer)
Number of words in the text field.
- **Calculation:** `len(text.split())`
- **Accuracy:** Must match actual text word count (±0 tolerance)
- **Range:** 50-1000 words typically

#### `chunk_index` (integer)
Sequential index of this chunk within the document.
- **Base:** 0-indexed (first chunk is 0)
- **Continuity:** Monotonically increasing within document
- **Example:** Document with 50 chunks has indices 0-49

#### `section_number` (integer)
For documents with sections (articles, essays, etc.), the section index.
- **Meaning:** Semantic section (Article 1, Essay 10, etc.)
- **Optional:** May be null for unsectioned documents
- **Hierarchy:** Used with chunk_in_section for document position

#### `chunk_size_category` (string)
Classification of chunk size.
- **Valid Values:**
  - `small` - < 200 words
  - `medium` - 200-500 words
  - `large` - 500-750 words
  - `xlarge` - 750-1000 words
- **Calculation:** Derived from word_count
- **Purpose:** UI/UX hint for rendering

#### `source_collection` (string)
Synonym for `collection`, provided for clarity.
- **Value:** Same as `collection` field
- **Purpose:** Redundancy for frontend convenience

#### `constitutional_clause_tags` (array of strings)
References to constitutional provisions discussed in this chunk.
- **Format:** `"Article.Section.Clause"` or clause_id
- **Valid Examples:**
  - `"preamble"`
  - `"I.1.legislative_power"`
  - `"II.2.commander_in_chief"`
  - `"V.amendment_process"`
- **Multiple Tags:** Chunk can reference multiple clauses
- **Empty OK:** `[]` if no clear constitutional reference
- **Confidence:** All tags assumed high confidence

#### `issue_tags` (array of strings)
Thematic issues or concepts discussed in this chunk.
- **Valid Values:**
  - `federalism`
  - `separation_of_powers`
  - `representation`
  - `commerce`
  - `rights`
  - `judicial_review`
  - `amending_process`
  - `ratification`
  - `executive_power`
  - `political_theory`
- **Multiple Tags:** Chunk can have multiple thematic associations
- **Empty OK:** `[]` if no clear thematic match
- **Overlapping OK:** Same chunk can have multiple issue tags

### Nested Object: `position_in_document`

Describes where in the source document this chunk is located.

#### `position_in_document.section_number` (integer)
Section/article/chapter number in the document.
- **Example:** Constitution chunk 1 is in `section_number: 1` (Article I)
- **0-indexed:** Starts from 0
- **Meaning:** Depends on document structure

#### `position_in_document.chunk_in_section` (integer)
Sequential chunk index within that section.
- **Example:** Second chunk of Article III is `chunk_in_section: 1`
- **0-indexed:** Starts from 0
- **Purpose:** Allows navigation within sections

### Nested Object: `metadata`

Additional metadata about the chunk processing.

#### `metadata.cleaned` (boolean)
Whether text has been cleaned/normalized.
- **Value:** Always `true` in final corpus
- **Meaning:** OCR artifacts removed, whitespace normalized

#### `metadata.manually_reviewed` (boolean)
Whether chunk has been manually reviewed for quality.
- **Current:** Mostly `false`
- **Future:** Set to `true` for high-confidence chunks (Constitution, Federalist Papers)
- **Purpose:** Quality indicator for scholarly use

#### `metadata.confidence_score` (float, 0-1)
Machine confidence score for automated tagging.
- **Range:** 0.0 to 1.0
- **Typical:** 0.85 for auto-tagged chunks
- **High confidence:** > 0.9
- **Manual review:** < 0.7
- **Purpose:** Indicates reliability of clause/issue tags

#### `metadata.created_timestamp` (string, ISO 8601)
When this chunk was created during processing.
- **Format:** `"2026-05-02T14:30:45.123456"`
- **Timezone:** UTC
- **Purpose:** Audit trail for data generation

## Corpus Metadata

The `constitution_full_corpus.json` file also contains a top-level metadata object:

```json
{
  "metadata": {
    "version": "string",
    "description": "string",
    "generated_timestamp": "string",
    "total_chunks": "integer",
    "total_documents": "integer"
  },
  "chunks": [...]
}
```

### Corpus Metadata Fields

#### `metadata.version`
Schema version number.
- **Current:** `"1.0"`
- **Format:** Semantic versioning

#### `metadata.description`
Human-readable description of corpus contents.

#### `metadata.generated_timestamp`
When the corpus was generated.
- **Format:** ISO 8601

#### `metadata.total_chunks`
Total number of chunks in corpus.
- **Validation:** Must equal `chunks.length`

#### `metadata.total_documents`
Number of distinct documents represented.
- **Calculation:** Count of unique `document_id` values

## Constitutional Clause Tags

Valid constitutional clause tags are defined in `config/constitutional_clauses.json`.

### Clause ID Format

Most clauses follow pattern: `Article.Section.Clause`

**Examples:**
- `I.1` - Article I, Section 1
- `I.8` - Article I, Section 8 (Enumerated Powers)
- `II.2` - Article II, Section 2 (Commander in Chief)
- `III.2` - Article III, Section 2 (Judicial Power Scope)

**Special Cases:**
- `preamble` - Constitution preamble
- `V` - Amendment Process (standalone article)
- `VI.1` - Supremacy Clause
- `VII` - Ratification (standalone article)

### Complete List

**Article I (Legislative):**
- I.1: Legislative Power
- I.2: House of Representatives
- I.3: Senate
- I.4: Elections and Meetings
- I.5: Proceedings
- I.6: Compensation
- I.7: Legislative Process
- I.8: Enumerated Powers
- I.9: Limitations on Congress
- I.10: Limitations on States

**Article II (Executive):**
- II.1: Executive Power
- II.2: Commander in Chief
- II.3: State of the Union
- II.4: Take Care Clause

**Article III (Judicial):**
- III.1: Judicial Power
- III.2: Scope of Judicial Power
- III.3: Treason

**Article IV (States):**
- IV.1: Full Faith and Credit
- IV.2: Privileges and Immunities
- IV.3: New States
- IV.4: Guarantee Clause

**Article V:** Amendment Process

**Article VI:** Supremacy and Oath

**Article VII:** Ratification

## Issue Tags

Valid issue tags:

1. **federalism** - Division of power between federal and state governments
2. **separation_of_powers** - Division among branches
3. **representation** - Issues of representation and voting
4. **commerce** - Interstate/international trade regulation
5. **rights** - Individual rights and liberties
6. **judicial_review** - Courts' authority to review laws
7. **amending_process** - How Constitution can be changed
8. **ratification** - Process of adopting Constitution
9. **executive_power** - Presidential authority
10. **political_theory** - Underlying political philosophy

## Data Validation

All chunks MUST satisfy:

1. **Completeness:** All required fields present and non-null
2. **Consistency:** `word_count` matches actual text
3. **Uniqueness:** `chunk_id` is globally unique
4. **Format Compliance:** Fields match expected types and patterns
5. **Tag Validity:** All clause and issue tags exist in configuration
6. **Text Quality:** Non-empty, valid UTF-8, reasonable length

## Example Chunk

```json
{
  "chunk_id": "federalist_papers_full_0010",
  "document_id": "federalist_papers_full",
  "collection": "federalist_papers",
  "title": "The Federalist Papers: A Collection of Essays",
  "author": "Alexander Hamilton, James Madison, John Jay",
  "date": "1787-10-27 to 1788-05-28",
  "document_type": "political_essay",
  "source_collection": "federalist_papers",
  "source_url": "https://www.gutenberg.org/ebooks/18",
  "text": "Among the numerous advantages promised by a union...",
  "word_count": 387,
  "chunk_index": 10,
  "section_number": 10,
  "chunk_size_category": "medium",
  "position_in_document": {
    "section_number": 10,
    "chunk_in_section": 0
  },
  "constitutional_clause_tags": ["I.1", "I.8", "federalism"],
  "issue_tags": ["federalism", "representation", "political_theory"],
  "metadata": {
    "cleaned": true,
    "manually_reviewed": false,
    "confidence_score": 0.92,
    "created_timestamp": "2026-05-02T14:30:45"
  }
}
```

---

**Last Updated:** May 2, 2026
**Schema Version:** 1.0
