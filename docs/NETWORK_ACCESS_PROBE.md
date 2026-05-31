# Network Access Probe — Delegate Writings Ingestion

**Date:** 2026-05-31 · **Environment:** Claude Code on the web (managed container)

## Question
Which founding-era source hosts can we reach from this environment to fetch the
55 delegates' public-domain writings?

## Method
Direct HTTPS GET (Python `requests`) against each candidate host; a 403 body of
`Host not in allowlist` / `Host not in allowlist` indicates the network policy's
allowlist blocked it.

## Result: archival hosts are ALL blocked; only dev-infra is allowed

| Host | Result |
|---|---|
| archive.org (+ ia6/ia8 nodes) | ❌ allowlist-blocked |
| gutenberg.org (+ pglaf/ibiblio mirrors) | ❌ allowlist-blocked |
| founders.archives.gov | ❌ allowlist-blocked |
| loc.gov / tile.loc.gov | ❌ allowlist-blocked |
| avalon.law.yale.edu | ❌ allowlist-blocked |
| oll.libertyfund.org | ❌ allowlist-blocked |
| teachingamericanhistory.org | ❌ allowlist-blocked |
| en.wikisource.org | ❌ allowlist-blocked |
| consource.org | ❌ allowlist-blocked |
| press-pubs.uchicago.edu | ❌ allowlist-blocked |
| constitution.org | ❌ allowlist-blocked |
| hathitrust.org / books.google.com | ❌ allowlist-blocked |
| **github.com / raw.githubusercontent.com / codeload.github.com** | ✅ reachable |
| **pypi.org / files.pythonhosted.org** | ✅ reachable |
| huggingface.co | ❌ allowlist-blocked |

**Interpretation:** the policy is a **developer-infrastructure allowlist** (Git
hosting + Python package registries), not a research/archival one. No primary
public-domain source host is reachable.

## The one open path, and why it doesn't help here

`raw.githubusercontent.com` is reachable, so any text mirrored on GitHub can be
fetched. Project Gutenberg is mirrored by the **GITenberg** org
(`raw.githubusercontent.com/GITenberg/<Title-Slug>_<id>/master/<id>.txt`), which
served control texts (Federalist #18, Franklin's *Autobiography*) successfully.

But the **targeted delegate editions are not on Project Gutenberg** — they are
Internet-Archive / HathiTrust scans:
- *The Works of James Wilson*, *The Diary and Letters of Gouverneur Morris*,
  Rowland's *Life of George Mason*, *Life and Correspondence of Rufus King*,
  *Life and Correspondence of George Read*, Luther Martin's *Genuine Information*,
  the Yates/Lansing *Secret Proceedings*, etc.
- Probes for these on GITenberg returned `404` (confirmed not present).

A GitHub repo search surfaced **no** dedicated founding-era primary-source corpus
repositories that could substitute.

## Conclusion
Delegate-writings ingestion **cannot be completed in this environment.** It must
run where the network policy allowlists `archive.org` (and ideally
`founders.archives.gov`, `loc.gov`). Everything else is already staged:

- `config/delegate_acquisition_targets.json` — per-delegate PD source + IA ids
- `config/sources_manifest.json` → `delegate_writings` collection (30 volumes)
- `scripts/build_delegate_acquisition_matrix.py` — offline coverage classifier

### To finish, in an archive.org-allowlisted environment
```bash
python3 scripts/ingest_sources.py          # fetches the 30 delegate_writings volumes
python3 scripts/clean_text.py
python3 scripts/chunk_documents.py
python3 scripts/build_search_index.py
python3 scripts/build_delegate_dossiers.py
python3 scripts/build_delegate_acquisition_matrix.py   # authored counts rise
```

> Configuring the environment's network policy is documented at
> https://code.claude.com/docs/en/claude-code-on-the-web — an environment whose
> policy permits `archive.org` is required for the fetch step.
