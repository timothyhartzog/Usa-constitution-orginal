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

## Notes on scope

- The pipeline is designed around stable public-domain text endpoints that can be downloaded without paid APIs.
- Some collections are represented through curated ratification-era texts or filtered correspondence sections so the resulting corpus remains local, reproducible, and searchable.
- Internet Archive OCR sources contain scan artifacts. The cleaner removes the front-matter boilerplate and normalizes whitespace, but the OCR still reflects the archival scan.

## Reproducibility

1. `scripts/ingest_sources.py` downloads each configured source into `data/raw/<collection>/<document_id>/`.
2. `scripts/clean_text.py` extracts or normalizes text into `data/clean/`.
3. `scripts/chunk_documents.py` creates search chunks and enriches them with issue and constitutional clause tags.
4. `scripts/build_search_index.py` creates the client-side search index.

If a source endpoint changes or becomes unavailable, rerunning the pipeline will continue to work as long as the cached copy remains in `data/raw/`.
