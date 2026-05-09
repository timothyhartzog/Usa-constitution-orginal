# Corpus expansion — May 2026

## What changed

The manifest at `config/sources_manifest.json` grew from 11 to 25 documents.
The 14 new entries cover the major Anti-Federalist essay series and the
personal-correspondence record of the principal founders during the
ratification period:

### Anti-Federalist (in `collections.anti_federalist`)

| `document_id` | Author (attributed) | Date range | Strategy |
|---|---|---|---|
| `brutus_essays` | Robert Yates | 1787-10-18 – 1788-04-10 | `numbered_essay_series` |
| `letters_from_a_federal_farmer` | Richard Henry Lee | 1787-10-08 – 1788-01-25 | `numbered_essay_series` |
| `cato_essays` | George Clinton | 1787-09-27 – 1788-01-03 | `numbered_essay_series` |
| `centinel_essays` | Samuel Bryan | 1787-10-05 – 1788-04-09 | `numbered_essay_series` |
| `mason_objections_1787` | George Mason | 1787-09-16 | `sliding_window` |
| `henry_speeches_virginia_1788` | Patrick Henry | 1788-06-04 – 1788-06-25 | `sliding_window` |

### Founders correspondence (in `collections.founders_correspondence`)

| `document_id` | Author | Date range | Strategy |
|---|---|---|---|
| `madison_writings_vol_5` | James Madison | 1787-1790 | `correspondence_letters` |
| `hamilton_correspondence_1787_1788` | Alexander Hamilton | 1787-1788 | `correspondence_letters` |
| `washington_letter_to_congress_1787` | George Washington | 1787-09-17 | `sliding_window` |
| `washington_correspondence_1787_1789` | George Washington | 1787-1789 | `correspondence_letters` |
| `adams_defence_constitutions_vol_1` | John Adams | 1787-01-01 | `sliding_window` |
| `adams_correspondence_1787_1789` | John Adams | 1787-1789 | `correspondence_letters` |
| `jay_correspondence_1787_1789` | John Jay | 1787-1789 | `correspondence_letters` |
| `franklin_speech_constitutional_convention` | Benjamin Franklin | 1787-09-17 | `sliding_window` |

## Two-phase build

1. **Manifest commit** (this change) — sources are declared in
   `config/sources_manifest.json` and the chunking strategies they need
   (`numbered_essay_series`, `correspondence_letters`) are implemented in
   `scripts/chunk_documents.py`. The pipeline now logs a clear `SKIPPED`
   record for any manifest entry whose raw source has not yet been fetched
   instead of raising `FileNotFoundError`.
2. **Network ingest** — when network access is available, run:

   ```bash
   python3 scripts/ingest_sources.py        # fetches new sources into data/raw/
   python3 scripts/clean_text.py             # normalizes raw → data/clean/
   python3 scripts/chunk_documents.py        # rebuilds data/chunks/
   python3 scripts/build_search_index.py     # rebuilds the static JSON index
   cargo run --release --bin constitution-archive -- build
   ```

   That last command rebuilds the binary archive that the WASM frontend
   loads, so the new chunks become searchable in the browser.

## New chunk strategies

`scripts/chunk_documents.py` grew two strategies:

- **`numbered_essay_series`** — splits a single source file on a configurable
  header pattern (default matches `BRUTUS, No. I`, `Letter I`,
  `CENTINEL, Number 1`, etc.). Each numbered essay becomes one record. Each
  manifest entry can override the regex via `chunk_options.header_pattern` and
  set the slug prefix via `chunk_options.series_label`.
- **`correspondence_letters`** — generic letter splitter for any author. By
  default matches `TO <NAME>.`, `FROM <NAME>`, or `Letter to <NAME>` headers,
  detects the year, and produces stable identifiers of the form
  `<source_doc>_to_<recipient>_<year>_<seq>`.

The legacy `jefferson_letters` strategy is retained for backward compatibility
with the existing Jefferson volume.

## Schema validation

`tests/test_manifest_schema.py` runs without requiring the corpus to be
generated. It verifies that:

- every manifest entry carries the required fields,
- `document_id` values are globally unique,
- `chunk_strategy` values match a known strategy in `chunk_documents.py`,
- `source_format` is `text` or `html`,
- `chunk_options` is only attached to strategies that consume it,
- `source_url` values are HTTP/HTTPS URLs.

These run on every `pytest` invocation and protect against typos in the
manifest before any expensive network step.

## Process timeline integration

`data/process_timeline.json` gained five new ratification-debate events
(`ratification_federal_farmer_letter_i`, `ratification_cato_no_1`,
`ratification_centinel_no_1`, `ratification_brutus_no_x_judiciary`,
`ratification_henry_liberty_or_empire`) and updated four existing events
(`ratification_brutus_no_1`, `convention_signing`,
`convention_bill_of_rights_rejected`, …) to point at the chunk identifiers
produced by the new manifest entries. After a successful ingest, the WASM
process explorer can pivot from any of these events directly to the
underlying primary-source passages.
