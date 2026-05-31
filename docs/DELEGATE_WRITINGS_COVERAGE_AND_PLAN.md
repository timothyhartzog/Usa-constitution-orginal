# Delegate Writings — Coverage Status & Completion Plan

**Question:** *Have we collected all available original writings, letters, etc. of
the members of the 1787 Constitutional Convention?*

**Short answer:** **Partly — and less than the file count suggests.** All **55
delegates** have a dossier and are richly *documented as subjects*, but the corpus
holds delegates' **own authored writings** for only a handful of framers. For 49 of
55 delegates we currently have **context about them, not a collection by them.**

> Scope: "Constitutional Congress" = the **Federal Convention of 1787**. 74 appointed,
> **55 attended**, **39 signed**. This repo tracks all 55 attendees.

---

## 1. The decisive metric: *authored* vs. *mentioned*

`data/delegates/reports/delegate_dossier_index.csv` reports, per delegate,
`matching_chunks` / `total_mentions` (passages that **mention** them) and
`authored_chunks` (passages **written by** them). For "original writings" only
`authored_chunks` counts — and it is concentrated:

| Delegate | Authored passages | Mentions |
|---|--:|--:|
| George Washington | 6,336 | 26,540 |
| Alexander Hamilton | 4,747 | 15,697 |
| James Madison | 3,581 | 14,369 |
| Robert Yates | 16 | 465 |
| George Mason | 4 | 1,298 |
| Benjamin Franklin | 2 | 1,813 |
| **All other 49 delegates** | **0** | varies |

**Dedicated authored corpora exist for ~3 delegates (Washington, Madison, Hamilton),
token amounts for Yates/Mason/Franklin, and zero for everyone else.** The large
dossiers for the rest are built from *mentions* in convention records, ratification
debates, and others' correspondence — valuable context, but not their own collected
papers.

### Data-quality caveat (don't trust raw mention counts)
Several high "mention" totals are inflated by **common-word surnames**: e.g. William
**Few** (4,608), Rufus **King** (2,851), George **Read** (2,773), Robert **Morris**
(2,290), Caleb **Strong** (2,034). Jacob Broom's top "mention" is an unrelated
Jefferson passage about reclaimed **marsh** land ("broom"). Attribution needs
disambiguation before these numbers mean anything.

---

## 2. What *is* solidly collected

- **55/55 delegate dossiers** (`data/delegates/dossiers/`), built by
  `scripts/build_delegate_dossiers.py`, with `scripts/delegate_coverage_report.py`
  and the index/coverage reports.
- A broad **public-domain source backbone** (~19k cached files in `data/raw/`):
  - **Founders Online** bulk (`data/founders_online/`) — the authored material for
    Washington, Madison, Hamilton, Franklin.
  - **Letters of Delegates to Congress** — **26 volumes (~18k files)**.
  - **Elliot's *Debates*** vols 1–5 — state ratifying-convention speeches.
  - **Farrand's *Records*** vols 1–3 + **Madison's Notes** (full) — Convention floor
    speeches of many delegates (secondhand).
  - **Works of Hamilton** (11 vols), **Writings of Jefferson** (8 vols, contextual),
    Federalist / Anti-Federalist / Bill of Rights sets.

This is a genuine 55-delegate *documentation* effort — but the **authored-writings**
layer is thin outside the major framers.

---

## 3. Honest gaps to reach "all available original writings"

1. **No dedicated authored corpora for 49 delegates.** We have not ingested the
   individual public-domain papers/works of the mid-tier framers, e.g.:
   *Works of James Wilson*; *Diary & Letters of Gouverneur Morris*; *Writings of John
   Dickinson* (Fabius); Rowland's *George Mason* (w/ papers); *Life & Correspondence
   of Rufus King*; *Life & Correspondence of George Read*; Luther Martin's *Genuine
   Information*; *Life of C.C. Pinckney*; Sherman / Ellsworth essays; etc.
2. **Other Convention note-takers not added as first-person records:** **Yates**
   (*Secret Proceedings*), **Lansing**, **Paterson**, **McHenry**.
3. **Attribution is mention-based and noisy** — `authored_chunks` is 0 for almost
   everyone because their own documents were never loaded *and* surname collisions
   pollute mention counts.
4. **Survival limits.** Some delegates left very few papers (Broom, Blair, Bassett,
   Gilman, Brearley, Houstoun) — "all available" for them is genuinely small.
5. **Copyright limits.** The most complete modern compilations — **DHRC** (Wisconsin)
   and UVA-Press *Papers of …* — are copyrighted and intentionally excluded; we cap
   at the public-domain frontier.

---

## 4. Completion plan (builds on the existing pipeline)

- **Phase 0 — Baseline.** Re-run `delegate_coverage_report.py`; rank delegates by
  `authored_chunks`. Tag: `authored-good` / `mentions-only` / `minimal-surviving`.
- **Phase 1 — Disambiguate attribution.** Fix surname collisions (Few / Read / King /
  Strong / Morris / Mason / Broom…) so `authored` vs `mentioned` is trustworthy;
  split the dossier schema into `authored_count` and `addressed/mentioned_count`.
- **Phase 2 — Founders Online maximization.** Confirm a full `fetch_founders_online.py`
  run and rebuild dossiers so every authored framer letter is attributed.
- **Phase 3 — Individual public-domain editions.** Add `scripts/fetch_ia_editions.py`
  (Internet-Archive item list) for the Wilson / G. Morris / Dickinson / Mason / Rufus
  King / George Read / Luther Martin / C.C. Pinckney volumes; route through
  clean→chunk→index; rebuild dossiers. Target: move these to `authored-good`.
- **Phase 4 — Convention note-takers.** Ingest Yates, Lansing, Paterson, McHenry as
  first-person delegate records; link to `data/process_timeline.json`.
- **Phase 5 — Audit.** Every delegate row is `authored-good`, or explicitly justified
  as `minimal-surviving` / `more-only-in-copyright`, with provenance in
  `docs/SOURCES.md`.

**Definition of done:** for each of the 55 delegates, the repo holds the *authored*
public-domain writings that exist, attribution is disambiguated, and remaining gaps
are labeled by cause (survival vs. copyright vs. not-yet-fetched).

---

## 5. Bottom line

- **Documented:** all 55 delegates (dossiers + reports). ✅
- **Their own original writings collected:** essentially only Washington, Madison,
  Hamilton (plus trivial Franklin / Mason / Yates). ❌ for the other 49.
- **Biggest wins available now:** ingest the individual public-domain *Works/Papers*
  editions + the non-Madison Convention notes, and fix surname-collision attribution.
- **Hard ceiling:** low-survival delegates and copyrighted modern editions (DHRC,
  UVA) — beyond those, Phases 1–4 reach the practical public-domain maximum.
