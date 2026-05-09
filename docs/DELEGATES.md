# Delegates of the 1787 Federal Convention

`data/delegates.json` is a registry of every delegate appointed to the
Convention. It is decoupled from the ingest manifest so it remains a
useful research resource even when the corresponding texts have not
been fetched.

## Contents

| Field | Description |
|---|---|
| `id` | Stable slug, e.g. `madison_james`. |
| `name` | Full given name as commonly cited. |
| `state` | One of the 12 states that sent delegates (Rhode Island sent none). |
| `born`, `died` | ISO-8601 dates. |
| `status` | One of `signed`, `signed_by_proxy`, `refused_to_sign`, `left_before_signing`. |
| `notable_role` | One-paragraph contextual note. |
| `archive_url` | Canonical public-domain compilation of the delegate's papers (Founders Online, Library of Congress, archive.org, Project Gutenberg, OLL). |
| `manifest_entry` | Optional `document_id` matching an entry in `config/sources_manifest.json`. |

## Composition

| | Count |
|---|---:|
| Total delegates | **55** |
| Personally signed | 38 |
| Signed by proxy (Dickinson) | 1 |
| Refused to sign (Mason, Gerry, Randolph) | 3 |
| Left before signing | 13 |

| State | Delegates |
|---|---:|
| Connecticut | 3 |
| Delaware | 5 |
| Georgia | 4 |
| Maryland | 5 |
| Massachusetts | 4 |
| New Hampshire | 2 |
| New Jersey | 5 |
| New York | 3 |
| North Carolina | 5 |
| Pennsylvania | 8 |
| South Carolina | 4 |
| Virginia | 7 |

## Manifest coverage

Of the 55 delegates, **24** have an associated manifest entry pointing
at a public-domain compilation of their writings. The remaining **31**
either:

- did not leave substantial public-domain papers beyond what is already
  captured in Madison's Notes / Farrand's Records (which the manifest
  ingests in full), or
- are covered by other delegates' compilations (e.g. McHenry's notes
  contain other delegates' speeches).

The 31 "no manifest" delegates remain in the registry so that future
research, citation extraction, or process-timeline cross-references
can attach to them by id.

## Two-phase build

Manifest entries point at remote URLs; the ingest pipeline fetches
them. To pick up the new delegate-paper sources after pulling this
branch:

```bash
python3 scripts/ingest_sources.py    # downloads new sources into data/raw/
python3 scripts/clean_text.py         # normalises raw → data/clean/
python3 scripts/chunk_documents.py    # rebuilds data/chunks/
cargo run --release --bin constitution-archive -- build
```

`scripts/clean_text.py` and `scripts/chunk_documents.py` log
`SKIPPED` for any manifest entry whose raw source has not been
fetched, so partial ingest is safe.

After ingest, run the coverage report to see what landed:

```bash
python3 scripts/coverage_report.py
python3 scripts/coverage_report.py --missing  # only entries not yet ingested
python3 scripts/coverage_report.py --json     # machine-readable
```

## Why some delegates have no manifest entry

The Convention met in secret. Most delegates spoke through Madison's
Notes, McHenry's notes, Yates's *Secret Proceedings*, or Farrand's
*Records*, all of which the manifest already ingests. For a delegate
who did not publish independently and whose surviving correspondence
is sparse — e.g. Jacob Broom, William Houstoun, James McClurg — the
returns from a per-delegate manifest entry are minimal and
duplicative.

As more public-domain digitisations appear (Founders Online continues
to grow), individual entries can be added without changing the
registry schema.

## Schema validation

`tests/test_delegates.py` enforces:

- exactly 55 entries,
- the historic 38/1/3/13 signing breakdown,
- unique ids, ISO-8601 dates, valid states,
- HTTPS-only archive URLs,
- every `manifest_entry` reference resolves to a real `document_id`.

These run on every `pytest` invocation.
