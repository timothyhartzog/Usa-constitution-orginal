# Public-Domain Source Notes

This project builds its corpus from public-domain or government-source historical texts and caches the retrieved source material into `data/raw/`.

## Collections and primary source URLs

| Collection | Documents in manifest | Primary public-domain source |
| --- | --- | --- |
| Constitution | Constitution of the United States | Project Gutenberg: `https://www.gutenberg.org/cache/epub/5/pg5.txt` |
| Madison's Notes | May 29, June 19, and July 17 debates | Yale Avalon Project daily debate pages under `https://avalon.law.yale.edu/18th_century/` |
| Farrand's Records | Volume 1 and Volume 2 OCR text | Internet Archive download endpoints for `recordsfederalc00farrgoog` and `recordsfederalc01farrgoog` |
| Federalist Papers | Complete Federalist Papers ebook | Project Gutenberg: `https://www.gutenberg.org/cache/epub/18/pg18.txt` |
| Anti-Federalist / objections | Virginia, New York, and North Carolina ratification texts with recommended rights language | Yale Avalon Project ratification texts |
| Founders correspondence | Thomas Jefferson, *Writings*, Volume 3, filtered to 1786-1789 correspondence | Project Gutenberg: `https://www.gutenberg.org/cache/epub/52878/pg52878.txt` |
| Bill of Rights | Complete collection of Bill of Rights documentation 1787-1791 | Compiled from National Archives, Constitution Center, Library of Congress, Teaching American History |

## Bill of Rights Collection (NEW)

The Bill of Rights collection provides comprehensive primary and secondary source materials documenting the creation, debate, and ratification of the first ten amendments (1787-1791).

### Documents included:

**Primary Sources:**
- `bill_of_rights_original_text_1791.txt` - Official text of all 10 ratified amendments with historical context
- `madison_speech_bill_of_rights_1789_06_08.txt` - James Madison's speech to Congress introducing amendments (June 8, 1789)
- `jefferson_madison_correspondence_bill_of_rights.txt` - Correspondence and analysis of Jefferson-Madison exchange on rights (1787-1789)

**Secondary Sources with Primary References:**
- `congressional_debates_bill_of_rights_1789.txt` - Timeline and detailed coverage of House and Senate debates, committee work, and modifications (June-September 1789)
- `anti_federalist_positions_bill_of_rights.txt` - Anti-Federalist movement, demands, state convention proposals, and influence on final document
- `bill_of_rights_key_figures.txt` - Biographical and historical analysis of 8+ major figures (Madison, Jefferson, Washington, Sherman, Henry, Mason, Adams, Hancock)

**Finding and Research Tools:**
- `bill_of_rights_index.txt` - Complete index with metadata, retrieval strategies, cross-references, and recommended search queries for RAG systems

### Source Documentation:

The Bill of Rights collection is compiled from authoritative public-domain and government sources:

- **National Archives**: Official Bill of Rights texts and documents (https://www.archives.gov/founding-docs/bill-of-rights)
- **Constitution Center**: Curated collections, correspondence analysis, educational resources (https://constitutioncenter.org/)
- **Library of Congress**: Digital collections and exhibits on constitutional history (https://www.loc.gov/exhibits/creating-the-united-states)
- **Teaching American History**: Primary document transcriptions and classroom resources (https://teachingamericanhistory.org/)
- **Founders Online**: Transcriptions of founding-era documents (https://founders.archives.gov/)
- **Bill of Rights Institute**: Scholarly essays and primary sources (https://billofrightsinstitute.org/)
- **National Humanities Center**: Teaching resources and document collections

### Rationale and Scope:

The Bill of Rights collection extends the existing constitutional research system by providing:
1. **Comprehensive Bill of Rights documentation** - From initial demands through ratification
2. **Multiple perspectives** - Federalist, Anti-Federalist, and neutral analytical views
3. **Key figures coverage** - Understanding individual roles and intellectual contributions
4. **RAG optimization** - Structured documents with clear metadata, cross-references, and search-friendly organization
5. **Connection to existing materials** - References state ratification documents already in the corpus (Virginia, New York, North Carolina ratification texts)

See `docs/BILL_OF_RIGHTS_README.md` for complete collection documentation.

## Notes on scope

- The pipeline is designed around stable public-domain text endpoints that can be downloaded without paid APIs.
- Some collections are represented through curated ratification-era texts or filtered correspondence sections so the resulting corpus remains local, reproducible, and searchable.
- Internet Archive OCR sources contain scan artifacts. The cleaner removes the front-matter boilerplate and normalizes whitespace, but the OCR still reflects the archival scan.
- The Bill of Rights collection is compiled from authoritative secondary sources (Constitution Center, Library of Congress, National Archives) rather than OCR sources, providing high-quality text suitable for RAG systems and research applications.

## Reproducibility

1. `scripts/ingest_sources.py` downloads each configured source into `data/raw/<collection>/<document_id>/`.
2. `scripts/clean_text.py` extracts or normalizes text into `data/clean/`.
3. `scripts/chunk_documents.py` creates search chunks and enriches them with issue and constitutional clause tags.
4. `scripts/build_search_index.py` creates the client-side search index.

If a source endpoint changes or becomes unavailable, rerunning the pipeline will continue to work as long as the cached copy remains in `data/raw/`.
